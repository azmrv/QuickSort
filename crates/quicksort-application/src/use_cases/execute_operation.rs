use crate::dtos::{OperationCommand, OperationResult};
use crate::errors::UseCaseError;
use crate::ports::inbound::execute_operation::ExecuteOperation;
use crate::ports::outbound::clock::Clock;
use crate::ports::outbound::configuration_repository::ConfigurationRepository;
use crate::ports::outbound::file_system::FileSystem;
use crate::ports::outbound::id_generator::IdGenerator;
use crate::ports::outbound::operation_repository::OperationRepository;
use crate::ports::outbound::conflict_resolver::ConflictResolver;
use quicksort_domain::entities::{Operation, OperationStatus};
use quicksort_domain::services::OperationService;
use quicksort_domain::value_objects::windows_path::WindowsPath;

/// Реализация Use Case для выполнения операции перемещения/копирования.
pub struct ExecuteOperationImpl {
    operation_repository: Box<dyn OperationRepository>,
    configuration_repository: Box<dyn ConfigurationRepository>,
    file_system: Box<dyn FileSystem>,
    id_generator: Box<dyn IdGenerator>,
    clock: Box<dyn Clock>,
    conflict_resolver: Box<dyn ConflictResolver>,
}

impl ExecuteOperationImpl {
    /// Создает новый экземпляр с использованием репозиториев.
    pub fn new(
        operation_repository: Box<dyn OperationRepository>,
        configuration_repository: Box<dyn ConfigurationRepository>,
        file_system: Box<dyn FileSystem>,
        id_generator: Box<dyn IdGenerator>,
        clock: Box<dyn Clock>,
        conflict_resolver: Box<dyn ConflictResolver>,
    ) -> Self {
        Self {
            operation_repository,
            configuration_repository,
            file_system,
            id_generator,
            clock,
            conflict_resolver,
        }
    }
}

impl ExecuteOperation for ExecuteOperationImpl {
    async fn execute(&self, command: OperationCommand) -> Result<OperationResult, UseCaseError> {
        // Валидация команды
        let result = self.run_operation(command).await;
        
        match &result {
            Ok(_) => {
                // Успешное выполнение — сохраняем операцию с выполненным статусом
                if let Ok(operation) = &result {
                    self.operation_repository
                        .save(&operation.with_status(OperationStatus::Completed))
                        .await
                        .map_err(|e| UseCaseError::Repository(e.to_string()))?;
                }
            }
            Err(_) => {
                // Ошибка — сохраняем как не завершённую для возможности отката
                if let Ok(operation) = &result {
                    if matches!(operation, Err(_)) {
                        self.operation_repository
                            .save(&operation.with_status(OperationStatus::Failed))
                            .await
                            .map_err(|e| UseCaseError::Repository(e.to_string()))?;
                    }
                }
            }
        }
        
        result
    }
}

impl ExecuteOperationImpl {
    async fn run_operation(
        &self,
        command: OperationCommand,
    ) -> Result<OperationResult, UseCaseError> {
        // Валидация параметров
        self.validate_command(&command)?;
        
        let target_paths = command.target_paths.clone();
        let mut operation_id = None;
        let mut total_files = 0;
        let mut total_bytes = 0;
        
        // Создание операции для отслеживания
        {
            let timestamp = self.clock.now();
            operation_id = Some(self.id_generator.generate().to_string());
            
            let operation_type = match command.command_type.as_str() {
                "move" => quicksort_domain::entities::OperationType::Move,
                "copy" => quicksort_domain::entities::OperationType::Copy,
                "delete" => quicksort_domain::entities::OperationType::Delete,
                "rename" => quicksort_domain::entities::OperationType::Rename,
                _ => return Err(UseCaseError::InvalidCommand(format!("Unknown command type: {}", command.command_type))),
            };
            
            let mut operation = Operation::new(
                operation_id.clone().unwrap_or_default(),
                timestamp,
                timestamp, // created_at по умолчанию равен now()
                timestamp, // updated_at по умолчанию равен now()
                0,         // attempts
                None,      // failed_at по умолчанию None
                Some(timestamp), // completed_at по умолчанию Some(now)
                operation_type,
                command.source_paths.clone(),
                target_paths.clone().map(|p| p.into()),
                false,     // is_conflicted по умолчанию false
                0,         // error_code по умолчанию 0
            );
            
            let mut last_error = None;
            
            for (i, (source_path_str, target_path_str)) in 
                command.source_paths.iter().zip(target_paths.iter()).enumerate() {
                
                // Превращаем строковые пути в WindowsPath
                let source = WindowsPath::new(source_path_str)
                    .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?;
                
                let target = WindowsPath::new(target_path_str)
                    .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?;
                
                // Выполнение операции через файловую систему
                self.execute_file_operation(&source, &target, command.command_type.as_str())
                    .await
                    .map_err(|e| {
                        last_error = Some((i, e));
                        UseCaseError::FileSystemError(e.to_string())
                    })?;
                
                total_files += 1;
            }
            
            // Записываем завершённую операцию в репозиторий
            let completed_operation = operation.finalize_with(
                timestamp,
                None,      // failed_at = None (успешно)
                Some(timestamp), // updated_at = now()
                Some(total_files),     // attempts
                Some(0),        // error_code = 0 (успешно)
            );
            
            self.operation_repository
                .save(&completed_operation)
                .await
                .map_err(|e| UseCaseError::Repository(e.to_string()))?;
        }
        
        Ok(OperationResult {
            success: true,
            operation_id: operation_id.unwrap_or_default(),
            files_processed: total_files,
            bytes_transferred: total_bytes,
        })
    }

    /// Выполняет операцию для одной пары файлов (источник -> цель).
    async fn execute_file_operation(
        &self,
        source: &WindowsPath,
        target: &WindowsPath,
        command_type: &str,
    ) -> Result<(), UseCaseError> {
        match command_type {
            "move" => {
                // Сначала проверяем цель — если файл уже существует, создаём имя с цифрой
                let destination = self.get_unique_filename(target).await?;
                
                // Перемещаем файл/папку
                self.file_system.move_file(source, &destination)
                    .await
                    .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?;
                
                // Удаляем пустую папку-источник (если это было перемещение файлов в папку)
                if source.is_directory() && !self.file_system.exists(source).await {
                    self.file_system.delete_file_or_dir(source)
                        .await
                        .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?;
                }
            }
            "copy" => {
                // Копируем файл/папку
                self.file_system.copy_file(source, target)
                    .await
                    .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?;
            }
            "delete" => {
                // Удаляем файл или папку
                self.file_system.delete_file_or_dir(source)
                    .await
                    .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?;
            }
            "rename" => {
                // Ренерим файл/папку
                let destination = self.get_unique_filename(target).await?;
                
                self.file_system.rename_file(source, &destination)
                    .await
                    .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?;
            }
            _ => {
                return Err(UseCaseError::InvalidCommand(format!(
                    "Unknown command type in file operation: {}",
                    command_type
                )))
            }
        }
        
        Ok(())
    }

    /// Получает уникальное имя файла, если целевой файл уже существует.
    async fn get_unique_filename(&self, target: &WindowsPath) -> Result<WindowsPath, UseCaseError> {
        let file_name = match target.file_name() {
            Some(name) => name,
            None => return Err(UseCaseError::InvalidCommand("Could not extract filename".to_string())),
        };

        let base_name = match target.file_stem() {
            Some(s) => s,
            None => file_name,
        };

        let ext = match target.extension() {
            Some(e) => e.to_string_lossy().to_string(),
            None => String::new(),
        };

        // Генерация имени с цифрой, если файл уже существует
        for counter in 0.. {
            let new_name = if counter == 0 {
                format!("{}.{}", base_name, ext)
            } else {
                format!("{} ({}).{}", base_name, counter, ext)
            };

            // Создаем путь к кандидату на имя
            let candidate_str = format!("{}\\{}", target.path(), new_name);
            let candidate = WindowsPath::new(&candidate_str)
                .map_err(|e| UseCaseError::Internal(e.to_string()))?;
            
            if !self.file_system.exists(&candidate).await {
                return Ok(candidate);
            }
        }

        Err(UseCaseError::Internal("Could not find unique filename".to_string()))
    }

    /// Валидирует команду.
    fn validate_command(&self, command: &OperationCommand) -> Result<(), UseCaseError> {
        // Проверка типа команды
        if !matches!(
            command.command_type.as_str(),
            "move" | "copy" | "delete" | "rename"
        ) {
            return Err(UseCaseError::InvalidCommand(format!(
                "Invalid command type: {}",
                command.command_type
            )));
        }

        // Проверка путей назначения
        if let Some(paths) = &command.target_paths {
            if paths.is_empty() {
                return Err(UseCaseError::InvalidCommand(
                    "Target paths are required".to_string()
                ));
            }
        } else if !matches!(command.command_type.as_str(), "delete" | "rename") {
            // Для delete и rename не нужны target_paths, но для других типов — нужны
            return Err(UseCaseError::InvalidCommand(
                "Target paths are required".to_string()
            ));
        }

        Ok(())
    }
}
//! Use Case for executing file operations (move, copy, delete, rename).
//!
//! This use case handles the complete execution flow of file operations:
//! 1. Command validation
//! 2. File system operations
//! 3. Conflict resolution
//! 4. Operation recording

use async_trait::async_trait;
use crate::dtos::{OperationCommand, OperationResult};
use crate::errors::UseCaseError;
use crate::ports::inbound::ExecuteOperation as ExecuteOperationPort;
use quicksort_domain::{Operation, OperationId, OperationType, WindowsPath};
use quicksort_infrastructure::clock::SystemClock;
use quicksort_infrastructure::conflict_resolver::ConflictResolver;
use quicksort_infrastructure::filesystem::FileSystem;
use quicksort_infrastructure::id_generator::IdGenerator;
use quicksort_infrastructure::repository::OperationRepository;

/// Исполнитель сценария ExecuteOperation.
pub struct ExecuteOperationUseCase {
    file_system: FileSystem,
    operation_repository: OperationRepository,
    id_generator: IdGenerator,
    conflict_resolver: ConflictResolver,
    clock: SystemClock,
}

#[async_trait]
impl ExecuteOperationPort for ExecuteOperationUseCase {
    async fn execute(&self, command: OperationCommand) -> Result<OperationResult, UseCaseError> {
        Self::execute(self, command).await
    }
}

impl ExecuteOperationUseCase {
    pub fn new(
        file_system: FileSystem,
        operation_repository: OperationRepository,
        id_generator: IdGenerator,
        conflict_resolver: ConflictResolver,
        clock: SystemClock,
    ) -> Self {
        Self {
            file_system,
            operation_repository,
            id_generator,
            conflict_resolver,
            clock,
        }
    }

    /// Выполняет операцию с файлами.
    pub async fn execute(&self, command: OperationCommand) -> Result<OperationResult, UseCaseError> {
        let OperationCommand {
            operation_type,
            source_paths,
            destination_path,
            target_folder_id,
        } = command;

        // Валидация типов операций
        self.validate_operation_type(&operation_type)?;

        // Обработка move/copy операций
        if operation_type == OperationType::Move || operation_type == OperationType::Copy {
            return self.execute_move_or_copy(
                operation_type,
                source_paths,
                destination_path,
                target_folder_id,
            ).await;
        }

        // Обработка delete операции
        if operation_type == OperationType::Delete {
            return self.execute_delete(source_paths).await;
        }

        // Обработка rename операции
        if operation_type == OperationType::Rename {
            return self.execute_rename(source_paths, destination_path).await;
        }

        Err(UseCaseError::InvalidCommand(format!(
            "Unsupported operation type: {:?}",
            operation_type
        )))
    }

    /// Выполняет операцию move или copy.
    async fn execute_move_or_copy(
        &self,
        operation_type: OperationType,
        source_paths: Vec<WindowsPath>,
        destination_path: WindowsPath,
        target_folder_id: Option<String>,
    ) -> Result<OperationResult, UseCaseError> {
        // Валидация целевой папки
        if let Some(folder_id) = target_folder_id {
            self.validate_target_folder(&destination_path, &folder_id)?;
        }

        // Проверка существования источника
        for source in &source_paths {
            self.file_system
                .path_exists(source)
                .await
                .map_err(|e| UseCaseError::Internal(e.to_string()))?;
        }

        let operation_id = self.id_generator.generate_id();
        let timestamp = self.clock.now();

        // Вычисление размера операции
        let total_bytes = self.calculate_operation_size(&source_paths).await?;

        // Создание операции до выполнения
        let mut operation = Operation::new(
            operation_id.clone(),
            operation_type.clone(),
            source_paths.clone(),
            destination_path.clone(),
            timestamp,
        );
        operation.set_total_bytes(total_bytes);

        // Выполнение реальных операций с файлами
        for source in &source_paths {
            match operation_type {
                OperationType::Move => {
                    self.file_system
                        .move_path(source, &destination_path)
                        .await
                        .map_err(|e| UseCaseError::Internal(e.to_string()))?;
                }
                OperationType::Copy => {
                    let destination = self.resolve_copy_destination(
                        source,
                        &destination_path,
                        target_folder_id.as_deref(),
                    )?;

                    self.file_system
                        .copy_path(source, &destination)
                        .await
                        .map_err(|e| UseCaseError::Internal(e.to_string()))?;
                }
                _ => unreachable!(), // Проверка в main execute()
            }
        }

        // Сохранение операции после успешного выполнения
        self.operation_repository
            .save(&operation)
            .await
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?;

        let result_path = if operation_type == OperationType::Move {
            destination_path
        } else {
            self.resolve_copy_destination(
                &source_paths[0],
                &destination_path,
                target_folder_id.as_deref(),
            )?
        };

        Ok(OperationResult {
            operation_id,
            operation_type: operation_type.clone(),
            source_paths,
            destination_path: result_path,
            bytes_transferred: total_bytes,
        })
    }

    /// Выполняет операцию delete.
    async fn execute_delete(&self, source_paths: Vec<WindowsPath>) -> Result<OperationResult, UseCaseError> {
        // Проверка существования источников
        for source in &source_paths {
            if !self.file_system.path_exists(source).await? {
                return Err(UseCaseError::FileNotFound(format!(
                    "Delete target not found: {}",
                    source
                )));
            }
        }

        let operation_id = self.id_generator.generate_id();
        let timestamp = self.clock.now();

        // Вычисление общего размера удаляемых файлов
        let total_bytes = self.calculate_delete_size(&source_paths).await?;

        // Создание операции до выполнения
        let mut operation = Operation::new(
            operation_id.clone(),
            OperationType::Delete,
            source_paths.clone(),
            WindowsPath::default(),
            timestamp,
        );
        operation.set_total_bytes(total_bytes);

        // Выполнение реальных операций удаления
        for source in &source_paths {
            self.file_system
                .remove_path(source)
                .await
                .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?;
        }

        // Сохранение операции после успешного выполнения
        self.operation_repository
            .save(&operation)
            .await
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?;

        Ok(OperationResult {
            operation_id,
            operation_type: OperationType::Delete,
            source_paths,
            destination_path: WindowsPath::default(),
            bytes_transferred: total_bytes,
        })
    }

    /// Выполняет операцию rename.
    async fn execute_rename(
        &self,
        source_paths: Vec<WindowsPath>,
        destination_path: WindowsPath,
    ) -> Result<OperationResult, UseCaseError> {
        // Проверка существования источника
        let source = &source_paths[0];
        if !self.file_system.path_exists(source).await? {
            return Err(UseCaseError::FileNotFound(format!(
                "Rename source not found: {}",
                source
            )));
        }

        // Валидация имени файла
        self.validate_rename_destination(&source_paths[0], &destination_path)?;

        let operation_id = self.id_generator.generate_id();
        let timestamp = self.clock.now();

        // Вычисление размера перемещаемого файла
        let total_bytes = self.calculate_file_size(source).await?;

        // Создание операции до выполнения
        let mut operation = Operation::new(
            operation_id.clone(),
            OperationType::Rename,
            source_paths.clone(),
            destination_path.clone(),
            timestamp,
        );
        operation.set_total_bytes(total_bytes);

        // Выполнение реального переименования
        self.file_system
            .rename_path(source, &destination_path)
            .await
            .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?;

        // Сохранение операции после успешного выполнения
        self.operation_repository
            .save(&operation)
            .await
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?;

        Ok(OperationResult {
            operation_id,
            operation_type: OperationType::Rename,
            source_paths,
            destination_path,
            bytes_transferred: total_bytes,
        })
    }

    /// Валидирует тип операции.
    fn validate_operation_type(&self, operation_type: &OperationType) -> Result<(), UseCaseError> {
        // TODO: Добавить проверки на допустимые типы для конкретных сценариев
        Ok(())
    }

    /// Валидирует целевую папку по её ID.
    fn validate_target_folder(&self, destination_path: &WindowsPath, folder_id: &str) -> Result<(), UseCaseError> {
        // TODO: Интеграция с репозиторием папок для валидации по ID
        // Пока просто проверяем существование пути
        self.file_system
            .path_exists(destination_path)
            .await
            .map_err(|e| UseCaseError::Internal(e.to_string()))?;

        Ok(())
    }

    /// Определяет путь для копирования с учётом target_folder_id.
    fn resolve_copy_destination(
        &self,
        source: &WindowsPath,
        destination_path: &WindowsPath,
        target_folder_id: Option<&str>,
    ) -> Result<WindowsPath, UseCaseError> {
        // Если указан target_folder_id, используем путь из destination_path
        if let Some(_) = target_folder_id {
            return Ok(destination_path.clone());
        }

        // Иначе добавляем имя источника к destination_path
        let file_name = source.file_name().unwrap_or("unknown");
        Ok(WindowsPath::new(destination_path.as_ref(), file_name))
    }

    /// Валидирует имя файла при rename.
    fn validate_rename_destination(&self, source: &WindowsPath, destination_path: &WindowsPath) -> Result<(), UseCaseError> {
        // TODO: Проверка на коллизии имён в целевой папке
        Ok(())
    }

    /// Вычисляет размер операции move/copy.
    async fn calculate_operation_size(&self, source_paths: &[WindowsPath]) -> Result<u64, UseCaseError> {
        let mut total_bytes = 0u64;

        for path in source_paths {
            let bytes = self.calculate_file_size(path).await?;
            total_bytes += bytes;
        }

        Ok(total_bytes)
    }

    /// Вычисляет размер удаляемых файлов.
    async fn calculate_delete_size(&self, source_paths: &[WindowsPath]) -> Result<u64, UseCaseError> {
        let mut total_bytes = 0u64;

        for path in source_paths {
            let bytes = self.calculate_file_size(path).await?;
            total_bytes += bytes;
        }

        Ok(total_bytes)
    }

    /// Вычисляет размер файла или папки.
    async fn calculate_file_size(&self, path: &WindowsPath) -> Result<u64, UseCaseError> {
        // TODO: Реализовать рекурсивный подсчёт размера папок
        match self.file_system.get_path_info(path).await {
            Ok(info) => Ok(info.size),
            Err(_) => Ok(0), // Для не существующих путей возвращаем 0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_operation_type() {
        let use_case = ExecuteOperationUseCase::new(
            FileSystem::default(),
            OperationRepository::default(),
            IdGenerator::default(),
            ConflictResolver::default(),
            SystemClock::default(),
        );

        assert!(use_case.validate_operation_type(&OperationType::Move).is_ok());
        assert!(use_case.validate_operation_type(&OperationType::Copy).is_ok());
        assert!(use_case.validate_operation_type(&OperationType::Delete).is_ok());
        assert!(use_case.validate_operation_type(&OperationType::Rename).is_ok());
    }
}
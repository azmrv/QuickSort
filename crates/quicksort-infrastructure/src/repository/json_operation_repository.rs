//! JSON реализация репозитория операций.
//! Хранит историю операций в JSON-файле для последующего восстановления состояния при откате.

use quicksort_domain::{
    entities::Operation,
    errors::ApplicationError,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Структура данных для сериализации операций в JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationLogEntry {
    pub id: String,
    pub operation_type: String,
    pub source_path: Option<String>,
    pub destination_path: Option<String>,
    pub old_name: Option<String>,
    pub new_name: Option<String>,
    pub timestamp: u64,
    pub file_count: usize,
    pub metadata: Option<serde_json::Value>,
}

impl OperationLogEntry {
    /// Создаёт новый записи из операции.
    fn from_operation(operation: &Operation) -> Self {
        Self {
            id: operation.id().clone(),
            operation_type: operation.operation_type().to_string(),
            source_path: operation.source_path().cloned(),
            destination_path: operation.destination_path().cloned(),
            old_name: operation.old_name().cloned(),
            new_name: operation.new_name().cloned(),
            timestamp: operation.timestamp().into(),
            file_count: operation.file_count(),
            metadata: operation.metadata().as_ref().map(|m| serde_json::to_value(m).unwrap_or_default()),
        }
    }

    /// Создаёт операцию из записи.
    fn into_operation(self) -> Operation {
        Operation::new(
            &self.id,
            &self.operation_type,
            self.source_path,
            self.destination_path,
            self.old_name,
            self.new_name,
            Some(&self.timestamp as u64),
            self.file_count,
            None, // metadata пока не используется
        )
    }
}

/// Репозиторий операций с сохранением в JSON-файл.
pub struct JsonOperationRepository {
    file_path: String,
}

impl JsonOperationRepository {
    /// Создаёт новый репозиторий операций с указанным путём к файлу журнала.
    pub fn new(file_path: impl Into<String>) -> Self {
        let path = file_path.into();
        if !Path::new(&path).exists() {
            // Создаём родительскую директорию, если она не существует
            if let Some(parent) = Path::new(&path).parent() {
                fs::create_dir_all(parent).ok();
            }
        }
        Self { file_path: path }
    }

    /// Загружает историю операций из файла.
    pub fn load_operations(&self) -> Vec<Operation> {
        if !Path::new(&self.file_path).exists() {
            return vec![];
        }

        let content = match fs::read_to_string(&self.file_path) {
            Ok(c) => c,
            Err(_) => return vec![],
        };

        let entries: Vec<OperationLogEntry> =
            match serde_json::from_str(&content) {
                Ok(e) => e,
                Err(_) => return vec![],
            };

        entries.into_iter().map(|entry| entry.into_operation()).collect()
    }

    /// Сохраняет историю операций в файл.
    pub fn save_operations(&self, operations: &[Operation]) -> Result<(), ApplicationError> {
        let entries: Vec<OperationLogEntry> = operations.iter().map(|op| OperationLogEntry::from_operation(op)).collect();

        let json = serde_json::to_string_pretty(&entries).map_err(|e| ApplicationError::Io(e))?;
        fs::write(&self.file_path, json).map_err(|e| ApplicationError::Io(e))?;

        Ok(())
    }

    /// Добавляет операцию в журнал.
    pub fn add_operation(&mut self, operation: &Operation) -> Result<(), ApplicationError> {
        // Загружаем существующий журнал
        let mut operations = self.load_operations();
        // Добавляем новую операцию
        operations.push(operation.clone());
        // Сохраняем обратно
        self.save_operations(&operations)
    }

    /// Удаляет операцию по ID (для отката).
    pub fn remove_operation(&mut self, operation_id: &str) -> Result<(), ApplicationError> {
        let mut entries = match fs::read_to_string(&self.file_path) {
            Ok(c) => c,
            Err(_) => return Ok(()),
        };

        if entries.is_empty() {
            return Ok(());
        }

        // Парсим JSON и удаляем операцию с указанным ID
        let parsed: Vec<OperationLogEntry> =
            serde_json::from_str(&entries).map_err(|e| ApplicationError::Io(e))?;
        let filtered: Vec<OperationLogEntry> = parsed.iter().filter(|e| e.id != operation_id).cloned().collect();

        entries = serde_json::to_string_pretty(&filtered).map_err(|e| ApplicationError::Io(e))?;
        fs::write(&self.file_path, entries).map_err(|e| ApplicationError::Io(e))?;

        Ok(())
    }

    /// Очищает весь журнал операций.
    pub fn clear_operations(&mut self) -> Result<(), ApplicationError> {
        fs::write(&self.file_path, "[]")?;
        Ok(())
    }

    /// Получает путь к файлу журнала.
    pub fn file_path(&self) -> &str {
        &self.file_path
    }
}

impl Default for JsonOperationRepository {
    fn default() -> Self {
        let path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("data"));
        Self::new(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_load_empty_repository() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("operations.json");
        let repo = JsonOperationRepository::new(&file_path);

        let operations = repo.load_operations();
        assert!(operations.is_empty());
    }

    #[test]
    fn test_save_and_load_operation() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("operations.json");
        let mut repo = JsonOperationRepository::new(&file_path);

        // Создаём тестовую операцию
        let operation = Operation::new(
            "test-uuid",
            "move",
            Some("source/file.txt"),
            Some("dest/file.txt"),
            None,
            None,
            None,
            1,
            None,
        );

        repo.add_operation(&operation).unwrap();

        let operations = repo.load_operations();
        assert_eq!(operations.len(), 1);
        assert_eq!(operations[0].id(), "test-uuid");
    }

    #[test]
    fn test_remove_operation() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("operations.json");
        let mut repo = JsonOperationRepository::new(&file_path);

        // Создаём тестовые операции
        let operation1 = Operation::new(
            "test-uuid-1",
            "move",
            None,
            None,
            None,
            None,
            None,
            1,
            None,
        );
        let operation2 = Operation::new(
            "test-uuid-2",
            "copy",
            None,
            None,
            None,
            None,
            None,
            2,
            None,
        );

        repo.add_operation(&operation1).unwrap();
        repo.add_operation(&operation2).unwrap();

        assert_eq!(repo.load_operations().len(), 2);

        // Удаляем первую операцию
        repo.remove_operation("test-uuid-1").unwrap();

        let operations = repo.load_operations();
        assert_eq!(operations.len(), 1);
        assert_eq!(operations[0].id(), "test-uuid-2");
    }
}
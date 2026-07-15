//! Data Transfer Objects (DTO) для взаимодействия между слоями.
//! 
//! DTO находятся в доменном слое и не зависят от внешнего мира,
//! что позволяет инфраструктуре использовать их без зависимости от application.

use serde::{Deserialize, Serialize};
use crate::{FolderId, OperationType, WindowsPath};

/// Команда для выполнения операции с файлами.
/// Создаётся адаптерами (GUI, CLI, Shell) и передаётся в порт ExecuteOperation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationCommand {
    /// Тип операции (Move, Copy, Delete, Rename).
    pub operation_type: OperationType,
    /// Пути к исходным файлам/папкам.
    pub source_paths: Vec<WindowsPath>,
    /// ID целевой папки (для Move/Copy когда цель — контейнер).
    pub target_folder_id: Option<FolderId>,
    /// Явный список целевых путей (требуется для Rename). Если указан, переопределяет target_folder_id.
    pub target_paths: Option<Vec<WindowsPath>>,
    /// Политика обработки конфликтов при наличии существующего файла.
    pub overwrite_policy: OverwritePolicy,
}

/// Стратегия разрешения конфликтов при существовании целевого файла.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OverwritePolicy {
    /// Пропускать файл, записать предупреждение в лог.
    Skip,
    /// Заменить существующий файл.
    Overwrite,
    /// Добавить суффикс (например, "file (1).txt").
    AutoRename,
    /// Спросить пользователя (обрабатывается портом ConflictResolver).
    Ask,
}

/// Результат выполнения операции.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResult {
    /// ID выполненной операции.
    pub operation_id: String,
    /// Статус операции (Success, PartialFailure, Failure).
    pub status: OperationStatus,
    /// Список успешно выполненных подопераций.
    pub succeeded_operations: Vec<OperationSummary>,
    /// Список неуспешных подопераций с причинами сбоя.
    pub failed_operations: Vec<OperationFailure>,
}

/// Статус выполнения операции.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OperationStatus {
    /// Все операции выполнены успешно.
    Success,
    /// Некоторые операции выполнены с ошибками.
    PartialFailure,
    /// Операция не выполнена ни разу (критическая ошибка).
    Failure,
}

/// Сводка об успешной подоперации.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationSummary {
    /// Тип операции.
    pub operation_type: OperationType,
    /// Источник.
    pub source_paths: Vec<WindowsPath>,
    /// Целевые пути (если применимо).
    pub target_paths: Option<Vec<WindowsPath>>,
}

/// Сбой подоперации.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationFailure {
    /// Тип операции.
    pub operation_type: OperationType,
    /// Источник.
    pub source_path: WindowsPath,
    /// Причина сбоя (ошибка).
    pub error_message: String,
}

/// Создает OperationFailure из строкового сообщения об ошибке.
pub fn create_operation_failure(
  operation_type: OperationType,
  source_path: WindowsPath,
  error_message: String,
) -> OperationFailure {
  OperationFailure {
    operation_type,
    source_path,
    error_message,
  }
}

/// Конструктор для создания OperationFailure с типом операции и сообщением.
pub fn create_failure_from_error(
  operation_type: OperationType,
  source_path: WindowsPath,
  error: impl std::fmt::Display,
) -> OperationFailure {
  create_operation_failure(operation_type, source_path, error.to_string())
}

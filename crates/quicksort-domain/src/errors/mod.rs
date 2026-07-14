//! Domain errors – business rule violations.
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DomainError {
    #[error("Путь пустой")]
    EmptyPath,

    #[error("Неверный путь: {0}")]
    InvalidPath(String),

    #[error("Некорректное имя папки: {0}")]
    InvalidFolderName(String),

    #[error("Недопустимая целевая директория (корень)")]
    IllegalDirectoryTarget,

    #[error("Неверный переход состояния операции")]
    InvalidStateTransition,

    #[error("Операция не найдена")]
    OperationNotFound,

    #[error("Папка не найдена")]
    FolderNotFound,

    #[error("Конфликт: {0}")]
    Conflict(String),

    #[error("Отказ в доступе: {0}")]
    PermissionDenied(String),

    #[error("Внутренняя ошибка домена: {0}")]
    Internal(String),
}
//! Use cases (application business logic orchestration).

mod execute_operation;
mod undo_operation;
mod get_folders;
mod manage_folders;

pub use execute_operation::ExecuteOperationUseCase;
pub use undo_operation::UndoOperationUseCase;
pub use get_folders::GetFoldersUseCase;
pub use manage_folders::ManageFoldersUseCase;
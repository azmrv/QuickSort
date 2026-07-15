//! Inbound ports define the public API of the Application Layer.
//! Adapters (GUI, CLI, Shell) depend on these interfaces.

mod execute_operation;
mod undo_operation;
mod get_folders;
mod manage_folders;
mod facade;
mod facade_impl;

pub use execute_operation::ExecuteOperation;
pub use undo_operation::UndoOperation;
pub use get_folders::GetFolders;
pub use manage_folders::ManageFolders;
pub use facade::ApplicationFacade;
pub use facade_impl::ApplicationFacadeImpl;
//! Unified facade combining all inbound ports.

use super::*;

/// Combined interface for all inbound operations.
pub trait ApplicationFacade:
    ExecuteOperation
    // + UndoOperation
    + GetFolders
    + ManageFolders
    + Send
    + Sync
{
}

// Blanket implementation for any type that implements all required traits.
impl<T> ApplicationFacade for T where
    T: ExecuteOperation
        // + UndoOperation
        + GetFolders
        + ManageFolders
        + Send
        + Sync
{

}

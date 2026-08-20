//! IPC protocol definitions – **DEPRECATED**.
//!
//! The canonical versions of these types now live in
//! `quicksort-ipc-contract`.  This module is kept temporarily so that
//! any code importing from here continues to compile during migration.
//!
//! New code should import directly from `quicksort_ipc_contract`.

pub use quicksort_ipc_contract::{
    CommandMessage, ExecuteOperationData, OperationType, OverwritePolicy, ResponseMessage,
    ResponseStatus,
};

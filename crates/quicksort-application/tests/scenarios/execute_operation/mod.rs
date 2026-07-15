//! Executable specifications for `ExecuteOperationUseCase`.
//!
//! This module contains scenario-based integration tests that verify
//! the behavior of the Execute Operation use case under various
//! conditions (move, copy, delete, rename, conflicts, errors, undo).
//!
//! # Running Tests
//! ```bash
//! cargo test --test integration scenarios::execute_operation
//! cargo test --test integration scenarios::execute_operation::move
//! cargo test --test integration scenarios::execute_operation::conflicts -- --nocapture
//! ```

mod conflicts;
mod copy;
mod delete;
mod errors;
mod move;
mod rename;
mod undo;
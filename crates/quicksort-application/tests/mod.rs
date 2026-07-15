//! Integration tests for the Application Layer.
//!
//! This module contains tests that verify the behavior of Use Cases
//! with real or mock implementations of outbound ports. These tests
//! are designed to be run with `cargo test --test integration`.
//!
//! # Test Organization
//! | Module | Purpose |
//! |--------|---------|
//! | `mocks` | Shared mock implementations of outbound ports (`ConfigurationRepository`, `FileSystem`, `Clock`, etc.) |
//! | `scenarios` | Executable specifications organized by Use Case and scenario type |
//!
//! # Running Tests
//! ```bash
//! # Run all integration tests
//! cargo test --test integration
//!
//! # Run only move operation tests
//! cargo test --test integration scenarios::execute_operation::move
//!
//! # Run with output
//! cargo test --test integration -- --nocapture
//! ```

pub mod mocks;
pub mod scenarios;
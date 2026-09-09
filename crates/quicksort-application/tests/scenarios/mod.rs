//! Shared helpers for the scenario-based integration tests.
//!
//! The mock implementations of the outbound ports live in `crate::mocks`
//! (the single source of truth for test doubles). This module only
//! provides common scenario helpers such as [`test_folder`].

use chrono::Utc;
use quicksort_domain::{AbsolutePath, Folder, FolderId};

// ============================================================================
// Test helpers
// ============================================================================

/// Creates a default folder for testing.
pub fn test_folder() -> Folder {
    let now = Utc::now();
    Folder {
        id: FolderId::from_string("00000000-0000-0000-0000-000000000001").unwrap(),
        name: "Documents".to_string(),
        path: AbsolutePath::new(if cfg!(target_os = "windows") {
            "C:\\Users\\Test\\Documents"
        } else {
            "/home/test/Documents"
        })
        .unwrap(),
        favorite: false,
        order: 0,
        color: None,
        stats: Default::default(),
        created_at: now,
        updated_at: now,
    }
}

pub mod execute_operation;
pub mod logging;

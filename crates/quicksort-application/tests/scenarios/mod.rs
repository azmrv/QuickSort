//! Test doubles (mocks/stubs) for all outbound ports.
//!
//! These are used in the executable specifications to test Use Cases
//! in isolation without touching real infrastructure.
//!
//! # Design Decisions
//! - All mocks use `std::sync::Mutex` for simplicity (tests are single-threaded).
//! - `MockIdGenerator` generates deterministic IDs for predictable test assertions.
//! - `MockClock` allows controlling time to test time-dependent logic.
//! - `MockConflictResolver` passes commands through unchanged for most tests.
//!
//! # Usage
//! ```rust
//! use crate::mocks::*;
//!
//! let config_repo = MockConfigurationRepository::new();
//! let folder = test_folder();
//! config_repo.add(folder.clone()).await.unwrap();
//! ```

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

use quicksort_domain::{
    Folder, FolderId, Operation, OperationId, WindowsPath,
};
use quicksort_application::ports::outbound::{
    ConfigurationRepository, OperationRepository, FileSystem,
    IdGenerator, Clock, ConflictResolver,
};
use quicksort_application::dtos::OperationCommand;
use quicksort_application::errors::UseCaseError;

// ============================================================================
// Mock: ConfigurationRepository
// ============================================================================

/// In-memory implementation of `ConfigurationRepository` for testing.
///
/// Allows pre-loading of folder data and captures save operations
/// for verification.
pub struct MockConfigurationRepository {
    folders: Mutex<Vec<Folder>>,
}

impl MockConfigurationRepository {
    /// Creates a new empty repository.
    pub fn new() -> Self {
        Self {
            folders: Mutex::new(Vec::new()),
        }
    }

    /// Pre-loads the repository with the given folders.
    pub fn set_folders(&self, folders: Vec<Folder>) {
        *self.folders.lock().unwrap() = folders;
    }

    /// Returns the current list of folders (for verification).
    pub fn get_folders(&self) -> Vec<Folder> {
        self.folders.lock().unwrap().clone()
    }
}

#[async_trait]
impl ConfigurationRepository for MockConfigurationRepository {
    async fn load_all(&self) -> Result<Vec<Folder>, UseCaseError> {
        Ok(self.folders.lock().unwrap().clone())
    }

    async fn save_all(&self, folders: &[Folder]) -> Result<(), UseCaseError> {
        *self.folders.lock().unwrap() = folders.to_vec();
        Ok(())
    }

    async fn add(&self, folder: Folder) -> Result<(), UseCaseError> {
        let mut folders = self.folders.lock().unwrap();
        if folders.iter().any(|f| f.id == folder.id) {
            return Err(UseCaseError::InvalidCommand(format!(
                "Folder with ID {} already exists",
                folder.id
            )));
        }
        folders.push(folder);
        Ok(())
    }

    async fn remove(&self, id: &FolderId) -> Result<(), UseCaseError> {
        let mut folders = self.folders.lock().unwrap();
        folders.retain(|f| f.id != *id);
        Ok(())
    }

    async fn find_by_id(&self, id: &FolderId) -> Result<Option<Folder>, UseCaseError> {
        let folders = self.folders.lock().unwrap();
        Ok(folders.iter().find(|f| f.id == *id).cloned())
    }

    async fn find_by_path(&self, path: &str) -> Result<Option<Folder>, UseCaseError> {
        let folders = self.folders.lock().unwrap();
        Ok(folders.iter().find(|f| f.path.to_string_lossy() == path).cloned())
    }

    async fn get_default_folder_id(&self) -> Result<FolderId, UseCaseError> {
        let folders = self.folders.lock().unwrap();
        folders.first()
            .map(|f| f.id.clone())
            .ok_or_else(|| UseCaseError::FolderNotFound("No default folder configured".to_string()))
    }
}

// ============================================================================
// Mock: OperationRepository
// ============================================================================

/// In-memory implementation of `OperationRepository` for testing.
///
/// Stores operations in a `HashMap<String, Operation>` keyed by
/// `OperationId::to_string()`.
pub struct MockOperationRepository {
    operations: Mutex<HashMap<String, Operation>>,
}

impl MockOperationRepository {
    /// Creates a new empty repository.
    pub fn new() -> Self {
        Self {
            operations: Mutex::new(HashMap::new()),
        }
    }

    /// Pre-loads an operation for testing.
    pub fn set_operation(&self, operation: Operation) {
        self.operations.lock().unwrap()
            .insert(operation.id.to_string(), operation);
    }

    /// Returns the number of stored operations (for verification).
    pub fn count(&self) -> usize {
        self.operations.lock().unwrap().len()
    }
}

#[async_trait]
impl OperationRepository for MockOperationRepository {
    async fn find_by_id(&self, id: &OperationId) -> Result<Option<Operation>, UseCaseError> {
        Ok(self.operations.lock().unwrap().get(&id.to_string()).cloned())
    }

    async fn save(&self, operation: &Operation) -> Result<(), UseCaseError> {
        self.operations.lock().unwrap()
            .insert(operation.id.to_string(), operation.clone());
        Ok(())
    }

    async fn delete(&self, id: &OperationId) -> Result<(), UseCaseError> {
        self.operations.lock().unwrap().remove(&id.to_string());
        Ok(())
    }

    async fn load_all(&self) -> Result<Vec<Operation>, UseCaseError> {
        Ok(self.operations.lock().unwrap().values().cloned().collect())
    }
}

// ============================================================================
// Mock: FileSystem
// ============================================================================

/// In-memory simulation of the file system for testing.
///
/// Uses a `HashMap<PathBuf, u64>` where the key is the file path
/// and the value is the file size in bytes.
pub struct MockFileSystem {
    files: Mutex<HashMap<PathBuf, u64>>,
}

impl MockFileSystem {
    /// Creates a new empty file system.
    pub fn new() -> Self {
        Self {
            files: Mutex::new(HashMap::new()),
        }
    }

    /// Pre-populates a file entry (used by tests to set up source files).
    pub fn add_file(&self, path: PathBuf, size: u64) {
        self.files.lock().unwrap().insert(path, size);
    }

    /// Helper to convert `WindowsPath` to `PathBuf` for internal storage.
    fn to_pathbuf(path: &WindowsPath) -> PathBuf {
        PathBuf::from(path.to_string_lossy().as_ref())
    }
}

#[async_trait]
impl FileSystem for MockFileSystem {
    async fn exists(&self, path: &WindowsPath) -> Result<bool, UseCaseError> {
        let path = Self::to_pathbuf(path);
        Ok(self.files.lock().unwrap().contains_key(&path))
    }

    async fn get_file_size(&self, path: &WindowsPath) -> Result<u64, UseCaseError> {
        let path = Self::to_pathbuf(path);
        let files = self.files.lock().unwrap();
        match files.get(&path) {
            Some(size) => Ok(*size),
            None => Err(UseCaseError::FileNotFound(format!("File not found: {}", path.display()))),
        }
    }

    async fn move_file(&self, from: &WindowsPath, to: &WindowsPath) -> Result<u64, UseCaseError> {
        let from_path = Self::to_pathbuf(from);
        let to_path = Self::to_pathbuf(to);
        let mut files = self.files.lock().unwrap();

        let size = files.remove(&from_path)
            .ok_or_else(|| UseCaseError::FileNotFound(format!("Source not found: {}", from_path.display())))?;

        files.insert(to_path, size);
        Ok(size)
    }

    async fn copy_file(&self, from: &WindowsPath, to: &WindowsPath) -> Result<u64, UseCaseError> {
        let from_path = Self::to_pathbuf(from);
        let to_path = Self::to_pathbuf(to);
        let files = self.files.lock().unwrap();

        let size = files.get(&from_path)
            .ok_or_else(|| UseCaseError::FileNotFound(format!("Source not found: {}", from_path.display())))?;
        let size_clone = *size;
        drop(files);

        self.files.lock().unwrap().insert(to_path, size_clone);
        Ok(size_clone)
    }

    async fn delete_file(&self, path: &WindowsPath) -> Result<(), UseCaseError> {
        let path = Self::to_pathbuf(path);
        let mut files = self.files.lock().unwrap();
        files.remove(&path)
            .ok_or_else(|| UseCaseError::FileNotFound(format!("File not found: {}", path.display())))?;
        Ok(())
    }

    async fn rename_file(&self, from: &WindowsPath, to: &WindowsPath) -> Result<(), UseCaseError> {
        let from_path = Self::to_pathbuf(from);
        let to_path = Self::to_pathbuf(to);
        let mut files = self.files.lock().unwrap();

        let size = files.remove(&from_path)
            .ok_or_else(|| UseCaseError::FileNotFound(format!("Source not found: {}", from_path.display())))?;

        files.insert(to_path, size);
        Ok(())
    }
}

// ============================================================================
// Mock: IdGenerator
// ============================================================================

/// A predictable ID generator for testing.
///
/// Returns incrementing IDs of the form `test-op-001`, `test-op-002`, etc.
pub struct MockIdGenerator {
    counter: Mutex<u32>,
}

impl MockIdGenerator {
    /// Creates a new generator starting at 1.
    pub fn new() -> Self {
        Self {
            counter: Mutex::new(1),
        }
    }
}

impl IdGenerator for MockIdGenerator {
    fn generate(&self) -> OperationId {
        let mut counter = self.counter.lock().unwrap();
        let id = format!("test-op-{:03}", *counter);
        *counter += 1;
        OperationId::from_string(&id)
    }
}

// ============================================================================
// Mock: Clock
// ============================================================================

/// A controllable clock for testing.
///
/// The time can be set manually, allowing tests to verify timestamp-based
/// logic (e.g., `created_at`, `updated_at`).
pub struct MockClock {
    now: Mutex<DateTime<Utc>>,
}

impl MockClock {
    /// Creates a new clock with a fixed timestamp.
    pub fn new(now: DateTime<Utc>) -> Self {
        Self {
            now: Mutex::new(now),
        }
    }

    /// Advances the clock by a given duration.
    pub fn advance(&self, duration: chrono::Duration) {
        let mut now = self.now.lock().unwrap();
        *now = *now + duration;
    }

    /// Sets the clock to a specific time.
    pub fn set(&self, now: DateTime<Utc>) {
        *self.now.lock().unwrap() = now;
    }
}

impl Clock for MockClock {
    fn now(&self) -> DateTime<Utc> {
        *self.now.lock().unwrap()
    }
}

// ============================================================================
// Mock: ConflictResolver
// ============================================================================

/// A simple conflict resolver that always returns the command unchanged.
///
/// For tests that need specific conflict resolution behavior,
/// create a custom mock.
pub struct MockConflictResolver;

#[async_trait]
impl ConflictResolver for MockConflictResolver {
    async fn resolve(&self, command: OperationCommand) -> Result<OperationCommand, UseCaseError> {
        // Return the command unchanged – tests that need different behavior
        // can replace this mock.
        Ok(command)
    }
}

// ============================================================================
// Test helpers
// ============================================================================

/// Creates a default folder for testing.
pub fn test_folder() -> Folder {
    Folder {
        id: FolderId::from_string("folder-1"),
        name: "Documents".to_string(),
        path: WindowsPath::new("C:\\Users\\Test\\Documents").unwrap(),
        favorite: false,
        order: 0,
        stats: Default::default(),
    }
}

/// Creates a `WindowsPath` from a string for testing.
pub fn test_file(path: &str) -> WindowsPath {
    WindowsPath::new(path).expect("Invalid test path")
}
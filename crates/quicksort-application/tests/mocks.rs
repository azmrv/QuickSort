//! Shared mock implementations of outbound ports for testing.
//!
//! Each mock provides a simple in-memory implementation that can be
//! controlled by the test scenario. They implement the corresponding
//! outbound port trait and can be injected into Use Cases during
//! integration testing.
//!
//! # Usage
//! ```rust
//! use crate::mocks::MockConfigurationRepository;
//! let repo = MockConfigurationRepository::new();
//! repo.set_folders(vec![/* test folders */]);
//! ```
//!
//! # Design Note
//! Mocks use `std::sync::Mutex` (not `parking_lot` or `tokio::sync::Mutex`)
//! to keep them simple and synchronous. Since all test scenarios are
//! single-threaded or use `tokio::task::spawn_blocking`, this is safe.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use async_trait::async_trait;
use quicksort_domain::{Folder, FolderId, Operation, OperationId, WindowsPath};
use crate::errors::UseCaseError;
use crate::ports::outbound::{
    ConfigurationRepository, OperationRepository, FileSystem,
    IdGenerator, Clock, ConflictResolver,
};
use crate::dtos::OperationCommand;

// ============================================================================
// Mock ConfigurationRepository
// ============================================================================

/// In-memory implementation of `ConfigurationRepository` for testing.
///
/// Allows pre-loading of folder data and captures save operations
/// for verification.
pub struct MockConfigurationRepository {
    /// Internal storage for folders, protected by a mutex.
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
    /// Useful for setting up test data before a scenario.
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
        // Check for duplicates – if the folder ID already exists, return an error.
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
// Mock OperationRepository
// ============================================================================

/// In-memory implementation of `OperationRepository` for testing.
///
/// Stores operations in a `HashMap<OperationId, Operation>` for
/// fast lookup and supports all CRUD operations.
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
// Mock FileSystem
// ============================================================================

/// In-memory simulation of the file system for testing.
///
/// Uses a `HashMap<PathBuf, (bool, u64)>` where the key is the file path,
/// the boolean indicates existence, and the u64 is the file size.
/// Supports basic operations: exists, move, copy, delete, rename.
pub struct MockFileSystem {
    /// Simulated file system state: (exists, size_in_bytes)
    files: Mutex<HashMap<PathBuf, (bool, u64)>>,
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
        self.files.lock().unwrap().insert(path, (true, size));
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
        Ok(self.files.lock().unwrap().get(&path).map(|(exists, _)| *exists).unwrap_or(false))
    }

    async fn get_file_size(&self, path: &WindowsPath) -> Result<u64, UseCaseError> {
        let path = Self::to_pathbuf(path);
        let files = self.files.lock().unwrap();
        match files.get(&path) {
            Some((true, size)) => Ok(*size),
            _ => Err(UseCaseError::FileNotFound(format!("File not found: {}", path.display()))),
        }
    }

    async fn move_file(&self, from: &WindowsPath, to: &WindowsPath) -> Result<u64, UseCaseError> {
        let from_path = Self::to_pathbuf(from);
        let to_path = Self::to_pathbuf(to);
        let mut files = self.files.lock().unwrap();

        // Check that the source exists and retrieve its size
        let (_, size) = files.get(&from_path)
            .filter(|(exists, _)| *exists)
            .ok_or_else(|| UseCaseError::FileNotFound(format!("Source not found: {}", from_path.display())))?;
        let size = *size;

        // Remove the source entry and create the destination
        files.remove(&from_path);
        files.insert(to_path, (true, size));

        Ok(size)
    }

    async fn copy_file(&self, from: &WindowsPath, to: &WindowsPath) -> Result<u64, UseCaseError> {
        let from_path = Self::to_pathbuf(from);
        let to_path = Self::to_pathbuf(to);
        let mut files = self.files.lock().unwrap();

        // Check that the source exists and retrieve its size
        let (_, size) = files.get(&from_path)
            .filter(|(exists, _)| *exists)
            .ok_or_else(|| UseCaseError::FileNotFound(format!("Source not found: {}", from_path.display())))?;
        let size = *size;

        // Create the copy at the destination
        files.insert(to_path, (true, size));

        Ok(size)
    }

    async fn delete_file(&self, path: &WindowsPath) -> Result<(), UseCaseError> {
        let path = Self::to_pathbuf(path);
        let mut files = self.files.lock().unwrap();
        if !files.contains_key(&path) {
            return Err(UseCaseError::FileNotFound(format!("File not found: {}", path.display())));
        }
        files.remove(&path);
        Ok(())
    }

    async fn rename_file(&self, from: &WindowsPath, to: &WindowsPath) -> Result<(), UseCaseError> {
        let from_path = Self::to_pathbuf(from);
        let to_path = Self::to_pathbuf(to);
        let mut files = self.files.lock().unwrap();

        // Check that the source exists
        let (_, size) = files.get(&from_path)
            .filter(|(exists, _)| *exists)
            .ok_or_else(|| UseCaseError::FileNotFound(format!("Source not found: {}", from_path.display())))?;
        let size = *size;

        // Remove the old entry and create the new one
        files.remove(&from_path);
        files.insert(to_path, (true, size));

        Ok(())
    }
}

// ============================================================================
// Mock IdGenerator
// ============================================================================

/// A predictable ID generator for testing.
///
/// Returns incrementing IDs of the form `test-op-001`, `test-op-002`, etc.
/// This makes test assertions deterministic.
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
// Mock Clock
// ============================================================================

/// A controllable clock for testing.
///
/// The time can be set manually, allowing tests to verify timestamp-based
/// logic (e.g., `created_at`, `updated_at`).
pub struct MockClock {
    /// The current time, settable by the test.
    now: Mutex<chrono::DateTime<chrono::Utc>>,
}

impl MockClock {
    /// Creates a new clock with a fixed timestamp.
    pub fn new(now: chrono::DateTime<chrono::Utc>) -> Self {
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
    pub fn set(&self, now: chrono::DateTime<chrono::Utc>) {
        *self.now.lock().unwrap() = now;
    }
}

impl Clock for MockClock {
    fn now(&self) -> chrono::DateTime<chrono::Utc> {
        *self.now.lock().unwrap()
    }
}

// ============================================================================
// Mock ConflictResolver
// ============================================================================

/// A simple conflict resolver that always returns the command unchanged.
///
/// For tests that need specific conflict resolution behavior,
/// create a custom mock or use a conditional resolver.
pub struct MockConflictResolver;

#[async_trait]
impl ConflictResolver for MockConflictResolver {
    async fn resolve(&self, command: OperationCommand) -> Result<OperationCommand, UseCaseError> {
        // By default, just return the command unchanged.
        // Tests can override this behavior by replacing the mock.
        Ok(command)
    }
}
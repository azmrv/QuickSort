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

use async_trait::async_trait;
use quicksort_application::UseCaseError;
use quicksort_domain::errors::DomainError;
use quicksort_domain::{
    AbsolutePath, DuplicateCheckMode, DuplicateCheckResult, Folder, FolderId, Operation,
    OperationId,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub use quicksort_application::ports::outbound::{
    Clock, ConfigurationRepository, DuplicateDetectionPort, FileSystem, IdGenerator,
    OperationRepository,
};

// ============================================================================
// Mock ConfigurationRepository
// ============================================================================

/// In-memory implementation of `ConfigurationRepository` for testing.
///
/// Allows pre-loading of folder data and captures save operations
/// for verification.
#[derive(Clone)]
pub struct MockConfigurationRepository {
    /// Internal storage for folders, protected by a mutex.
    folders: Arc<Mutex<Vec<Folder>>>,
}

impl MockConfigurationRepository {
    /// Creates a new empty repository.
    pub fn new() -> Self {
        Self {
            folders: Arc::new(Mutex::new(Vec::new())),
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
        Ok(folders
            .iter()
            .find(|f| f.path.to_string_lossy() == path)
            .cloned())
    }

    async fn get_default_folder_id(&self) -> Result<FolderId, UseCaseError> {
        let folders = self.folders.lock().unwrap();
        folders
            .first()
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
#[derive(Clone)]
pub struct MockOperationRepository {
    operations: Arc<Mutex<HashMap<String, Operation>>>,
}

impl MockOperationRepository {
    /// Creates a new empty repository.
    pub fn new() -> Self {
        Self {
            operations: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Pre-loads an operation for testing.
    pub fn set_operation(&self, operation: Operation) {
        self.operations
            .lock()
            .unwrap()
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
        Ok(self
            .operations
            .lock()
            .unwrap()
            .get(&id.to_string())
            .cloned())
    }

    async fn save(&self, operation: &Operation) -> Result<(), UseCaseError> {
        self.operations
            .lock()
            .unwrap()
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

    async fn clear(&self) -> Result<(), UseCaseError> {
        self.operations.lock().unwrap().clear();
        Ok(())
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
#[derive(Clone)]
pub struct MockFileSystem {
    /// Simulated file system state: (exists, size_in_bytes)
    files: Arc<Mutex<HashMap<PathBuf, (bool, u64)>>>,
}

impl MockFileSystem {
    /// Creates a new empty file system.
    pub fn new() -> Self {
        Self {
            files: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Pre-populates a file entry (used by tests to set up source files).
    pub fn add_file(&self, path: PathBuf, size: u64) {
        self.files.lock().unwrap().insert(path, (true, size));
    }

    /// Helper to convert `AbsolutePath` to `PathBuf` for internal storage.
    fn to_pathbuf(path: &AbsolutePath) -> PathBuf {
        PathBuf::from(path.to_string_lossy().as_ref())
    }
}

#[async_trait]
impl FileSystem for MockFileSystem {
    async fn exists(&self, path: &AbsolutePath) -> Result<bool, UseCaseError> {
        let path = Self::to_pathbuf(path);
        Ok(self
            .files
            .lock()
            .unwrap()
            .get(&path)
            .map(|(exists, _)| *exists)
            .unwrap_or(false))
    }

    async fn get_file_size(&self, path: &AbsolutePath) -> Result<u64, UseCaseError> {
        let path = Self::to_pathbuf(path);
        let files = self.files.lock().unwrap();
        match files.get(&path) {
            Some((true, size)) => Ok(*size),
            _ => Err(UseCaseError::FileNotFound(format!(
                "File not found: {}",
                path.display()
            ))),
        }
    }

    async fn move_file(&self, from: &AbsolutePath, to: &AbsolutePath) -> Result<u64, UseCaseError> {
        let from_path = Self::to_pathbuf(from);
        let to_path = Self::to_pathbuf(to);
        let mut files = self.files.lock().unwrap();

        // Check that the source exists and retrieve its size
        let (_, size) = files
            .get(&from_path)
            .filter(|(exists, _)| *exists)
            .ok_or_else(|| {
                UseCaseError::FileNotFound(format!("Source not found: {}", from_path.display()))
            })?;
        let size = *size;

        // Remove the source entry and create the destination
        files.remove(&from_path);
        files.insert(to_path, (true, size));

        Ok(size)
    }

    async fn copy_file(&self, from: &AbsolutePath, to: &AbsolutePath) -> Result<u64, UseCaseError> {
        let from_path = Self::to_pathbuf(from);
        let to_path = Self::to_pathbuf(to);
        let mut files = self.files.lock().unwrap();

        // Check that the source exists and retrieve its size
        let (_, size) = files
            .get(&from_path)
            .filter(|(exists, _)| *exists)
            .ok_or_else(|| {
                UseCaseError::FileNotFound(format!("Source not found: {}", from_path.display()))
            })?;
        let size = *size;

        // Create the copy at the destination
        files.insert(to_path, (true, size));

        Ok(size)
    }

    async fn is_dir(&self, _path: &AbsolutePath) -> Result<bool, UseCaseError> {
        // The in-memory mock tracks only files; every registered entry is
        // treated as a regular file.
        Ok(false)
    }

    async fn copy_tree(&self, from: &AbsolutePath, to: &AbsolutePath) -> Result<u64, UseCaseError> {
        self.copy_file(from, to).await
    }

    async fn move_tree(&self, from: &AbsolutePath, to: &AbsolutePath) -> Result<u64, UseCaseError> {
        self.move_file(from, to).await
    }

    async fn delete_file(&self, path: &AbsolutePath) -> Result<(), UseCaseError> {
        let path = Self::to_pathbuf(path);
        let mut files = self.files.lock().unwrap();
        if !files.contains_key(&path) {
            return Err(UseCaseError::FileNotFound(format!(
                "File not found: {}",
                path.display()
            )));
        }
        files.remove(&path);
        Ok(())
    }

    async fn rename_file(
        &self,
        from: &AbsolutePath,
        to: &AbsolutePath,
    ) -> Result<(), UseCaseError> {
        let from_path = Self::to_pathbuf(from);
        let to_path = Self::to_pathbuf(to);
        let mut files = self.files.lock().unwrap();

        // Check that the source exists
        let (_, size) = files
            .get(&from_path)
            .filter(|(exists, _)| *exists)
            .ok_or_else(|| {
                UseCaseError::FileNotFound(format!("Source not found: {}", from_path.display()))
            })?;
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

/// A random ID generator for testing.
///
/// Returns a fresh `OperationId` on every call, mirroring the real
/// generator. Tests that need to inspect the operation id use the value
/// returned by the use case.
#[derive(Clone, Default)]
pub struct MockIdGenerator;

impl MockIdGenerator {
    /// Creates a new generator.
    pub fn new() -> Self {
        Self
    }
}

impl IdGenerator for MockIdGenerator {
    fn generate(&self) -> OperationId {
        OperationId::new()
    }
}

// ============================================================================
// Mock Clock
// ============================================================================

/// A controllable clock for testing.
///
/// The time can be set manually, allowing tests to verify timestamp-based
/// logic (e.g., `created_at`, `updated_at`).
#[derive(Clone)]
pub struct MockClock {
    /// The current time, settable by the test.
    now: Arc<Mutex<chrono::DateTime<chrono::Utc>>>,
}

impl MockClock {
    /// Creates a new clock with a fixed timestamp.
    pub fn new(now: chrono::DateTime<chrono::Utc>) -> Self {
        Self {
            now: Arc::new(Mutex::new(now)),
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
// Mock DuplicateDetector
// ============================================================================

/// A duplicate detector that never reports a duplicate.
///
/// The use case treats the real destination `exists()` check as
/// authoritative for conflict resolution (the duplicate detector result
/// is informational), so returning `exists: false` is safe for every
/// overwrite-policy scenario exercised by the tests.
#[derive(Clone, Default)]
pub struct MockDuplicateDetector;

#[async_trait]
impl DuplicateDetectionPort for MockDuplicateDetector {
    async fn check_duplicate(
        &self,
        source: &AbsolutePath,
        destination: &AbsolutePath,
        mode: &DuplicateCheckMode,
    ) -> Result<DuplicateCheckResult, DomainError> {
        Ok(DuplicateCheckResult {
            source: source.clone(),
            destination: destination.clone(),
            exists: false,
            mode: mode.clone(),
        })
    }
}

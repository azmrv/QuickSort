//! Test doubles (mocks/stubs) for all outbound ports.
//!
//! These are used in the executable specifications to test Use Cases
//! in isolation without touching real infrastructure.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use async_trait::async_trait;

use quicksort_domain::{
    Folder, FolderId, Operation, OperationId, OperationType,
    WindowsPath, DomainEvent,
};
use quicksort_application::ports::outbound::{
    ConfigurationRepository, OperationRepository, FileSystem,
    IdGenerator, Clock, ConflictResolver,
};
use quicksort_application::errors::UseCaseError;

// ============================================================================
// Mock: ConfigurationRepository
// ============================================================================

#[derive(Default, Clone)]
pub struct MockConfigurationRepository {
    pub folders: Arc<Mutex<Vec<Folder>>>,
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
        self.folders.lock().unwrap().push(folder);
        Ok(())
    }

    async fn remove(&self, id: &FolderId) -> Result<(), UseCaseError> {
        let mut folders = self.folders.lock().unwrap();
        folders.retain(|f| f.id != *id);
        Ok(())
    }

    async fn find_by_id(&self, id: &FolderId) -> Result<Option<Folder>, UseCaseError> {
        Ok(self.folders.lock().unwrap().iter().find(|f| f.id == *id).cloned())
    }

    async fn find_by_path(&self, path: &str) -> Result<Option<Folder>, UseCaseError> {
        Ok(self.folders.lock().unwrap().iter().find(|f| f.path.as_str() == path).cloned())
    }
}

// ============================================================================
// Mock: OperationRepository
// ============================================================================

#[derive(Default, Clone)]
pub struct MockOperationRepository {
    pub storage: Arc<Mutex<HashMap<String, Operation>>>,
}

#[async_trait]
impl OperationRepository for MockOperationRepository {
    async fn find_by_id(&self, id: &OperationId) -> Result<Option<Operation>, UseCaseError> {
        let storage = self.storage.lock().unwrap();
        Ok(storage.get(id.as_str()).cloned())
    }

    async fn save(&self, operation: &Operation) -> Result<(), UseCaseError> {
        let mut storage = self.storage.lock().unwrap();
        storage.insert(operation.id.as_str().to_string(), operation.clone());
        Ok(())
    }

    async fn delete(&self, id: &OperationId) -> Result<(), UseCaseError> {
        let mut storage = self.storage.lock().unwrap();
        storage.remove(id.as_str());
        Ok(())
    }

    async fn load_all(&self) -> Result<Vec<Operation>, UseCaseError> {
        let storage = self.storage.lock().unwrap();
        Ok(storage.values().cloned().collect())
    }
}

// ============================================================================
// Mock: FileSystem
// ============================================================================

#[derive(Clone)]
pub struct MockFileSystem {
    pub files: Arc<Mutex<HashMap<String, u64>>>,
    pub should_fail: Arc<Mutex<Option<UseCaseError>>>,
}

impl MockFileSystem {
    pub fn new() -> Self {
        Self {
            files: Arc::new(Mutex::new(HashMap::new())),
            should_fail: Arc::new(Mutex::new(None)),
        }
    }

    /// Creates a file in the virtual file system with the given size.
    pub fn create_file(&self, path: &WindowsPath, size: u64) {
        self.files.lock().unwrap().insert(path.as_str().to_string(), size);
    }

    /// Sets the next operation to fail with a specific error.
    pub fn set_next_error(&self, error: UseCaseError) {
        *self.should_fail.lock().unwrap() = Some(error);
    }

    /// Clears the error state.
    pub fn clear_error(&self) {
        *self.should_fail.lock().unwrap() = None;
    }

    /// Returns true if a file exists at the given path.
    pub fn file_exists(&self, path: &WindowsPath) -> bool {
        self.files.lock().unwrap().contains_key(path.as_str())
    }
}

#[async_trait]
impl FileSystem for MockFileSystem {
    async fn exists(&self, path: &WindowsPath) -> Result<bool, UseCaseError> {
        if let Some(err) = self.should_fail.lock().unwrap().as_ref() {
            return Err(err.clone());
        }
        Ok(self.files.lock().unwrap().contains_key(path.as_str()))
    }

    async fn move_file(&self, from: &WindowsPath, to: &WindowsPath) -> Result<u64, UseCaseError> {
        if let Some(err) = self.should_fail.lock().unwrap().as_ref() {
            return Err(err.clone());
        }
        let mut files = self.files.lock().unwrap();
        let size = files.remove(from.as_str())
            .ok_or_else(|| UseCaseError::FileNotFound(from.as_str().to_string()))?;
        files.insert(to.as_str().to_string(), size);
        Ok(size)
    }

    async fn copy_file(&self, from: &WindowsPath, to: &WindowsPath) -> Result<u64, UseCaseError> {
        if let Some(err) = self.should_fail.lock().unwrap().as_ref() {
            return Err(err.clone());
        }
        let files = self.files.lock().unwrap();
        let size = files.get(from.as_str())
            .ok_or_else(|| UseCaseError::FileNotFound(from.as_str().to_string()))?;
        // Release lock before inserting new file
        let size_clone = *size;
        drop(files);
        self.files.lock().unwrap().insert(to.as_str().to_string(), size_clone);
        Ok(size_clone)
    }

    async fn delete_file(&self, path: &WindowsPath) -> Result<(), UseCaseError> {
        if let Some(err) = self.should_fail.lock().unwrap().as_ref() {
            return Err(err.clone());
        }
        let mut files = self.files.lock().unwrap();
        files.remove(path.as_str())
            .ok_or_else(|| UseCaseError::FileNotFound(path.as_str().to_string()))?;
        Ok(())
    }

    async fn rename_file(&self, from: &WindowsPath, to: &WindowsPath) -> Result<(), UseCaseError> {
        if let Some(err) = self.should_fail.lock().unwrap().as_ref() {
            return Err(err.clone());
        }
        let mut files = self.files.lock().unwrap();
        let size = files.remove(from.as_str())
            .ok_or_else(|| UseCaseError::FileNotFound(from.as_str().to_string()))?;
        files.insert(to.as_str().to_string(), size);
        Ok(())
    }
}

impl Default for MockFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Stub: IdGenerator
// ============================================================================

#[derive(Clone)]
pub struct StubIdGenerator {
    pub next_id: String,
}

impl StubIdGenerator {
    pub fn new(next_id: impl Into<String>) -> Self {
        Self { next_id: next_id.into() }
    }
}

impl IdGenerator for StubIdGenerator {
    fn generate(&self) -> String {
        self.next_id.clone()
    }
}

// ============================================================================
// Stub: Clock (fixed time)
// ============================================================================

#[derive(Clone)]
pub struct FrozenClock {
    pub time: SystemTime,
}

impl FrozenClock {
    pub fn new(time: SystemTime) -> Self {
        Self { time }
    }
}

impl Clock for FrozenClock {
    fn now(&self) -> SystemTime {
        self.time
    }
}

// ============================================================================
// Stub: ConflictResolver (auto-rename)
// ============================================================================

#[derive(Clone)]
pub struct StubConflictResolver;

#[async_trait]
impl ConflictResolver for StubConflictResolver {
    async fn resolve(&self, command: OperationCommand, folders: &[Folder]) -> Result<OperationCommand, ConflictResolutionError> {
        // For tests, we just auto-rename by appending "_copy"
        // In real implementation, this would be more sophisticated
        Ok(command)
    }
}


// ============================================================================
// Helper: Build a default folder for tests
// ============================================================================

pub fn test_folder() -> Folder {
    Folder::new(
        FolderId::from_string("folder-1"),
        "Documents".to_string(),
        WindowsPath::new("C:\\Users\\Test\\Documents").unwrap(),
    )
}

pub fn test_file(path: &str) -> WindowsPath {
    WindowsPath::new(path).unwrap()
}
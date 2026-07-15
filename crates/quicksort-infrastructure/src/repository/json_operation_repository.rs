//! JSON implementation of OperationRepository.
//! Stores operation history in a JSON file for audit and undo support.

use std::fs;
use std::path::{Path, PathBuf};
use async_trait::async_trait;

use quicksort_domain::{
    Operation, OperationId, OperationType, OperationState, WindowsPath,
};
use quicksort_application::ports::outbound::OperationRepository;
use quicksort_application::errors::UseCaseError;

/// Repository that persists operations to a JSON file.
///
/// The file contains an array of `Operation` objects serialized as JSON.
/// It is read on startup and written whenever an operation is saved or deleted.
pub struct JsonOperationRepository {
    file_path: PathBuf,
}

impl JsonOperationRepository {
    /// Creates a new repository backed by the given file path.
    ///
    /// If the parent directory does not exist, it is created.
    pub fn new(file_path: impl Into<PathBuf>) -> Self {
        let path = file_path.into();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).ok();
        }
        Self { file_path: path }
    }

    /// Loads all operations from the JSON file.
    fn load_from_file(&self) -> Result<Vec<Operation>, UseCaseError> {
        if !self.file_path.exists() {
            return Ok(vec![]);
        }
        let content = fs::read_to_string(&self.file_path)
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?;
        if content.trim().is_empty() {
            return Ok(vec![]);
        }
        let operations: Vec<Operation> = serde_json::from_str(&content)
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?;
        Ok(operations)
    }

    /// Saves all operations to the JSON file.
    fn save_to_file(&self, operations: &[Operation]) -> Result<(), UseCaseError> {
        let json = serde_json::to_string_pretty(operations)
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?;
        fs::write(&self.file_path, json)
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?;
        Ok(())
    }
}

#[async_trait]
impl OperationRepository for JsonOperationRepository {
    async fn find_by_id(&self, id: &OperationId) -> Result<Option<Operation>, UseCaseError> {
        let operations = self.load_from_file()?;
        Ok(operations.into_iter().find(|op| op.id == *id))
    }

    async fn save(&self, operation: &Operation) -> Result<(), UseCaseError> {
        let mut operations = self.load_from_file()?;
        // Replace existing operation with the same ID, or insert new one.
        if let Some(existing) = operations.iter_mut().find(|op| op.id == operation.id) {
            *existing = operation.clone();
        } else {
            operations.push(operation.clone());
        }
        self.save_to_file(&operations)
    }

    async fn delete(&self, id: &OperationId) -> Result<(), UseCaseError> {
        let mut operations = self.load_from_file()?;
        operations.retain(|op| op.id != *id);
        self.save_to_file(&operations)
    }

    async fn load_all(&self) -> Result<Vec<Operation>, UseCaseError> {
        self.load_from_file()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use chrono::Utc;

    fn test_path(path: &str) -> WindowsPath {
        WindowsPath::new(path).unwrap()
    }

    #[test]
    fn test_load_empty_repository() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("operations.json");
        let repo = JsonOperationRepository::new(file_path);

        let operations = repo.load_from_file().unwrap();
        assert!(operations.is_empty());
    }

    #[test]
    fn test_save_and_load_operation() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("operations.json");
        let repo = JsonOperationRepository::new(file_path);

        // Create a test operation using the new domain API
        let mut op = Operation::new_move(
            vec![test_path("C:\\src.txt")],
            test_path("C:\\dst.txt"),
            Utc::now(),
        );
        op.start().unwrap();
        op.complete(1, 1024).unwrap();

        // Save it
        repo.save(&op).unwrap();

        // Load all operations and verify
        let operations = repo.load_all().unwrap();
        assert_eq!(operations.len(), 1);
        assert_eq!(operations[0].id, op.id);
        assert!(matches!(operations[0].state, OperationState::Completed { .. }));
    }

    #[test]
    fn test_delete_operation() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("operations.json");
        let repo = JsonOperationRepository::new(file_path);

        // Create and save two operations
        let op1 = Operation::new_move(
            vec![test_path("C:\\a.txt")],
            test_path("C:\\b.txt"),
            Utc::now(),
        );
        let op2 = Operation::new_copy(
            vec![test_path("C:\\c.txt")],
            test_path("C:\\d.txt"),
            Utc::now(),
        );
        repo.save(&op1).unwrap();
        repo.save(&op2).unwrap();

        // Delete the first one
        repo.delete(&op1.id).unwrap();

        // Verify only the second remains
        let operations = repo.load_all().unwrap();
        assert_eq!(operations.len(), 1);
        assert_eq!(operations[0].id, op2.id);
    }

    #[test]
    fn test_find_by_id() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("operations.json");
        let repo = JsonOperationRepository::new(file_path);

        let op = Operation::new_delete(
            vec![test_path("C:\\trash.txt")],
            Utc::now(),
        );
        repo.save(&op).unwrap();

        let found = repo.find_by_id(&op.id).unwrap().unwrap();
        assert_eq!(found.id, op.id);

        let not_found = repo.find_by_id(&OperationId::new()).unwrap();
        assert!(not_found.is_none());
    }
}
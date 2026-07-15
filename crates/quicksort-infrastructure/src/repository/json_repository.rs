//! JSON-backed repository for persisting operation history.
//!
//! Implements the `OperationRepository` outbound port defined in
//! `quicksort-application`.  Stores operations as a JSON array in
//! `%APPDATA%\QuickSort\operations.json`.

use std::path::PathBuf;
use async_trait::async_trait;

use quicksort_domain::Operation;
use quicksort_domain::value_objects::OperationId;
use quicksort_application::ports::outbound::OperationRepository;
use quicksort_application::errors::UseCaseError;

/// A persistent, JSON-file-based implementation of `OperationRepository`.
///
/// The file is stored at a fixed location derived from `%APPDATA%`.
/// All methods are synchronous (the JSON file is small enough that
/// blocking I/O is acceptable) but declared `async` to satisfy the port.
pub struct JsonOperationRepository {
    path: PathBuf,
}

impl JsonOperationRepository {
    /// Creates a new repository.  The file will be stored in
    /// `%APPDATA%\QuickSort\operations.json`.
    // OLD: used `directories::ProjectDirs` – removed to reduce dependency
    // footprint.  The path is now directly derived from `APPDATA` for
    // consistency with other configuration files.
    pub fn new() -> Result<Self, UseCaseError> {
        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
        let dir = PathBuf::from(&appdata).join("QuickSort");
        std::fs::create_dir_all(&dir)
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?;
        Ok(Self {
            path: dir.join("operations.json"),
        })
    }

    // ---- private helpers ----

    fn read_all(&self) -> Result<Vec<Operation>, UseCaseError> {
        if !self.path.exists() {
            return Ok(vec![]);
        }
        let data = std::fs::read_to_string(&self.path)
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?;
        if data.trim().is_empty() {
            return Ok(vec![]);
        }
        serde_json::from_str(&data)
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))
    }

    fn write_all(&self, operations: &[Operation]) -> Result<(), UseCaseError> {
        let json = serde_json::to_string_pretty(operations)
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?;
        std::fs::write(&self.path, json)
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))
    }
}

#[async_trait]
impl OperationRepository for JsonOperationRepository {
    /// Finds an operation by its unique identifier.
    async fn find_by_id(&self, id: &OperationId) -> Result<Option<Operation>, UseCaseError> {
        let ops = self.read_all()?;
        Ok(ops.into_iter().find(|o| o.id == *id))
    }

    /// Saves (inserts or updates) an operation.
    async fn save(&self, operation: &Operation) -> Result<(), UseCaseError> {
        let mut ops = self.read_all()?;
        // OLD: checked `o.id == operation.id`
        // NEW: `OperationId` supports equality, so the comparison is unchanged.
        if let Some(existing) = ops.iter_mut().find(|o| o.id == operation.id) {
            *existing = operation.clone();
        } else {
            ops.push(operation.clone());
        }
        self.write_all(&ops)
    }

    /// Deletes an operation by ID.  If the operation does not exist,
    /// this is a no-op.
    async fn delete(&self, id: &OperationId) -> Result<(), UseCaseError> {
        let mut ops = self.read_all()?;
        ops.retain(|o| o.id != *id);
        self.write_all(&ops)
    }

    /// Loads all stored operations.
    async fn load_all(&self) -> Result<Vec<Operation>, UseCaseError> {
        self.read_all()
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
    use quicksort_domain::value_objects::WindowsPath;

    fn test_path(path: &str) -> WindowsPath {
        WindowsPath::new(path).unwrap()
    }

    #[test]
    fn test_empty_repository() {
        let temp_dir = tempdir().unwrap();
        let repo = JsonOperationRepository {
            path: temp_dir.path().join("ops.json"),
        };
        assert!(repo.load_all().unwrap().is_empty());
    }

    #[test]
    fn test_save_and_find() {
        let temp_dir = tempdir().unwrap();
        let repo = JsonOperationRepository {
            path: temp_dir.path().join("ops.json"),
        };

        let mut op = Operation::new_move(
            vec![test_path("C:\\src.txt")],
            test_path("C:\\dst.txt"),
            Utc::now(),
        );
        op.start().unwrap();
        op.complete(1, 1024).unwrap();

        repo.save(&op).unwrap();
        let found = repo.find_by_id(&op.id).unwrap().unwrap();
        assert_eq!(found.id, op.id);
        assert!(matches!(found.state, OperationState::Completed { .. }));
    }

    #[test]
    fn test_delete() {
        let temp_dir = tempdir().unwrap();
        let repo = JsonOperationRepository {
            path: temp_dir.path().join("ops.json"),
        };

        let op = Operation::new_delete(
            vec![test_path("C:\\trash.txt")],
            Utc::now(),
        );
        repo.save(&op).unwrap();
        assert_eq!(repo.load_all().unwrap().len(), 1);

        repo.delete(&op.id).unwrap();
        assert_eq!(repo.load_all().unwrap().len(), 0);
    }
}
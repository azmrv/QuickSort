//! In-memory implementation of OperationRepository for testing.
//!
//! Stores operations in a `HashMap<String, Operation>` keyed by the
//! string representation of `OperationId`.  Not intended for production use.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;

use quicksort_domain::{Operation, OperationId};
use quicksort_application::ports::outbound::OperationRepository;
use quicksort_application::errors::UseCaseError;

/// In-memory repository for operation history.
///
/// All data is lost when the process exits.  Use `JsonOperationRepository`
/// for persistent storage.
pub struct InMemoryOperationRepository {
    storage: Arc<Mutex<HashMap<String, Operation>>>,
}

impl InMemoryOperationRepository {
    /// Creates a new empty repository.
    pub fn new() -> Self {
        Self {
            storage: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl OperationRepository for InMemoryOperationRepository {
    async fn find_by_id(&self, id: &OperationId) -> Result<Option<Operation>, UseCaseError> {
        // OLD: storage.get(id.as_str()).cloned()
        // NEW: OperationId::to_string() returns the UUID string
        let storage = self.storage.lock().unwrap();
        Ok(storage.get(&id.to_string()).cloned())
    }

    async fn save(&self, operation: &Operation) -> Result<(), UseCaseError> {
        let mut storage = self.storage.lock().unwrap();
        // OLD: operation.id.as_str() – OperationId does not have as_str()
        // NEW: use to_string() via the Display implementation
        storage.insert(operation.id.to_string(), operation.clone());
        Ok(())
    }

    async fn delete(&self, id: &OperationId) -> Result<(), UseCaseError> {
        let mut storage = self.storage.lock().unwrap();
        storage.remove(&id.to_string());
        Ok(())
    }

    async fn load_all(&self) -> Result<Vec<Operation>, UseCaseError> {
        let storage = self.storage.lock().unwrap();
        Ok(storage.values().cloned().collect())
    }
}
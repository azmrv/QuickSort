use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;

use quicksort_domain::{Operation, OperationId};
use quicksort_application::ports::outbound::OperationRepository;
use quicksort_application::errors::UseCaseError;

pub struct InMemoryOperationRepository {
    storage: Arc<Mutex<HashMap<String, Operation>>>,
}

impl InMemoryOperationRepository {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl OperationRepository for InMemoryOperationRepository {
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
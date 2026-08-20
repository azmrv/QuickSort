// mocks/operation_repository_mock.rs
use async_trait::async_trait;
use crate::application::ports::OperationRepositoryPort;
use crate::domain::{Operation, OperationId, DomainError};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug)]
pub struct OperationRepositoryMock {
    // Simulated database/storage
    store: Mutex<HashMap<OperationId, Operation>>,
}

impl OperationRepositoryMock {
    pub fn new() -> Self {
        OperationRepositoryMock {
            store: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl OperationRepositoryPort for OperationRepositoryMock {
    async fn get_operation(&self, id: &OperationId) -> Result<Operation, DomainError> {
        let store = self.store.lock().unwrap();
        match store.get(id) {
            Some(op) => Ok(op.clone()),
            None => Err(DomainError::OperationNotFound(format!("Operation ID {} not found", id.as_str()))),
        }
    }

    async fn save(&self, operation: &Operation) -> Result<(), DomainError> {
        let mut store = self.store.lock().unwrap();
        store.insert(operation.id.clone(), operation.clone());
        println!("MOCK: Operation {} saved to repository.", operation.id.as_str());
        Ok(())
    }
}
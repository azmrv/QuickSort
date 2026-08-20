// mocks/conflict_resolver_mock.rs
use async_trait::async_trait;
use crate::application::ports::ConflictResolverPort;
use crate::domain::{Operation, DomainError};

#[derive(Debug)]
pub struct ConflictResolverMock;

impl ConflictResolverMock {
    pub fn new() -> Self {
        ConflictResolverMock {}
    }
}

#[async_trait]
impl ConflictResolverPort for ConflictResolverMock {
    async fn resolve(&self, operation: &Operation) -> Result<Option<String>, DomainError> {
        // Mock: Assume no conflict for now. In a real scenario, this checks target file existence/locks.
        println!("MOCK: Conflict Resolver checking operation ID: {}", operation.id.as_str());
        Ok(Ok(None)) // Return None if no conflict is found
    }
}
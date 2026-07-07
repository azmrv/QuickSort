//! Outbound port for generating unique identifiers.

pub trait IdGenerator: Send + Sync {
    fn generate(&self) -> String;
}
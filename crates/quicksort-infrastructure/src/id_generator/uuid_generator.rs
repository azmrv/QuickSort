use uuid::Uuid;
use quicksort_domain::OperationId;
use quicksort_application::ports::outbound::IdGenerator;

/// Generates operation IDs using UUID v4.
pub struct UuidGenerator;

impl IdGenerator for UuidGenerator {
    // OLD: returned `String`
    // NEW: returns `OperationId` as required by the updated port
    fn generate(&self) -> OperationId {
        OperationId::from_uuid(Uuid::new_v4())
    }
}
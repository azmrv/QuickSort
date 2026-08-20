use quicksort_application::ports::outbound::IdGenerator;
use quicksort_domain::OperationId;
use uuid::Uuid;

/// Generates operation IDs using UUID v4.
pub struct UuidGenerator;

impl IdGenerator for UuidGenerator {
    // returns `OperationId` as required by the updated port
    fn generate(&self) -> OperationId {
        OperationId::from_uuid(Uuid::new_v4())
    }
}

use uuid::Uuid;
use quicksort_application::ports::outbound::IdGenerator;

pub struct UuidGenerator;

impl IdGenerator for UuidGenerator {
    fn generate(&self) -> String {
        Uuid::new_v4().to_string()
    }
}
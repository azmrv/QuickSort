use quicksort_application::ports::outbound::FileSystem;
use quicksort_application::ApplicationFacadeImpl;
use std::sync::Arc;

pub struct AppState {
    pub facade: Arc<ApplicationFacadeImpl>,
    pub queue: Arc<crate::queue::JobQueue>,
    pub fs: Arc<dyn FileSystem>,
}

use std::sync::Arc;
use quicksort_application::ApplicationFacadeImpl;

pub struct AppState {
    pub facade: Arc<ApplicationFacadeImpl>,
}

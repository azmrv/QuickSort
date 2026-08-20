use quicksort_application::ApplicationFacadeImpl;
use std::sync::Arc;

pub struct AppState {
    pub facade: Arc<ApplicationFacadeImpl>,
}

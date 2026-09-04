use std::sync::{Arc, Mutex};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::mocks::MockConfigurationRepository;
use quicksort_application::GetFolders;
use quicksort_application::use_cases::GetFoldersUseCase;

struct SpanRecorder {
    names: Mutex<Vec<String>>,
}

impl SpanRecorder {
    fn new() -> Self {
        Self {
            names: Mutex::new(Vec::new()),
        }
    }

    fn has_span(&self, name: &str) -> bool {
        self.names.lock().unwrap().iter().any(|n| n == name)
    }
}

struct RecordLayer {
    recorder: Arc<SpanRecorder>,
}

impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for RecordLayer {
    fn on_new_span(
        &self,
        attrs: &tracing::span::Attributes<'_>,
        _id: &tracing::span::Id,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut visitor = SpanFieldVisitor(String::new());
        attrs.record(&mut visitor);

        let name = attrs.metadata().name().to_string();
        self.recorder.names.lock().unwrap().push(name);
    }
}

struct SpanFieldVisitor(String);

impl tracing::field::Visit for SpanFieldVisitor {
    fn record_debug(&mut self, _field: &tracing::field::Field, _value: &dyn std::fmt::Debug) {}
}

#[tokio::test]
async fn get_folders_emits_tracing_span() {
    let recorder = Arc::new(SpanRecorder::new());
    let layer = RecordLayer {
        recorder: Arc::clone(&recorder),
    };

    let subscriber = tracing_subscriber::registry()
        .with(layer)
        .with(tracing_subscriber::fmt::layer().with_test_writer());

    let _guard = subscriber.set_default();

    let mock_repo = MockConfigurationRepository::new();
    let use_case = GetFoldersUseCase::new(Arc::new(mock_repo));

    let _ = use_case.get_all().await;

    assert!(
        recorder.has_span("get_folders"),
        "Expected a tracing span named 'get_folders' on GetFoldersUseCase::get_all(). \
         This means #[instrument(skip_all, fields(folder_count))] is missing from the method. \
         Recorded spans: {:?}",
        recorder.names.lock().unwrap()
    );
}

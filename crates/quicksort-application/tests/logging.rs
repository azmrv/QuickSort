//! Standalone integration test binary for ADR-019 Structured Logging.
//!
//! This file is separate from `mod.rs` (which includes `execute_operation` tests
//! that have pre-existing compilation errors related to a changed mock API).
//! By being its own test binary, logging tests can compile and run independently.
//!
//! The shared mock implementations are pulled in via `#[path = "mocks.rs"]` so
//! there is exactly ONE source of mock truth — `tests/mocks.rs`.

#[path = "mocks.rs"]
mod mocks;

use std::fs;
use std::sync::{Arc, Mutex, OnceLock};

use mocks::MockConfigurationRepository;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

// ===========================================================================
// Test 1: File output — logs appear in daily-rotated file
// ===========================================================================

static GUARD: OnceLock<tracing_appender::non_blocking::WorkerGuard> = OnceLock::new();

#[test]
fn logs_appear_in_daily_rotated_file() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("quicksort-test")
        .build(dir.path())
        .expect("create rolling file appender");

    let (non_blocking, guard) = tracing_appender::non_blocking(appender);
    // Store guard so it is never dropped (keeps writer alive).
    let _ = GUARD.set(guard);

    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_writer(non_blocking)
        .with_env_filter("info")
        .finish();

    let _guard = tracing::subscriber::set_default(subscriber);

    tracing::info!("test_log_marker_xyz_123");

    // Flush: drop the subscriber so the non_blocking writer flushes.
    drop(_guard);

    let entries: Vec<_> = fs::read_dir(dir.path())
        .expect("read temp dir")
        .filter_map(|e| e.ok())
        .collect();

    assert!(
        !entries.is_empty(),
        "Expected at least one log file in {:?}",
        dir.path()
    );

    let content = fs::read_to_string(&entries[0].path()).expect("read log file");
    assert!(
        content.contains("test_log_marker_xyz_123"),
        "Log file should contain the emitted message. Content: {}",
        content
    );
}

// ===========================================================================
// Test 2: #[instrument] span on GetFoldersUseCase::get_all()
// ===========================================================================

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
        let name = attrs.metadata().name().to_string();
        self.recorder.names.lock().unwrap().push(name);
    }
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
    let use_case = quicksort_application::use_cases::GetFoldersUseCase::new(Arc::new(mock_repo));

    // Bring the GetFolders trait into scope so `get_all()` is callable.
    use quicksort_application::GetFolders;
    let _ = use_case.get_all().await;

    assert!(
        recorder.has_span("get_folders"),
        "Expected a tracing span named 'get_folders' on GetFoldersUseCase::get_all(). \
         This means #[instrument(skip_all)] is missing from the method. \
         Recorded spans: {:?}",
        recorder.names.lock().unwrap()
    );
}

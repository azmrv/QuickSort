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

use std::collections::HashMap;
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

#[derive(Default)]
struct SpanData {
    name: String,
    creation_fields: HashMap<String, String>,
    recorded_fields: HashMap<String, String>,
}

struct SpanRecorder {
    spans: Mutex<HashMap<u64, SpanData>>,
}

impl SpanRecorder {
    fn new() -> Self {
        Self {
            spans: Mutex::new(HashMap::new()),
        }
    }

    fn has_span(&self, name: &str) -> bool {
        self.spans
            .lock()
            .unwrap()
            .values()
            .any(|span| span.name == name)
    }
}

struct SpanVisitor(HashMap<String, String>);

impl tracing::field::Visit for SpanVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.0.insert(field.name().to_string(), format!("{value:?}"));
    }
}

struct RecordLayer {
    recorder: Arc<SpanRecorder>,
}

impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for RecordLayer {
    fn on_new_span(
        &self,
        attrs: &tracing::span::Attributes<'_>,
        id: &tracing::span::Id,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut visitor = SpanVisitor(HashMap::new());
        attrs.values().record(&mut visitor);

        let mut spans = self.recorder.spans.lock().unwrap();
        spans.insert(
            id.into_u64(),
            SpanData {
                name: attrs.metadata().name().to_string(),
                creation_fields: visitor.0,
                recorded_fields: HashMap::new(),
            },
        );
    }

    fn on_record(
        &self,
        id: &tracing::span::Id,
        values: &tracing::span::Record<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut visitor = SpanVisitor(HashMap::new());
        values.record(&mut visitor);

        let mut spans = self.recorder.spans.lock().unwrap();
        if let Some(span) = spans.get_mut(&id.into_u64()) {
            span.recorded_fields.extend(visitor.0);
        }
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
        recorder
            .spans
            .lock()
            .unwrap()
            .values()
            .map(|s| s.name.clone())
            .collect::<Vec<_>>()
    );
}

// ===========================================================================
// Test 3: #[instrument] span with fields on SearchFilesUseCase::search()
// ===========================================================================

#[tokio::test]
async fn search_files_emits_tracing_span() {
    let recorder = Arc::new(SpanRecorder::new());
    let layer = RecordLayer {
        recorder: Arc::clone(&recorder),
    };

    let subscriber = tracing_subscriber::registry()
        .with(layer)
        .with(tracing_subscriber::fmt::layer().with_test_writer());

    let _guard = subscriber.set_default();

    let mock_search = mocks::MockFileSearchPort::new();
    mock_search.set_result(mocks::SearchResult {
        files: vec![mocks::FileSearchResult {
            path: "C:\\test\\file.txt".to_string(),
            name: "file.txt".to_string(),
            size: 128,
            is_directory: false,
            modified_at: Some(1_700_000_000),
        }],
        total_count: 1,
        search_time_ms: 4,
        truncated: false,
    });

    let use_case =
        quicksort_application::use_cases::SearchFilesUseCase::new(Arc::new(mock_search.clone()));

    // Bring the SearchFiles trait into scope so `search()` is callable.
    use quicksort_application::SearchFiles;
    let result = use_case
        .search("report", &["C:\\test".to_string()])
        .await;

    assert!(result.is_ok());

    assert_eq!(
        mock_search.last_query(),
        "report",
        "SearchFilesUseCase should forward the query text to FileSearchPort"
    );
    assert_eq!(
        mock_search.last_directories(),
        vec!["C:\\test".to_string()],
        "SearchFilesUseCase should forward the root directories to FileSearchPort"
    );

    let spans = recorder.spans.lock().unwrap();
    let span = spans.values().find(|s| s.name == "search_files");
    assert!(
        span.is_some(),
        "Expected a tracing span named 'search_files' on SearchFilesUseCase::search(). \
         This means #[instrument(skip_all)] is missing from the method. \
         Recorded spans: {:?}",
        spans.values().map(|s| s.name.clone()).collect::<Vec<_>>()
    );

    let span = span.expect("search_files span");
    assert_eq!(
        span.creation_fields.get("query_text").map(String::as_str),
        Some("report"),
        "search_files span should carry the query_text creation field"
    );
    assert_eq!(
        span.creation_fields.get("directory_count").map(String::as_str),
        Some("1"),
        "search_files span should carry the directory_count creation field"
    );
    assert_eq!(
        span.recorded_fields.get("result_count").map(String::as_str),
        Some("1"),
        "search_files span should record the result_count field"
    );
    assert!(
        span.recorded_fields.contains_key("duration_ms"),
        "search_files span should record the duration_ms field"
    );
}

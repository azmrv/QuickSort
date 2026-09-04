use std::fs;
use std::sync::OnceLock;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt;

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
    GUARD.set(guard).expect("set guard once");

    let subscriber = fmt::Subscriber::builder()
        .with_writer(non_blocking)
        .with_env_filter("info")
        .finish();

    let _guard = tracing::subscriber::set_default(subscriber);

    tracing::info!("test_log_marker_xyz_123");

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

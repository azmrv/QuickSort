use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};
use tauri::Emitter;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

static APP_HANDLE: OnceLock<Mutex<Option<tauri::AppHandle>>> = OnceLock::new();

// Rolling buffer of recent backend log entries served to the frontend via the
// `get_logs` command, so the LOG tab shows history from app start — not only
// events emitted after the page mounted.
static LOG_BUFFER: Mutex<VecDeque<serde_json::Value>> = Mutex::new(VecDeque::new());

const MAX_BUFFERED_LOGS: usize = 500;

static _FILE_GUARD: OnceLock<tracing_appender::non_blocking::WorkerGuard> = OnceLock::new();

struct FrontendLayer;

impl<S> tracing_subscriber::Layer<S> for FrontendLayer
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let meta = event.metadata();
        let level = match *meta.level() {
            tracing::Level::ERROR => "ERROR",
            tracing::Level::WARN => "WARN",
            tracing::Level::INFO => "INFO",
            tracing::Level::DEBUG => "DEBUG",
            tracing::Level::TRACE => "TRACE",
        };

        let target = meta.target();
        let mut visitor = FieldVisitor(String::new());
        event.record(&mut visitor);

        let log_entry = serde_json::json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "level": level,
            "target": target,
            "message": visitor.0,
        });

        if let Ok(mut buffer) = LOG_BUFFER.lock() {
            buffer.push_back(log_entry.clone());
            while buffer.len() > MAX_BUFFERED_LOGS {
                buffer.pop_front();
            }
        }

        if let Some(handle) = APP_HANDLE.get() {
            if let Ok(guard) = handle.lock() {
                if let Some(ref h) = *guard {
                    let _ = h.emit("backend-log", &log_entry);
                }
            }
        }
    }
}

/// Returns recent buffered backend log entries (oldest first), used by the
/// `get_logs` Tauri command so the LOG tab can show history from app start.
pub fn get_recent_logs() -> Vec<serde_json::Value> {
    if let Ok(buffer) = LOG_BUFFER.lock() {
        buffer.iter().cloned().collect()
    } else {
        Vec::new()
    }
}

struct FieldVisitor(String);

impl tracing::field::Visit for FieldVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if !self.0.is_empty() {
            self.0.push(' ');
        }
        self.0.push_str(&format!("{}={:?}", field.name(), value));
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if !self.0.is_empty() {
            self.0.push(' ');
        }
        self.0.push_str(&format!("{}={}", field.name(), value));
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        if !self.0.is_empty() {
            self.0.push(' ');
        }
        self.0.push_str(&format!("{}={}", field.name(), value));
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        if !self.0.is_empty() {
            self.0.push(' ');
        }
        self.0.push_str(&format!("{}={}", field.name(), value));
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        if !self.0.is_empty() {
            self.0.push(' ');
        }
        self.0.push_str(&format!("{}={}", field.name(), value));
    }
}

pub fn init() {
    let env_filter = tracing_subscriber::filter::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::filter::EnvFilter::new("info"));

    let log_dir = crate::platform::paths::config_dir().join("logs");
    let _ = std::fs::create_dir_all(&log_dir);

    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("quicksort")
        .build(&log_dir)
        .expect("create rolling file appender");

    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    let _ = _FILE_GUARD.set(guard);

    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .with(file_layer)
        .with(FrontendLayer)
        .init();
}

pub fn set_app_handle(handle: tauri::AppHandle) {
    let _ = APP_HANDLE.set(Mutex::new(Some(handle)));
}

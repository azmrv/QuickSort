//! Tauri adapter for progress reporting.
//!
//! Emits `operation-progress` events to the frontend during long-running
//! operations. Uses the same global handle pattern as `logging.rs`.
//!
//! Emission is throttled: at most one event per 500ms, unless the processed
//! fraction moved by at least 1% since the last emitted tick. This keeps the
//! IPC churn low during fast file loops while staying smooth on slow ones.

use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use tauri::Emitter;

use quicksort_application::ports::outbound::{ProgressInfo, ProgressReporter};

const THROTTLE_INTERVAL: Duration = Duration::from_millis(500);
const THROTTLE_PERCENT_STEP: f64 = 1.0;

static APP_HANDLE: OnceLock<Mutex<Option<tauri::AppHandle>>> = OnceLock::new();

pub fn set_app_handle(handle: tauri::AppHandle) {
    let _ = APP_HANDLE.set(Mutex::new(Some(handle)));
}

/// Pure throttling decision, extracted for unit testing.
/// `now` is injected so tests can simulate the passage of time.
#[derive(Debug, Clone, Copy)]
struct ThrottleState {
    last_emit: Option<Instant>,
    last_percent: f64,
}

impl ThrottleState {
    /// Decide whether the given tick should be emitted. A tick with
    /// `current == 0` starts a new operation and resets the throttle.
    fn should_emit(&mut self, current: u32, total: u32, now: Instant) -> bool {
        if current == 0 {
            self.last_emit = None;
            self.last_percent = 0.0;
        }
        let percent = if total > 0 {
            ((current as f64 / total as f64) * 100.0).min(100.0)
        } else {
            100.0
        };
        // Always emit the final tick (the use case reports current == total
        // with phase "complete") so the UI/log reach 100% deterministically
        // even when the last step falls inside the throttle window.
        let is_final = total > 0 && current >= total;
        let time_ok = match self.last_emit {
            None => true,
            Some(last) => now.saturating_duration_since(last) >= THROTTLE_INTERVAL,
        };
        let delta_ok = (percent - self.last_percent).abs() >= THROTTLE_PERCENT_STEP;
        if is_final || time_ok || delta_ok {
            self.last_emit = Some(now);
            self.last_percent = percent;
            return true;
        }
        false
    }
}

/// Tauri-based progress reporter that emits throttled events to the frontend.
///
/// NOTE: this single reporter instance is shared across all queued
/// operations (the app queue serializes background work via its lock).
/// If concurrent job execution is ever enabled, the throttle state must be
/// split per operation, otherwise a `current == 0` from one operation would
/// reset another's baseline mid-flight.
pub struct TauriProgressReporter {
    state: Mutex<ThrottleState>,
}

impl TauriProgressReporter {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(ThrottleState {
                last_emit: None,
                last_percent: 0.0,
            }),
        }
    }
}

#[async_trait::async_trait]
impl ProgressReporter for TauriProgressReporter {
    async fn report(&self, progress: ProgressInfo) {
        let should_emit = {
            let Ok(mut state) = self.state.lock() else {
                return;
            };
            state.should_emit(progress.current, progress.total, Instant::now())
        };
        if !should_emit {
            return;
        }
        if let Some(handle) = APP_HANDLE.get() {
            if let Ok(guard) = handle.lock() {
                if let Some(ref h) = *guard {
                    let _ = h.emit("operation-progress", &progress);
                }
            }
        }
        // Log every emitted tick so the LOG tab exposes operation progress
        // during long copies (re-QA 12.09.2026 D4). Bounded by the throttle
        // above; one line per emitted tick.
        tracing::info!(
            current = progress.current,
            total = progress.total,
            phase = %progress.phase,
            detail = ?progress.detail,
            "operation progress"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn anchor() -> Instant {
        Instant::now()
    }

    #[test]
    fn emits_on_cold_start() {
        let mut state = ThrottleState {
            last_emit: None,
            last_percent: 0.0,
        };
        assert!(state.should_emit(0, 100, anchor()));
    }

    #[test]
    fn skips_recent_unchanged_tick() {
        let mut state = ThrottleState {
            last_emit: None,
            last_percent: 0.0,
        };
        let t0 = anchor();
        assert!(state.should_emit(10, 100, t0));
        assert!(!state.should_emit(10, 100, t0 + Duration::from_millis(100)));
    }

    #[test]
    fn emits_once_the_interval_passed() {
        let mut state = ThrottleState {
            last_emit: None,
            last_percent: 0.0,
        };
        let t0 = anchor();
        assert!(state.should_emit(10, 100, t0));
        assert!(state.should_emit(10, 100, t0 + Duration::from_millis(500)));
    }

    #[test]
    fn always_emits_final_tick() {
        let mut state = ThrottleState {
            last_emit: None,
            last_percent: 0.0,
        };
        let t0 = anchor();
        assert!(state.should_emit(99, 100, t0));
        // 99 -> 100 within 100ms: delta is 1% and no interval has passed,
        // but the final tick must still be emitted.
        assert!(state.should_emit(100, 100, t0 + Duration::from_millis(100)));
    }

    #[test]
    fn emits_on_percent_step_within_interval() {
        let mut state = ThrottleState {
            last_emit: None,
            last_percent: 0.0,
        };
        let t0 = anchor();
        assert!(state.should_emit(10, 100, t0));
        // 10% delta but only 100ms passed — percent gate still triggers.
        assert!(state.should_emit(20, 100, t0 + Duration::from_millis(100)));
    }

    #[test]
    fn new_operation_resets_throttle_state() {
        let mut state = ThrottleState {
            last_emit: Some(anchor()),
            last_percent: 96.0,
        };
        let t0 = anchor();
        // Fresh operation: baseline resets to 0%, so a small tick must NOT emit.
        assert!(state.should_emit(0, 200, t0));
        assert!(!state.should_emit(1, 200, t0 + Duration::from_millis(100)));
        // ...but a big jump still does.
        assert!(state.should_emit(20, 200, t0 + Duration::from_millis(200)));
    }

    #[test]
    fn percent_is_clamped_to_one_hundred() {
        let mut state = ThrottleState {
            last_emit: None,
            last_percent: 0.0,
        };
        let t0 = anchor();
        assert!(state.should_emit(120, 100, t0));
    }
}

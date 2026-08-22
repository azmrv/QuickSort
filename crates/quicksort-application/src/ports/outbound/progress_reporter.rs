//! Outbound port for progress reporting during long-running operations.
//!
//! This port defines the interface for reporting progress to the user
//! during operations like duplicate checking or batch file processing.
//! The Application Layer defines what progress information it needs to report,
//! and the Infrastructure/Adapter Layer provides the concrete implementation
//! (e.g., emitting Tauri events to the frontend).

use async_trait::async_trait;

/// Progress information for a long-running operation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProgressInfo {
    /// Current step (0-based).
    pub current: u32,
    /// Total number of steps.
    pub total: u32,
    /// Human-readable description of the current phase.
    pub phase: String,
    /// Optional detail message (e.g., filename being processed).
    pub detail: Option<String>,
}

/// Port for reporting progress during long-running operations.
///
/// Implementations should emit progress events to the frontend
/// (e.g., via Tauri events, WebSocket, or callback).
#[async_trait]
pub trait ProgressReporter: Send + Sync {
    /// Report progress for the current phase of an operation.
    ///
    /// Implementations should be non-blocking and best-effort:
    /// if the frontend is not listening, progress events can be dropped.
    async fn report(&self, progress: ProgressInfo);
}

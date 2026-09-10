use parking_lot::Mutex;
use quicksort_application::ports::outbound::FileSystem;
use quicksort_application::ApplicationFacadeImpl;
use std::sync::Arc;

/// Cumulative disk/network counters from the previous poll, stored in managed
/// state so successive `get_system_info` calls can compute per-second deltas.
#[derive(Debug, Clone, Default)]
pub struct SystemInfoSample {
    /// Cumulative disk read bytes at the previous poll.
    pub disk_read: u64,
    /// Cumulative disk write bytes at the previous poll.
    pub disk_write: u64,
    /// Cumulative network receive bytes at the previous poll.
    pub network_rx: u64,
    /// Cumulative network transmit bytes at the previous poll.
    pub network_tx: u64,
    /// Wall-clock millis of the previous poll.
    pub timestamp_ms: u64,
}

pub struct AppState {
    pub facade: Arc<ApplicationFacadeImpl>,
    pub queue: Arc<crate::queue::JobQueue>,
    pub fs: Arc<dyn FileSystem>,
    /// Previous disk/network counters used to derive per-second throughput
    /// deltas in `get_system_info`.
    pub system_sample: Mutex<SystemInfoSample>,
}

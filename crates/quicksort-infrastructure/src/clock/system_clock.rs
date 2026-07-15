//! SystemClock - реализация временной метки на основе системного времени.

use std::time::{SystemTime, UNIX_EPOCH};
use quicksort_infrastructure_contract::timestamp::Timestamp;

pub struct SystemClock;

impl Timestamp for SystemClock {
    fn current_timestamp(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
    }
}
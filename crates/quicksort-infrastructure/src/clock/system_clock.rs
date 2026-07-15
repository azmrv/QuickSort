//! SystemClock – implementation of the Clock port using the real system time.

use chrono::{DateTime, Utc};
use quicksort_application::ports::outbound::Clock;

/// Returns the current UTC time.
pub struct SystemClock;

impl Clock for SystemClock {
    // OLD: returned a Unix timestamp (u64)
    // NEW: returns DateTime<Utc> as required by the updated port
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

// OLD: use std::time::{SystemTime, UNIX_EPOCH};
// OLD: use quicksort_infrastructure_contract::timestamp::Timestamp;
// The old implementation was based on a non-existent trait.  The new
// implementation matches the `Clock` port from `quicksort-application`.
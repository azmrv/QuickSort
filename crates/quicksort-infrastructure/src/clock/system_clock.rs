//! SystemClock – implementation of the Clock port using the real system time.

use chrono::{DateTime, Utc};
use quicksort_application::ports::outbound::Clock;

/// Returns the current UTC time.
pub struct SystemClock;

impl Clock for SystemClock {
    // returns DateTime<Utc> as required by the updated port
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

// The old implementation was based on a non-existent trait.  The new
// implementation matches the `Clock` port from `quicksort-application`.

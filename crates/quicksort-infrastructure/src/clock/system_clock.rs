use std::time::SystemTime;
use quicksort_application::ports::outbound::Clock;

pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> SystemTime {
        SystemTime::now()
    }
}
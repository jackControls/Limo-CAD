//! Windows PowerShell startup and Add-Type compilation precede the bounded
//! input operation. Slow hosted startup must not consume the input deadline.
use std::time::{Duration, Instant};

pub(super) struct HelperDeadline {
    deadline: Instant,
    starting: bool,
}

impl HelperDeadline {
    pub(super) fn new(now: Instant, windows: bool) -> Self {
        Self {
            deadline: now + Duration::from_secs(if windows { 60 } else { 20 }),
            starting: windows,
        }
    }

    pub(super) fn ready(&mut self, now: Instant) {
        if self.starting {
            self.deadline = now + Duration::from_secs(20);
            self.starting = false;
        }
    }

    pub(super) fn expired(&self, now: Instant) -> Option<&'static str> {
        (now >= self.deadline).then_some(if self.starting {
            "60 seconds during Windows helper initialization"
        } else {
            "20 seconds during input"
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cold_windows_startup_does_not_consume_or_extend_the_input_budget() {
        let start = Instant::now();
        let mut deadline = HelperDeadline::new(start, true);
        assert!(deadline.expired(start + Duration::from_secs(35)).is_none());
        deadline.ready(start + Duration::from_secs(35));
        assert!(deadline.expired(start + Duration::from_secs(54)).is_none());
        // Polling a persistent ready signal must not keep extending the limit.
        deadline.ready(start + Duration::from_secs(54));
        assert_eq!(
            deadline.expired(start + Duration::from_secs(55)),
            Some("20 seconds during input")
        );
    }

    #[test]
    fn stalled_windows_initialization_and_other_platform_input_remain_bounded() {
        let start = Instant::now();
        let windows = HelperDeadline::new(start, true);
        assert!(windows.expired(start + Duration::from_secs(59)).is_none());
        assert_eq!(
            windows.expired(start + Duration::from_secs(60)),
            Some("60 seconds during Windows helper initialization")
        );
        let mut other = HelperDeadline::new(start, false);
        other.ready(start + Duration::from_secs(19));
        assert_eq!(
            other.expired(start + Duration::from_secs(20)),
            Some("20 seconds during input")
        );
    }
}

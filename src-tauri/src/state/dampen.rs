//! Event dampening — 2-second sliding window dedup.
//!
//! Prevents the "toxic loop strobe" failure mode (Gemini review).
//! `Severity::Critical` bypasses dampening unconditionally.

use super::canonical::Severity;
use crate::event::schema::EventType;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

pub struct Dampener {
    window: Duration,
    capacity: usize,
    recent: VecDeque<(EventType, Severity, Instant)>,
}

impl Dampener {
    pub fn new(window_ms: u64) -> Self {
        Self {
            window: Duration::from_millis(window_ms),
            capacity: 20,
            recent: VecDeque::with_capacity(20),
        }
    }

    /// Returns `true` when the event should proceed down the pipeline.
    /// Returns `false` when it should be silently dropped (duplicate).
    pub fn observe(&mut self, event_type: EventType, severity: Severity, now: Instant) -> bool {
        if severity == Severity::Critical {
            self.record(event_type, severity, now);
            return true;
        }
        self.prune(now);
        let duplicate = self
            .recent
            .iter()
            .any(|(t, s, _)| *t == event_type && *s == severity);
        if duplicate {
            false
        } else {
            self.record(event_type, severity, now);
            true
        }
    }

    fn prune(&mut self, now: Instant) {
        while let Some(&(_, _, ts)) = self.recent.front() {
            if now.duration_since(ts) > self.window {
                self.recent.pop_front();
            } else {
                break;
            }
        }
    }

    fn record(&mut self, t: EventType, s: Severity, now: Instant) {
        if self.recent.len() >= self.capacity {
            self.recent.pop_front();
        }
        self.recent.push_back((t, s, now));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dedups_identical_within_window() {
        let mut d = Dampener::new(2000);
        let t0 = Instant::now();
        assert!(d.observe(EventType::ToolComplete, Severity::Info, t0));
        assert!(!d.observe(
            EventType::ToolComplete,
            Severity::Info,
            t0 + Duration::from_millis(500)
        ));
    }

    #[test]
    fn passes_after_window() {
        let mut d = Dampener::new(2000);
        let t0 = Instant::now();
        assert!(d.observe(EventType::ToolComplete, Severity::Info, t0));
        assert!(d.observe(
            EventType::ToolComplete,
            Severity::Info,
            t0 + Duration::from_millis(2100)
        ));
    }

    #[test]
    fn critical_bypasses_dampening() {
        let mut d = Dampener::new(2000);
        let t0 = Instant::now();
        assert!(d.observe(EventType::DestructiveOpDetected, Severity::Critical, t0));
        assert!(d.observe(
            EventType::DestructiveOpDetected,
            Severity::Critical,
            t0 + Duration::from_millis(100)
        ));
    }

    #[test]
    fn different_severity_not_deduped() {
        let mut d = Dampener::new(2000);
        let t0 = Instant::now();
        assert!(d.observe(EventType::ToolComplete, Severity::Info, t0));
        assert!(d.observe(
            EventType::ToolComplete,
            Severity::Warning,
            t0 + Duration::from_millis(100)
        ));
    }
}

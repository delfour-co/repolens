//! Timing utilities for measuring and formatting durations
//!
//! This module provides utilities for measuring elapsed time and formatting
//! durations in a human-readable way.

use std::time::{Duration, Instant};

/// A simple timer for measuring elapsed time
#[derive(Debug, Clone)]
pub struct Timer {
    start: Instant,
}

impl Timer {
    /// Create a new timer that starts immediately
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// Get the elapsed duration since the timer started
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Get the elapsed time formatted as a human-readable string
    #[allow(dead_code)]
    pub fn elapsed_formatted(&self) -> String {
        format_duration(self.elapsed())
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::start()
    }
}

/// Format a duration into a human-readable string
///
/// # Examples
///
/// - Durations >= 1 second: "1.23s"
/// - Durations >= 1 millisecond: "456ms"
/// - Durations < 1 millisecond: "< 1ms"
///
/// # Arguments
///
/// * `duration` - The duration to format
///
/// # Returns
///
/// A formatted string representation of the duration
pub fn format_duration(duration: Duration) -> String {
    let millis = duration.as_millis();

    if millis == 0 {
        "< 1ms".to_string()
    } else if millis >= 1000 {
        let secs = duration.as_secs_f64();
        format!("{:.2}s", secs)
    } else {
        format!("{}ms", millis)
    }
}

/// Timing information for a single category execution
#[derive(Debug, Clone)]
pub struct CategoryTiming {
    /// Name of the category
    pub name: String,
    /// Number of rules executed in this category (reserved for future use)
    #[allow(dead_code)]
    pub rule_count: usize,
    /// Number of findings produced
    pub findings_count: usize,
    /// Duration of execution
    pub duration: Duration,
}

impl CategoryTiming {
    /// Create a new category timing record
    pub fn new(name: &str, rule_count: usize, findings_count: usize, duration: Duration) -> Self {
        Self {
            name: name.to_string(),
            rule_count,
            findings_count,
            duration,
        }
    }

    /// Get the duration formatted as a human-readable string
    pub fn duration_formatted(&self) -> String {
        format_duration(self.duration)
    }
}

/// Collection of timing information for an entire audit run
#[derive(Debug, Clone, Default)]
pub struct AuditTiming {
    /// Timing for each category
    pub categories: Vec<CategoryTiming>,
    /// Total duration of the audit
    pub total_duration: Duration,
}

impl AuditTiming {
    /// Create a new audit timing collection
    pub fn new() -> Self {
        Self {
            categories: Vec::new(),
            total_duration: Duration::ZERO,
        }
    }

    /// Add a category timing record
    pub fn add_category(&mut self, timing: CategoryTiming) {
        self.categories.push(timing);
    }

    /// Set the total audit duration
    pub fn set_total_duration(&mut self, duration: Duration) {
        self.total_duration = duration;
    }

    /// Get the total duration formatted as a human-readable string
    pub fn total_duration_formatted(&self) -> String {
        format_duration(self.total_duration)
    }

    /// Get all category timings
    pub fn categories(&self) -> &[CategoryTiming] {
        &self.categories
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration_less_than_1ms() {
        let duration = Duration::from_micros(500);
        assert_eq!(format_duration(duration), "< 1ms");

        let duration = Duration::from_nanos(100);
        assert_eq!(format_duration(duration), "< 1ms");

        let duration = Duration::ZERO;
        assert_eq!(format_duration(duration), "< 1ms");
    }

    #[test]
    fn test_format_duration_milliseconds() {
        let duration = Duration::from_millis(1);
        assert_eq!(format_duration(duration), "1ms");

        let duration = Duration::from_millis(456);
        assert_eq!(format_duration(duration), "456ms");

        let duration = Duration::from_millis(999);
        assert_eq!(format_duration(duration), "999ms");
    }

    #[test]
    fn test_format_duration_seconds() {
        let duration = Duration::from_millis(1000);
        assert_eq!(format_duration(duration), "1.00s");

        let duration = Duration::from_millis(1234);
        assert_eq!(format_duration(duration), "1.23s");

        let duration = Duration::from_secs(2);
        assert_eq!(format_duration(duration), "2.00s");

        let duration = Duration::from_secs_f64(3.456);
        assert_eq!(format_duration(duration), "3.46s");

        let duration = Duration::from_secs(60);
        assert_eq!(format_duration(duration), "60.00s");
    }

    #[test]
    fn test_timer_basic() {
        let timer = Timer::start();
        std::thread::sleep(Duration::from_millis(10));
        let elapsed = timer.elapsed();
        assert!(elapsed >= Duration::from_millis(10));
    }

    #[test]
    fn test_timer_elapsed_formatted() {
        let timer = Timer::start();
        std::thread::sleep(Duration::from_millis(50));
        let formatted = timer.elapsed_formatted();
        // Should be something like "50ms" or close to it
        assert!(
            formatted.ends_with("ms") || formatted.ends_with('s'),
            "Expected formatted duration, got: {}",
            formatted
        );
    }

    #[test]
    fn test_timer_default() {
        let timer = Timer::default();
        let elapsed = timer.elapsed();
        assert!(elapsed < Duration::from_secs(1));
    }

    #[test]
    fn test_category_timing() {
        let duration = Duration::from_millis(123);
        let timing = CategoryTiming::new("secrets", 12, 3, duration);

        assert_eq!(timing.name, "secrets");
        assert_eq!(timing.rule_count, 12);
        assert_eq!(timing.findings_count, 3);
        assert_eq!(timing.duration, duration);
        assert_eq!(timing.duration_formatted(), "123ms");
    }

    #[test]
    fn test_audit_timing() {
        let mut audit_timing = AuditTiming::new();

        audit_timing.add_category(CategoryTiming::new(
            "secrets",
            10,
            2,
            Duration::from_millis(45),
        ));
        audit_timing.add_category(CategoryTiming::new(
            "files",
            8,
            1,
            Duration::from_millis(23),
        ));
        audit_timing.set_total_duration(Duration::from_millis(1234));

        assert_eq!(audit_timing.categories().len(), 2);
        assert_eq!(audit_timing.total_duration_formatted(), "1.23s");
    }

    #[test]
    fn test_audit_timing_default() {
        let timing = AuditTiming::default();
        assert!(timing.categories.is_empty());
        assert_eq!(timing.total_duration, Duration::ZERO);
    }
}

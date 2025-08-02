//! Memory profiling and optimization utilities for libpostal-rs

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Memory usage statistics for tracking allocations
#[derive(Debug, Clone, Default)]
pub struct MemoryStats {
    /// Peak memory usage in bytes
    pub peak_memory_bytes: usize,
    /// Current memory usage in bytes
    pub current_memory_bytes: usize,
    /// Total number of allocations
    pub total_allocations: usize,
    /// Total number of deallocations
    pub total_deallocations: usize,
    /// Number of active allocations
    pub active_allocations: usize,
}

/// Simple memory tracker for profiling libpostal-rs usage
#[derive(Debug)]
pub struct MemoryTracker {
    stats: Arc<MemoryStats>,
    start_time: Instant,
}

impl MemoryTracker {
    /// Create a new memory tracker
    pub fn new() -> Self {
        Self {
            stats: Arc::new(MemoryStats::default()),
            start_time: Instant::now(),
        }
    }

    /// Get current memory statistics
    pub fn stats(&self) -> MemoryStats {
        self.stats.as_ref().clone()
    }

    /// Get elapsed time since tracking started
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Sample current memory usage from system
    pub fn sample_system_memory(&self) -> Option<usize> {
        #[cfg(target_os = "linux")]
        {
            self.sample_linux_memory()
        }
        #[cfg(target_os = "macos")]
        {
            self.sample_macos_memory()
        }
        #[cfg(target_os = "windows")]
        {
            self.sample_windows_memory()
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            None
        }
    }

    #[cfg(target_os = "linux")]
    fn sample_linux_memory(&self) -> Option<usize> {
        let status = std::fs::read_to_string("/proc/self/status").ok()?;
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let kb = parts[1].parse::<usize>().ok()?;
                    return Some(kb * 1024); // Convert KB to bytes
                }
            }
        }
        None
    }

    #[cfg(target_os = "macos")]
    fn sample_macos_memory(&self) -> Option<usize> {
        // On macOS, we can use the mach API, but for simplicity we'll use a basic approach
        // In a real implementation, you'd use mach_task_basic_info
        None
    }

    #[cfg(target_os = "windows")]
    fn sample_windows_memory(&self) -> Option<usize> {
        // On Windows, we'd use GetProcessMemoryInfo
        // For now, return None as this requires additional dependencies
        None
    }
}

impl Default for MemoryTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance profiler for libpostal operations
#[derive(Debug)]
pub struct PerformanceProfiler {
    memory_tracker: MemoryTracker,
    operation_count: AtomicUsize,
    total_parse_time: AtomicUsize,     // microseconds
    total_normalize_time: AtomicUsize, // microseconds
}

impl PerformanceProfiler {
    /// Create a new performance profiler
    pub fn new() -> Self {
        Self {
            memory_tracker: MemoryTracker::new(),
            operation_count: AtomicUsize::new(0),
            total_parse_time: AtomicUsize::new(0),
            total_normalize_time: AtomicUsize::new(0),
        }
    }

    /// Record a parsing operation
    pub fn record_parse_operation(&self, duration: Duration) {
        self.operation_count.fetch_add(1, Ordering::Relaxed);
        self.total_parse_time
            .fetch_add(duration.as_micros() as usize, Ordering::Relaxed);
    }

    /// Record a normalization operation
    pub fn record_normalize_operation(&self, duration: Duration) {
        self.operation_count.fetch_add(1, Ordering::Relaxed);
        self.total_normalize_time
            .fetch_add(duration.as_micros() as usize, Ordering::Relaxed);
    }

    /// Get performance summary
    pub fn summary(&self) -> PerformanceSummary {
        let operation_count = self.operation_count.load(Ordering::Relaxed);
        let total_parse_time_us = self.total_parse_time.load(Ordering::Relaxed);
        let total_normalize_time_us = self.total_normalize_time.load(Ordering::Relaxed);

        PerformanceSummary {
            total_operations: operation_count,
            total_runtime: self.memory_tracker.elapsed(),
            average_parse_time: if operation_count > 0 {
                Duration::from_micros((total_parse_time_us / operation_count) as u64)
            } else {
                Duration::ZERO
            },
            average_normalize_time: if operation_count > 0 {
                Duration::from_micros((total_normalize_time_us / operation_count) as u64)
            } else {
                Duration::ZERO
            },
            memory_stats: self.memory_tracker.stats(),
            current_memory_usage: self.memory_tracker.sample_system_memory(),
        }
    }

    /// Reset all counters
    pub fn reset(&self) {
        self.operation_count.store(0, Ordering::Relaxed);
        self.total_parse_time.store(0, Ordering::Relaxed);
        self.total_normalize_time.store(0, Ordering::Relaxed);
    }
}

impl Default for PerformanceProfiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of performance metrics
#[derive(Debug, Clone)]
pub struct PerformanceSummary {
    /// Total number of operations performed
    pub total_operations: usize,
    /// Total runtime of the profiler
    pub total_runtime: Duration,
    /// Average time per parse operation
    pub average_parse_time: Duration,
    /// Average time per normalize operation
    pub average_normalize_time: Duration,
    /// Memory usage statistics
    pub memory_stats: MemoryStats,
    /// Current system memory usage in bytes
    pub current_memory_usage: Option<usize>,
}

impl PerformanceSummary {
    /// Calculate operations per second
    pub fn operations_per_second(&self) -> f64 {
        if self.total_runtime.as_secs_f64() > 0.0 {
            self.total_operations as f64 / self.total_runtime.as_secs_f64()
        } else {
            0.0
        }
    }

    /// Format memory usage as human-readable string
    pub fn format_memory_usage(&self) -> String {
        if let Some(bytes) = self.current_memory_usage {
            format_bytes(bytes)
        } else {
            "Unknown".to_string()
        }
    }
}

/// Format bytes as human-readable string
pub fn format_bytes(bytes: usize) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_tracker() {
        let tracker = MemoryTracker::new();
        let stats = tracker.stats();

        // Should start with zero stats
        assert_eq!(stats.peak_memory_bytes, 0);
        assert_eq!(stats.current_memory_bytes, 0);
        assert_eq!(stats.total_allocations, 0);

        // Should have positive elapsed time
        std::thread::sleep(Duration::from_millis(1));
        assert!(tracker.elapsed() > Duration::ZERO);
    }

    #[test]
    fn test_performance_profiler() {
        let profiler = PerformanceProfiler::new();

        // Record some operations
        profiler.record_parse_operation(Duration::from_micros(100));
        profiler.record_normalize_operation(Duration::from_micros(50));

        let summary = profiler.summary();
        assert_eq!(summary.total_operations, 2);
        assert!(summary.average_parse_time > Duration::ZERO);
        assert!(summary.average_normalize_time > Duration::ZERO);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1536), "1.50 KB");
        assert_eq!(format_bytes(1048576), "1.00 MB");
        assert_eq!(format_bytes(1073741824), "1.00 GB");
    }
}

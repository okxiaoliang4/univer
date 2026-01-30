use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use sysinfo::{ProcessRefreshKind, RefreshKind, System};
use tokio::time::sleep;

/// Memory snapshot at a specific point in time
#[derive(Debug, Clone)]
pub struct MemorySnapshot {
    pub timestamp: Instant,
    pub rss_bytes: u64,
    pub label: String,
}

impl MemorySnapshot {
    /// Calculate memory growth since another snapshot
    pub fn growth_since(&self, baseline: &MemorySnapshot) -> i64 {
        self.rss_bytes as i64 - baseline.rss_bytes as i64
    }

    /// Format memory in human-readable form (MB)
    pub fn rss_mb(&self) -> f64 {
        self.rss_bytes as f64 / (1024.0 * 1024.0)
    }
}

/// Memory monitor that tracks RSS usage and prints periodic updates
pub struct MemoryMonitor {
    baseline: MemorySnapshot,
    snapshots: Vec<MemorySnapshot>,
    stop_signal: Arc<AtomicBool>,
}

impl MemoryMonitor {
    /// Create a new memory monitor and take initial snapshot
    pub fn new(label: &str) -> Self {
        let snapshot = Self::take_snapshot(label);
        println!("\n═══════════════════════════════════════════════════════");
        println!("🔍 Memory Monitor Started");
        println!("═══════════════════════════════════════════════════════");
        println!("📊 Baseline RSS: {:.2} MB", snapshot.rss_mb());
        println!("───────────────────────────────────────────────────────\n");

        Self {
            baseline: snapshot.clone(),
            snapshots: vec![snapshot],
            stop_signal: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Take a memory snapshot at current time
    fn take_snapshot(label: &str) -> MemorySnapshot {
        let mut system = System::new_all();
        system.refresh_all();

        let pid = sysinfo::get_current_pid().expect("Failed to get current PID");
        let process = system.process(pid).expect("Failed to get current process");

        MemorySnapshot {
            timestamp: Instant::now(),
            rss_bytes: process.memory(),
            label: label.to_string(),
        }
    }

    /// Record a memory snapshot with a label
    pub fn record(&mut self, label: &str) {
        let snapshot = Self::take_snapshot(label);
        let growth = snapshot.growth_since(&self.baseline);
        let growth_mb = growth as f64 / (1024.0 * 1024.0);

        println!(
            "📌 Checkpoint: {} | RSS: {:.2} MB | Growth: {:+.2} MB",
            label,
            snapshot.rss_mb(),
            growth_mb
        );

        self.snapshots.push(snapshot);
    }

    /// Start background task that prints RSS every second
    pub fn start_periodic_logging(&self) -> tokio::task::JoinHandle<()> {
        let stop_signal = self.stop_signal.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            let mut counter = 0u64;

            while !stop_signal.load(Ordering::Relaxed) {
                interval.tick().await;
                counter += 1;

                let snapshot = Self::take_snapshot(&format!("periodic_{}", counter));
                println!(
                    "⏱️  [{}s] RSS: {:.2} MB",
                    counter,
                    snapshot.rss_mb()
                );
            }
        })
    }

    /// Stop the periodic logging task
    pub fn stop_periodic_logging(&self) {
        self.stop_signal.store(true, Ordering::Relaxed);
    }

    /// Print a summary report of all snapshots
    pub fn print_summary(&self) {
        println!("\n═══════════════════════════════════════════════════════");
        println!("📊 Memory Monitor Summary");
        println!("═══════════════════════════════════════════════════════");

        let baseline_mb = self.baseline.rss_mb();
        println!("Baseline: {:.2} MB ({})", baseline_mb, self.baseline.label);
        println!("───────────────────────────────────────────────────────");

        for snapshot in &self.snapshots[1..] {
            let growth = snapshot.growth_since(&self.baseline);
            let growth_mb = growth as f64 / (1024.0 * 1024.0);
            let elapsed = snapshot.timestamp.duration_since(self.baseline.timestamp);

            println!(
                "  {} | RSS: {:.2} MB | Growth: {:+.2} MB | Elapsed: {:.2}s",
                snapshot.label,
                snapshot.rss_mb(),
                growth_mb,
                elapsed.as_secs_f64()
            );
        }

        if let Some(final_snapshot) = self.snapshots.last() {
            let total_growth = final_snapshot.growth_since(&self.baseline);
            let total_growth_mb = total_growth as f64 / (1024.0 * 1024.0);
            println!("───────────────────────────────────────────────────────");
            println!("Total Memory Growth: {:+.2} MB", total_growth_mb);
        }

        println!("═══════════════════════════════════════════════════════\n");
    }
}

/// Performance timer for measuring operation duration
pub struct PerfTimer {
    label: String,
    start: Instant,
}

impl PerfTimer {
    /// Start a new performance timer
    pub fn start(label: &str) -> Self {
        println!("⏱️  Starting: {}", label);
        Self {
            label: label.to_string(),
            start: Instant::now(),
        }
    }

    /// Stop the timer and print elapsed time
    pub fn stop(self) -> Duration {
        let elapsed = self.start.elapsed();
        println!(
            "✅ Completed: {} | Duration: {:.3}s",
            self.label,
            elapsed.as_secs_f64()
        );
        elapsed
    }

    /// Stop the timer with custom message
    pub fn stop_with_message(self, message: &str) -> Duration {
        let elapsed = self.start.elapsed();
        println!(
            "✅ Completed: {} | {} | Duration: {:.3}s",
            self.label,
            message,
            elapsed.as_secs_f64()
        );
        elapsed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_monitor() {
        let mut monitor = MemoryMonitor::new("test_start");

        // Allocate some memory
        let _data: Vec<u8> = vec![0; 10 * 1024 * 1024]; // 10 MB
        monitor.record("after_allocation");

        sleep(Duration::from_millis(100)).await;
        monitor.print_summary();
    }

    #[test]
    fn test_perf_timer() {
        let timer = PerfTimer::start("test_operation");
        std::thread::sleep(Duration::from_millis(100));
        let elapsed = timer.stop();
        assert!(elapsed.as_millis() >= 100);
    }
}

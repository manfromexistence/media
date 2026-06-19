//! Progress tracking for long-running media operations.
//!
//! Provides a thread-safe way to report progress from media
//! processing operations to UI components or CLI progress bars.

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

/// A callback type for progress updates.
pub type ProgressCallback = Arc<dyn Fn(f32, &str) + Send + Sync>;

/// Thread-safe progress tracker.
pub struct ProgressTracker {
    /// Current progress (0-10000, representing 0.00% to 100.00%).
    progress: AtomicU32,
    /// Progress callback.
    callback: ProgressCallback,
}

impl ProgressTracker {
    /// Create a new progress tracker with a callback.
    pub fn new<F>(callback: F) -> Self
    where
        F: Fn(f32, &str) + Send + Sync + 'static,
    {
        Self {
            progress: AtomicU32::new(0),
            callback: Arc::new(callback),
        }
    }

    /// Report progress (0.0 to 1.0) with a message.
    pub fn report(&self, progress: f32, message: &str) {
        let progress = progress.clamp(0.0, 1.0);
        let stored = (progress * 10000.0) as u32;
        self.progress.store(stored, Ordering::SeqCst);
        (self.callback)(progress, message);
    }

    /// Get the current progress (0.0 to 1.0).
    pub fn current(&self) -> f32 {
        self.progress.load(Ordering::SeqCst) as f32 / 10000.0
    }

    /// Check if the operation is complete.
    pub fn is_complete(&self) -> bool {
        self.current() >= 1.0
    }

    /// Reset the progress to zero.
    pub fn reset(&self) {
        self.progress.store(0, Ordering::SeqCst);
    }
}

/// A multi-stage progress tracker that aggregates progress from multiple stages.
///
/// Provides weighted progress tracking across multiple stages.
/// Available for CLI progress display integration.
#[allow(dead_code)] // Public API for CLI progress display integration
pub struct MultiStageProgress {
    /// Stage weights (for weighted progress calculation).
    weights: Vec<f32>,
    /// Stage progress values.
    progress: Vec<AtomicU32>,
    /// Overall callback.
    callback: Option<ProgressCallback>,
}

#[allow(dead_code)] // Public API for CLI progress display integration
impl MultiStageProgress {
    /// Create a new multi-stage progress tracker.
    pub fn new(stage_count: usize) -> Self {
        Self {
            weights: vec![1.0; stage_count],
            progress: (0..stage_count).map(|_| AtomicU32::new(0)).collect(),
            callback: None,
        }
    }

    /// Create with custom weights for each stage.
    pub fn with_weights(weights: Vec<f32>) -> Self {
        let stage_count = weights.len();
        Self {
            weights,
            progress: (0..stage_count).map(|_| AtomicU32::new(0)).collect(),
            callback: None,
        }
    }

    /// Set the overall progress callback.
    pub fn with_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(f32, &str) + Send + Sync + 'static,
    {
        self.callback = Some(Arc::new(callback));
        self
    }

    /// Report progress for a specific stage.
    pub fn report_stage(&self, stage: usize, progress: f32, message: &str) {
        if stage >= self.progress.len() {
            return;
        }

        let progress = progress.clamp(0.0, 1.0);
        let stored = (progress * 10000.0) as u32;
        self.progress[stage].store(stored, Ordering::SeqCst);

        if let Some(callback) = &self.callback {
            let overall = self.overall_progress();
            callback(overall, message);
        }
    }

    /// Get the overall progress (weighted average).
    pub fn overall_progress(&self) -> f32 {
        let total_weight: f32 = self.weights.iter().sum();
        if total_weight == 0.0 {
            return 0.0;
        }

        let weighted_sum: f32 = self
            .progress
            .iter()
            .zip(self.weights.iter())
            .map(|(p, w)| {
                let progress = p.load(Ordering::SeqCst) as f32 / 10000.0;
                progress * w
            })
            .sum();

        weighted_sum / total_weight
    }

    /// Check if all stages are complete.
    pub fn is_complete(&self) -> bool {
        self.progress
            .iter()
            .all(|p| p.load(Ordering::SeqCst) >= 10000)
    }

    /// Reset all stages to zero.
    pub fn reset(&self) {
        for p in &self.progress {
            p.store(0, Ordering::SeqCst);
        }
    }
}

/// A simple progress bar for console output.
///
/// Provides a visual progress bar for terminal output.
/// Available for CLI progress display integration.
#[allow(dead_code)] // Public API for CLI progress display integration
pub struct ConsoleProgress {
    /// Width of the progress bar in characters.
    width: usize,
    /// Last printed message (for clearing).
    last_len: std::sync::atomic::AtomicUsize,
}

#[allow(dead_code)] // Public API for CLI progress display integration
impl ConsoleProgress {
    /// Create a new console progress bar.
    pub fn new(width: usize) -> Self {
        Self {
            width,
            last_len: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// Render the progress bar.
    pub fn render(&self, progress: f32, message: &str) -> String {
        let filled = ((progress * self.width as f32) as usize).min(self.width);
        let empty = self.width - filled;
        let percentage = (progress * 100.0) as u32;

        format!(
            "\r[{}{}] {:3}% {}",
            "█".repeat(filled),
            "░".repeat(empty),
            percentage,
            message
        )
    }

    /// Print progress to stdout (with carriage return for updating).
    pub fn print(&self, progress: f32, message: &str) {
        let output = self.render(progress, message);
        let output_len = output.len();

        // Clear any extra characters from previous longer messages
        let last = self.last_len.swap(output_len, Ordering::SeqCst);
        let padding = if last > output_len {
            " ".repeat(last - output_len)
        } else {
            String::new()
        };

        print!("{}{}", output, padding);
        use std::io::Write;
        std::io::stdout().flush().ok();
    }

    /// Print completion (with newline).
    pub fn complete(&self, message: &str) {
        println!("\r[{}] 100% {}", "█".repeat(self.width), message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::AtomicU32;

    #[test]
    fn test_progress_tracker() {
        let reported = Arc::new(AtomicU32::new(0));
        let reported_clone = reported.clone();

        let tracker = ProgressTracker::new(move |p, _| {
            reported_clone.store((p * 100.0) as u32, Ordering::SeqCst);
        });

        tracker.report(0.5, "halfway");
        assert_eq!(reported.load(Ordering::SeqCst), 50);
        assert!((tracker.current() - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_multi_stage_progress() {
        let multi = MultiStageProgress::with_weights(vec![1.0, 2.0, 1.0]);

        multi.report_stage(0, 1.0, "stage 1 done");
        multi.report_stage(1, 0.5, "stage 2 half");
        multi.report_stage(2, 0.0, "stage 3 not started");

        // (1.0 * 1.0 + 0.5 * 2.0 + 0.0 * 1.0) / 4.0 = 2.0 / 4.0 = 0.5
        assert!((multi.overall_progress() - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_console_progress_render() {
        let console = ConsoleProgress::new(20);
        let output = console.render(0.5, "Processing");
        assert!(output.contains("50%"));
        assert!(output.contains("█".repeat(10).as_str()));
    }
}

//! Progress tracking for migration operations.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Callback type for progress updates.
pub type ProgressCallback = Box<dyn Fn(ProgressUpdate) + Send + Sync>;

/// Progress update information.
#[derive(Debug, Clone)]
pub struct ProgressUpdate {
    /// Current phase of migration.
    pub phase: MigrationPhase,

    /// Current item being processed.
    pub current_item: Option<String>,

    /// Items completed in current phase.
    pub completed: u64,

    /// Total items in current phase.
    pub total: u64,

    /// Optional message.
    pub message: Option<String>,
}

/// Phases of the migration process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationPhase {
    /// Initializing migration.
    Initializing,
    /// Creating repository on Guts.
    CreatingRepository,
    /// Cloning source repository.
    CloningRepository,
    /// Pushing to Guts.
    PushingRepository,
    /// Migrating labels.
    MigratingLabels,
    /// Migrating milestones.
    MigratingMilestones,
    /// Migrating issues.
    MigratingIssues,
    /// Migrating pull requests.
    MigratingPullRequests,
    /// Migrating releases.
    MigratingReleases,
    /// Migrating wiki.
    MigratingWiki,
    /// Verifying migration.
    Verifying,
    /// Migration complete.
    Complete,
}

impl std::fmt::Display for MigrationPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Initializing => write!(f, "Initializing"),
            Self::CreatingRepository => write!(f, "Creating repository"),
            Self::CloningRepository => write!(f, "Cloning source repository"),
            Self::PushingRepository => write!(f, "Pushing to Guts"),
            Self::MigratingLabels => write!(f, "Migrating labels"),
            Self::MigratingMilestones => write!(f, "Migrating milestones"),
            Self::MigratingIssues => write!(f, "Migrating issues"),
            Self::MigratingPullRequests => write!(f, "Migrating pull requests"),
            Self::MigratingReleases => write!(f, "Migrating releases"),
            Self::MigratingWiki => write!(f, "Migrating wiki"),
            Self::Verifying => write!(f, "Verifying migration"),
            Self::Complete => write!(f, "Complete"),
        }
    }
}

/// Progress tracker for migration operations.
pub struct MigrationProgress {
    phase: std::sync::atomic::AtomicU8,
    completed: AtomicU64,
    total: AtomicU64,
    callback: Option<Arc<ProgressCallback>>,
}

impl MigrationProgress {
    /// Create a new progress tracker.
    pub fn new() -> Self {
        Self {
            phase: std::sync::atomic::AtomicU8::new(0),
            completed: AtomicU64::new(0),
            total: AtomicU64::new(0),
            callback: None,
        }
    }

    /// Create a progress tracker with a callback.
    pub fn with_callback(callback: ProgressCallback) -> Self {
        Self {
            phase: std::sync::atomic::AtomicU8::new(0),
            completed: AtomicU64::new(0),
            total: AtomicU64::new(0),
            callback: Some(Arc::new(callback)),
        }
    }

    /// Set the current phase.
    pub fn set_phase(&self, phase: MigrationPhase, total: u64) {
        self.phase.store(phase as u8, Ordering::SeqCst);
        self.completed.store(0, Ordering::SeqCst);
        self.total.store(total, Ordering::SeqCst);
        self.notify(None, None);
    }

    /// Increment progress.
    pub fn increment(&self, item: Option<&str>) {
        self.completed.fetch_add(1, Ordering::SeqCst);
        self.notify(item.map(|s| s.to_string()), None);
    }

    /// Set a message.
    pub fn message(&self, msg: &str) {
        self.notify(None, Some(msg.to_string()));
    }

    /// Get current progress percentage.
    pub fn percentage(&self) -> f64 {
        let total = self.total.load(Ordering::SeqCst);
        if total == 0 {
            return 0.0;
        }
        let completed = self.completed.load(Ordering::SeqCst);
        (completed as f64 / total as f64) * 100.0
    }

    /// Get current phase.
    pub fn current_phase(&self) -> MigrationPhase {
        match self.phase.load(Ordering::SeqCst) {
            0 => MigrationPhase::Initializing,
            1 => MigrationPhase::CreatingRepository,
            2 => MigrationPhase::CloningRepository,
            3 => MigrationPhase::PushingRepository,
            4 => MigrationPhase::MigratingLabels,
            5 => MigrationPhase::MigratingMilestones,
            6 => MigrationPhase::MigratingIssues,
            7 => MigrationPhase::MigratingPullRequests,
            8 => MigrationPhase::MigratingReleases,
            9 => MigrationPhase::MigratingWiki,
            10 => MigrationPhase::Verifying,
            _ => MigrationPhase::Complete,
        }
    }

    fn notify(&self, current_item: Option<String>, message: Option<String>) {
        if let Some(callback) = &self.callback {
            let update = ProgressUpdate {
                phase: self.current_phase(),
                current_item,
                completed: self.completed.load(Ordering::SeqCst),
                total: self.total.load(Ordering::SeqCst),
                message,
            };
            callback(update);
        }
    }
}

impl Default for MigrationProgress {
    fn default() -> Self {
        Self::new()
    }
}

/// Console progress reporter using indicatif.
pub struct ConsoleProgressReporter {
    progress_bar: indicatif::ProgressBar,
    multi_progress: indicatif::MultiProgress,
}

impl ConsoleProgressReporter {
    /// Create a new console progress reporter.
    pub fn new() -> Self {
        let multi_progress = indicatif::MultiProgress::new();
        let progress_bar = multi_progress.add(indicatif::ProgressBar::new(100));

        progress_bar.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );

        Self {
            progress_bar,
            multi_progress,
        }
    }

    /// Create a progress callback for use with migration.
    pub fn callback(&self) -> ProgressCallback {
        let pb = self.progress_bar.clone();
        Box::new(move |update: ProgressUpdate| {
            pb.set_length(update.total);
            pb.set_position(update.completed);

            let mut msg = update.phase.to_string();
            if let Some(item) = &update.current_item {
                msg = format!("{msg}: {item}");
            }
            if let Some(message) = &update.message {
                msg = format!("{msg} - {message}");
            }
            pb.set_message(msg);
        })
    }

    /// Finish the progress bar.
    pub fn finish(&self, message: &str) {
        self.progress_bar.finish_with_message(message.to_string());
    }

    /// Get the multi-progress handle.
    pub fn multi_progress(&self) -> &indicatif::MultiProgress {
        &self.multi_progress
    }
}

impl Default for ConsoleProgressReporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_tracker() {
        let progress = MigrationProgress::new();

        progress.set_phase(MigrationPhase::MigratingIssues, 10);
        assert_eq!(progress.current_phase(), MigrationPhase::MigratingIssues);
        assert_eq!(progress.percentage(), 0.0);

        progress.increment(Some("Issue #1"));
        assert!((progress.percentage() - 10.0).abs() < 0.01);

        for _ in 0..9 {
            progress.increment(None);
        }
        assert!((progress.percentage() - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_progress_with_callback() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = call_count.clone();

        let progress = MigrationProgress::with_callback(Box::new(move |_| {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
        }));

        progress.set_phase(MigrationPhase::MigratingIssues, 5);
        progress.increment(None);
        progress.increment(None);

        assert!(call_count.load(Ordering::SeqCst) >= 3);
    }
}

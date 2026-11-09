use crate::analysis::{Candidate, CandidateAction};
use crate::cleaner::verify::PreDeleteVerifier;
use crate::error::{DevCleanError, Result};
use crate::git::TrackedPathIndex;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Debug, Default)]
pub struct CleanStats {
  pub attempted_operations: usize,
  pub successful_operations: usize,
  pub skipped_operations: usize,
}

pub struct TrashCleaner;

impl TrashCleaner {
  /// Moves approved candidates to OS trash with pre-delete verification and Ctrl+C check.
  pub fn clean_candidates(
    project_root: &Path,
    candidates: &[Candidate],
    tracked_index: &TrackedPathIndex,
    interrupted: Arc<AtomicBool>,
  ) -> Result<CleanStats> {
    let mut stats = CleanStats::default();

    for candidate in candidates {
      // Check interruption
      if interrupted.load(Ordering::SeqCst) {
        return Err(DevCleanError::Interrupted);
      }

      // Only clean candidates marked for aggregation or individual file
      match &candidate.action {
        CandidateAction::AggregateDirectory | CandidateAction::IndividualFile => {}
        CandidateAction::Skip(_) => {
          stats.skipped_operations += 1;
          continue;
        }
      }

      stats.attempted_operations += 1;

      // 1. Pre-delete TOCTOU verification
      if let Err(e) = PreDeleteVerifier::verify(project_root, candidate, tracked_index) {
        eprintln!(
          "Pre-delete verification skipped {}: {}",
          candidate.relative_path.display(),
          e
        );
        stats.skipped_operations += 1;
        continue;
      }

      // 2. Perform trash deletion
      match trash::delete(&candidate.path) {
        Ok(_) => {
          stats.successful_operations += 1;
        }
        Err(e) => {
          eprintln!(
            "Failed to move {} to trash: {e}",
            candidate.relative_path.display()
          );
          stats.skipped_operations += 1;
          // IMPORTANT: Do NOT automatically fallback to permanent deletion!
        }
      }
    }

    Ok(stats)
  }
}

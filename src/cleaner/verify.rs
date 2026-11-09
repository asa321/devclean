use crate::analysis::Candidate;
use crate::error::{DevCleanError, Result};
use crate::git::{GitRepository, TrackedPathIndex};
use std::fs;
use std::path::Path;
use std::time::Duration;

pub struct PreDeleteVerifier;

impl PreDeleteVerifier {
  /// Re-validates the candidate immediately before trash operation to mitigate TOCTOU.
  pub fn verify(
    _project_root: &Path,
    candidate: &Candidate,
    tracked_index: &TrackedPathIndex,
  ) -> Result<()> {
    // 1. Verify existence and obtain metadata
    let metadata =
      fs::symlink_metadata(&candidate.path).map_err(|e| DevCleanError::PreDeleteVerification {
        path: candidate.path.clone(),
        reason: format!("File or directory disappeared or cannot be read: {e}"),
      })?;

    // 2. Verify file type didn't unexpectedly change
    if candidate.is_dir && !metadata.is_dir() {
      return Err(DevCleanError::PreDeleteVerification {
        path: candidate.path.clone(),
        reason: "Target was expected to be a directory but is now a file or symlink".to_string(),
      });
    }

    if !candidate.is_dir && metadata.is_dir() {
      return Err(DevCleanError::PreDeleteVerification {
        path: candidate.path.clone(),
        reason: "Target was expected to be a file but is now a directory".to_string(),
      });
    }

    // 3. Symlink / Junction check: do not delete symlink targets or corrupted links
    if metadata.file_type().is_symlink() {
      return Err(DevCleanError::PreDeleteVerification {
        path: candidate.path.clone(),
        reason: "Symlink or junction reparse point detected at deletion target".to_string(),
      });
    }

    // 4. Re-check Tracked protection
    if candidate.is_dir {
      if tracked_index.has_tracked_under(&candidate.relative_path) {
        return Err(DevCleanError::PreDeleteVerification {
          path: candidate.path.clone(),
          reason: "Tracked file was added under directory prior to deletion".to_string(),
        });
      }
    } else if tracked_index.is_tracked_exact(&candidate.relative_path) {
      return Err(DevCleanError::PreDeleteVerification {
        path: candidate.path.clone(),
        reason: "Target is tracked in Git index".to_string(),
      });
    }

    Ok(())
  }

  /// Refreshes tracked index for the project before clean execution.
  pub fn refresh_tracked(project_root: &Path, timeout: Duration) -> Result<TrackedPathIndex> {
    let repo = GitRepository::open(project_root)?;
    repo.get_tracked_index(timeout)
  }
}

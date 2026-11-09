use crate::error::{DevCleanError, Result};
use crate::git::tracked::TrackedPathIndex;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

pub struct GitRepository {
  pub root: PathBuf,
}

impl GitRepository {
  /// Opens a Git repository at root, verifying with git rev-parse.
  pub fn open(path: &Path) -> Result<Self> {
    let output = Command::new("git")
      .args(["rev-parse", "--show-toplevel"])
      .current_dir(path)
      .output()
      .map_err(|e| DevCleanError::Git(format!("Failed to execute git rev-parse: {e}")))?;

    if !output.status.success() {
      return Err(DevCleanError::Git(format!(
        "Path {:?} is not inside a git repository",
        path
      )));
    }

    let root_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    #[cfg(windows)]
    let root = PathBuf::from(root_str.replace('/', "\\"));
    #[cfg(not(windows))]
    let root = PathBuf::from(root_str);

    Ok(Self { root })
  }

  /// Fetches all tracked files in this repository in a single batch using `git ls-files -z`.
  pub fn get_tracked_index(&self, _timeout: Duration) -> Result<TrackedPathIndex> {
    let output = Command::new("git")
      .args(["ls-files", "-z"])
      .current_dir(&self.root)
      .output()
      .map_err(|e| DevCleanError::Git(format!("Failed to execute git ls-files: {e}")))?;

    if !output.status.success() {
      let err_msg = String::from_utf8_lossy(&output.stderr);
      return Err(DevCleanError::Git(format!(
        "git ls-files failed: {err_msg}"
      )));
    }

    let mut index = TrackedPathIndex::new();
    let parts: Vec<&[u8]> = output.stdout.split(|&b| b == 0).collect();

    for raw in parts {
      if raw.is_empty() {
        continue;
      }
      #[cfg(windows)]
      let rel_path = PathBuf::from(String::from_utf8_lossy(raw).replace('/', "\\"));
      #[cfg(not(windows))]
      let rel_path = {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;
        PathBuf::from(OsStr::from_bytes(raw))
      };

      index.insert(rel_path);
    }

    Ok(index)
  }
}

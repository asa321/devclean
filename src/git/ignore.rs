use crate::error::{DevCleanError, Result};
use std::collections::HashSet;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct GitIgnoreMatch {
  pub source: String,
  pub line: usize,
  pub pattern: String,
  pub path: PathBuf,
  pub is_negation: bool,
}

/// Executes `git check-ignore --no-index -v -z --stdin` in batches.
pub struct GitIgnoreChecker;

impl GitIgnoreChecker {
  pub fn check_batch(
    repo_root: &Path,
    relative_paths: &[PathBuf],
    _timeout: Duration,
  ) -> Result<HashSet<PathBuf>> {
    if relative_paths.is_empty() {
      return Ok(HashSet::new());
    }

    let mut child = Command::new("git")
      .args(["check-ignore", "--no-index", "-v", "-z", "--stdin"])
      .current_dir(repo_root)
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()
      .map_err(|e| DevCleanError::Git(format!("Failed to spawn git check-ignore: {e}")))?;

    // Write all paths separated by NUL
    if let Some(mut stdin) = child.stdin.take() {
      for p in relative_paths {
        #[cfg(windows)]
        let bytes = p.to_string_lossy().replace('\\', "/").into_bytes();
        #[cfg(not(windows))]
        let bytes = {
          use std::os::unix::ffi::OsStrExt;
          p.as_os_str().as_bytes().to_vec()
        };

        let _ = stdin.write_all(&bytes);
        let _ = stdin.write_all(&[0]);
      }
    }

    let output = child
      .wait_with_output()
      .map_err(|e| DevCleanError::Git(format!("git check-ignore execution error: {e}")))?;

    // Return code 0 (at least one match) or 1 (no match) are normal. Others are errors.
    if let Some(code) = output.status.code() {
      if code != 0 && code != 1 {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        return Err(DevCleanError::Git(format!(
          "git check-ignore failed with code {code}: {err_msg}"
        )));
      }
    }

    let mut ignored_paths = HashSet::new();
    let parts: Vec<&[u8]> = output.stdout.split(|&b| b == 0).collect();

    let mut i = 0;
    while i + 4 <= parts.len() {
      if parts[i].is_empty() && i + 1 == parts.len() {
        break;
      }
      let _source = String::from_utf8_lossy(parts[i]);
      let _lineno = String::from_utf8_lossy(parts[i + 1]);
      let pattern = String::from_utf8_lossy(parts[i + 2]);
      let pathname_raw = parts[i + 3];

      let is_negation = pattern.starts_with('!');

      #[cfg(windows)]
      let path_str = String::from_utf8_lossy(pathname_raw).to_string();
      #[cfg(windows)]
      let rel_path = PathBuf::from(path_str.replace('/', "\\"));

      #[cfg(not(windows))]
      let rel_path = {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;
        PathBuf::from(OsStr::from_bytes(pathname_raw))
      };

      if !is_negation {
        ignored_paths.insert(crate::git::tracked::normalize_relative_path(&rel_path));
      }

      i += 4;
    }

    Ok(ignored_paths)
  }
}

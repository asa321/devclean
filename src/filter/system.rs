use std::path::Path;

pub struct SystemFilter;

impl SystemFilter {
  /// Strictly checks if a path is a system protected area or critical file that MUST NEVER be deleted.
  pub fn is_absolutely_excluded(path: &Path, project_root: &Path) -> bool {
    // 1. Project root itself
    if path == project_root {
      return true;
    }

    // 2. .git directory itself or .git file
    if let Some(file_name) = path.file_name() {
      if file_name == ".git" {
        return true;
      }
    }

    // 3. Current executable path check (devclean itself)
    if let Ok(current_exe) = std::env::current_exe() {
      if path == current_exe {
        return true;
      }
    }

    // 4. Filesystem root checks
    #[cfg(windows)]
    {
      if let Some(path_str) = path.to_str() {
        let upper = path_str.to_uppercase();
        // Drive roots e.g. "C:\"
        if upper.len() <= 3 && upper.ends_with(":\\") {
          return true;
        }
        // Windows system directories
        if upper.starts_with("C:\\WINDOWS")
          || upper.starts_with("C:\\PROGRAM FILES")
          || upper.starts_with("C:\\PROGRAM FILES (X86)")
          || upper.starts_with("C:\\SYSTEM VOLUME INFORMATION")
        {
          return true;
        }
      }
    }

    #[cfg(not(windows))]
    {
      if path == Path::new("/") {
        return true;
      }
      let forbidden = [
        "/bin", "/sbin", "/usr", "/etc", "/lib", "/dev", "/sys", "/proc", "/System", "/Library",
      ];
      for &f in &forbidden {
        if path.starts_with(f) {
          return true;
        }
      }
    }

    false
  }
}

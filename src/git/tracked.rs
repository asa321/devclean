use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};

/// High-performance prefix-matching tree for tracked paths.
/// Ensures path boundary correctness (e.g. `target/foo` is inside `target`,
/// but `target-old/foo` is NOT inside `target`).
#[derive(Debug, Default, Clone)]
pub struct TrackedPathIndex {
  exact_files: HashSet<PathBuf>,
}

impl TrackedPathIndex {
  pub fn new() -> Self {
    Self {
      exact_files: HashSet::new(),
    }
  }

  /// Inserts a tracked file path relative to repository root.
  pub fn insert(&mut self, path: PathBuf) {
    self.exact_files.insert(normalize_relative_path(&path));
  }

  /// Checks if the exact path is tracked.
  pub fn is_tracked_exact(&self, path: &Path) -> bool {
    let norm = normalize_relative_path(path);
    self.exact_files.contains(&norm)
  }

  /// Checks if any tracked file exists under the given directory path.
  /// If true, the directory MUST NOT be aggregated/deleted as a whole.
  pub fn has_tracked_under(&self, dir_path: &Path) -> bool {
    let norm_dir = normalize_relative_path(dir_path);
    for tracked in &self.exact_files {
      if tracked.starts_with(&norm_dir) && tracked != &norm_dir {
        return true;
      }
    }
    false
  }

  pub fn len(&self) -> usize {
    self.exact_files.len()
  }

  pub fn is_empty(&self) -> bool {
    self.exact_files.is_empty()
  }
}

/// Normalizes relative path components into a canonical standard representation.
pub fn normalize_relative_path(path: &Path) -> PathBuf {
  let mut clean = PathBuf::new();
  for comp in path.components() {
    match comp {
      Component::Normal(c) => clean.push(c),
      Component::CurDir => {}
      Component::ParentDir => {
        clean.pop();
      }
      _ => {}
    }
  }
  clean
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_tracked_boundary_matching() {
    let mut index = TrackedPathIndex::new();
    index.insert(PathBuf::from("src/main.rs"));
    index.insert(PathBuf::from("target/forced.txt"));
    index.insert(PathBuf::from("target-old/file.txt"));

    assert!(index.is_tracked_exact(Path::new("src/main.rs")));
    assert!(!index.is_tracked_exact(Path::new("src/lib.rs")));

    // target has target/forced.txt under it
    assert!(index.has_tracked_under(Path::new("target")));

    // target-old has target-old/file.txt, but target does NOT match target-old
    assert!(!index.has_tracked_under(Path::new("build")));

    // target-ol should NOT match target-old/file.txt
    assert!(!index.has_tracked_under(Path::new("target-ol")));
  }
}

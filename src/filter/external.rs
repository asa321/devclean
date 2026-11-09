use globset::{Glob, GlobSet, GlobSetBuilder};
use std::fs;
use std::path::{Path, PathBuf};

const CONFIG_FILE_NAME: &str = ".devclean-ignore";

/// Manages external protection patterns loaded from devclean-ignore.txt files.
#[derive(Debug, Clone, Default)]
pub struct ExternalIgnoreRules {
  matchers: Option<GlobSet>,
  patterns: Vec<String>,
}

impl ExternalIgnoreRules {
  /// Loads ignore patterns according to precedence:
  /// 1. CLI explicit config (if provided)
  /// 2. Binary directory devclean-ignore.txt
  /// 3. Project root devclean-ignore.txt
  /// 4. User global config directory
  pub fn load(project_root: &Path, explicit_config: Option<&Path>) -> Self {
    let mut patterns = Vec::new();

    // 1. Explicit CLI config
    if let Some(path) = explicit_config {
      if let Ok(content) = fs::read_to_string(path) {
        Self::parse_lines(&content, &mut patterns);
      }
    }

    // 2. Binary directory
    if let Ok(exe_path) = std::env::current_exe() {
      if let Some(exe_dir) = exe_path.parent() {
        let bin_config = exe_dir.join(CONFIG_FILE_NAME);
        if bin_config.exists() {
          if let Ok(content) = fs::read_to_string(&bin_config) {
            Self::parse_lines(&content, &mut patterns);
          }
        }
      }
    }

    // 3. Project root
    let project_config = project_root.join(CONFIG_FILE_NAME);
    if project_config.exists() {
      if let Ok(content) = fs::read_to_string(&project_config) {
        Self::parse_lines(&content, &mut patterns);
      }
    }

    // 4. User global config directory
    if let Some(user_config) = Self::user_config_path() {
      if user_config.exists() {
        if let Ok(content) = fs::read_to_string(&user_config) {
          Self::parse_lines(&content, &mut patterns);
        }
      }
    }

    // Build GlobSet
    let mut builder = GlobSetBuilder::new();
    for pat in &patterns {
      if let Ok(glob) = Glob::new(pat) {
        builder.add(glob);
      }
    }

    let matchers = builder.build().ok();

    Self { matchers, patterns }
  }

  /// Parses non-empty, non-comment lines.
  fn parse_lines(content: &str, out: &mut Vec<String>) {
    for line in content.lines() {
      let trimmed = line.trim();
      if trimmed.is_empty() || trimmed.starts_with('#') {
        continue;
      }
      if !out.contains(&trimmed.to_string()) {
        out.push(trimmed.to_string());
      }
    }
  }

  /// Resolves the OS-specific user configuration path.
  fn user_config_path() -> Option<PathBuf> {
    #[cfg(windows)]
    {
      std::env::var("APPDATA").ok().map(|appdata| {
        PathBuf::from(appdata)
          .join("devclean")
          .join(CONFIG_FILE_NAME)
      })
    }

    #[cfg(not(windows))]
    {
      std::env::var("XDG_CONFIG_HOME")
        .ok()
        .map(|xdg| PathBuf::from(xdg).join("devclean").join(CONFIG_FILE_NAME))
        .or_else(|| {
          std::env::var("HOME").ok().map(|home| {
            PathBuf::from(home)
              .join(".config")
              .join("devclean")
              .join(CONFIG_FILE_NAME)
          })
        })
    }
  }

  /// Checks if a file name or path matches any external protection pattern.
  pub fn matches(&self, path: &Path) -> bool {
    if let Some(ref set) = self.matchers {
      if set.is_match(path) {
        return true;
      }
      if let Some(name) = path.file_name() {
        if set.is_match(name) {
          return true;
        }
      }
    }
    false
  }

  pub fn patterns(&self) -> &[String] {
    &self.patterns
  }
}

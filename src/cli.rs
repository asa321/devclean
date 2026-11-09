use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Format {
  Text,
  Json,
}

#[derive(Parser, Debug, Clone)]
#[command(
  name = "devclean",
  author,
  version,
  about = "Safely scan and clean build artifacts and caches to OS trash",
  long_about = None
)]
pub struct Cli {
  /// Target root directory (defaults to current directory)
  #[arg(value_name = "PATH")]
  pub path: Option<PathBuf>,

  /// Target root directory (alias / compatibility option)
  #[arg(short = 'p', long = "path", value_name = "PATH")]
  pub path_flag: Option<PathBuf>,

  /// Recursive scan (enabled by default)
  #[arg(short = 'r', long = "recursive", default_value_t = true)]
  pub recursive: bool,

  /// Maximum scan depth (root is depth 0)
  #[arg(long = "depth", value_name = "N")]
  pub depth: Option<usize>,

  /// Target only Git repositories
  #[arg(long = "git-only")]
  pub git_only: bool,

  /// Scan and summarize only without delete planning
  #[arg(long = "scan")]
  pub scan: bool,

  /// Move safe candidates to OS trash
  #[arg(long = "clean")]
  pub clean: bool,

  /// Show plan without deleting (default mode)
  #[arg(long = "dry-run")]
  pub dry_run: bool,

  /// Skip interactive confirmation
  #[arg(short = 'y', long = "yes")]
  pub yes: bool,

  /// Allow cleaning non-Git projects (requires interactive confirmation)
  #[arg(long = "allow-non-git")]
  pub allow_non_git: bool,

  /// Relax sensitive protection inside build artifact directories
  #[arg(long = "include-sensitive")]
  pub include_sensitive: bool,

  /// Additional inclusion pattern
  #[arg(long = "include", value_name = "PATTERN")]
  pub include: Option<String>,

  /// Exclusion pattern (takes precedence over include)
  #[arg(long = "exclude", value_name = "PATTERN")]
  pub exclude: Option<String>,

  /// Minimum file size (e.g. 10MB, 1GB)
  #[arg(long = "min-size", value_name = "SIZE")]
  pub min_size: Option<String>,

  /// Maximum file size or aggregate directory size
  #[arg(long = "max-size", value_name = "SIZE")]
  pub max_size: Option<String>,

  /// Maximum total files to be deleted
  #[arg(long = "max-files", value_name = "N")]
  pub max_files: Option<usize>,

  /// Maximum total delete operations
  #[arg(long = "max-operations", value_name = "N")]
  pub max_operations: Option<usize>,

  /// Include empty directories in scan candidates
  #[arg(long = "empty-dirs")]
  pub empty_dirs: bool,

  /// Permanently delete instead of moving to trash (requires --clean --yes)
  #[arg(long = "permanent")]
  pub permanent: bool,

  /// Output format
  #[arg(long = "format", value_enum, default_value_t = Format::Text)]
  pub format: Format,

  /// Verbose detailed output
  #[arg(short = 'v', long = "verbose")]
  pub verbose: bool,

  /// Minimal quiet output
  #[arg(short = 'q', long = "quiet")]
  pub quiet: bool,

  /// Check for latest release on GitHub
  #[arg(long = "check-update")]
  pub check_update: bool,

  /// Self-update devclean binary to the latest release
  #[arg(long = "update")]
  pub update: bool,
}

impl Cli {
  /// Resolves target directory and validates CLI options against safety rules.
  pub fn validate_and_resolve(&self) -> Result<PathBuf, crate::error::DevCleanError> {
    // 0. Update modes exclusivity
    if self.check_update || self.update {
      if self.clean || self.scan || self.dry_run {
        return Err(crate::error::DevCleanError::Cli(
          "Cannot combine update options (--check-update, --update) with clean/scan/dry-run modes"
            .to_string(),
        ));
      }
      return Ok(PathBuf::from("."));
    }
    let target_path = self
      .path
      .clone()
      .or_else(|| self.path_flag.clone())
      .unwrap_or_else(|| PathBuf::from("."));

    // 2. Validate mutually exclusive modes
    if self.clean && self.dry_run {
      return Err(crate::error::DevCleanError::Cli(
        "Cannot specify both --clean and --dry-run".to_string(),
      ));
    }
    if self.scan && self.clean {
      return Err(crate::error::DevCleanError::Cli(
        "Cannot specify both --scan and --clean".to_string(),
      ));
    }

    // 3. Validate safety rules
    if self.allow_non_git && self.yes {
      return Err(crate::error::DevCleanError::SafetyGuard(
        "--allow-non-git cannot be combined with --yes (interactive confirmation is mandatory)"
          .to_string(),
      ));
    }

    if self.allow_non_git && self.include.is_some() {
      return Err(crate::error::DevCleanError::SafetyGuard(
        "--include cannot be used with --allow-non-git".to_string(),
      ));
    }

    if self.format == Format::Json && self.clean && !self.yes {
      return Err(crate::error::DevCleanError::SafetyGuard(
        "--format json with --clean requires --yes (interactive confirmation cannot be used in JSON mode)"
          .to_string(),
      ));
    }

    if self.permanent && (!self.clean || !self.yes) {
      return Err(crate::error::DevCleanError::SafetyGuard(
        "--permanent requires both --clean and --yes".to_string(),
      ));
    }

    Ok(target_path)
  }
}

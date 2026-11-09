use std::path::PathBuf;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, DevCleanError>;

#[derive(Debug, Error)]
pub enum DevCleanError {
  #[error("CLI argument error: {0}")]
  Cli(String),

  #[error("Safety guard violation: {0}")]
  SafetyGuard(String),

  #[error("Git error: {0}")]
  Git(String),

  #[error("Git command timed out after {0}s")]
  GitTimeout(u64),

  #[error("I/O error at {path}: {source}")]
  Io {
    path: PathBuf,
    #[source]
    source: std::io::Error,
  },

  #[error("Trash operation failed for {path}: {message}")]
  Trash { path: PathBuf, message: String },

  #[error("Verification failed before delete for {path}: {reason}")]
  PreDeleteVerification { path: PathBuf, reason: String },

  #[error("Interrupted by user (Ctrl+C)")]
  Interrupted,

  #[error("Fatal error: {0}")]
  Fatal(String),
}

impl DevCleanError {
  /// Maps the error to the CLI exit code specification.
  /// 0: Success / no candidates
  /// 1: Partial skip
  /// 2: CLI argument error
  /// 3: Interrupted (Ctrl+C)
  /// 4: Safety guard rejection
  /// 5: Fatal error
  pub fn exit_code(&self) -> i32 {
    match self {
      Self::Cli(_) => 2,
      Self::Interrupted => 3,
      Self::SafetyGuard(_) => 4,
      Self::Git(_) | Self::GitTimeout(_) | Self::Fatal(_) => 5,
      Self::Trash { .. } | Self::Io { .. } | Self::PreDeleteVerification { .. } => 1,
    }
  }
}

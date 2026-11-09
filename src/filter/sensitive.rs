use std::path::Path;

/// Level 1: Depth 0, 1, 2 from project root (strictly protected, cannot be bypassed).
/// Level 2: Inside build artifact directories (depth >= 3, protected by default, can be relaxed with --include-sensitive).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensitiveLevel {
  Level1Strict,
  Level2Internal,
  NotSensitive,
}

pub struct SensitiveFilter;

impl SensitiveFilter {
  /// Checks if the file name matches known sensitive patterns.
  pub fn matches_pattern(file_name: &str) -> bool {
    let lower = file_name.to_lowercase();

    // Exact matches
    if lower == ".env" || lower == "credentials" || lower == "secrets" {
      return true;
    }

    // Prefix matches
    if lower.starts_with(".env.")
      || lower.starts_with("id_rsa")
      || lower.starts_with("credentials.")
      || lower.starts_with("credentials_")
    {
      return true;
    }

    // Substring matches
    if lower.contains("secret") {
      return true;
    }

    // Extensions
    if lower.ends_with(".key")
      || lower.ends_with(".pem")
      || lower.ends_with(".p12")
      || lower.ends_with(".pfx")
    {
      return true;
    }

    false
  }

  /// Determines sensitivity given relative depth from project root.
  pub fn classify(file_path: &Path, depth: usize) -> SensitiveLevel {
    if let Some(file_name) = file_path.file_name().and_then(|n| n.to_str()) {
      if Self::matches_pattern(file_name) {
        if depth <= 2 {
          return SensitiveLevel::Level1Strict;
        } else {
          return SensitiveLevel::Level2Internal;
        }
      }
    }
    SensitiveLevel::NotSensitive
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::path::Path;

  #[test]
  fn test_sensitive_patterns() {
    assert!(SensitiveFilter::matches_pattern(".env"));
    assert!(SensitiveFilter::matches_pattern(".env.production"));
    assert!(SensitiveFilter::matches_pattern("id_rsa"));
    assert!(SensitiveFilter::matches_pattern("id_rsa.pub"));
    assert!(SensitiveFilter::matches_pattern("client_secret.json"));
    assert!(SensitiveFilter::matches_pattern("server.key"));
    assert!(SensitiveFilter::matches_pattern("cert.pem"));
    assert!(SensitiveFilter::matches_pattern("keystore.p12"));

    assert!(!SensitiveFilter::matches_pattern("index.ts"));
    assert!(!SensitiveFilter::matches_pattern("main.rs"));
  }

  #[test]
  fn test_sensitive_depth_classification() {
    assert_eq!(
      SensitiveFilter::classify(Path::new(".env"), 0),
      SensitiveLevel::Level1Strict
    );
    assert_eq!(
      SensitiveFilter::classify(Path::new("config/.env.local"), 1),
      SensitiveLevel::Level1Strict
    );
    assert_eq!(
      SensitiveFilter::classify(Path::new("packages/app/.env"), 2),
      SensitiveLevel::Level1Strict
    );
    assert_eq!(
      SensitiveFilter::classify(Path::new("packages/app/deep/.env"), 3),
      SensitiveLevel::Level2Internal
    );
  }
}

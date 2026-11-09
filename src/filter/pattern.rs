use globset::{Glob, GlobMatcher};
use std::path::Path;

pub struct PatternFilter {
  include_matcher: Option<GlobMatcher>,
  exclude_matcher: Option<GlobMatcher>,
}

impl PatternFilter {
  pub fn new(
    include_pattern: Option<&str>,
    exclude_pattern: Option<&str>,
  ) -> Result<Self, globset::Error> {
    let include_matcher = match include_pattern {
      Some(pat) => Some(Glob::new(pat)?.compile_matcher()),
      None => None,
    };
    let exclude_matcher = match exclude_pattern {
      Some(pat) => Some(Glob::new(pat)?.compile_matcher()),
      None => None,
    };

    Ok(Self {
      include_matcher,
      exclude_matcher,
    })
  }

  /// Checks if path is excluded. --exclude takes precedence over --include.
  pub fn is_excluded(&self, path: &Path) -> bool {
    if let Some(ref matcher) = self.exclude_matcher {
      if matcher.is_match(path) {
        return true;
      }
    }
    false
  }

  /// Checks if path matches include pattern (if specified).
  pub fn matches_include(&self, path: &Path) -> bool {
    match self.include_matcher {
      Some(ref matcher) => matcher.is_match(path),
      None => true, // Default to true if no include pattern is specified
    }
  }
}

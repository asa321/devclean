use crate::error::{DevCleanError, Result};

pub struct SizeParser;

impl SizeParser {
  /// Parses human-readable size strings (e.g., "100B", "10KB", "500MB", "2GB") to bytes.
  pub fn parse(s: &str) -> Result<u64> {
    let trimmed = s.trim().to_uppercase();
    if trimmed.is_empty() {
      return Err(DevCleanError::Cli("Empty size string".to_string()));
    }

    let (num_part, unit_mult) = if trimmed.ends_with("GB") || trimmed.ends_with("G") {
      let num_str = trimmed.trim_end_matches("GB").trim_end_matches('G');
      (num_str, 1024 * 1024 * 1024)
    } else if trimmed.ends_with("MB") || trimmed.ends_with("M") {
      let num_str = trimmed.trim_end_matches("MB").trim_end_matches('M');
      (num_str, 1024 * 1024)
    } else if trimmed.ends_with("KB") || trimmed.ends_with("K") {
      let num_str = trimmed.trim_end_matches("KB").trim_end_matches('K');
      (num_str, 1024)
    } else if trimmed.ends_with('B') {
      let num_str = trimmed.trim_end_matches('B');
      (num_str, 1)
    } else {
      (trimmed.as_str(), 1)
    };

    let num: f64 = num_part
      .trim()
      .parse()
      .map_err(|_| DevCleanError::Cli(format!("Invalid size number format: {s}")))?;

    if num < 0.0 {
      return Err(DevCleanError::Cli("Size cannot be negative".to_string()));
    }

    Ok((num * unit_mult as f64) as u64)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_size_parsing() {
    assert_eq!(SizeParser::parse("100B").unwrap(), 100);
    assert_eq!(SizeParser::parse("10KB").unwrap(), 10 * 1024);
    assert_eq!(SizeParser::parse("500M").unwrap(), 500 * 1024 * 1024);
    assert_eq!(SizeParser::parse("2GB").unwrap(), 2 * 1024 * 1024 * 1024);
    assert_eq!(SizeParser::parse("1024").unwrap(), 1024);
  }
}

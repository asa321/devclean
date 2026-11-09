use base64::Engine;
use std::path::Path;

/// Cross-platform path encoding representation according to v3.2 specification.
#[derive(Debug, serde::Serialize)]
pub struct EncodedPath {
  pub path_display: String,
  pub path_encoding: String,
  pub path_raw_base64: String,
}

impl EncodedPath {
  pub fn from_path(path: &Path) -> Self {
    let path_display = path.to_string_lossy().to_string();

    #[cfg(windows)]
    {
      use std::os::windows::ffi::OsStrExt;
      let wide_chars: Vec<u16> = path.as_os_str().encode_wide().collect();
      // Little-endian byte conversion
      let mut bytes = Vec::with_capacity(wide_chars.len() * 2);
      for w in wide_chars {
        bytes.extend_from_slice(&w.to_le_bytes());
      }
      let path_raw_base64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
      Self {
        path_display,
        path_encoding: "windows_utf16le".to_string(),
        path_raw_base64,
      }
    }

    #[cfg(not(windows))]
    {
      use std::os::unix::ffi::OsStrExt;
      let bytes = path.as_os_str().as_bytes();
      let path_raw_base64 = base64::engine::general_purpose::STANDARD.encode(bytes);
      Self {
        path_display,
        path_encoding: "unix_bytes".to_string(),
        path_raw_base64,
      }
    }
  }
}

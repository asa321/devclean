use std::fs;
use tempfile::tempdir;

#[test]
fn test_complex_paths_and_json_serialization() {
  let dir = tempdir().unwrap();
  let root = dir.path();

  // Test special filenames: Japanese, Spaces, Emoji
  let special_name = "日本語 フォルダ 🚀";
  let target_sub = root.join(special_name);
  fs::create_dir_all(&target_sub).unwrap();
  fs::write(target_sub.join("dummy.log"), "log").unwrap();

  let encoded = devclean::output::EncodedPath::from_path(&target_sub);
  assert!(encoded.path_display.contains("日本語 フォルダ 🚀"));
  assert!(!encoded.path_raw_base64.is_empty());

  #[cfg(windows)]
  assert_eq!(encoded.path_encoding, "windows_utf16le");

  #[cfg(not(windows))]
  assert_eq!(encoded.path_encoding, "unix_bytes");
}

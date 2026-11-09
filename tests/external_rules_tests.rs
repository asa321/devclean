use devclean::filter::ExternalIgnoreRules;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_external_ignore_rules_loading_and_matching() {
  let dir = tempdir().unwrap();
  let root = dir.path();

  let config_path = root.join(".devclean-ignore");
  let content = r#"
# Custom comment
custom-config.json
*.secret.db
nested/safe-file.txt
"#;
  fs::write(&config_path, content).unwrap();

  let rules = ExternalIgnoreRules::load(root, None);

  // Match root-level custom pattern
  assert!(rules.matches(Path::new("custom-config.json")));
  assert!(rules.matches(Path::new("foo/custom-config.json")));

  // Match extension pattern
  assert!(rules.matches(Path::new("mydb.secret.db")));
  assert!(rules.matches(Path::new("deep/dir/mydb.secret.db")));

  // Non-matching file
  assert!(!rules.matches(Path::new("index.ts")));
  assert!(!rules.matches(Path::new("regular.db")));
}

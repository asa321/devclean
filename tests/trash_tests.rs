use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_clean_to_trash_and_toctou_verification() {
  let dir = tempdir().unwrap();
  let root = dir.path();

  // Initialize git repo
  Command::new("git")
    .args(["init"])
    .current_dir(root)
    .output()
    .unwrap();
  Command::new("git")
    .args(["config", "user.name", "Tester"])
    .current_dir(root)
    .output()
    .unwrap();
  Command::new("git")
    .args(["config", "user.email", "test@example.com"])
    .current_dir(root)
    .output()
    .unwrap();

  // .gitignore ignores temp_artifacts/
  fs::write(root.join(".gitignore"), "temp_artifacts/\n").unwrap();
  fs::write(root.join("README.md"), "# Project").unwrap();
  Command::new("git")
    .args(["add", "README.md", ".gitignore"])
    .current_dir(root)
    .output()
    .unwrap();
  Command::new("git")
    .args(["commit", "-m", "init"])
    .current_dir(root)
    .output()
    .unwrap();

  // Create cleanable directory
  let artifact_dir = root.join("temp_artifacts");
  fs::create_dir_all(&artifact_dir).unwrap();
  fs::write(artifact_dir.join("build.log"), "build output").unwrap();

  let project = devclean::scanner::Project {
    root: root.to_path_buf(),
    kind: devclean::scanner::ProjectKind::GitRepository,
  };

  let external_rules = devclean::filter::ExternalIgnoreRules::default();
  let candidates = devclean::analysis::SafetyEngine::evaluate_project(
    &project,
    false,
    &external_rules,
    std::time::Duration::from_secs(10),
  )
  .unwrap();

  assert_eq!(candidates.len(), 1);
  assert_eq!(
    candidates[0].action,
    devclean::analysis::CandidateAction::AggregateDirectory
  );

  // Perform clean
  let tracked = devclean::cleaner::PreDeleteVerifier::refresh_tracked(
    &project.root,
    std::time::Duration::from_secs(10),
  )
  .unwrap();

  let interrupted = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
  let stats = devclean::cleaner::TrashCleaner::clean_candidates(
    &project.root,
    &candidates,
    &tracked,
    interrupted,
  )
  .unwrap();

  assert_eq!(stats.successful_operations, 1);
  assert_eq!(stats.skipped_operations, 0);

  // Verify that artifact_dir no longer exists in root (moved to trash)
  assert!(!artifact_dir.exists());
}

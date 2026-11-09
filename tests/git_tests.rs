use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_git_tracked_protection_and_aggregation_veto() {
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

  // Create .gitignore
  fs::write(root.join(".gitignore"), "target/\n").unwrap();

  // Create normal tracked file
  fs::write(root.join("Cargo.toml"), "[package]").unwrap();
  Command::new("git")
    .args(["add", "Cargo.toml", ".gitignore"])
    .current_dir(root)
    .output()
    .unwrap();
  Command::new("git")
    .args(["commit", "-m", "init"])
    .current_dir(root)
    .output()
    .unwrap();

  // Create target/ with untracked artifact AND a force-tracked file
  fs::create_dir_all(root.join("target").join("debug")).unwrap();
  fs::write(root.join("target").join("debug").join("app.exe"), "bin").unwrap();

  let forced_file = root.join("target").join("keep_me.txt");
  fs::write(&forced_file, "must not be deleted").unwrap();
  Command::new("git")
    .args(["add", "-f", "target/keep_me.txt"])
    .current_dir(root)
    .output()
    .unwrap();
  Command::new("git")
    .args(["commit", "-m", "force add"])
    .current_dir(root)
    .output()
    .unwrap();

  // Evaluate project using devclean core
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

  // Because target contains target/keep_me.txt (tracked), target/ MUST NOT be aggregated!
  for c in candidates {
    if c.relative_path == std::path::Path::new("target") {
      match c.action {
        devclean::analysis::CandidateAction::AggregateDirectory => {
          panic!("CRITICAL SAFETY BUG: target/ was approved for aggregation despite having a tracked file inside!");
        }
        devclean::analysis::CandidateAction::Skip(reason) => {
          println!(
            "PASS: target/ aggregation correctly vetoed. Reason: {}",
            reason
          );
        }
        _ => {}
      }
    }
  }
}

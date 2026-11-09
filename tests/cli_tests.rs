use std::process::Command;

#[test]
fn test_cli_argument_rejections_and_exit_codes() {
  let exe = env!("CARGO_BIN_EXE_devclean");

  // 1. --clean and --dry-run mutually exclusive -> exit code 2
  let output = Command::new(exe)
    .args(["--clean", "--dry-run"])
    .output()
    .unwrap();
  assert_eq!(output.status.code(), Some(2));

  // 2. --allow-non-git and --yes forbidden -> exit code 4 (Safety guard)
  let output = Command::new(exe)
    .args(["--clean", "--allow-non-git", "--yes"])
    .output()
    .unwrap();
  assert_eq!(output.status.code(), Some(4));

  // 3. --format json --clean without --yes forbidden -> exit code 4 (Safety guard)
  let output = Command::new(exe)
    .args(["--format", "json", "--clean"])
    .output()
    .unwrap();
  assert_eq!(output.status.code(), Some(4));

  // 4. --permanent without --clean --yes forbidden -> exit code 4 (Safety guard)
  let output = Command::new(exe).args(["--permanent"]).output().unwrap();
  assert_eq!(output.status.code(), Some(4));
}

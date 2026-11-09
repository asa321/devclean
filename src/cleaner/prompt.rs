use std::io::{self, BufRead, Write};

pub struct ConfirmPrompt;

impl ConfirmPrompt {
  /// Prompts user for interactive confirmation.
  /// Returns true only on 'y' or 'yes' (case-insensitive).
  pub fn ask(
    project_count: usize,
    candidate_ops: usize,
    has_non_git: bool,
    has_sensitive_relaxed: bool,
  ) -> bool {
    eprintln!("\n=== Clean Confirmation ===");
    eprintln!("  Projects to clean:      {}", project_count);
    eprintln!("  Estimated operations:   {}", candidate_ops);
    if has_non_git {
      eprintln!("  [WARNING] Target contains non-Git projects!");
    }
    if has_sensitive_relaxed {
      eprintln!("  [WARNING] Sensitive file protection is relaxed in artifact folders!");
    }
    eprint!("Proceed with moving items to OS trash? [y/N]: ");
    let _ = io::stderr().flush();

    let mut input = String::new();
    let stdin = io::stdin();
    if stdin.lock().read_line(&mut input).is_err() {
      return false;
    }

    let trimmed = input.trim().to_lowercase();
    trimmed == "y" || trimmed == "yes"
  }
}

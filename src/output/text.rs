use crate::analysis::{Candidate, CandidateAction};
use crate::cli::Cli;
use crate::scanner::Project;

pub struct TextReporter;

impl TextReporter {
  pub fn render(cli: &Cli, projects: &[Project], project_candidates: &[Vec<Candidate>]) {
    println!("devclean v0.1.0\n");

    let mode_str = if cli.clean {
      "CLEAN (Trash)"
    } else if cli.scan {
      "SCAN ONLY"
    } else {
      "DRY-RUN"
    };

    let mut total_dirs = 0;
    let mut total_files = 0;
    let mut total_estimated_ops = 0;

    for (project, candidates) in projects.iter().zip(project_candidates) {
      println!("Project: {}", project.root.display());
      println!("Mode: {}\n", mode_str);

      if candidates.is_empty() {
        println!("  No cleanable candidates found.\n");
        continue;
      }

      println!("Candidates:");
      for c in candidates {
        match &c.action {
          CandidateAction::AggregateDirectory => {
            total_dirs += 1;
            total_estimated_ops += 1;
            println!(
              "  {:<24} (directory aggregate, size not scanned)",
              format!("{}/", c.relative_path.display())
            );
          }
          CandidateAction::IndividualFile => {
            total_files += 1;
            total_estimated_ops += 1;
            let size_str = c
              .size_bytes
              .map(|s| format!("{} bytes", s))
              .unwrap_or_else(|| "size not scanned".to_string());
            println!("  {:<24} {}", c.relative_path.display(), size_str);
          }
          CandidateAction::Skip(reason) => {
            if cli.verbose {
              println!("  [SKIP] {:<18} ({})", c.relative_path.display(), reason);
            }
          }
        }
      }
      println!();
    }

    println!("Summary:");
    println!("  Directories: {}", total_dirs);
    println!("  Individual Files: {}", total_files);
    println!("  Estimated operations: {}", total_estimated_ops);

    if !cli.clean {
      println!("\nNo files were deleted.");
    }
  }
}

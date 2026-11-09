use clap::Parser;
use devclean::analysis::{CandidateAction, SafetyEngine};
use devclean::cleaner::{ConfirmPrompt, PreDeleteVerifier, TrashCleaner};
use devclean::cli::Cli;
use devclean::error::DevCleanError;
use devclean::output::TextReporter;
use devclean::scanner::{Project, ProjectKind};
use std::io::IsTerminal;
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

fn main() -> ExitCode {
  let cli = Cli::parse();

  // Setup Ctrl+C handler
  let interrupted = Arc::new(AtomicBool::new(false));
  let r = interrupted.clone();
  if let Err(e) = ctrlc::set_handler(move || {
    r.store(true, Ordering::SeqCst);
    eprintln!("\nReceived Ctrl+C, safely stopping operations...");
  }) {
    // If setting handler fails, log and continue
    eprintln!("Warning: could not set Ctrl+C handler: {e}");
  }

  let target_path = match cli.validate_and_resolve() {
    Ok(p) => p,
    Err(e) => {
      eprintln!("Error: {e}");
      return ExitCode::from(e.exit_code() as u8);
    }
  };

  let projects = Project::discover(&target_path, cli.git_only);
  if projects.is_empty() {
    println!("No projects found at {:?}", target_path);
    return ExitCode::SUCCESS;
  }

  let mut all_candidates = Vec::new();
  let timeout = Duration::from_secs(30);

  for project in &projects {
    match SafetyEngine::evaluate_project(project, cli.include_sensitive, timeout) {
      Ok(c) => all_candidates.push(c),
      Err(e) => {
        eprintln!("Error evaluating project {:?}: {e}", project.root);
        return ExitCode::from(e.exit_code() as u8);
      }
    }
  }

  // Calculate operation counts and limits
  let mut total_candidate_ops = 0;
  for candidates in &all_candidates {
    for c in candidates {
      match c.action {
        CandidateAction::AggregateDirectory | CandidateAction::IndividualFile => {
          total_candidate_ops += 1;
        }
        CandidateAction::Skip(_) => {}
      }
    }
  }

  // Check safety limit --max-operations
  if let Some(max_ops) = cli.max_operations {
    if total_candidate_ops > max_ops {
      eprintln!(
        "Safety guard error: total operations ({}) exceeds --max-operations limit ({})",
        total_candidate_ops, max_ops
      );
      return ExitCode::from(DevCleanError::SafetyGuard(String::new()).exit_code() as u8);
    }
  }

  // Output scan / dry-run report
  if cli.format == devclean::cli::Format::Json {
    let json_output =
      devclean::output::JsonReporter::render(&cli, &projects, &all_candidates, 0, 0, &[]);
    if !cli.clean {
      println!("{}", json_output);
    }
  } else {
    TextReporter::render(&cli, &projects, &all_candidates);
  }

  // If not clean mode, stop here
  if !cli.clean {
    return ExitCode::SUCCESS;
  }

  if total_candidate_ops == 0 {
    println!("No cleanable items to delete.");
    return ExitCode::SUCCESS;
  }

  // Safety guard: Non-TTY check
  let is_tty = std::io::stdin().is_terminal();
  if !cli.yes && !is_tty {
    eprintln!("Safety guard error: standard input is not a TTY and --yes was not provided.");
    return ExitCode::from(DevCleanError::SafetyGuard(String::new()).exit_code() as u8);
  }

  // Interactive confirmation
  if !cli.yes {
    let has_non_git = projects
      .iter()
      .any(|p| p.kind == ProjectKind::NonGitWithGitignore);
    let confirmed = ConfirmPrompt::ask(
      projects.len(),
      total_candidate_ops,
      has_non_git,
      cli.include_sensitive,
    );
    if !confirmed {
      println!("Clean operation cancelled by user.");
      return ExitCode::SUCCESS;
    }
  }

  // Execute clean (OS trash)
  let mut had_skips = false;
  for (project, candidates) in projects.iter().zip(&all_candidates) {
    if interrupted.load(Ordering::SeqCst) {
      eprintln!("Interrupted before completing all projects.");
      return ExitCode::from(DevCleanError::Interrupted.exit_code() as u8);
    }

    let tracked_index = match PreDeleteVerifier::refresh_tracked(&project.root, timeout) {
      Ok(idx) => idx,
      Err(e) => {
        eprintln!(
          "Failed to re-index tracked files for {:?}: {e}",
          project.root
        );
        had_skips = true;
        continue;
      }
    };

    match TrashCleaner::clean_candidates(
      &project.root,
      candidates,
      &tracked_index,
      interrupted.clone(),
    ) {
      Ok(stats) => {
        if stats.skipped_operations > 0 {
          had_skips = true;
        }
        println!(
          "Cleaned {:?}: {} moved to trash, {} skipped.",
          project.root, stats.successful_operations, stats.skipped_operations
        );
      }
      Err(DevCleanError::Interrupted) => {
        return ExitCode::from(DevCleanError::Interrupted.exit_code() as u8);
      }
      Err(e) => {
        eprintln!("Error cleaning project {:?}: {e}", project.root);
        return ExitCode::from(e.exit_code() as u8);
      }
    }
  }

  if had_skips {
    ExitCode::from(1)
  } else {
    ExitCode::SUCCESS
  }
}

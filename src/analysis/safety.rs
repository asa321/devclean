use crate::error::{DevCleanError, Result};
use crate::filter::{ExternalIgnoreRules, SensitiveFilter, SensitiveLevel, SystemFilter};
use crate::git::{GitIgnoreChecker, GitRepository, TrackedPathIndex};
use crate::scanner::{Project, ProjectKind};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CandidateAction {
  AggregateDirectory,
  IndividualFile,
  Skip(String),
}

#[derive(Debug, Clone)]
pub struct Candidate {
  pub path: PathBuf,
  pub relative_path: PathBuf,
  pub is_dir: bool,
  pub action: CandidateAction,
  pub file_count: Option<usize>,
  pub size_bytes: Option<u64>,
}

pub struct SafetyEngine;

impl SafetyEngine {
  /// Evaluates candidates within a project for safe cleaning.
  pub fn evaluate_project(
    project: &Project,
    include_sensitive: bool,
    external_rules: &ExternalIgnoreRules,
    timeout: Duration,
  ) -> Result<Vec<Candidate>> {
    let mut candidates = Vec::new();

    match project.kind {
      ProjectKind::NonGitWithGitignore => {
        // Non-git projects cannot have tracked protection guaranteed
        // Default to detection only
        return Ok(candidates);
      }
      ProjectKind::GitRepository => {
        let repo = GitRepository::open(&project.root)?;
        let tracked_index = repo.get_tracked_index(timeout)?;

        // Find top-level ignored entries (e.g. node_modules, target, dist, build, .next, etc.)
        let entries = fs::read_dir(&project.root).map_err(|e| DevCleanError::Io {
          path: project.root.clone(),
          source: e,
        })?;

        let mut candidate_dirs = Vec::new();
        let mut candidate_files = Vec::new();

        for entry_res in entries {
          let entry = match entry_res {
            Ok(e) => e,
            Err(_) => continue,
          };
          let path = entry.path();
          let file_name = entry.file_name();
          let name_str = file_name.to_string_lossy();

          // Skip .git and system exclusions
          if SystemFilter::is_absolutely_excluded(&path, &project.root) {
            continue;
          }

          let rel = match path.strip_prefix(&project.root) {
            Ok(r) => r.to_path_buf(),
            Err(_) => continue,
          };

          // Never aggregate hidden files like .git or .gitignore
          if name_str == ".gitignore" || name_str == ".git" {
            continue;
          }

          if path.is_dir() {
            candidate_dirs.push(rel);
          } else {
            candidate_files.push(rel);
          }
        }

        // Run check-ignore on candidate directories & files
        let all_rel: Vec<PathBuf> = candidate_dirs
          .iter()
          .chain(candidate_files.iter())
          .cloned()
          .collect();
        let ignored_set = GitIgnoreChecker::check_batch(&project.root, &all_rel, timeout)?;

        // 1. Process candidate directories for safe aggregation
        for rel_dir in candidate_dirs {
          let full_path = project.root.join(&rel_dir);

          if !ignored_set.contains(&rel_dir) {
            continue; // Not ignored by Git
          }

          // Safe directory aggregation check
          let can_aggregate = Self::can_aggregate_directory(
            &project.root,
            &full_path,
            &rel_dir,
            &tracked_index,
            include_sensitive,
            external_rules,
          );

          match can_aggregate {
            Ok(true) => {
              candidates.push(Candidate {
                path: full_path,
                relative_path: rel_dir,
                is_dir: true,
                action: CandidateAction::AggregateDirectory,
                file_count: None, // Not scanned in standard mode
                size_bytes: None,
              });
            }
            Ok(false) | Err(_) => {
              // Cannot aggregate: skip or mark with reason
              candidates.push(Candidate {
                path: full_path,
                relative_path: rel_dir,
                is_dir: true,
                action: CandidateAction::Skip(
                  "Contains tracked files, nested git, symlinks, or sensitive files".to_string(),
                ),
                file_count: None,
                size_bytes: None,
              });
            }
          }
        }

        // 2. Process candidate single files
        for rel_file in candidate_files {
          let full_path = project.root.join(&rel_file);

          if !ignored_set.contains(&rel_file) {
            continue;
          }

          if tracked_index.is_tracked_exact(&rel_file) {
            continue; // Tracked protection!
          }

          // Check sensitive Level 1 & external rules
          if SensitiveFilter::classify(&rel_file, 0) == SensitiveLevel::Level1Strict
            || external_rules.matches(&full_path)
          {
            continue; // Strictly protected
          }

          let metadata = fs::symlink_metadata(&full_path).ok();
          let size = metadata.map(|m| m.len());

          candidates.push(Candidate {
            path: full_path,
            relative_path: rel_file,
            is_dir: false,
            action: CandidateAction::IndividualFile,
            file_count: Some(1),
            size_bytes: size,
          });
        }
      }
    }

    Ok(candidates)
  }

  /// Verifies all conditions required for directory aggregation.
  fn can_aggregate_directory(
    _project_root: &Path,
    dir_path: &Path,
    rel_dir: &Path,
    tracked_index: &TrackedPathIndex,
    include_sensitive: bool,
    external_rules: &ExternalIgnoreRules,
  ) -> Result<bool> {
    // Condition 1: Must not have any tracked file under this directory
    if tracked_index.has_tracked_under(rel_dir) {
      return Ok(false);
    }

    // Condition 2: Deep scan inspection for sensitive files, nested git, or symlinks
    let walker = ignore::WalkBuilder::new(dir_path)
      .follow_links(false)
      .hidden(false)
      .parents(false)
      .ignore(false)
      .git_global(false)
      .git_ignore(false)
      .git_exclude(false)
      .build();

    for entry_res in walker {
      let entry = match entry_res {
        Ok(e) => e,
        Err(_) => return Ok(false), // Permission denied or unreadable -> skip aggregation
      };

      let path = entry.path();
      if path == dir_path {
        continue;
      }

      // Check symlink / reparse point
      if let Ok(meta) = fs::symlink_metadata(path) {
        if meta.file_type().is_symlink() {
          return Ok(false); // Do not aggregate directories containing symlinks
        }
      }

      // Check nested git repository
      if entry.file_name() == ".git" {
        return Ok(false);
      }

      // Check external protection rules inside
      if external_rules.matches(path) {
        return Ok(false);
      }

      // Check sensitive files inside
      if !include_sensitive {
        if let Some(name) = entry.file_name().to_str() {
          if SensitiveFilter::matches_pattern(name) {
            return Ok(false);
          }
        }
      }
    }

    Ok(true)
  }
}

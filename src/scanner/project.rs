use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectKind {
  GitRepository,
  NonGitWithGitignore,
}

#[derive(Debug, Clone)]
pub struct Project {
  pub root: PathBuf,
  pub kind: ProjectKind,
}

impl Project {
  /// Scans candidates under root to find projects (.git or .gitignore).
  pub fn discover(root: &Path, git_only: bool) -> Vec<Project> {
    let mut projects = Vec::new();

    // Check if root itself is a project
    if root.join(".git").exists() {
      projects.push(Project {
        root: root.to_path_buf(),
        kind: ProjectKind::GitRepository,
      });
      return projects;
    } else if !git_only && root.join(".gitignore").exists() {
      projects.push(Project {
        root: root.to_path_buf(),
        kind: ProjectKind::NonGitWithGitignore,
      });
      return projects;
    }

    // Otherwise, discover child directories using walkdir (shallow or depth limited)
    let walker = ignore::WalkBuilder::new(root)
      .max_depth(Some(4))
      .follow_links(false)
      .build();

    for entry in walker.flatten() {
      let path = entry.path();
      if path.is_dir() {
        if path.join(".git").exists() {
          projects.push(Project {
            root: path.to_path_buf(),
            kind: ProjectKind::GitRepository,
          });
        } else if !git_only && path.join(".gitignore").exists() {
          projects.push(Project {
            root: path.to_path_buf(),
            kind: ProjectKind::NonGitWithGitignore,
          });
        }
      }
    }

    projects
  }
}

use crate::analysis::{Candidate, CandidateAction};
use crate::cli::Cli;
use crate::output::path_codec::EncodedPath;
use crate::scanner::Project;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct JsonCandidate {
  #[serde(flatten)]
  pub path: EncodedPath,
  pub is_directory: bool,
  pub action: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub skip_reason: Option<String>,
  pub file_count: Option<usize>,
  pub size_bytes: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct JsonProject {
  #[serde(flatten)]
  pub root: EncodedPath,
  pub kind: String,
  pub candidates: Vec<JsonCandidate>,
}

#[derive(Debug, Serialize)]
pub struct JsonSummary {
  pub candidate_operations: usize,
  pub deleted_operations: usize,
  pub skipped_operations: usize,
}

#[derive(Debug, Serialize)]
pub struct JsonReport {
  pub schema_version: String,
  pub app_version: String,
  pub mode: String,
  pub projects: Vec<JsonProject>,
  pub summary: JsonSummary,
  pub errors: Vec<String>,
}

pub struct JsonReporter;

impl JsonReporter {
  pub fn render(
    cli: &Cli,
    projects: &[Project],
    project_candidates: &[Vec<Candidate>],
    deleted_ops: usize,
    skipped_ops: usize,
    errors: &[String],
  ) -> String {
    let mode = if cli.clean {
      "clean"
    } else if cli.scan {
      "scan"
    } else {
      "dry-run"
    };

    let mut total_candidate_ops = 0;
    let mut json_projects = Vec::new();

    for (p, cands) in projects.iter().zip(project_candidates) {
      let mut json_cands = Vec::new();
      for c in cands {
        let (action_str, skip_reason) = match &c.action {
          CandidateAction::AggregateDirectory => {
            total_candidate_ops += 1;
            ("aggregate_directory".to_string(), None)
          }
          CandidateAction::IndividualFile => {
            total_candidate_ops += 1;
            ("individual_file".to_string(), None)
          }
          CandidateAction::Skip(reason) => ("skip".to_string(), Some(reason.clone())),
        };

        json_cands.push(JsonCandidate {
          path: EncodedPath::from_path(&c.path),
          is_directory: c.is_dir,
          action: action_str,
          skip_reason,
          file_count: c.file_count,
          size_bytes: c.size_bytes,
        });
      }

      json_projects.push(JsonProject {
        root: EncodedPath::from_path(&p.root),
        kind: format!("{:?}", p.kind),
        candidates: json_cands,
      });
    }

    let report = JsonReport {
      schema_version: "1.0.0".to_string(),
      app_version: env!("CARGO_PKG_VERSION").to_string(),
      mode: mode.to_string(),
      projects: json_projects,
      summary: JsonSummary {
        candidate_operations: total_candidate_ops,
        deleted_operations: deleted_ops,
        skipped_operations: skipped_ops,
      },
      errors: errors.to_vec(),
    };

    serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string())
  }
}

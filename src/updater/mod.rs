use crate::error::{DevCleanError, Result};
use std::env;

const REPO_OWNER: &str = "asa321";
const REPO_NAME: &str = "devclean";
const BIN_NAME: &str = "devclean";

pub struct AppUpdater;

impl AppUpdater {
  /// Resolves the current target triple based on compile target.
  pub fn current_target() -> &'static str {
    if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
      "windows-x86_64"
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
      "linux-x86_64"
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
      "macos-x86_64"
    } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
      "macos-arm64"
    } else {
      "unknown"
    }
  }

  /// Checks for the latest release on GitHub without downloading or updating.
  pub fn check_latest() -> Result<()> {
    let current_version = env!("CARGO_PKG_VERSION");
    println!("Current version: v{}", current_version);
    println!(
      "Checking for updates from https://github.com/{}/{}...",
      REPO_OWNER, REPO_NAME
    );

    let mut builder = self_update::backends::github::ReleaseList::configure();
    builder.repo_owner(REPO_OWNER).repo_name(REPO_NAME);

    if let Ok(token) = env::var("GITHUB_TOKEN").or_else(|_| env::var("DEV_CLEAN_GITHUB_TOKEN")) {
      if !token.is_empty() {
        builder.auth_token(&token);
      }
    }

    let release_list = builder
      .build()
      .map_err(|e| DevCleanError::Fatal(format!("Failed to build release list query: {e}")))?;

    let releases = match release_list.fetch() {
      Ok(r) => r.with_current_version(current_version),
      Err(ref e) if e.to_string().contains("404") => {
        println!(
          "\nNo releases found on GitHub (or repository is private and requires GITHUB_TOKEN)."
        );
        return Ok(());
      }
      Err(e) => {
        return Err(DevCleanError::Fatal(format!(
          "Failed to fetch releases: {e}"
        )))
      }
    };

    if let Some(latest) = releases.latest() {
      let latest_version = latest.version().trim_start_matches('v');
      if releases.is_update_available().unwrap_or(false) {
        println!(
          "\nA new version is available: v{} (current: v{})",
          latest_version, current_version
        );
        println!("Run 'devclean --update' to upgrade.");
      } else {
        println!("\ndevclean is up to date (v{}).", current_version);
      }
    } else {
      println!("\nNo releases found on GitHub.");
    }

    Ok(())
  }

  /// Downloads and replaces current binary with the latest release from GitHub.
  pub fn update(no_confirm: bool) -> Result<()> {
    let current_version = env!("CARGO_PKG_VERSION");
    let target_name = Self::current_target();

    println!("Current version: v{}", current_version);
    println!("Target platform: {}", target_name);
    println!(
      "Fetching latest release from https://github.com/{}/{}...",
      REPO_OWNER, REPO_NAME
    );

    let mut builder = self_update::backends::github::Update::configure();
    builder
      .repo_owner(REPO_OWNER)
      .repo_name(REPO_NAME)
      .bin_name(BIN_NAME)
      .target(target_name)
      .show_download_progress(true)
      .no_confirm(no_confirm)
      .current_version(current_version);

    if let Ok(token) = env::var("GITHUB_TOKEN").or_else(|_| env::var("DEV_CLEAN_GITHUB_TOKEN")) {
      if !token.is_empty() {
        builder.auth_token(&token);
      }
    }

    let updater = builder
      .build()
      .map_err(|e| DevCleanError::Fatal(format!("Failed to build updater: {e}")))?;

    let status = updater
      .update()
      .map_err(|e| DevCleanError::Fatal(format!("Update failed: {e}")))?;

    println!("\nSuccessfully updated devclean to v{}!", status.version());

    Ok(())
  }
}

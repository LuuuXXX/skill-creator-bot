use std::path::Path;
use std::process::Command;

use anyhow::Context;
use which::which;

use crate::provider::{PreflightError, PublishMetadata, PublishResult, RegistryProvider};

/// Registry provider that delegates to the `clawhub` CLI tool.
///
/// Install with:  npm i -g clawhub
/// Login with:    clawhub login
pub struct ClawHubProvider;

impl RegistryProvider for ClawHubProvider {
    fn name(&self) -> &str {
        "ClawHub"
    }

    fn preflight(&self) -> Result<(), PreflightError> {
        // 1. Ensure `clawhub` binary is available.
        which("clawhub").map_err(|_| {
            PreflightError::CliNotFound(
                "The `clawhub` CLI is not installed or not found in PATH.\n  \
                 Install it with:  npm i -g clawhub\n  \
                 Then log in with: clawhub login"
                    .to_string(),
            )
        })?;

        // 2. Verify the user is logged in by running `clawhub whoami`.
        let output = Command::new("clawhub")
            .arg("whoami")
            .output()
            .map_err(|e| PreflightError::Other(format!("Failed to run `clawhub whoami`: {}", e)))?;

        if !output.status.success() {
            return Err(PreflightError::NotAuthenticated(
                "You are not logged in to ClawHub.\n\
                 Run `clawhub login` to authenticate, then try again."
                    .to_string(),
            ));
        }

        Ok(())
    }

    fn publish(&self, skill_path: &Path, metadata: &PublishMetadata) -> anyhow::Result<PublishResult> {
        let mut cmd = Command::new("clawhub");
        cmd.arg("publish")
            .arg(skill_path)
            .arg("--slug")
            .arg(&metadata.slug)
            .arg("--name")
            .arg(&metadata.display_name)
            .arg("--version")
            .arg(&metadata.version)
            .arg("--changelog")
            .arg(&metadata.changelog);

        for tag in &metadata.tags {
            cmd.arg("--tag").arg(tag);
        }

        let output = cmd.output().context("Failed to run `clawhub publish`")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("clawhub publish failed:\n{}", stderr.trim());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Best-effort URL extraction: look for a URL in the output.
        let url = extract_url(&stdout);

        Ok(PublishResult {
            url,
            slug: metadata.slug.clone(),
            version: metadata.version.clone(),
        })
    }
}

/// Attempt to extract a URL from `clawhub publish` stdout.
fn extract_url(output: &str) -> Option<String> {
    for word in output.split_whitespace() {
        if word.starts_with("https://") || word.starts_with("http://") {
            return Some(word.to_string());
        }
    }
    None
}

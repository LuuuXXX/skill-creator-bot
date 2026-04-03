use std::path::Path;
use std::process::Command;

use which::which;

use crate::provider::{PreflightError, PublishMetadata, PublishResult, RegistryProvider};

/// Substrings (lowercased) that identify a `clawhub whoami` failure as an
/// authentication error rather than a generic tool/environment failure.
const AUTH_ERROR_SIGNALS: &[&str] = &[
    "not logged",
    "not authenticated",
    "unauthenticated",
    "unauthorized",
    "please login",
    "please log in",
];

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
        which("clawhub").map_err(|_| PreflightError::CliNotFound("clawhub".to_string()))?;

        // 2. Verify the user is logged in by running `clawhub whoami`.
        let output = Command::new("clawhub")
            .arg("whoami")
            .output()
            .map_err(|e| PreflightError::Other(anyhow::Error::from(e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr_str = stderr.trim();
            let stdout_str = stdout.trim();

            // Prefer stderr for diagnostics; fall back to stdout.
            let diagnostic = if !stderr_str.is_empty() {
                stderr_str
            } else {
                stdout_str
            };

            if diagnostic.is_empty() {
                // No output from `clawhub whoami` — most likely the user is not
                // logged in (the typical case for an unauthenticated whoami).
                return Err(PreflightError::NotAuthenticated);
            }

            // Inspect the output for known authentication-error signals.
            // Any other failure (corrupted config, incompatible CLI version,
            // network error) is returned as Other with the raw diagnostic
            // attached so the CLI layer can display it as a technical detail.
            let lower = diagnostic.to_lowercase();
            let is_auth_error = AUTH_ERROR_SIGNALS.iter().any(|s| lower.contains(s));

            if is_auth_error {
                return Err(PreflightError::NotAuthenticated);
            }

            return Err(PreflightError::Other(anyhow::anyhow!("{}", diagnostic)));
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

        let output = cmd.output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = stderr.trim();
            let stdout = stdout.trim();

            if !stderr.is_empty() {
                anyhow::bail!("{}", stderr);
            } else if !stdout.is_empty() {
                anyhow::bail!("{}", stdout);
            } else {
                match output.status.code() {
                    Some(code) => anyhow::bail!("exit {}", code),
                    None => anyhow::bail!("signal"),
                }
            }
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

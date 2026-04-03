use std::path::Path;

/// Metadata required to publish a skill.
#[derive(Debug, Clone)]
pub struct PublishMetadata {
    /// URL-safe slug (e.g. "my-skill").
    pub slug: String,
    /// Human-readable display name.
    pub display_name: String,
    /// Semantic version string.
    pub version: String,
    /// Changelog / release notes.
    pub changelog: String,
    /// Optional tags (e.g. ["latest", "stable"]).
    pub tags: Vec<String>,
}

/// Result returned after a successful publish.
#[derive(Debug)]
pub struct PublishResult {
    /// Registry-provided URL of the published skill (best-effort).
    pub url: Option<String>,
    /// The slug that was published.
    pub slug: String,
    /// The version that was published.
    pub version: String,
}

/// Structured error kinds returned by `preflight()`.
///
/// Using a typed enum lets callers select the right localized message without
/// fragile substring matching on English error text.
///
/// `Other` carries the raw underlying error so the CLI layer can append the
/// OS/tool detail to a localized prefix rather than embedding English strings
/// in the registry layer.
#[derive(Debug)]
pub enum PreflightError {
    /// The required CLI tool is not installed or not in PATH.
    /// Carries the name of the missing command (e.g. `"clawhub"`) as structured data.
    CliNotFound(String),
    /// The user is not authenticated with the registry.
    NotAuthenticated,
    /// Any other preflight failure (carries the raw source error).
    Other(Box<dyn std::error::Error + Send + Sync>),
}

impl std::fmt::Display for PreflightError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CliNotFound(cmd) => write!(f, "CLI not found: {}", cmd),
            Self::NotAuthenticated => write!(f, "not authenticated"),
            Self::Other(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for PreflightError {}

/// Trait that every registry provider must implement.
///
/// Add new registries by implementing this trait in a new module and
/// wiring it up in `scb-cli`.
pub trait RegistryProvider {
    /// Human-readable name of this registry (e.g. "ClawHub").
    fn name(&self) -> &str;

    /// Check that the required CLI / credentials are available.
    /// Returns a typed `PreflightError` on failure so callers can select
    /// the appropriate localized message without brittle string matching.
    fn preflight(&self) -> Result<(), PreflightError>;

    /// Publish the skill at `skill_path` with the given metadata.
    fn publish(&self, skill_path: &Path, metadata: &PublishMetadata) -> anyhow::Result<PublishResult>;
}

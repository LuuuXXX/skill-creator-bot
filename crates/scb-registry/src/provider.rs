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

/// Trait that every registry provider must implement.
///
/// Add new registries by implementing this trait in a new module and
/// wiring it up in `scb-cli`.
pub trait RegistryProvider {
    /// Human-readable name of this registry (e.g. "ClawHub").
    fn name(&self) -> &str;

    /// Check that the required CLI / credentials are available.
    /// Returns an `Err` with a human-readable message if pre-flight fails.
    fn preflight(&self) -> anyhow::Result<()>;

    /// Publish the skill at `skill_path` with the given metadata.
    fn publish(&self, skill_path: &Path, metadata: &PublishMetadata) -> anyhow::Result<PublishResult>;
}

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Top-level project config stored in `scb.project.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Human-readable skill name (matches directory).
    pub skill_name: String,
    /// Path to the skill root directory.
    pub skill_dir: PathBuf,
    /// Default language for UI output.
    #[serde(default)]
    pub lang: crate::i18n::Lang,
    /// Registry configuration.
    #[serde(default)]
    pub registry: RegistryConfig,
    /// Engine configuration.
    #[serde(default)]
    pub engine: EngineConfig,
    /// Last-known publish metadata (populated after `scb publish`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_publish: Option<PublishRecord>,
}

impl ProjectConfig {
    pub fn new(skill_name: impl Into<String>, skill_dir: PathBuf) -> Self {
        let name = skill_name.into();
        Self {
            skill_name: name,
            skill_dir,
            lang: crate::i18n::Lang::ZhCn,
            registry: RegistryConfig::default(),
            engine: EngineConfig::default(),
            last_publish: None,
        }
    }

    /// Load from `scb.project.json` inside the given skill directory.
    ///
    /// The `skill_dir` field is set to the canonicalized form of the
    /// caller-provided path so that a subsequent `save()` always writes to an
    /// absolute, stable location — regardless of what was originally persisted
    /// (e.g. a relative path) and regardless of the working directory from
    /// which `scb` was invoked.
    pub fn load(skill_dir: &std::path::Path) -> anyhow::Result<Self> {
        let path = skill_dir.join("scb.project.json");
        let data = std::fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("Cannot read {}: {}", path.display(), e))?;
        let mut config: Self = serde_json::from_str(&data)
            .map_err(|e| anyhow::anyhow!("Invalid scb.project.json: {}", e))?;
        // Canonicalize the caller-provided path so save() always uses an absolute path.
        config.skill_dir = skill_dir
            .canonicalize()
            .unwrap_or_else(|_| skill_dir.to_path_buf());
        Ok(config)
    }

    /// Save to `scb.project.json` inside the skill directory.
    pub fn save(&self) -> anyhow::Result<()> {
        let path = self.skill_dir.join("scb.project.json");
        let data = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, data)
            .map_err(|e| anyhow::anyhow!("Cannot write {}: {}", path.display(), e))?;
        Ok(())
    }
}

/// Registry/publish settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    /// Default registry provider identifier (e.g. "clawhub").
    pub default_registry: String,
    /// ClawHub-specific settings.
    #[serde(default)]
    pub clawhub: ClawHubConfig,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            default_registry: "clawhub".to_string(),
            clawhub: ClawHubConfig::default(),
        }
    }
}

/// ClawHub registry settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClawHubConfig {
    /// Skill slug on ClawHub (e.g. "my-skill").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    /// Display name shown on the marketplace.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

/// Engine/eval configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    /// Name of the selected engine (e.g. "claude-cli", "custom").
    pub engine: String,
    /// External command or path for running evals.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    /// Additional arguments passed to the engine command.
    #[serde(default)]
    pub args: Vec<String>,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            engine: "claude-cli".to_string(),
            command: None,
            args: Vec::new(),
        }
    }
}

/// Metadata recorded after a successful publish.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishRecord {
    pub registry: String,
    pub slug: String,
    pub version: String,
    pub published_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

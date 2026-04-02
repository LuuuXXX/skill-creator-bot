use serde::{Deserialize, Serialize};

/// Describes a named engine that can execute evals.
///
/// Users select one engine in `scb.project.json → engine.engine`.
/// Additional engines can be added by extending this list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineDefinition {
    /// Short identifier (e.g. "claude-cli", "custom").
    pub id: String,
    /// Human-readable label.
    pub label: String,
    /// Command to invoke (e.g. "claude").
    pub command: String,
    /// Default argument template; `{prompt}` is replaced at runtime.
    pub args_template: Vec<String>,
}

impl EngineDefinition {
    pub fn claude_cli() -> Self {
        Self {
            id: "claude-cli".to_string(),
            label: "Claude CLI".to_string(),
            command: "claude".to_string(),
            args_template: vec!["-p".to_string(), "{prompt}".to_string()],
        }
    }

    pub fn custom(command: &str, args: &[&str]) -> Self {
        Self {
            id: "custom".to_string(),
            label: "Custom Engine".to_string(),
            command: command.to_string(),
            args_template: args.iter().map(|s| s.to_string()).collect(),
        }
    }
}

use serde::{Deserialize, Serialize};

/// A single evaluation item inside `evals/evals.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalItem {
    /// Unique identifier for this eval.
    pub id: String,
    /// The prompt to send to the agent.
    pub prompt: String,
    /// Optional expected output assertions.
    #[serde(default)]
    pub assertions: Vec<String>,
    /// Optional tags (e.g. "should-trigger", "should-not-trigger").
    #[serde(default)]
    pub tags: Vec<String>,
}

/// The top-level structure of `evals/evals.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalsSchema {
    /// Schema version (currently "1").
    pub version: String,
    /// List of eval items.
    pub evals: Vec<EvalItem>,
}

impl Default for EvalsSchema {
    fn default() -> Self {
        Self {
            version: "1".to_string(),
            evals: vec![
                EvalItem {
                    id: "eval-001".to_string(),
                    prompt: "Your test prompt here".to_string(),
                    assertions: Vec::new(),
                    tags: vec!["should-trigger".to_string()],
                },
                EvalItem {
                    id: "eval-002".to_string(),
                    prompt: "Your test prompt here".to_string(),
                    assertions: Vec::new(),
                    tags: vec!["should-trigger".to_string()],
                },
            ],
        }
    }
}

impl EvalsSchema {
    /// Load from a file path.
    ///
    /// Returns the underlying I/O or JSON parse error directly so callers can
    /// add localized context rather than receiving a hard-coded English string.
    pub fn load(path: &std::path::Path) -> anyhow::Result<Self> {
        let data = std::fs::read_to_string(path)?;
        let schema: Self = serde_json::from_str(&data)?;
        Ok(schema)
    }
}

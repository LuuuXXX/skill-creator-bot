use std::path::Path;
use std::process::Command;

use anyhow::Context;
use scb_core::schema::{EvalItem, EvalsSchema};

use crate::config::EngineDefinition;

/// Result of running a single eval item.
#[derive(Debug)]
pub struct EvalRunResult {
    pub eval_id: String,
    pub output: String,
    /// Exit code of the engine process, or `None` if the process was terminated by a signal.
    pub exit_code: Option<i32>,
    pub duration_ms: u128,
}

/// Orchestrates running evals against an engine.
pub struct EvalRunner {
    engine: EngineDefinition,
}

impl EvalRunner {
    pub fn new(engine: EngineDefinition) -> Self {
        Self { engine }
    }

    /// Load and run all evals from `evals/evals.json` in the given skill directory.
    pub fn run_all(&self, skill_dir: &Path) -> anyhow::Result<Vec<EvalRunResult>> {
        let evals_path = skill_dir.join("evals").join("evals.json");
        let schema = EvalsSchema::load(&evals_path)?;

        let mut results = Vec::new();
        for item in &schema.evals {
            let result = self.run_item(item)?;
            results.push(result);
        }
        Ok(results)
    }

    fn run_item(&self, item: &EvalItem) -> anyhow::Result<EvalRunResult> {
        let args: Vec<String> = self
            .engine
            .args_template
            .iter()
            .map(|a| a.replace("{prompt}", &item.prompt))
            .collect();

        let start = std::time::Instant::now();

        let output = Command::new(&self.engine.command)
            .args(&args)
            .output()
            .with_context(|| {
                format!(
                    "Failed to run engine command `{}`.\n\
                     Make sure the engine is installed and available in PATH.",
                    self.engine.command
                )
            })?;

        let duration_ms = start.elapsed().as_millis();
        let exit_code = output.status.code();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        // On failure, append stderr so error diagnostics are not silently dropped.
        let mut combined_output = stdout;
        if exit_code.unwrap_or(1) != 0 && !stderr.is_empty() {
            if !combined_output.is_empty() && !combined_output.ends_with('\n') {
                combined_output.push('\n');
            }
            combined_output.push_str(&stderr);
        }

        Ok(EvalRunResult {
            eval_id: item.id.clone(),
            output: combined_output,
            exit_code,
            duration_ms,
        })
    }
}

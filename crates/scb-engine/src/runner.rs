use std::process::Command;

use scb_core::schema::{EvalItem, EvalsSchema};

use crate::config::EngineDefinition;

/// Typed error returned when the engine process cannot be launched.
///
/// Callers (e.g. `scb-cli`) should match on this to produce a localized
/// user-facing message rather than displaying the raw English context string.
#[derive(Debug)]
pub struct EngineCommandNotFound {
    /// The engine command that could not be found/executed.
    pub command: String,
    /// The underlying OS error.
    pub source: std::io::Error,
}

impl std::fmt::Display for EngineCommandNotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "engine command `{}` could not be executed: {}", self.command, self.source)
    }
}

impl std::error::Error for EngineCommandNotFound {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

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

    /// Run all evals from a pre-loaded schema.
    ///
    /// Callers are expected to load `EvalsSchema` themselves (with localized
    /// error handling) and pass it here, so the file is only read once.
    pub fn run_all(&self, schema: &EvalsSchema) -> anyhow::Result<Vec<EvalRunResult>> {
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
            .map_err(|io_err| EngineCommandNotFound {
                command: self.engine.command.clone(),
                source: io_err,
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

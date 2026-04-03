use clap::{Args, Subcommand};
use colored::Colorize;
use anyhow::Context as _;
use scb_core::i18n::I18n;
use scb_core::schema::EvalsSchema;
use scb_engine::config::EngineDefinition;
use scb_engine::runner::EvalRunner;
use scb_engine::EngineCommandLaunchError;
use std::io::ErrorKind;
use std::path::PathBuf;

#[derive(Args)]
pub struct EvalArgs {
    #[command(subcommand)]
    pub command: EvalCommand,
}

#[derive(Subcommand)]
pub enum EvalCommand {
    /// List all evals in evals/evals.json
    List(EvalListArgs),
    /// Run evals against the configured engine
    Run(EvalRunArgs),
}

#[derive(Args)]
pub struct EvalListArgs {
    /// Path to the skill directory (default: current directory)
    #[arg(short, long, default_value = ".")]
    pub path: PathBuf,
}

#[derive(Args)]
pub struct EvalRunArgs {
    /// Path to the skill directory (default: current directory)
    #[arg(short, long, default_value = ".")]
    pub path: PathBuf,

    /// Engine to use: "claude-cli" or "custom"
    #[arg(long, default_value = "claude-cli", value_parser = ["claude-cli", "custom"])]
    pub engine: String,

    /// Custom command for the engine (required when --engine custom)
    #[arg(long, required_if_eq("engine", "custom"))]
    pub engine_command: Option<String>,

    /// Skip running evals and print the skipped message
    #[arg(long)]
    pub skip: bool,
}

pub fn run(args: EvalArgs, i18n: &I18n) -> anyhow::Result<()> {
    match args.command {
        EvalCommand::List(list_args) => run_list(list_args, i18n),
        EvalCommand::Run(run_args) => run_evals(run_args, i18n),
    }
}

fn run_list(args: EvalListArgs, i18n: &I18n) -> anyhow::Result<()> {
    let evals_path = args.path.join("evals").join("evals.json");
    let schema = EvalsSchema::load(&evals_path)
        .with_context(|| i18n.t("eval.evals_load_failed").into_owned())?;

    println!("{}", i18n.t("eval.list_header").cyan().bold());
    for item in &schema.evals {
        let tags = if item.tags.is_empty() {
            String::new()
        } else {
            format!(" [{}]", item.tags.join(", "))
        };
        println!("  • {} — {}{}", item.id.bold(), item.prompt, tags.dimmed());
    }

    Ok(())
}

fn run_evals(args: EvalRunArgs, i18n: &I18n) -> anyhow::Result<()> {
    if args.skip {
        println!("{}", i18n.t("eval.skipped").dimmed());
        return Ok(());
    }

    let engine_def = build_engine(&args.engine, args.engine_command.as_deref(), i18n)?;

    // Load evals/evals.json once here for localized error reporting; the same
    // schema is passed directly to the runner to avoid redundant I/O.
    let evals_path = args.path.join("evals").join("evals.json");
    let schema = EvalsSchema::load(&evals_path)
        .with_context(|| i18n.t("eval.evals_load_failed").into_owned())?;

    let msg = i18n
        .t("eval.run_start")
        .replace("{engine}", &engine_def.label);
    println!("{}", msg.cyan());

    let runner = EvalRunner::new(engine_def);

    match runner.run_all(&schema) {
        Ok(results) => {
            let mut any_failed = false;
            for r in &results {
                let msg = i18n.t("eval.run_item").replace("{id}", &r.eval_id);
                let success = r.exit_code == Some(0);
                if success {
                    println!("  {} {}", "✔".green(), msg);
                } else {
                    any_failed = true;
                    let code_str = r
                        .exit_code
                        .map(|c| c.to_string())
                        .unwrap_or_else(|| "signal".to_string());
                    let fail = i18n
                        .t("eval.run_failed")
                        .replace("{id}", &r.eval_id)
                        .replace("{error}", &format!("exit code {}", code_str));
                    println!("  {} {}", "✖".red(), fail);
                    if !r.output.trim().is_empty() {
                        for line in r.output.lines() {
                            println!("      {}", line);
                        }
                    }
                }
            }
            if any_failed {
                anyhow::bail!("{}", i18n.t("eval.run_some_failed"));
            }
            println!("{}", i18n.t("eval.run_done").green().bold());
        }
        Err(e) => {
            // Map EngineCommandNotFound to a localized message.
            // Distinguish "binary not in PATH" (NotFound) from other launch
            // failures (e.g. permission denied, invalid executable) so the
            // user gets an accurate diagnosis.
            if let Some(not_found) = e.downcast_ref::<EngineCommandLaunchError>() {
                let msg = if not_found.source.kind() == ErrorKind::NotFound {
                    i18n.t("eval.engine_not_found")
                        .replace("{command}", &not_found.command)
                } else {
                    i18n.t("eval.engine_launch_failed")
                        .replace("{command}", &not_found.command)
                        .replace("{error}", &not_found.source.to_string())
                };
                anyhow::bail!(msg);
            }
            return Err(e);
        }
    }

    Ok(())
}

fn build_engine(engine_id: &str, custom_cmd: Option<&str>, i18n: &I18n) -> anyhow::Result<EngineDefinition> {
    match engine_id {
        "claude-cli" => Ok(EngineDefinition::claude_cli()),
        "custom" => {
            let cmd = custom_cmd.ok_or_else(|| {
                anyhow::anyhow!("{}", i18n.t("eval.engine_missing_command"))
            })?;
            Ok(EngineDefinition::custom(cmd, &["{prompt}"]))
        }
        _ => anyhow::bail!("unknown engine id: {}", engine_id),
    }
}

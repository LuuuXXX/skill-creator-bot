use clap::{Args, Subcommand};
use colored::Colorize;
use scb_core::i18n::I18n;
use scb_core::schema::EvalsSchema;
use scb_engine::config::EngineDefinition;
use scb_engine::runner::EvalRunner;
use scb_engine::EngineCommandNotFound;
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

    /// Custom command for the engine (used when --engine custom)
    #[arg(long)]
    pub engine_command: Option<String>,

    /// Skip running evals (print summary only)
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
        .map_err(|_| anyhow::anyhow!("{}", i18n.t("eval.evals_load_failed")))?;

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
        println!("{}", i18n.t("eval.not_implemented").yellow());
        println!("{}", i18n.t("eval.skip_hint").dimmed());
        return Ok(());
    }

    let engine_def = build_engine(&args.engine, args.engine_command.as_deref(), i18n)?;

    // Verify evals/evals.json is loadable before starting the engine run,
    // so the user gets a localized error rather than an English anyhow string.
    let evals_path = args.path.join("evals").join("evals.json");
    EvalsSchema::load(&evals_path)
        .map_err(|_| anyhow::anyhow!("{}", i18n.t("eval.evals_load_failed")))?;

    let msg = i18n
        .t("eval.run_start")
        .replace("{engine}", &engine_def.label);
    println!("{}", msg.cyan());

    let runner = EvalRunner::new(engine_def);

    match runner.run_all(&args.path) {
        Ok(results) => {
            for r in &results {
                let msg = i18n.t("eval.run_item").replace("{id}", &r.eval_id);
                let success = r.exit_code == Some(0);
                if success {
                    println!("  {} {}", "✔".green(), msg);
                } else {
                    let code_str = r
                        .exit_code
                        .map(|c| c.to_string())
                        .unwrap_or_else(|| "signal".to_string());
                    let fail = i18n
                        .t("eval.run_failed")
                        .replace("{id}", &r.eval_id)
                        .replace("{error}", &format!("exit code {}", code_str));
                    println!("  {} {}", "✖".red(), fail);
                }
            }
            println!("{}", i18n.t("eval.run_done").green().bold());
        }
        Err(e) => {
            // If the engine binary was not found, produce a localized error
            // message; otherwise propagate the raw error to main's handler.
            if let Some(not_found) = e.downcast_ref::<EngineCommandNotFound>() {
                let msg = i18n
                    .t("eval.engine_not_found")
                    .replace("{command}", &not_found.command);
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
        // clap's value_parser restricts --engine to "claude-cli" | "custom", so
        // the "custom" arm is the only other reachable case here.
        _ => {
            let cmd = custom_cmd.ok_or_else(|| {
                anyhow::anyhow!("{}", i18n.t("eval.engine_missing_command"))
            })?;
            Ok(EngineDefinition::custom(cmd, &["{prompt}"]))
        }
    }
}

use clap::Args;
use colored::Colorize;
use scb_core::i18n::I18n;
use scb_core::validate::{validate_skill, ValidationIssue};
use std::path::PathBuf;

#[derive(Args)]
pub struct ValidateArgs {
    /// Path to the skill directory (default: current directory)
    #[arg(short, long, default_value = ".")]
    pub path: PathBuf,
}

/// Render a `ValidationIssue` to a localized string.
fn render_issue(issue: &ValidationIssue, i18n: &I18n) -> String {
    let template = i18n.t(issue.i18n_key());
    if let Some(detail) = issue.detail() {
        template.replace("{detail}", detail)
    } else {
        template.to_string()
    }
}

pub fn run(args: ValidateArgs, i18n: &I18n) -> anyhow::Result<()> {
    println!("{}", i18n.t("validate.checking").cyan());

    let result = validate_skill(&args.path)?;

    if !result.warnings.is_empty() {
        println!("{}", i18n.t("validate.warnings").yellow().bold());
        for w in &result.warnings {
            println!("  {} {}", "⚠".yellow(), render_issue(w, i18n));
        }
    }

    if result.errors.is_empty() {
        println!("{}", i18n.t("validate.ok").green().bold());
    } else {
        println!("{}", i18n.t("validate.errors").red().bold());
        for e in &result.errors {
            println!("  {} {}", "✖".red(), render_issue(e, i18n));
        }
        println!("{}", i18n.t("validate.failed").red());
        anyhow::bail!("validation failed");
    }

    Ok(())
}

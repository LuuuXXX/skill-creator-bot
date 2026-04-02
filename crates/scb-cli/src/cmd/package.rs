use clap::Args;
use colored::Colorize;
use scb_core::i18n::I18n;
use std::path::PathBuf;
use which::which;

#[derive(Args)]
pub struct PackageArgs {
    /// Path to the skill directory (default: current directory)
    #[arg(short, long, default_value = ".")]
    pub path: PathBuf,
}

pub fn run(args: PackageArgs, i18n: &I18n) -> anyhow::Result<()> {
    println!("{}", i18n.t("package.not_implemented").yellow().bold());

    // Detect Python availability and give guidance.
    match which("python3").or_else(|_| which("python")) {
        Ok(python_path) => {
            let msg = i18n
                .t("package.python_found")
                .replace("{python}", &python_path.to_string_lossy());
            println!("{}", msg.green());
        }
        Err(_) => {
            println!("{}", i18n.t("package.python_not_found").red());
        }
    }

    let hint = i18n
        .t("package.python_hint")
        .replace("{path}", &args.path.to_string_lossy());
    println!("\n{}", hint.dimmed());

    Ok(())
}

mod cmd;

use clap::{Parser, Subcommand};
use cmd::{eval, init, package, publish, validate};
use scb_core::i18n::{I18n, Lang};

#[derive(Parser)]
#[command(
    name = "scb",
    about = "Skill Creator Bot — create, validate, and publish Claude skills",
    version
)]
struct Cli {
    /// UI language (zh-CN or en-US). Default: zh-CN
    #[arg(long, global = true, default_value = "zh-CN")]
    lang: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new skill project
    Init(init::InitArgs),
    /// Validate an existing skill directory
    Validate(validate::ValidateArgs),
    /// Publish a skill to a registry (default: ClawHub)
    Publish(publish::PublishArgs),
    /// Package a skill into a .skill bundle (stub)
    Package(package::PackageArgs),
    /// Manage and run skill evaluations
    Eval(eval::EvalArgs),
}

fn main() {
    let cli = Cli::parse();

    let lang: Lang = cli.lang.parse().unwrap_or_default();
    let i18n = I18n::load(&lang);

    let result = match cli.command {
        Commands::Init(args) => init::run(args, &i18n),
        Commands::Validate(args) => validate::run(args, &i18n),
        Commands::Publish(args) => publish::run(args, &i18n),
        Commands::Package(args) => package::run(args, &i18n),
        Commands::Eval(args) => eval::run(args, &i18n),
    };

    if let Err(e) = result {
        eprintln!("{}: {}", colored::Colorize::red("error"), e);
        std::process::exit(1);
    }
}

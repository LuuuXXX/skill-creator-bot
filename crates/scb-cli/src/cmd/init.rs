use clap::Args;
use colored::Colorize;
use scb_core::i18n::I18n;
use scb_core::template::scaffold_skill;
use std::path::PathBuf;

#[derive(Args)]
pub struct InitArgs {
    /// Name of the skill to create
    pub skill_name: String,

    /// Base directory in which to create the skill folder (default: current directory)
    #[arg(short, long, default_value = ".")]
    pub dir: PathBuf,
}

pub fn run(args: InitArgs, i18n: &I18n) -> anyhow::Result<()> {
    let skill_dir = args.dir.join(&args.skill_name);

    if skill_dir.exists() {
        println!(
            "{} {}",
            i18n.t("init.already_exists").yellow(),
            skill_dir.display()
        );
        anyhow::bail!(
            "Directory '{}' already exists. Remove it or choose a different name.",
            skill_dir.display()
        );
    }

    println!("{}", i18n.t("init.creating").cyan());

    let created = scaffold_skill(&args.skill_name, &args.dir, i18n.lang())?;

    println!(
        "{} {}",
        i18n.t("init.success").green().bold(),
        created.display()
    );

    let hint = i18n
        .t("init.hint_next")
        .replace("{path}", &created.to_string_lossy());
    println!("\n{}", hint.dimmed());

    Ok(())
}

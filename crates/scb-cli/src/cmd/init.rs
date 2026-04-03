use clap::Args;
use colored::Colorize;
use scb_core::i18n::I18n;
use scb_core::template::scaffold_skill;
use std::path::PathBuf;

/// Validate that a skill name is a single plain directory name with no path
/// separators, `.`, or `..` components — preventing accidental directory
/// traversal when the name is joined to a base directory.
///
/// Note: this runs at clap argument-parse time, before `--lang` is resolved,
/// so the rejection message is always in English (same constraint as the other
/// `value_parser` validators in this binary).
fn validate_skill_name(s: &str) -> Result<String, String> {
    use std::path::Component;
    let mut components = std::path::Path::new(s).components();
    match (components.next(), components.next()) {
        (Some(Component::Normal(_)), None) => Ok(s.to_string()),
        _ => Err(format!(
            "invalid skill name '{}': must be a single plain directory name (no '/', '\\', or '..')",
            s
        )),
    }
}

#[derive(Args)]
pub struct InitArgs {
    /// Name of the skill to create (must be a single directory name, no '/', '\\', or '..')
    #[arg(value_parser = validate_skill_name)]
    pub skill_name: String,

    /// Base directory in which to create the skill folder (default: current directory)
    #[arg(short, long, default_value = ".")]
    pub dir: PathBuf,
}

pub fn run(args: InitArgs, i18n: &I18n) -> anyhow::Result<()> {
    let skill_dir = args.dir.join(&args.skill_name);

    println!("{}", i18n.t("init.creating").cyan());

    let created = scaffold_skill(&args.skill_name, &args.dir, i18n.lang()).map_err(|e| {
        // If the underlying IO error is AlreadyExists, surface a localized message.
        if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
            if io_err.kind() == std::io::ErrorKind::AlreadyExists {
                return anyhow::anyhow!(
                    "{} {}",
                    i18n.t("init.already_exists"),
                    skill_dir.display()
                );
            }
        }
        e
    })?;

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

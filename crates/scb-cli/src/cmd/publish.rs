use clap::Args;
use colored::Colorize;
use scb_core::i18n::I18n;
use scb_core::project::{ProjectConfig, PublishRecord};
use scb_registry::provider::{PreflightError, PublishMetadata, RegistryProvider};
use scb_registry::ClawHubProvider;
use std::path::PathBuf;

#[derive(Args)]
pub struct PublishArgs {
    /// Path to the skill directory (default: current directory)
    #[arg(short, long, default_value = ".")]
    pub path: PathBuf,

    /// Registry to publish to (default: clawhub)
    #[arg(long, default_value = "clawhub")]
    pub registry: String,

    /// URL-safe slug for the skill (e.g. my-skill)
    #[arg(long)]
    pub slug: String,

    /// Human-readable display name
    #[arg(long)]
    pub name: String,

    /// Semantic version (e.g. 1.0.0)
    #[arg(long)]
    pub version: String,

    /// Changelog / release notes
    #[arg(long)]
    pub changelog: String,

    /// Optional tags (repeatable)
    #[arg(long = "tag")]
    pub tags: Vec<String>,
}

pub fn run(args: PublishArgs, i18n: &I18n) -> anyhow::Result<()> {
    let provider: Box<dyn RegistryProvider> = match args.registry.to_lowercase().as_str() {
        "clawhub" => Box::new(ClawHubProvider),
        other => anyhow::bail!("Unknown registry '{}'. Supported: clawhub", other),
    };

    // Pre-flight checks
    println!("{}", i18n.t("publish.preflight").cyan());
    if let Err(e) = provider.preflight() {
        // Match on the structured error kind to select the appropriate localized message.
        match &e {
            PreflightError::CliNotFound(_) => {
                println!("{}", i18n.t("publish.clawhub_not_found").yellow());
            }
            PreflightError::NotAuthenticated(_) => {
                println!("{}", i18n.t("publish.not_logged_in").yellow());
            }
            PreflightError::Other(msg) => {
                println!("{} {}", i18n.t("publish.failed").red().bold(), msg);
            }
        }
        anyhow::bail!("pre-flight failed");
    }

    let registry_label = provider.name().to_string();
    let publishing_msg = i18n
        .t("publish.publishing")
        .replace("{registry}", &registry_label);
    println!("{}", publishing_msg.cyan());

    let metadata = PublishMetadata {
        slug: args.slug.clone(),
        display_name: args.name.clone(),
        version: args.version.clone(),
        changelog: args.changelog.clone(),
        tags: args.tags.clone(),
    };

    match provider.publish(&args.path, &metadata) {
        Ok(result) => {
            println!("{}", i18n.t("publish.success").green().bold());
            if let Some(url) = &result.url {
                println!("{}", i18n.t("publish.url").replace("{url}", url));
            } else {
                println!("{}", i18n.t("publish.slug").replace("{slug}", &result.slug));
            }

            // Persist publish metadata
            if let Ok(mut config) = ProjectConfig::load(&args.path) {
                config.last_publish = Some(PublishRecord {
                    registry: args.registry.clone(),
                    slug: result.slug.clone(),
                    version: result.version.clone(),
                    published_at: unix_epoch_seconds_now(),
                    url: result.url.clone(),
                });
                if config.save().is_ok() {
                    println!("{}", i18n.t("publish.saved").dimmed());
                }
            }
        }
        Err(e) => {
            println!("{} {}", i18n.t("publish.failed").red().bold(), e);
            anyhow::bail!("publish failed");
        }
    }

    Ok(())
}

/// Return the current time as a UNIX epoch seconds string.
fn unix_epoch_seconds_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{}", secs)
}

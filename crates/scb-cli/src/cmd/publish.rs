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
    // Normalize registry to lowercase once; this is the canonical id persisted
    // in scb.project.json and used for display/dispatch throughout.
    let registry_id = args.registry.to_lowercase();
    let provider: Box<dyn RegistryProvider> = match registry_id.as_str() {
        "clawhub" => Box::new(ClawHubProvider),
        other => {
            let msg = i18n
                .t("publish.unknown_registry")
                .replace("{registry}", other);
            anyhow::bail!(msg);
        }
    };

    // Pre-flight checks
    println!("{}", i18n.t("publish.preflight").cyan());
    if let Err(e) = provider.preflight() {
        // Build a single localized error message and propagate it — avoids
        // printing a user-facing line here *and* a second non-localized line
        // from main's error handler. Raw OS/provider detail is indented so
        // the primary localized line is always the first thing the user sees.
        let localized = match &e {
            PreflightError::CliNotFound(_) => i18n.t("publish.clawhub_not_found").to_string(),
            PreflightError::NotAuthenticated => i18n.t("publish.not_logged_in").to_string(),
            PreflightError::Other(msg) => format!("{}\n  {}", i18n.t("publish.failed"), msg),
        };
        anyhow::bail!(localized);
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

            // Persist publish metadata; warn if load or save fails so the user
            // knows to fix the config and can recover the metadata manually.
            match ProjectConfig::load(&args.path) {
                Ok(mut config) => {
                    config.last_publish = Some(PublishRecord {
                        // Persist the canonical lowercase id rather than the
                        // user-supplied casing so future comparisons are consistent.
                        registry: registry_id.clone(),
                        slug: result.slug.clone(),
                        version: result.version.clone(),
                        published_at: unix_epoch_seconds_now(),
                        url: result.url.clone(),
                    });
                    if let Err(e) = config.save() {
                        eprintln!("{}: {}", i18n.t("publish.save_failed"), e);
                    } else {
                        println!("{}", i18n.t("publish.saved").dimmed());
                    }
                }
                Err(e) => {
                    eprintln!("{}: {}", i18n.t("publish.load_failed"), e);
                }
            }
        }
        Err(e) => {
            // Keep the localized prefix as the primary error line; append the
            // raw provider/OS detail as an indented line so users see a
            // translated message first and the technical detail below.
            anyhow::bail!("{}\n  {}", i18n.t("publish.failed"), e);
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

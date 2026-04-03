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
    // Validate --version as SemVer now that i18n is available, so the rejection
    // message is fully localized rather than the English-only parse-time error
    // that a clap value_parser would emit.
    if let Err(e) = semver::Version::parse(&args.version) {
        let msg = i18n
            .t("publish.invalid_version")
            .replace("{version}", &args.version)
            .replace("{detail}", &e.to_string());
        anyhow::bail!(msg);
    }

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
        // For known variants the localized message is self-contained; attaching
        // the PreflightError as a source would only add raw variant-name noise.
        // For Other, preserve the underlying OS/tool error as a chain source so
        // main's layered display can show it as an indented technical detail.
        return Err(match e {
            PreflightError::CliNotFound(cmd) => {
                anyhow::anyhow!(i18n.t("publish.cli_not_found").replace("{command}", &cmd))
            }
            PreflightError::NotAuthenticated => {
                anyhow::anyhow!(i18n.t("publish.not_logged_in").into_owned())
            }
            PreflightError::Other(err) => err.context(i18n.t("publish.failed").into_owned()),
        });
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
            // Wrap the raw provider error with the localized summary as context
            // so main can print the localized message as the primary line and
            // the technical cause (provider/OS detail) as indented follow-on.
            return Err(e.context(i18n.t("publish.failed").to_string()));
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

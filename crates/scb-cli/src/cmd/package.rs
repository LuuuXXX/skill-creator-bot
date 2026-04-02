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
    // `python3` is definitely Python 3; `python` may be Python 2 or 3, so we
    // verify by running it with `--version` before printing "Python 3 detected".
    let python_result = which("python3")
        .map(|p| (p, true)) // python3 → confirmed Python 3
        .or_else(|_| {
            which("python").map(|p| {
                // Run `python --version` and check whether it reports Python 3.
                let is_v3 = std::process::Command::new(&p)
                    .arg("--version")
                    .output()
                    .map(|out| {
                        // Python 3 prints "Python 3.x.y" on stdout or stderr.
                        let out_text = String::from_utf8_lossy(&out.stdout);
                        let err_text = String::from_utf8_lossy(&out.stderr);
                        out_text.starts_with("Python 3") || err_text.starts_with("Python 3")
                    })
                    .unwrap_or(false);
                (p, is_v3)
            })
        });

    match python_result {
        Ok((python_path, true)) => {
            let msg = i18n
                .t("package.python_found")
                .replace("{python}", &python_path.to_string_lossy());
            println!("{}", msg.green());
        }
        Ok((_, false)) => {
            println!("{}", i18n.t("package.python_not_found").red());
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

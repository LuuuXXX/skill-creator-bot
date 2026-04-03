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

    // Determine which Python 3 command was found to use in the packaging hint.
    let python_cmd = match &python_result {
        Ok((path, true)) => path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("python3")
            .to_string(),
        _ => "python3".to_string(),
    };

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

    // Canonicalize the skill path so the printed command is correct regardless
    // of the user's working directory.  Fall back to the original path when the
    // directory does not yet exist (e.g. used with a future `--path` value).
    let skill_dir = std::fs::canonicalize(&args.path).unwrap_or_else(|_| args.path.clone());
    let script_path = skill_dir.join("scripts").join("package_skill.py");
    // Wrap the script path in double quotes so the suggested command is
    // copy/paste-able even when the path contains spaces. Only escape embedded
    // double quotes; backslashes are kept as-is so Windows paths remain valid
    // (e.g. `C:\Users\…` must not be doubled to `C:\\Users\\…`).
    let path_str = script_path.to_string_lossy();
    let escaped_path = path_str.replace('"', "\\\"");
    let quoted_path = format!("\"{}\"", escaped_path);
    let hint = i18n
        .t("package.python_hint")
        .replace("{path}", &quoted_path)
        .replace("{python_cmd}", &python_cmd);
    println!("\n{}", hint.dimmed());

    Ok(())
}

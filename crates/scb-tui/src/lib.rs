// scb-tui: Linux TUI for the Skill Creator Bot.
//
// TODO: Implement a Textual-style terminal user interface for Linux.
//       This crate is intentionally left as a stub in this PR.
//       Future implementation should provide:
//         - Wizard-style skill creation forms
//         - Eval management and progress display
//         - Publish flow with live log output
//
// The TUI will share all domain logic through scb-core, scb-registry, and scb-engine.

/// Placeholder: launch the TUI application.
/// Returns an error indicating the TUI is not yet implemented.
pub fn run() -> anyhow::Result<()> {
    Err(anyhow::anyhow!(
        "The Linux TUI is not yet implemented. Use `scb --help` for the CLI interface."
    ))
}

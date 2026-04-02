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

/// Error type for the `scb-tui` crate.
#[derive(Debug)]
pub enum TuiError {
    /// The TUI has not been implemented yet.
    NotImplemented,
}

impl std::fmt::Display for TuiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TuiError::NotImplemented => write!(f, "tui_not_implemented"),
        }
    }
}

impl std::error::Error for TuiError {}

/// Placeholder: launch the TUI application.
/// Currently returns a structured error indicating the TUI is not yet implemented.
pub fn run() -> anyhow::Result<()> {
    Err(TuiError::NotImplemented.into())
}

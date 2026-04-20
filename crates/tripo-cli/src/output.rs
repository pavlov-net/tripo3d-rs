//! Output formatting: TTY vs JSON.

use is_terminal::IsTerminal;

/// Whether the CLI should emit JSON to stdout.
///
/// Forced on by `--json`; otherwise auto-detected based on whether stdout is a TTY.
#[allow(dead_code)] // Consumed by subcommands added in Tasks 4+.
pub fn use_json(arg_json: bool) -> bool {
    arg_json || !std::io::stdout().is_terminal()
}

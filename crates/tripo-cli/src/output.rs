//! Output formatting: TTY vs JSON.

/// Whether the CLI should emit JSON to stdout.
///
/// Controlled by `--json`. We don't auto-detect piped stdout because
/// human text output is often more useful when piped to pagers / less / grep.
pub fn use_json(arg_json: bool) -> bool {
    arg_json
}

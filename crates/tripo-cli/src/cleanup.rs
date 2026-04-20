//! Best-effort cleanup of `.partial` files left behind by interrupted downloads.

use std::path::Path;

/// Remove any files in `dir` whose extension ends with `partial`.
///
/// Errors are swallowed — this is a best-effort cleanup after SIGINT.
pub async fn partial_files(dir: &Path) {
    let Ok(mut entries) = tokio::fs::read_dir(dir).await else {
        return;
    };
    while let Ok(Some(e)) = entries.next_entry().await {
        let p = e.path();
        if p.extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.ends_with("partial"))
        {
            let _ = tokio::fs::remove_file(p).await;
        }
    }
}

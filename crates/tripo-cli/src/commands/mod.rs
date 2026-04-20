// Expanded in Task 4.

pub mod balance;
pub mod completions;
pub mod task;
pub mod upload;

#[allow(clippy::unused_async)] // Becomes `async` in Task 4.
pub async fn dispatch(_args: crate::cli::Cli) -> anyhow::Result<()> {
    anyhow::bail!("unimplemented — see Task 4")
}

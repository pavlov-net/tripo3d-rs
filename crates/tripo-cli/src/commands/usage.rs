//! `usage` subcommand.

use anyhow::Result;

use crate::cli::GlobalArgs;

/// `usage` arguments.
#[derive(Debug, clap::Args)]
pub struct UsageArgs {
    /// Maximum number of records to return (server default when unset).
    #[arg(long)]
    pub limit: Option<u32>,
    /// Number of records to skip.
    #[arg(long)]
    pub offset: Option<u32>,
}

/// Run `usage`: print per-task credit consumption as JSON or human text.
pub async fn run(g: &GlobalArgs, a: UsageArgs) -> Result<()> {
    let client = crate::resolve::build_client(g)?;
    let records = client
        .get_usage(tripo_api::UsageQuery {
            limit: a.limit,
            offset: a.offset,
        })
        .await?;
    if g.json {
        serde_json::to_writer_pretty(std::io::stdout(), &records)?;
        println!();
        return Ok(());
    }
    for r in &records {
        let field = |v: Option<String>| v.unwrap_or_else(|| "-".into());
        println!(
            "{}  {}  {}  {}  {}",
            field(r.created_at.as_ref().map(ToString::to_string)),
            field(r.task_id.as_ref().map(ToString::to_string)),
            field(r.task_type.clone()),
            field(r.status.clone()),
            field(r.credits_consumed.map(|c| format!("{c:.2}"))),
        );
    }
    Ok(())
}

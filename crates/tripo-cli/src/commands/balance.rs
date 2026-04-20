//! `balance` subcommand.

use anyhow::Result;

use crate::cli::GlobalArgs;

/// Run `balance`: print the account balance as JSON or human text.
pub async fn run(g: &GlobalArgs) -> Result<()> {
    let client = crate::resolve::build_client(g)?;
    let bal = client.get_balance().await?;
    if g.json {
        serde_json::to_writer_pretty(std::io::stdout(), &bal)?;
        println!();
    } else {
        println!("balance: {:.2}", bal.balance);
        println!("frozen:  {:.2}", bal.frozen);
    }
    Ok(())
}

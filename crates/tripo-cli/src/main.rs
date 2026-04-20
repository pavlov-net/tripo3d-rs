//! `tripo` — command-line client for the Tripo 3D Generation API.

mod cli;
mod commands;
mod exit;
mod output;
mod progress;
mod resolve;
mod signals;

use clap::Parser;

use crate::cli::Cli;
use crate::exit::ExitCode;

#[tokio::main]
async fn main() -> std::process::ExitCode {
    let args = Cli::parse();
    init_tracing(&args.global);
    signals::install_global();
    let code = match run(args).await {
        Ok(()) => ExitCode::Success,
        Err(err) => exit::code_for_error(&err),
    };
    std::process::ExitCode::from(code as u8)
}

async fn run(args: Cli) -> anyhow::Result<()> {
    commands::dispatch(args).await
}

fn init_tracing(g: &crate::cli::GlobalArgs) {
    use tracing_subscriber::{EnvFilter, fmt, prelude::*};
    let level = if g.quiet {
        "warn"
    } else if g.verbose >= 2 {
        "trace"
    } else if g.verbose == 1 {
        "debug"
    } else {
        "info"
    };
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("tripo=info,tripo_api={level}")));
    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_writer(std::io::stderr)
                .with_target(false)
                .with_ansi(!g.no_color),
        )
        .with(filter)
        .init();
}

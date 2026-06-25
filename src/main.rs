use clap::Parser;
use jswitch::{Cli, Result, commands};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    commands::dispatch(cli).await
}

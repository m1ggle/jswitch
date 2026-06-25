use clap::Parser;
use jswitch::{Cli, Result, commands, utils::logging};

#[tokio::main]
async fn main() -> Result<()> {
    logging::init_logging();
    let cli = Cli::parse();
    commands::dispatch(cli).await
}

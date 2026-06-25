use jswitch::{commands, Cli, JswitchError};
use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), JswitchError> {
    let cli = Cli::parse();
    commands::dispatch(cli).await
}

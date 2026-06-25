use clap::Args;
use serde::{Deserialize, Serialize};

use crate::JswitchError;

#[derive(Debug, Clone, Args, Serialize, Deserialize)]
pub struct ListArgs {
    #[arg(long)]
    pub installed: bool,

    #[arg(long)]
    pub remote: bool,

    #[arg(long)]
    pub verbose: bool,
}

pub async fn run(args: ListArgs) -> Result<(), JswitchError> {
    println!(
        "list Java versions, installed: {}, remote: {}, verbose: {}",
        args.installed, args.remote, args.verbose
    );
    Ok(())
}

use clap::Args;
use serde::{Deserialize, Serialize};

use crate::Result;

#[derive(Debug, Clone, Args, Serialize, Deserialize)]
pub struct SwitchArgs {
    pub version: Option<String>,

    #[arg(long)]
    pub global: bool,

    #[arg(long)]
    pub local: bool,

    #[arg(long)]
    pub session: bool,
}

pub async fn run(args: SwitchArgs) -> Result<()> {
    println!(
        "switch Java version: {:?}, global: {}, local: {}, session: {}",
        args.version, args.global, args.local, args.session
    );
    Ok(())
}

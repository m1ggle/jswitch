use clap::Args;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Args, Serialize, Deserialize)]
pub struct RemoveArgs {
    pub version: String,

    #[arg(long)]
    pub force: bool,
}

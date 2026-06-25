use clap::Args;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Args, Serialize, Deserialize)]
pub struct SelfUpdateArgs {
    #[arg(long)]
    pub check_only: bool,
}

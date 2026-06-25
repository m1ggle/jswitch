use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Args, Serialize, Deserialize)]
pub struct AliasArgs {
    #[command(subcommand)]
    pub action: AliasAction,
}

#[derive(Debug, Clone, Subcommand, Serialize, Deserialize)]
pub enum AliasAction {
    Set { name: String, version: String },
    Remove { name: String },
    List,
}

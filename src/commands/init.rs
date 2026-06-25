use clap::Args;
use serde::{Deserialize, Serialize};

use crate::JswitchError;

#[derive(Debug, Clone, Args, Serialize, Deserialize)]
pub struct InitArgs {
    #[arg(value_enum)]
    pub shell: Option<Shell>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, clap::ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
}

pub async fn run(args: InitArgs) -> Result<(), JswitchError> {
    println!("initialize shell integration: {:?}", args.shell);
    Ok(())
}

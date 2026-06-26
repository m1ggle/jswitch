use clap::Args;
use serde::{Deserialize, Serialize};

use crate::{
    Result,
    env::{Shell, hook::init_script},
};

#[derive(Debug, Clone, Args, Serialize, Deserialize)]
pub struct InitArgs {
    #[arg(value_enum)]
    pub shell: Option<Shell>,
}

pub async fn run(args: InitArgs) -> Result<()> {
    match args.shell {
        Some(shell) => {
            print!("{}", init_script(shell));
        }
        None => {
            eprintln!("usage: jswitch init <shell>");
            eprintln!("supported shells: bash, zsh, fish, powershell");
        }
    }
    Ok(())
}

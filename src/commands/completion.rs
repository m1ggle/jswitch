use std::io;

use clap::{Args, ValueEnum};
use clap_complete::{Shell, generate};
use serde::{Deserialize, Serialize};

use crate::{Cli, Result};

#[derive(Debug, Clone, Args, Serialize, Deserialize)]
pub struct CompletionArgs {
    #[arg(value_enum)]
    pub shell: CompletionShell,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CompletionShell {
    Bash,
    Elvish,
    Fish,
    PowerShell,
    Zsh,
}

impl From<CompletionShell> for Shell {
    fn from(shell: CompletionShell) -> Self {
        match shell {
            CompletionShell::Bash => Self::Bash,
            CompletionShell::Elvish => Self::Elvish,
            CompletionShell::Fish => Self::Fish,
            CompletionShell::PowerShell => Self::PowerShell,
            CompletionShell::Zsh => Self::Zsh,
        }
    }
}

pub async fn run(args: CompletionArgs) -> Result<()> {
    let mut command = <Cli as clap::CommandFactory>::command();
    generate(
        Shell::from(args.shell),
        &mut command,
        "jswitch",
        &mut io::stdout(),
    );
    Ok(())
}

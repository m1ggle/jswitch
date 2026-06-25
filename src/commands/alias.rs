use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};

use crate::{Result, config::Config};

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

pub async fn run(args: AliasArgs) -> Result<()> {
    match args.action {
        AliasAction::Set { name, version } => {
            let mut config = Config::load_or_default()?;
            config.aliases.insert(name.clone(), version.clone());
            config.save()?;
            println!("set alias {name} to {version}");
        }
        AliasAction::Remove { name } => {
            let mut config = Config::load_or_default()?;
            config.aliases.remove(&name);
            config.save()?;
            println!("removed alias {name}");
        }
        AliasAction::List => {
            let config = Config::load_or_default()?;
            if config.aliases.is_empty() {
                println!("no aliases configured");
            } else {
                for (name, version) in config.aliases {
                    println!("{name}={version}");
                }
            }
        }
    }

    Ok(())
}

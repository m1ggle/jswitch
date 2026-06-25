use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};

use crate::{Result, config::Config};

#[derive(Debug, Clone, Args, Serialize, Deserialize)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub action: ConfigAction,
}

#[derive(Debug, Clone, Subcommand, Serialize, Deserialize)]
pub enum ConfigAction {
    Show,
    Get { key: String },
    Set { key: String, value: String },
    List,
}

pub async fn run(args: ConfigArgs) -> Result<()> {
    match args.action {
        ConfigAction::Show => {
            let config = Config::load_or_default()?;
            print_config(&config);
        }
        ConfigAction::Get { key } => {
            let config = Config::load_or_default()?;
            if let Some(value) = config.get(&key)? {
                println!("{value}");
            }
        }
        ConfigAction::Set { key, value } => {
            let mut config = Config::load_or_default()?;
            config.set(&key, value)?;
            config.save()?;
            println!("updated {key}");
        }
        ConfigAction::List => {
            let config = Config::load_or_default()?;
            for (key, value) in config.entries() {
                println!("{key}={value}");
            }
        }
    }

    Ok(())
}

fn print_config(config: &Config) {
    for (key, value) in config.entries() {
        println!("{key}: {value}");
    }
}

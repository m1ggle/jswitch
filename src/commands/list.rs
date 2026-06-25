use clap::Args;
use serde::{Deserialize, Serialize};

use crate::{Result, version::VersionManager};

#[derive(Debug, Clone, Args, Serialize, Deserialize)]
pub struct ListArgs {
    #[arg(long)]
    pub installed: bool,

    #[arg(long)]
    pub remote: bool,

    #[arg(long)]
    pub verbose: bool,
}

pub async fn run(args: ListArgs) -> Result<()> {
    if args.remote {
        println!("remote version listing is not implemented yet");
        return Ok(());
    }

    let manager = VersionManager::from_default_root()?;
    let versions = manager.list_installed()?;

    if versions.is_empty() {
        println!("no installed Java versions");
        return Ok(());
    }

    for version in versions {
        if args.verbose {
            println!(
                "{}\t{}",
                version,
                manager.versions_dir().join(&version.value).display()
            );
        } else {
            println!("{version}");
        }
    }

    Ok(())
}

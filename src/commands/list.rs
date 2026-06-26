use std::collections::HashSet;

use clap::Args;
use serde::{Deserialize, Serialize};

use crate::{Result, config::Config, download::VersionFetcher, version::VersionManager};

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
        return list_remote(&args).await;
    }

    list_installed(&args)
}

fn list_installed(args: &ListArgs) -> Result<()> {
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

async fn list_remote(args: &ListArgs) -> Result<()> {
    let config = Config::load_or_default()?;
    let fetcher = VersionFetcher::new(reqwest::Client::new(), config.sources);

    let remote_versions = fetcher.list_remote().await?;

    if remote_versions.is_empty() {
        println!("no remote Java versions available");
        return Ok(());
    }

    // Cross-reference with installed versions to show markers.
    let installed: HashSet<String> = VersionManager::from_default_root()?
        .list_installed()?
        .into_iter()
        .map(|v| v.value)
        .collect();

    println!("Available remote versions:");
    for remote in &remote_versions {
        let marker = if installed.contains(&remote.version) {
            "  (installed)"
        } else {
            ""
        };

        if args.verbose {
            println!("  {}{}\t{}", remote.version, marker, remote.archive_name);
        } else {
            println!("  {}{}", remote.version, marker);
        }
    }

    Ok(())
}

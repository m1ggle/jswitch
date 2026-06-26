use clap::Args;
use serde::{Deserialize, Serialize};

use crate::{
    Result,
    config::Config,
    download::{JavaInstaller, VersionFetcher},
    error::jswitch_error::VersionError,
    version::{JavaVersion, VersionManager, VersionResolver},
};

#[derive(Debug, Clone, Args, Serialize, Deserialize)]
pub struct InstallArgs {
    pub version: Option<String>,

    #[arg(long)]
    pub use_now: bool,
}

pub async fn run(args: InstallArgs) -> Result<()> {
    let requested = args
        .version
        .as_deref()
        .and_then(JavaVersion::parse)
        .ok_or_else(|| VersionError::Invalid("missing version".to_owned()))?;
    let config = Config::load_or_default()?;
    let resolved = VersionResolver::new(config.clone()).resolve(&requested.value);
    let client = reqwest::Client::new();
    let fetcher = VersionFetcher::new(client.clone(), config.sources.clone());
    let remote = fetcher.fetch(&resolved).await?;
    let manager = VersionManager::from_default_root()?;

    println!("installing Java {}", remote.version);
    JavaInstaller::new(client, manager).install(&remote).await?;
    println!("installed Java {}", remote.version);

    if args.use_now {
        let mut config = config;
        config.global.default_version = Some(remote.version.clone());
        config.save()?;
        println!("set global Java version to {}", remote.version);
    }

    Ok(())
}

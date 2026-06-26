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

    #[arg(long, value_enum)]
    pub source: Option<JavaSource>,

    #[arg(long)]
    pub use_now: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, clap::ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum JavaSource {
    OpenJdk,
    Oracle,
    Corretto,
    Adoptopenjdk,
}

pub async fn run(args: InstallArgs) -> Result<()> {
    let requested = args
        .version
        .as_deref()
        .and_then(JavaVersion::parse)
        .ok_or_else(|| VersionError::Invalid("missing version".to_owned()))?;
    let config = Config::load_or_default()?;
    let resolved = VersionResolver::new(config.clone()).resolve(&requested.value);
    let source = args.source.unwrap_or(JavaSource::Adoptopenjdk);
    let client = reqwest::Client::new();
    let fetcher = VersionFetcher::new(client.clone());
    let remote = fetcher.fetch(&resolved, source).await?;
    let manager = VersionManager::from_default_root()?;

    println!(
        "installing Java {} from {:?}",
        remote.version, remote.source
    );
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

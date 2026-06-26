use clap::Args;
use serde::{Deserialize, Serialize};

use crate::{
    Result,
    config::Config,
    error::jswitch_error::VersionError,
    version::{JavaVersion, VersionManager, VersionResolver},
};

#[derive(Debug, Clone, Args, Serialize, Deserialize)]
pub struct SwitchArgs {
    pub version: Option<String>,

    #[arg(long)]
    pub global: bool,

    #[arg(long)]
    pub local: bool,

    #[arg(long)]
    pub session: bool,
}

pub async fn run(args: SwitchArgs) -> Result<()> {
    let requested = args
        .version
        .as_deref()
        .and_then(JavaVersion::parse)
        .ok_or_else(|| VersionError::Invalid("missing version".to_owned()))?;
    let mut config = Config::load_or_default()?;
    let resolved = VersionResolver::new(config.clone()).resolve(&requested.value);
    let manager = VersionManager::from_default_root()?;

    if !manager.is_installed(&resolved) {
        return Err(VersionError::NotFound(resolved).into());
    }

    if args.local {
        write_local_version(&resolved)?;
        println!("set local Java version to {}", resolved);
    } else if args.session {
        println!("export JSWITCH_SESSION_VERSION={}", resolved);
    } else {
        config.global.default_version = Some(resolved.clone());
        config.save()?;
        println!("set global Java version to {}", resolved);
    }

    Ok(())
}

fn write_local_version(version: &str) -> std::result::Result<(), VersionError> {
    let path = std::env::current_dir()
        .map_err(VersionError::CurrentDir)?
        .join(".java-version");
    std::fs::write(&path, format!("{version}\n"))
        .map_err(|source| VersionError::WriteFile { path, source })
}

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
}

pub async fn run(args: SwitchArgs) -> Result<()> {
    let requested = args
        .version
        .as_deref()
        .and_then(JavaVersion::parse)
        .ok_or_else(|| VersionError::Invalid("missing version".to_owned()))?;

    let config = Config::load_or_default()?;
    let resolved = VersionResolver::new(config.clone()).resolve(&requested.value);
    let manager = VersionManager::from_default_root()?;

    if !manager.is_installed(&resolved) {
        return Err(VersionError::NotFound(resolved).into());
    }

    // 设置全局默认版本并持久化到 config.toml
    let mut config = config;
    config.global.default_version = Some(resolved.clone());
    config.save()?;

    println!("set global Java version to {}", resolved);

    Ok(())
}

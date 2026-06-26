use clap::Args;
use serde::{Deserialize, Serialize};

use crate::{
    Result,
    config::Config,
    error::jswitch_error::VersionError,
    version::{VersionManager, VersionResolver},
};

#[derive(Debug, Clone, Args, Serialize, Deserialize)]
pub struct RemoveArgs {
    pub version: String,

    #[arg(long)]
    pub force: bool,
}

pub async fn run(args: RemoveArgs) -> Result<()> {
    let config = Config::load_or_default()?;
    let resolver = VersionResolver::new(config);
    let resolved = resolver.resolve(&args.version);
    let manager = VersionManager::from_default_root()?;

    if !args.force
        && let Some(current) = resolver.current()?
        && current.version == resolved
    {
        return Err(VersionError::ActiveVersion { version: resolved }.into());
    }

    manager.remove(&resolved)?;
    println!("removed Java version {resolved}");
    Ok(())
}

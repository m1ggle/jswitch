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
    let version = resolver.resolve(&args.version);

    if !args.force
        && let Some(current) = resolver.current()?
        && current.version == version
    {
        return Err(VersionError::ActiveVersion { version }.into());
    }

    VersionManager::from_default_root()?.remove(&version)?;
    println!("removed Java version {version}");
    Ok(())
}

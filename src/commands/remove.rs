use clap::Args;
use serde::{Deserialize, Serialize};

use crate::{
    Result,
    commands::JavaSource,
    config::Config,
    error::jswitch_error::VersionError,
    version::{VersionManager, VersionResolver},
};

#[derive(Debug, Clone, Args, Serialize, Deserialize)]
pub struct RemoveArgs {
    pub version: String,

    #[arg(long, value_enum)]
    pub source: Option<JavaSource>,

    #[arg(long)]
    pub force: bool,
}

pub async fn run(args: RemoveArgs) -> Result<()> {
    let config = Config::load_or_default()?;
    let resolver = VersionResolver::new(config);
    let resolved = resolver.resolve(&args.version);
    let manager = VersionManager::from_default_root()?;

    // Resolve the installed path: either the user gave an explicit --source,
    // the version string already contains a source prefix (e.g. "corretto/17"),
    // or we search across all sources for a unique match.
    let installed_id = if resolved.contains('/') {
        resolved
    } else if let Some(source) = args.source {
        format!("{}/{}", source.dir_name(), resolved)
    } else {
        let matches = manager.find_installed(&resolved)?;
        match matches.len() {
            0 => return Err(VersionError::NotFound(resolved).into()),
            1 => matches[0].clone(),
            _ => {
                return Err(VersionError::AmbiguousVersion {
                    version: resolved,
                    matches,
                }
                .into());
            }
        }
    };

    if !args.force
        && let Some(current) = resolver.current()?
        && current.version == installed_id
    {
        return Err(VersionError::ActiveVersion {
            version: installed_id,
        }
        .into());
    }

    manager.remove(&installed_id)?;
    println!("removed Java version {installed_id}");
    Ok(())
}

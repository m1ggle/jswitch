use clap::Args;
use serde::{Deserialize, Serialize};

use crate::{
    Result,
    commands::JavaSource,
    config::Config,
    error::jswitch_error::VersionError,
    version::{JavaVersion, VersionManager, VersionResolver},
};

#[derive(Debug, Clone, Args, Serialize, Deserialize)]
pub struct SwitchArgs {
    pub version: Option<String>,

    #[arg(long, value_enum)]
    pub source: Option<JavaSource>,

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

    // Resolve the installed path: either the user gave an explicit --source,
    // the version string already contains a source prefix (e.g. "corretto/17"),
    // or we search across all sources for a unique match.
    let installed_id = if resolved.contains('/') {
        resolved.clone()
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

    if !manager.is_installed(&installed_id) {
        return Err(VersionError::NotFound(installed_id).into());
    }

    if args.local {
        write_local_version(&installed_id)?;
        println!("set local Java version to {}", installed_id);
    } else if args.session {
        println!("export JSWITCH_SESSION_VERSION={}", installed_id);
    } else {
        config.global.default_version = Some(installed_id.clone());
        config.save()?;
        println!("set global Java version to {}", installed_id);
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

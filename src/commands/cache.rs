use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};

use crate::{Result, fs::CacheManager, version::VersionManager};

#[derive(Debug, Clone, Args, Serialize, Deserialize)]
pub struct CacheArgs {
    #[command(subcommand)]
    pub action: CacheAction,
}

#[derive(Debug, Clone, Subcommand, Serialize, Deserialize)]
pub enum CacheAction {
    /// Remove all cached archives.
    Clean,
    /// List cached archive files and their sizes.
    List,
    /// Remove cached archives for versions that are no longer installed.
    Prune,
}

pub async fn run(args: CacheArgs) -> Result<()> {
    match args.action {
        CacheAction::Clean => clean().await,
        CacheAction::List => list().await,
        CacheAction::Prune => prune().await,
    }
}

async fn clean() -> Result<()> {
    let cache = CacheManager::from_default_root()?;
    let removed = cache.list_archives()?.len();
    cache.clean()?;
    println!("removed {removed} cached archive(s)");
    Ok(())
}

async fn list() -> Result<()> {
    let cache = CacheManager::from_default_root()?;
    let archives = cache.list_archives()?;

    if archives.is_empty() {
        println!("cache is empty");
        return Ok(());
    }

    let total: u64 = archives.iter().map(|a| a.size).sum();
    for archive in &archives {
        println!("  {}\t{}", archive.name, format_size(archive.size));
    }
    println!("  ────────────────────────────────");
    println!("  total\t{}", format_size(total));

    Ok(())
}

async fn prune() -> Result<()> {
    let cache = CacheManager::from_default_root()?;
    let archives = cache.list_archives()?;
    let installed: Vec<String> = VersionManager::from_default_root()?
        .list_installed()?
        .into_iter()
        .map(|v| v.value)
        .collect();

    let mut removed = 0u64;
    for archive in &archives {
        // An archive is useful if any installed version string appears in its filename.
        // e.g. "17.0.19+10" matches "OpenJDK17U-jdk_x64_mac_hotspot_17.0.19_10.tar.gz"
        // but also "17" matches "amazon-corretto-17-x64-macos-jdk.tar.gz".
        let is_orphan = !installed
            .iter()
            .any(|version| archive.name.contains(version));

        if is_orphan {
            cache.remove_archive(&archive.name)?;
            println!(
                "  removed: {} ({})",
                archive.name,
                format_size(archive.size)
            );
            removed += archive.size;
        }
    }

    if removed == 0 {
        println!("no orphaned archives found");
    } else {
        println!("freed {}", format_size(removed));
    }

    Ok(())
}

/// Format a byte count as a human-readable string.
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_bytes() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn formats_fractional_sizes() {
        assert_eq!(format_size(1536), "1.50 KB");
        assert_eq!(format_size(1024 * 1024 * 3 + 1024 * 512), "3.50 MB");
    }
}

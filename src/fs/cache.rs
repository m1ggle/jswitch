use std::{io, path::PathBuf};

use tracing::debug;

use crate::error::jswitch_error::IoError;

use super::operations;

/// Manages the download cache directory under `~/.jswitch/cache/`.
#[derive(Debug, Clone)]
pub struct CacheManager {
    cache_dir: PathBuf,
}

impl CacheManager {
    /// Create a `CacheManager` rooted at the given jswitch home directory.
    pub fn new(root: &std::path::Path) -> Self {
        Self {
            cache_dir: root.join("cache"),
        }
    }

    /// Create a `CacheManager` using the default jswitch home (`~/.jswitch`).
    pub fn from_default_root() -> Result<Self, IoError> {
        let home = dirs::home_dir().ok_or(IoError::Access {
            path: std::path::PathBuf::from("~"),
            source: io::Error::new(io::ErrorKind::NotFound, "home directory not available"),
        })?;
        Ok(Self::new(&home.join(".jswitch")))
    }

    /// Return the cache directory path.
    pub fn cache_dir(&self) -> &std::path::Path {
        &self.cache_dir
    }

    /// Return the full path for a cached archive with the given file name.
    pub fn archive_path(&self, name: &str) -> PathBuf {
        self.cache_dir.join(name)
    }

    /// Create the cache directory if it does not exist.
    pub fn ensure_dir(&self) -> Result<(), IoError> {
        if self.cache_dir.is_dir() {
            return Ok(());
        }
        operations::create_dir_all(&self.cache_dir).map_err(|source| IoError::Access {
            path: self.cache_dir.clone(),
            source,
        })
    }

    /// Check whether a cached archive with the given name exists.
    pub fn has(&self, name: &str) -> bool {
        operations::file_exists(&self.archive_path(name))
    }

    /// Remove all cached archives.
    ///
    /// If the cache directory does not exist, this is a no-op.
    pub fn clean(&self) -> Result<(), IoError> {
        if !self.cache_dir.exists() {
            debug!(cache_dir = %self.cache_dir.display(), "cache dir absent, nothing to clean");
            return Ok(());
        }
        operations::reset_dir(&self.cache_dir).map_err(|source| IoError::Access {
            path: self.cache_dir.clone(),
            source,
        })
    }

    /// Compute the total size (in bytes) of all files in the cache directory.
    pub fn size(&self) -> Result<u64, IoError> {
        if !self.cache_dir.exists() {
            return Ok(0);
        }
        let entries =
            operations::read_dir_entries(&self.cache_dir).map_err(|source| IoError::Access {
                path: self.cache_dir.clone(),
                source,
            })?;
        let mut total = 0u64;
        for entry in entries {
            if entry.is_file() {
                total += std::fs::metadata(&entry)
                    .map_err(|source| IoError::Access {
                        path: entry,
                        source,
                    })?
                    .len();
            }
        }
        Ok(total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_and_queries_cache_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = CacheManager::new(tmp.path());

        cache.ensure_dir().unwrap();
        assert!(cache.cache_dir().is_dir());

        let archive = cache.archive_path("jdk-17.tar.gz");
        std::fs::write(&archive, "data").unwrap();
        assert!(cache.has("jdk-17.tar.gz"));
        assert!(!cache.has("nonexistent.zip"));
    }

    #[test]
    fn computes_cache_size() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = CacheManager::new(tmp.path());
        cache.ensure_dir().unwrap();

        std::fs::write(cache.archive_path("a"), "aaaa").unwrap();
        std::fs::write(cache.archive_path("b"), "bb").unwrap();

        assert_eq!(cache.size().unwrap(), 6);
    }

    #[test]
    fn cleans_cache() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = CacheManager::new(tmp.path());
        cache.ensure_dir().unwrap();
        std::fs::write(cache.archive_path("old"), "old").unwrap();

        cache.clean().unwrap();

        assert!(cache.cache_dir().is_dir());
        assert!(!cache.has("old"));
        assert_eq!(cache.size().unwrap(), 0);
    }

    #[test]
    fn size_is_zero_for_missing_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = CacheManager::new(tmp.path());
        assert_eq!(cache.size().unwrap(), 0);
    }

    #[test]
    fn clean_is_noop_when_dir_absent() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = CacheManager::new(tmp.path());
        cache.clean().unwrap();
    }
}

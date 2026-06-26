use std::{fs, io, path::PathBuf};

use crate::error::jswitch_error::VersionError;

use super::JavaVersion;

#[derive(Debug, Clone)]
pub struct VersionManager {
    root: PathBuf,
}

impl VersionManager {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn from_default_root() -> Result<Self, VersionError> {
        let home = dirs::home_dir().ok_or(VersionError::HomeDirUnavailable)?;
        Ok(Self::new(home.join(".jswitch")))
    }

    pub fn root_dir(&self) -> &std::path::Path {
        &self.root
    }

    pub fn versions_dir(&self) -> PathBuf {
        self.root.join("versions")
    }

    pub fn list_installed(&self) -> Result<Vec<JavaVersion>, VersionError> {
        let versions_dir = self.versions_dir();
        let entries = match fs::read_dir(&versions_dir) {
            Ok(entries) => entries,
            Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(source) => {
                return Err(VersionError::ReadDir {
                    path: versions_dir,
                    source,
                });
            }
        };

        let mut versions = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|source| VersionError::ReadDirEntry {
                path: versions_dir.clone(),
                source,
            })?;
            let file_type = entry.file_type().map_err(|source| VersionError::Metadata {
                path: entry.path(),
                source,
            })?;

            if file_type.is_dir()
                && let Some(version) = entry.file_name().to_str().and_then(JavaVersion::parse)
            {
                versions.push(version);
            }
        }

        versions.sort();
        Ok(versions)
    }

    pub fn is_installed(&self, version: &str) -> bool {
        self.versions_dir().join(version).is_dir()
    }

    pub fn remove(&self, version: &str) -> Result<(), VersionError> {
        let path = self.versions_dir().join(version);
        if !path.exists() {
            return Err(VersionError::NotFound(version.to_owned()));
        }

        fs::remove_dir_all(&path).map_err(|source| VersionError::RemoveDir { path, source })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_installed_versions_from_directories() {
        let dir = tempfile::tempdir().unwrap();
        let versions_dir = dir.path().join("versions");
        fs::create_dir_all(versions_dir.join("17")).unwrap();
        fs::create_dir_all(versions_dir.join("11.0.2")).unwrap();
        fs::write(versions_dir.join("README"), "not a version").unwrap();

        let manager = VersionManager::new(dir.path().to_path_buf());
        let versions = manager.list_installed().unwrap();

        assert_eq!(
            versions
                .into_iter()
                .map(|version| version.value)
                .collect::<Vec<_>>(),
            vec!["11.0.2", "17"]
        );
    }

    #[test]
    fn removes_installed_version_directory() {
        let dir = tempfile::tempdir().unwrap();
        let version_dir = dir.path().join("versions").join("17");
        fs::create_dir_all(&version_dir).unwrap();

        let manager = VersionManager::new(dir.path().to_path_buf());
        manager.remove("17").unwrap();

        assert!(!version_dir.exists());
    }
}

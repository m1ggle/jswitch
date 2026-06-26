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

    /// List all installed versions as `source/version` strings (e.g. `corretto/17`).
    pub fn list_installed(&self) -> Result<Vec<JavaVersion>, VersionError> {
        let versions_dir = self.versions_dir();
        let sources = match fs::read_dir(&versions_dir) {
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
        for source_entry in sources {
            let source_entry = source_entry.map_err(|source| VersionError::ReadDirEntry {
                path: versions_dir.clone(),
                source,
            })?;
            let file_type = source_entry
                .file_type()
                .map_err(|source| VersionError::Metadata {
                    path: source_entry.path(),
                    source,
                })?;

            if !file_type.is_dir() {
                continue;
            }

            let source_name = match source_entry.file_name().to_str() {
                Some(name) => name.to_owned(),
                None => continue,
            };
            let source_dir = source_entry.path();

            let version_entries = match fs::read_dir(&source_dir) {
                Ok(entries) => entries,
                Err(source) if source.kind() == io::ErrorKind::NotFound => continue,
                Err(source) => {
                    return Err(VersionError::ReadDir {
                        path: source_dir,
                        source,
                    });
                }
            };

            for version_entry in version_entries {
                let version_entry = version_entry.map_err(|source| VersionError::ReadDirEntry {
                    path: source_dir.clone(),
                    source,
                })?;
                let file_type =
                    version_entry
                        .file_type()
                        .map_err(|source| VersionError::Metadata {
                            path: version_entry.path(),
                            source,
                        })?;

                if file_type.is_dir()
                    && let Some(version) = version_entry
                        .file_name()
                        .to_str()
                        .and_then(JavaVersion::parse)
                {
                    let installed_id = format!("{source_name}/{}", version.value);
                    versions.push(JavaVersion::new(installed_id));
                }
            }
        }

        versions.sort();
        Ok(versions)
    }

    /// Find all installed versions matching a version string, returning
    /// `source/version` paths (e.g. `["corretto/17", "openjdk/17"]`).
    pub fn find_installed(&self, version: &str) -> Result<Vec<String>, VersionError> {
        let versions_dir = self.versions_dir();
        let sources = match fs::read_dir(&versions_dir) {
            Ok(entries) => entries,
            Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(source) => {
                return Err(VersionError::ReadDir {
                    path: versions_dir,
                    source,
                });
            }
        };

        let mut matches = Vec::new();
        for entry in sources {
            let entry = entry.map_err(|source| VersionError::ReadDirEntry {
                path: versions_dir.clone(),
                source,
            })?;
            let file_type = entry.file_type().map_err(|source| VersionError::Metadata {
                path: entry.path(),
                source,
            })?;

            if file_type.is_dir() {
                let source_name = entry.file_name().to_string_lossy().into_owned();
                if entry.path().join(version).is_dir() {
                    matches.push(format!("{source_name}/{version}"));
                }
            }
        }

        matches.sort();
        Ok(matches)
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
        fs::create_dir_all(versions_dir.join("corretto").join("17")).unwrap();
        fs::create_dir_all(versions_dir.join("adoptopenjdk").join("11.0.2")).unwrap();
        fs::write(versions_dir.join("README"), "not a version").unwrap();

        let manager = VersionManager::new(dir.path().to_path_buf());
        let versions = manager.list_installed().unwrap();

        let values: Vec<_> = versions.into_iter().map(|v| v.value).collect();
        assert!(values.contains(&"corretto/17".to_owned()));
        assert!(values.contains(&"adoptopenjdk/11.0.2".to_owned()));
        assert!(!values.iter().any(|v| v.contains("README")));
    }

    #[test]
    fn finds_installed_versions_across_sources() {
        let dir = tempfile::tempdir().unwrap();
        let versions_dir = dir.path().join("versions");
        fs::create_dir_all(versions_dir.join("corretto").join("17")).unwrap();
        fs::create_dir_all(versions_dir.join("openjdk").join("17")).unwrap();
        fs::create_dir_all(versions_dir.join("corretto").join("21")).unwrap();

        let manager = VersionManager::new(dir.path().to_path_buf());
        let matches = manager.find_installed("17").unwrap();

        assert_eq!(matches, vec!["corretto/17", "openjdk/17"]);
    }

    #[test]
    fn removes_installed_version_directory() {
        let dir = tempfile::tempdir().unwrap();
        let version_dir = dir.path().join("versions").join("corretto").join("17");
        fs::create_dir_all(&version_dir).unwrap();

        let manager = VersionManager::new(dir.path().to_path_buf());
        manager.remove("corretto/17").unwrap();

        assert!(!version_dir.exists());
    }
}

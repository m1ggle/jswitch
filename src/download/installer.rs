use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use flate2::read::GzDecoder;
use futures_util::StreamExt;
use tracing::debug;

use crate::{
    error::jswitch_error::NetworkError,
    version::{VersionManager, VersionMetadata},
};

use super::{RemoteVersion, progress::DownloadProgress, verify_sha256};

#[derive(Debug, Clone)]
pub struct JavaInstaller {
    client: reqwest::Client,
    manager: VersionManager,
}

impl JavaInstaller {
    pub fn new(client: reqwest::Client, manager: VersionManager) -> Self {
        Self { client, manager }
    }

    pub async fn install(&self, remote: &RemoteVersion) -> Result<(), NetworkError> {
        let cache_dir = self.manager.root_dir().join("cache");
        let temp_dir = self.manager.root_dir().join("tmp").join(&remote.version);
        let archive_path = cache_dir.join(&remote.archive_name);
        let install_dir = self.manager.versions_dir().join(&remote.version);

        debug!(version = %remote.version, "starting install");
        fs::create_dir_all(&cache_dir).map_err(|source| NetworkError::CreateDir {
            path: cache_dir.clone(),
            source,
        })?;
        reset_dir(&temp_dir)?;

        self.download(&remote.download_url, &archive_path).await?;
        if let Some(checksum_url) = &remote.checksum_url {
            debug!(%checksum_url, "fetching checksum");
            let checksum = self
                .client
                .get(checksum_url)
                .send()
                .await?
                .error_for_status()?
                .text()
                .await?;
            verify_sha256(&archive_path, &checksum)?;
        }

        debug!(archive = %archive_path.display(), "unpacking archive");
        unpack_archive(&archive_path, &temp_dir)?;
        write_metadata(&temp_dir, remote)?;

        if install_dir.exists() {
            fs::remove_dir_all(&install_dir).map_err(|source| NetworkError::Remove {
                path: install_dir.clone(),
                source,
            })?;
        }
        fs::rename(&temp_dir, &install_dir).map_err(|source| NetworkError::Rename {
            from: temp_dir,
            to: install_dir.clone(),
            source,
        })?;

        debug!(path = %install_dir.display(), "install complete");
        Ok(())
    }

    async fn download(&self, url: &str, path: &Path) -> Result<(), NetworkError> {
        debug!(%url, path = %path.display(), "downloading archive");
        let response = self.client.get(url).send().await?.error_for_status()?;
        let mut progress = DownloadProgress::new(response.content_length());
        let mut stream = response.bytes_stream();
        let mut file = File::create(path).map_err(|source| NetworkError::WriteFile {
            path: path.to_path_buf(),
            source,
        })?;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk)
                .map_err(|source| NetworkError::WriteFile {
                    path: path.to_path_buf(),
                    source,
                })?;
            progress.advance(chunk.len() as u64);
        }
        progress.finish();

        Ok(())
    }
}

fn reset_dir(path: &Path) -> Result<(), NetworkError> {
    if path.exists() {
        fs::remove_dir_all(path).map_err(|source| NetworkError::Remove {
            path: path.to_path_buf(),
            source,
        })?;
    }

    fs::create_dir_all(path).map_err(|source| NetworkError::CreateDir {
        path: path.to_path_buf(),
        source,
    })
}

fn unpack_archive(archive: &Path, destination: &Path) -> Result<(), NetworkError> {
    let archive_name = archive
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();

    if archive_name.ends_with(".tar.gz") || archive_name.ends_with(".tgz") {
        let file = File::open(archive).map_err(|source| NetworkError::ReadFile {
            path: archive.to_path_buf(),
            source,
        })?;
        let decoder = GzDecoder::new(file);
        let mut archive_reader = tar::Archive::new(decoder);
        archive_reader
            .unpack(destination)
            .map_err(|source| NetworkError::Unpack {
                path: archive.to_path_buf(),
                source,
            })?;
        flatten_single_root(destination)?;
        return Ok(());
    }

    if archive_name.ends_with(".zip") {
        unpack_zip(archive, destination)?;
        flatten_single_root(destination)?;
        return Ok(());
    }

    Err(NetworkError::UnsupportedArchive(archive_name.to_owned()))
}

fn unpack_zip(archive: &Path, destination: &Path) -> Result<(), NetworkError> {
    let file = File::open(archive).map_err(|source| NetworkError::ReadFile {
        path: archive.to_path_buf(),
        source,
    })?;
    let mut archive_reader = zip::ZipArchive::new(file).map_err(|source| NetworkError::Unzip {
        path: archive.to_path_buf(),
        source,
    })?;

    for index in 0..archive_reader.len() {
        let mut file = archive_reader
            .by_index(index)
            .map_err(|source| NetworkError::Unzip {
                path: archive.to_path_buf(),
                source,
            })?;
        let Some(enclosed_name) = file.enclosed_name() else {
            continue;
        };
        let output_path = destination.join(enclosed_name);

        if file.is_dir() {
            fs::create_dir_all(&output_path).map_err(|source| NetworkError::CreateDir {
                path: output_path,
                source,
            })?;
        } else {
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent).map_err(|source| NetworkError::CreateDir {
                    path: parent.to_path_buf(),
                    source,
                })?;
            }
            let mut output =
                File::create(&output_path).map_err(|source| NetworkError::WriteFile {
                    path: output_path.clone(),
                    source,
                })?;
            std::io::copy(&mut file, &mut output).map_err(|source| NetworkError::WriteFile {
                path: output_path,
                source,
            })?;
        }
    }

    Ok(())
}

fn flatten_single_root(destination: &Path) -> Result<(), NetworkError> {
    let entries = fs::read_dir(destination)
        .map_err(|source| NetworkError::ReadFile {
            path: destination.to_path_buf(),
            source,
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|source| NetworkError::ReadFile {
            path: destination.to_path_buf(),
            source,
        })?;

    if entries.len() != 1 || !entries[0].path().is_dir() {
        return Ok(());
    }

    let root = entries[0].path();
    for entry in fs::read_dir(&root).map_err(|source| NetworkError::ReadFile {
        path: root.clone(),
        source,
    })? {
        let entry = entry.map_err(|source| NetworkError::ReadFile {
            path: root.clone(),
            source,
        })?;
        let from = entry.path();
        let to = destination.join(entry.file_name());
        fs::rename(&from, &to).map_err(|source| NetworkError::Rename { from, to, source })?;
    }

    fs::remove_dir_all(&root).map_err(|source| NetworkError::Remove { path: root, source })
}

fn write_metadata(destination: &Path, remote: &RemoteVersion) -> Result<(), NetworkError> {
    let metadata = VersionMetadata {
        version: remote.version.clone(),
        source: Some("adoptopenjdk".to_owned()),
    };
    let content = toml::to_string_pretty(&metadata).map_err(NetworkError::SerializeMetadata)?;
    let path = destination.join(".jswitch-version.toml");
    fs::write(&path, content).map_err(|source| NetworkError::WriteFile { path, source })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flattens_single_archive_root() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("jdk-17");
        fs::create_dir_all(root.join("bin")).unwrap();
        fs::write(root.join("release"), "JAVA_VERSION=17").unwrap();

        flatten_single_root(dir.path()).unwrap();

        assert!(dir.path().join("bin").exists());
        assert!(dir.path().join("release").exists());
        assert!(!root.exists());
    }
}

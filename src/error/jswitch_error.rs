use std::{io, path::PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum JswitchError {
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    Io(#[from] IoError),
    #[error(transparent)]
    Network(#[from] NetworkError),
    #[error(transparent)]
    Version(#[from] VersionError),
}

pub type Result<T> = std::result::Result<T, JswitchError>;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("home directory not available")]
    HomeDirUnavailable,
    #[error("unsupported config key: {0}")]
    UnsupportedKey(String),
    #[error("invalid boolean value for {key}: {value}")]
    InvalidBoolean { key: String, value: String },
    #[error("failed to create config directory {path}: {source}")]
    CreateDir {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to read config file {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to write config file {path}: {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to parse config file {path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("failed to serialize config: {0}")]
    Serialize(#[from] toml::ser::Error),
}

#[derive(Debug, Error)]
pub enum IoError {
    #[error("failed to access {path}: {source}")]
    Access {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("remote Java version not found: {0}")]
    RemoteVersionNotFound(String),
    #[error("unsupported Java version request: {0}")]
    UnsupportedVersion(String),
    #[error("checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },
    #[error("unsupported archive format: {0}")]
    UnsupportedArchive(String),
    #[error("failed to create directory {path}: {source}")]
    CreateDir {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to read file {path}: {source}")]
    ReadFile {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to write file {path}: {source}")]
    WriteFile {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to remove path {path}: {source}")]
    Remove {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to rename {from} to {to}: {source}")]
    Rename {
        from: PathBuf,
        to: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to serialize version metadata: {0}")]
    SerializeMetadata(#[from] toml::ser::Error),
    #[error("failed to unpack archive {path}: {source}")]
    Unpack {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to unpack zip archive {path}: {source}")]
    Unzip {
        path: PathBuf,
        #[source]
        source: zip::result::ZipError,
    },
}

#[derive(Debug, Error)]
pub enum VersionError {
    #[error("home directory not available")]
    HomeDirUnavailable,
    #[error("Java version not found: {0}")]
    NotFound(String),
    #[error("invalid Java version: {0}")]
    Invalid(String),
    #[error("cannot remove active Java version {version}; use --force to override")]
    ActiveVersion { version: String },
    #[error("failed to read versions directory {path}: {source}")]
    ReadDir {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to read entry in versions directory {path}: {source}")]
    ReadDirEntry {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to read metadata for {path}: {source}")]
    Metadata {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to remove version directory {path}: {source}")]
    RemoveDir {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to read file {path}: {source}")]
    ReadFile {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to write file {path}: {source}")]
    WriteFile {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to get current directory: {0}")]
    CurrentDir(#[source] io::Error),
}

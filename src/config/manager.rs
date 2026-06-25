use std::{
    fs, io,
    path::{Path, PathBuf},
};

use crate::error::jswitch_error::ConfigError;

use super::global::Config;

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let path = default_config_path()?;
        Self::load_from_path(path)
    }

    pub fn load_or_default() -> Result<Self, ConfigError> {
        let path = default_config_path()?;
        Self::load_from_path_or_default(path)
    }

    pub fn load_from_path_or_default(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref();

        match Self::load_from_path(path) {
            Ok(config) => Ok(config),
            Err(ConfigError::Read { source, .. }) if source.kind() == io::ErrorKind::NotFound => {
                Ok(Self::default())
            }
            Err(error) => Err(error),
        }
    }

    pub fn load_from_path(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let content = fs::read_to_string(path).map_err(|source| ConfigError::Read {
            path: path.to_path_buf(),
            source,
        })?;

        toml::from_str(&content).map_err(|source| ConfigError::Parse {
            path: path.to_path_buf(),
            source,
        })
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        let path = default_config_path()?;
        self.save_to_path(path)
    }

    pub fn save_to_path(&self, path: impl AsRef<Path>) -> Result<(), ConfigError> {
        let path = path.as_ref();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| ConfigError::CreateDir {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let content = toml::to_string_pretty(self)?;
        fs::write(path, content).map_err(|source| ConfigError::Write {
            path: path.to_path_buf(),
            source,
        })
    }
}

pub fn default_config_path() -> Result<PathBuf, ConfigError> {
    let home = dirs::home_dir().ok_or(ConfigError::HomeDirUnavailable)?;
    Ok(home.join(".jswitch").join("config.toml"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_missing_config_as_default() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");

        let config = Config::load_from_path_or_default(path).unwrap();

        assert_eq!(config, Config::default());
    }

    #[test]
    fn saves_and_loads_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".jswitch").join("config.toml");
        let mut config = Config::default();
        config
            .set("global.default_version", "17".to_owned())
            .unwrap();
        config.set("aliases.lts", "21".to_owned()).unwrap();
        config.set("global.auto_update", "true".to_owned()).unwrap();
        config
            .set(
                "sources.openjdk",
                "https://download.java.net/java/GA/jdk".to_owned(),
            )
            .unwrap();
        config
            .set(
                "proxy.http_proxy",
                "http://proxy.company.com:8080".to_owned(),
            )
            .unwrap();
        config.set("plugins.maven", "true".to_owned()).unwrap();

        config.save_to_path(&path).unwrap();
        let loaded = Config::load_from_path(path).unwrap();

        assert_eq!(
            loaded.get("global.default_version").unwrap(),
            Some("17".to_owned())
        );
        assert_eq!(loaded.get("aliases.lts").unwrap(), Some("21".to_owned()));
        assert_eq!(
            loaded.get("global.auto_update").unwrap(),
            Some("true".to_owned())
        );
        assert_eq!(
            loaded.get("sources.openjdk").unwrap(),
            Some("https://download.java.net/java/GA/jdk".to_owned())
        );
        assert_eq!(
            loaded.get("proxy.http_proxy").unwrap(),
            Some("http://proxy.company.com:8080".to_owned())
        );
        assert_eq!(
            loaded.get("plugins.maven").unwrap(),
            Some("true".to_owned())
        );
    }
}

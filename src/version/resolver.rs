use crate::{config::Config, error::jswitch_error::VersionError};

#[derive(Debug, Clone)]
pub struct VersionResolver {
    config: Config,
}

impl VersionResolver {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn resolve(&self, value: &str) -> String {
        self.config
            .aliases
            .get(value)
            .cloned()
            .unwrap_or_else(|| value.to_owned())
    }

    pub fn current(&self) -> Result<Option<CurrentVersion>, VersionError> {
        if let Some(version) = read_local_version()? {
            return Ok(Some(CurrentVersion {
                version: self.resolve(&version),
                source: VersionSource::Local,
            }));
        }

        if let Ok(version) = std::env::var("JSWITCH_SESSION_VERSION")
            && !version.trim().is_empty()
        {
            return Ok(Some(CurrentVersion {
                version: self.resolve(version.trim()),
                source: VersionSource::Session,
            }));
        }

        Ok(self
            .config
            .global
            .default_version
            .as_ref()
            .map(|version| CurrentVersion {
                version: self.resolve(version),
                source: VersionSource::Global,
            }))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrentVersion {
    pub version: String,
    pub source: VersionSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionSource {
    Local,
    Session,
    Global,
}

impl std::fmt::Display for VersionSource {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            VersionSource::Local => "local",
            VersionSource::Session => "session",
            VersionSource::Global => "global",
        };
        formatter.write_str(value)
    }
}

fn read_local_version() -> Result<Option<String>, VersionError> {
    let path = std::env::current_dir()
        .map_err(VersionError::CurrentDir)?
        .join(".java-version");

    match std::fs::read_to_string(&path) {
        Ok(content) => {
            let version = content.trim();
            if version.is_empty() {
                Ok(None)
            } else {
                Ok(Some(version.to_owned()))
            }
        }
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(VersionError::ReadFile { path, source }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_aliases_from_config() {
        let mut config = Config::default();
        config.aliases.insert("lts".to_owned(), "21".to_owned());

        let resolver = VersionResolver::new(config);

        assert_eq!(resolver.resolve("lts"), "21");
        assert_eq!(resolver.resolve("17"), "17");
    }
}

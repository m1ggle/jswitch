use crate::{config::Config, error::jswitch_error::VersionError};

#[derive(Debug, Clone)]
pub struct VersionResolver {
    config: Config,
}

impl VersionResolver {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// 将别名解析为实际版本号，非别名则原样返回。
    pub fn resolve(&self, value: &str) -> String {
        self.config
            .aliases
            .get(value)
            .cloned()
            .unwrap_or_else(|| value.to_owned())
    }

    /// 返回当前全局默认版本（唯一来源）。
    pub fn current(&self) -> Result<Option<CurrentVersion>, VersionError> {
        Ok(self
            .config
            .global
            .default_version
            .as_ref()
            .map(|version| CurrentVersion {
                version: self.resolve(version),
            }))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrentVersion {
    pub version: String,
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

    #[test]
    fn current_returns_global_default() {
        let mut config = Config::default();
        config.global.default_version = Some("17".to_owned());

        let resolver = VersionResolver::new(config);

        assert_eq!(
            resolver.current().unwrap(),
            Some(CurrentVersion {
                version: "17".to_owned()
            })
        );
    }

    #[test]
    fn current_returns_none_when_no_default() {
        let config = Config::default();
        let resolver = VersionResolver::new(config);

        assert_eq!(resolver.current().unwrap(), None);
    }
}

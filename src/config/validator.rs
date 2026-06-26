use crate::error::jswitch_error::ConfigError;

use super::global::Config;

impl Config {
    pub fn get(&self, key: &str) -> Result<Option<String>, ConfigError> {
        let value = match key {
            "global.default_version" => self.global.default_version.clone(),
            "global.auto_update" => Some(self.global.auto_update.to_string()),
            "global.check_updates" => Some(self.global.check_updates.to_string()),
            "global.quiet_mode" => Some(self.global.quiet_mode.to_string()),
            key if key.starts_with("aliases.") => self.aliases.get(&key[8..]).cloned(),
            "plugins.maven" => Some(self.plugins.maven.to_string()),
            "plugins.gradle" => Some(self.plugins.gradle.to_string()),
            _ => return Err(ConfigError::UnsupportedKey(key.to_owned())),
        };

        Ok(value)
    }

    pub fn set(&mut self, key: &str, value: String) -> Result<(), ConfigError> {
        match key {
            "global.default_version" => self.global.default_version = Some(value),
            "global.auto_update" => self.global.auto_update = parse_bool(key, &value)?,
            "global.check_updates" => self.global.check_updates = parse_bool(key, &value)?,
            "global.quiet_mode" => self.global.quiet_mode = parse_bool(key, &value)?,
            key if key.starts_with("aliases.") => {
                self.aliases.insert(key[8..].to_owned(), value);
            }
            "plugins.maven" => self.plugins.maven = parse_bool(key, &value)?,
            "plugins.gradle" => self.plugins.gradle = parse_bool(key, &value)?,
            _ => return Err(ConfigError::UnsupportedKey(key.to_owned())),
        }

        Ok(())
    }

    pub fn entries(&self) -> Vec<(String, String)> {
        let mut entries = vec![
            (
                "global.default_version".to_owned(),
                format_option(&self.global.default_version),
            ),
            (
                "global.auto_update".to_owned(),
                self.global.auto_update.to_string(),
            ),
            (
                "global.check_updates".to_owned(),
                self.global.check_updates.to_string(),
            ),
            (
                "global.quiet_mode".to_owned(),
                self.global.quiet_mode.to_string(),
            ),
        ];

        entries.extend(
            self.aliases
                .iter()
                .map(|(alias, version)| (format!("aliases.{alias}"), version.clone())),
        );

        entries.extend([
            ("plugins.maven".to_owned(), self.plugins.maven.to_string()),
            ("plugins.gradle".to_owned(), self.plugins.gradle.to_string()),
        ]);

        entries
    }
}

fn parse_bool(key: &str, value: &str) -> Result<bool, ConfigError> {
    value.parse().map_err(|_| ConfigError::InvalidBoolean {
        key: key.to_owned(),
        value: value.to_owned(),
    })
}

fn format_option(value: &Option<String>) -> String {
    value.clone().unwrap_or_else(|| "<unset>".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supports_nested_config_keys() {
        let mut config = Config::default();
        config
            .set("global.default_version", "17".to_owned())
            .unwrap();
        config.set("aliases.lts", "21".to_owned()).unwrap();
        config.set("plugins.maven", "true".to_owned()).unwrap();

        assert_eq!(
            config.get("global.default_version").unwrap(),
            Some("17".to_owned())
        );
        assert_eq!(config.get("aliases.lts").unwrap(), Some("21".to_owned()));
        assert_eq!(
            config.get("plugins.maven").unwrap(),
            Some("true".to_owned())
        );
    }

    #[test]
    fn entries_use_sectioned_keys() {
        let mut config = Config::default();
        config.global.default_version = Some("17".to_owned());
        config.aliases.insert("lts".to_owned(), "21".to_owned());

        let entries = config.entries();

        assert!(
            entries
                .iter()
                .any(|(key, value)| key == "global.default_version" && value == "17")
        );
        assert!(
            entries
                .iter()
                .any(|(key, value)| key == "aliases.lts" && value == "21")
        );
    }

    #[test]
    fn rejects_unknown_config_keys() {
        let mut config = Config::default();
        let error = config.set("unknown", "value".to_owned()).unwrap_err();

        assert!(matches!(error, ConfigError::UnsupportedKey(_)));
    }
}

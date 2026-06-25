use super::{global::Config, manager::ConfigError};

impl Config {
    pub fn get(&self, key: &str) -> Result<Option<String>, ConfigError> {
        let value = match key {
            "default_version" => self.default_version.clone(),
            "mirror_url" => self.mirror_url.clone(),
            "auto_update" => Some(self.auto_update.to_string()),
            "check_updates" => Some(self.check_updates.to_string()),
            "quiet_mode" => Some(self.quiet_mode.to_string()),
            key if key.starts_with("alias.") => self.aliases.get(&key[6..]).cloned(),
            _ => return Err(ConfigError::UnsupportedKey(key.to_owned())),
        };

        Ok(value)
    }

    pub fn set(&mut self, key: &str, value: String) -> Result<(), ConfigError> {
        match key {
            "default_version" => self.default_version = Some(value),
            "mirror_url" => self.mirror_url = Some(value),
            "auto_update" => self.auto_update = parse_bool(key, &value)?,
            "check_updates" => self.check_updates = parse_bool(key, &value)?,
            "quiet_mode" => self.quiet_mode = parse_bool(key, &value)?,
            key if key.starts_with("alias.") => {
                self.aliases.insert(key[6..].to_owned(), value);
            }
            _ => return Err(ConfigError::UnsupportedKey(key.to_owned())),
        }

        Ok(())
    }

    pub fn entries(&self) -> Vec<(String, String)> {
        let mut entries = vec![
            ("default_version".to_owned(), format_option(&self.default_version)),
            ("mirror_url".to_owned(), format_option(&self.mirror_url)),
            ("auto_update".to_owned(), self.auto_update.to_string()),
            ("check_updates".to_owned(), self.check_updates.to_string()),
            ("quiet_mode".to_owned(), self.quiet_mode.to_string()),
        ];

        entries.extend(
            self.aliases
                .iter()
                .map(|(alias, version)| (format!("alias.{alias}"), version.clone())),
        );

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
    fn rejects_unknown_config_keys() {
        let mut config = Config::default();
        let error = config.set("unknown", "value".to_owned()).unwrap_err();

        assert!(matches!(error, ConfigError::UnsupportedKey(_)));
    }
}

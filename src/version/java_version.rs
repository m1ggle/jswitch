use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct JavaVersion {
    pub value: String,
}

impl JavaVersion {
    pub fn parse(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        let trimmed = value.trim();

        if trimmed.is_empty() {
            return None;
        }

        Some(Self {
            value: trimmed.to_owned(),
        })
    }

    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

impl std::fmt::Display for JavaVersion {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_blank_versions() {
        assert_eq!(JavaVersion::parse("   "), None);
    }

    #[test]
    fn trims_versions() {
        assert_eq!(JavaVersion::parse(" 17 ").unwrap().value, "17");
    }
}

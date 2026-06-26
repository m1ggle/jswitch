use std::collections::HashMap;

use crate::commands::install::JavaSource;

/// Resolves the final download URL for a given distribution source by applying
/// mirror overrides from `config.toml` or the `JSWITCH_MIRROR_URL` env var.
///
/// Resolution priority (highest wins):
/// 1. Per-source override (`[sources]` table in config)
/// 2. Global mirror (`global.mirror_url` or `JSWITCH_MIRROR_URL` env)
/// 3. Original upstream URL (no rewrite)
#[derive(Debug, Clone, Default)]
pub struct MirrorResolver {
    global_mirror: Option<String>,
    source_overrides: HashMap<JavaSource, String>,
}

impl MirrorResolver {
    pub fn from_config(config: &crate::config::Config) -> Self {
        let global_mirror = std::env::var("JSWITCH_MIRROR_URL")
            .ok()
            .or_else(|| config.global.mirror_url.clone());

        let mut source_overrides = HashMap::new();
        if let Some(url) = &config.sources.openjdk {
            source_overrides.insert(JavaSource::OpenJdk, url.clone());
        }
        if let Some(url) = &config.sources.corretto {
            source_overrides.insert(JavaSource::Corretto, url.clone());
        }
        if let Some(url) = &config.sources.adoptopenjdk {
            source_overrides.insert(JavaSource::Adoptopenjdk, url.clone());
        }
        if let Some(url) = &config.sources.oracle {
            source_overrides.insert(JavaSource::Oracle, url.clone());
        }

        Self {
            global_mirror,
            source_overrides,
        }
    }

    /// Return the effective download URL for `source`, rewriting the base
    /// when a mirror is configured.  If no mirror is set the original URL
    /// is returned unchanged.
    pub fn resolve(&self, source: JavaSource, original_url: &str) -> String {
        if let Some(override_base) = self.source_overrides.get(&source) {
            return replace_base(original_url, override_base);
        }
        if let Some(mirror_base) = &self.global_mirror {
            return replace_base(original_url, mirror_base);
        }
        original_url.to_owned()
    }

    /// True when at least one mirror (global or per-source) is active.
    pub fn has_mirror(&self) -> bool {
        self.global_mirror.is_some() || !self.source_overrides.is_empty()
    }
}

/// Replace the `scheme://host[:port]` prefix of `url` with `new_base`,
/// preserving the path and query string.
fn replace_base(url: &str, new_base: &str) -> String {
    let Some(scheme_end) = url.find("://") else {
        return url.to_owned();
    };

    let after_scheme = &url[scheme_end + 3..];
    let path_start = after_scheme.find('/');

    let path = match path_start {
        Some(idx) => &after_scheme[idx..],
        None => "",
    };

    format!("{}{}", new_base.trim_end_matches('/'), path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn returns_original_when_no_mirror() {
        let resolver = MirrorResolver::default();
        let url = "https://api.adoptium.net/v3/assets/feature_releases/17/ga";

        assert_eq!(resolver.resolve(JavaSource::Adoptopenjdk, url), url);
    }

    #[test]
    fn replaces_base_with_global_mirror() {
        let mut config = Config::default();
        config.global.mirror_url = Some("https://mirrors.tuna.tsinghua.edu.cn".to_owned());

        let resolver = MirrorResolver::from_config(&config);
        let url = "https://api.adoptium.net/v3/assets/feature_releases/17/ga";

        assert_eq!(
            resolver.resolve(JavaSource::Adoptopenjdk, url),
            "https://mirrors.tuna.tsinghua.edu.cn/v3/assets/feature_releases/17/ga"
        );
    }

    #[test]
    fn per_source_override_takes_priority() {
        let mut config = Config::default();
        config.global.mirror_url = Some("https://global-mirror.com".to_owned());
        config.sources.corretto = Some("https://corretto-mirror.com".to_owned());

        let resolver = MirrorResolver::from_config(&config);

        let corretto_url =
            "https://corretto.aws/downloads/latest/amazon-corretto-17-x64-linux-jdk.tar.gz";
        let adoptium_url = "https://api.adoptium.net/v3/assets";

        assert_eq!(
            resolver.resolve(JavaSource::Corretto, corretto_url),
            "https://corretto-mirror.com/downloads/latest/amazon-corretto-17-x64-linux-jdk.tar.gz"
        );
        assert_eq!(
            resolver.resolve(JavaSource::Adoptopenjdk, adoptium_url),
            "https://global-mirror.com/v3/assets"
        );
    }

    #[test]
    fn strips_trailing_slash_from_base() {
        let mut config = Config::default();
        config.global.mirror_url = Some("https://mirror.com/".to_owned());

        let resolver = MirrorResolver::from_config(&config);
        let url = "https://upstream.com/path/to/file";

        assert_eq!(
            resolver.resolve(JavaSource::Adoptopenjdk, url),
            "https://mirror.com/path/to/file"
        );
    }

    #[test]
    fn preserves_query_string() {
        let mut config = Config::default();
        config.global.mirror_url = Some("https://mirror.com".to_owned());

        let resolver = MirrorResolver::from_config(&config);
        let url = "https://upstream.com/path?architecture=x64&page_size=1";

        assert_eq!(
            resolver.resolve(JavaSource::Adoptopenjdk, url),
            "https://mirror.com/path?architecture=x64&page_size=1"
        );
    }

    #[test]
    fn handles_url_without_path() {
        let result = replace_base("https://upstream.com", "https://mirror.com");
        assert_eq!(result, "https://mirror.com");
    }

    #[test]
    fn returns_original_for_malformed_url() {
        let result = replace_base("not-a-url", "https://mirror.com");
        assert_eq!(result, "not-a-url");
    }
}

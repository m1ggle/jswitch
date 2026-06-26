use std::collections::HashMap;

use crate::commands::install::JavaSource;

/// Resolves the final download URL for a given distribution source.
///
/// Two layers of URL rewriting are supported:
///
/// 1. **Per-source override** (`[sources]` table in config or `sources.*` keys):
///    completely replaces the upstream base URL for that distribution.
///    The fetcher uses this value directly and appends only the archive
///    filename — no host-level rewriting is applied.
///
/// 2. **Global mirror** (`global.mirror_url` or `JSWITCH_MIRROR_URL` env):
///    replaces the `scheme://host[:port]` prefix of every URL, preserving
///    the path and query string.  Applied only when no per-source override
///    is active for the distribution.
///
/// Priority (highest wins):
/// 1. Per-source override — terminal, global mirror is not applied.
/// 2. Global mirror — host replacement.
/// 3. Original upstream URL (no rewrite).
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

    /// Returns the effective base URL for a distribution.
    /// If a per-source override is configured, return it; otherwise return
    /// `default_base` unchanged.
    pub fn base_url<'a>(&'a self, source: JavaSource, default_base: &'a str) -> &'a str {
        self.source_overrides
            .get(&source)
            .map(|s| s.as_str())
            .unwrap_or(default_base)
    }

    /// Returns true when a per-source override is configured for `source`.
    pub fn has_source_override(&self, source: JavaSource) -> bool {
        self.source_overrides.contains_key(&source)
    }

    /// Apply the global mirror (host replacement) to a fully-constructed URL.
    /// If no global mirror is set, the URL is returned unchanged.
    pub fn resolve_global(&self, url: &str) -> String {
        if let Some(mirror_base) = &self.global_mirror {
            return replace_base(url, mirror_base);
        }
        url.to_owned()
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

        assert_eq!(resolver.resolve_global(url), url);
    }

    #[test]
    fn replaces_host_with_global_mirror() {
        let mut config = Config::default();
        config.global.mirror_url = Some("https://mirrors.tuna.tsinghua.edu.cn".to_owned());

        let resolver = MirrorResolver::from_config(&config);
        let url = "https://api.adoptium.net/v3/assets/feature_releases/17/ga";

        assert_eq!(
            resolver.resolve_global(url),
            "https://mirrors.tuna.tsinghua.edu.cn/v3/assets/feature_releases/17/ga"
        );
    }

    #[test]
    fn source_override_replaces_base_url_completely() {
        let mut config = Config::default();
        config.sources.corretto = Some("https://my-mirror.com/corretto".to_owned());

        let resolver = MirrorResolver::from_config(&config);

        assert_eq!(
            resolver.base_url(
                JavaSource::Corretto,
                "https://corretto.aws/downloads/latest"
            ),
            "https://my-mirror.com/corretto"
        );
        assert!(resolver.has_source_override(JavaSource::Corretto));
    }

    #[test]
    fn source_override_does_not_affect_other_distributions() {
        let mut config = Config::default();
        config.sources.corretto = Some("https://corretto-mirror.com".to_owned());

        let resolver = MirrorResolver::from_config(&config);

        assert_eq!(
            resolver.base_url(JavaSource::Adoptopenjdk, "https://api.adoptium.net"),
            "https://api.adoptium.net"
        );
        assert!(!resolver.has_source_override(JavaSource::Adoptopenjdk));
    }

    #[test]
    fn strips_trailing_slash_from_base() {
        let mut config = Config::default();
        config.global.mirror_url = Some("https://mirror.com/".to_owned());

        let resolver = MirrorResolver::from_config(&config);
        let url = "https://upstream.com/path/to/file";

        assert_eq!(
            resolver.resolve_global(url),
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
            resolver.resolve_global(url),
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

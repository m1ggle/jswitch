use tracing_subscriber::{EnvFilter, fmt};

pub fn init_logging() {
    let filter = if std::env::var_os("RUST_LOG").is_some() {
        EnvFilter::from_default_env()
    } else if env_enabled("JSWITCH_DEBUG") {
        EnvFilter::new("jswitch=debug")
    } else {
        EnvFilter::new("warn")
    };

    let _ = fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_writer(std::io::stderr)
        .try_init();
}

pub fn quiet() -> bool {
    env_enabled("JSWITCH_QUIET")
}

fn env_enabled(name: &str) -> bool {
    std::env::var(name)
        .map(|value| {
            matches!(
                value.to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_missing_env_values_are_false() {
        assert!(!env_enabled("JSWITCH_TEST_ENV_THAT_SHOULD_NOT_EXIST"));
    }
}

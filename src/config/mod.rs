pub mod global;
pub mod local;
pub mod manager;
pub mod validator;

pub use crate::error::jswitch_error::ConfigError;
pub use global::Config;
pub use local::LocalConfig;
pub use manager::default_config_path;

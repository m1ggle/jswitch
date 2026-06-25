use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

pub mod cli;
pub mod commands;
pub mod config;
pub mod download;
pub mod env;
pub mod error;
pub mod utils;
pub mod version;

pub use error::{JswitchError, Result};

#[derive(Debug, Clone, Parser, Serialize, Deserialize)]
#[command(
    name = "jswitch",
    version,
    about = "A fast Java version switcher",
    arg_required_else_help = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, Subcommand, Serialize, Deserialize)]
pub enum Command {
    /// Initialize shell integration.
    Init(commands::init::InitArgs),
    /// Install a Java version.
    Install(commands::install::InstallArgs),
    /// Switch Java versions.
    Switch(commands::switch::SwitchArgs),
    /// List installed or remote Java versions.
    List(commands::list::ListArgs),
    /// Show the current active Java version.
    Current,
    /// Manage configuration.
    Config(commands::config::ConfigArgs),
    /// Generate shell completion scripts.
    Completion(commands::completion::CompletionArgs),
    /// Run health checks.
    Doctor,
}

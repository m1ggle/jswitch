use clap::Args;
use serde::{Deserialize, Serialize};

use crate::JswitchError;

#[derive(Debug, Clone, Args, Serialize, Deserialize)]
pub struct InstallArgs {
    pub version: Option<String>,

    #[arg(long, value_enum)]
    pub source: Option<JavaSource>,

    #[arg(long)]
    pub use_now: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, clap::ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum JavaSource {
    OpenJdk,
    Oracle,
    Corretto,
    Adoptopenjdk,
}

pub async fn run(args: InstallArgs) -> Result<(), JswitchError> {
    println!(
        "install Java version: {:?}, source: {:?}, use-now: {}",
        args.version, args.source, args.use_now
    );
    Ok(())
}

use crate::{Cli, Command, Result};

pub async fn dispatch(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Init(args) => crate::commands::init::run(args).await,
        Command::Install(args) => crate::commands::install::run(args).await,
        Command::Switch(args) => crate::commands::switch::run(args).await,
        Command::List(args) => crate::commands::list::run(args).await,
        Command::Current => crate::commands::current::run().await,
        Command::Config(args) => crate::commands::config::run(args).await,
        Command::Doctor => crate::commands::doctor::run().await,
    }
}

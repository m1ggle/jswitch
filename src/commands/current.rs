use crate::{Result, config::Config, version::VersionResolver};

pub async fn run() -> Result<()> {
    let config = Config::load_or_default()?;
    let resolver = VersionResolver::new(config);

    match resolver.current()? {
        Some(current) => println!("{}", current.version),
        None => println!("no Java version selected"),
    }

    Ok(())
}

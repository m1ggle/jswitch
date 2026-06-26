use clap::Args;
use serde::{Deserialize, Serialize};

use crate::{
    Result,
    env::{JavaEnvironment, Shell},
};

#[derive(Debug, Clone, Args, Serialize, Deserialize)]
pub struct EnvArgs {
    /// Show raw environment variable values (no 'export' prefix).
    #[arg(long)]
    pub raw: bool,

    /// Verify the Java installation and print diagnostics.
    #[arg(long)]
    pub verify: bool,
}

pub async fn run(args: EnvArgs) -> Result<()> {
    let env = match JavaEnvironment::from_current_version()? {
        Some(e) => e,
        None => {
            println!("# no Java version selected");
            return Ok(());
        }
    };

    if args.verify {
        let valid = env.verify()?;
        let executables = env.list_executables()?;

        println!(
            "JAVA_HOME: {}",
            env.java_home.as_deref().unwrap_or("<unset>")
        );
        println!("valid installation: {}", if valid { "yes" } else { "NO" });
        println!("bin executables found: {}", executables.len());

        if !executables.is_empty() {
            println!("\navailable tools:");
            for exe in &executables {
                println!("  - {}", exe);
            }
        }

        return Ok(());
    }

    let statements = env.export_for_shell(Shell::Bash);

    if statements.is_empty() {
        println!("# no Java environment to export");
        return Ok(());
    }

    if args.raw {
        for stmt in &statements {
            // Strip "export " prefix and quotes for raw output
            let cleaned = stmt
                .strip_prefix("export ")
                .unwrap_or(stmt)
                .replace('"', "");
            println!("{}", cleaned);
        }
    } else {
        for stmt in &statements {
            println!("{}", stmt);
        }
    }

    Ok(())
}

use clap::Parser;
use jswitch::{
    Cli, Command,
    commands::{alias::AliasAction, completion::CompletionShell, install::JavaSource},
};

#[test]
fn parses_install_arguments() {
    let cli = Cli::parse_from([
        "jswitch",
        "install",
        "17",
        "--source",
        "corretto",
        "--use-now",
    ]);

    match cli.command {
        Command::Install(args) => {
            assert_eq!(args.version.as_deref(), Some("17"));
            assert_eq!(args.source, Some(JavaSource::Corretto));
            assert!(args.use_now);
        }
        command => panic!("expected install command, got {command:?}"),
    }
}

#[test]
fn parses_switch_scope_flags() {
    let cli = Cli::parse_from(["jswitch", "switch", "21", "--global"]);

    match cli.command {
        Command::Switch(args) => {
            assert_eq!(args.version.as_deref(), Some("21"));
            assert!(args.global);
            assert!(!args.local);
            assert!(!args.session);
        }
        command => panic!("expected switch command, got {command:?}"),
    }
}

#[test]
fn parses_remove_command() {
    let cli = Cli::parse_from(["jswitch", "remove", "11", "--force"]);

    match cli.command {
        Command::Remove(args) => {
            assert_eq!(args.version, "11");
            assert!(args.force);
        }
        command => panic!("expected remove command, got {command:?}"),
    }
}

#[test]
fn parses_alias_set_command() {
    let cli = Cli::parse_from(["jswitch", "alias", "set", "lts", "21"]);

    match cli.command {
        Command::Alias(args) => match args.action {
            AliasAction::Set { name, version } => {
                assert_eq!(name, "lts");
                assert_eq!(version, "21");
            }
            action => panic!("expected alias set action, got {action:?}"),
        },
        command => panic!("expected alias command, got {command:?}"),
    }
}

#[test]
fn parses_config_set() {
    let cli = Cli::parse_from(["jswitch", "config", "set", "default_version", "17"]);

    match cli.command {
        Command::Config(args) => {
            let debug = format!("{:?}", args.action);
            assert!(debug.contains("Set"));
            assert!(debug.contains("default_version"));
            assert!(debug.contains("17"));
        }
        command => panic!("expected config command, got {command:?}"),
    }
}

#[test]
fn parses_completion_shell() {
    let cli = Cli::parse_from(["jswitch", "completion", "zsh"]);

    match cli.command {
        Command::Completion(args) => assert_eq!(args.shell, CompletionShell::Zsh),
        command => panic!("expected completion command, got {command:?}"),
    }
}

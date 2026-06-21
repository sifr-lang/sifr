use crate::cli_model_and_entrypoint::{BridgeCommands, Cli, Commands};
use clap::Parser;

#[test]
fn bridge_check_cli_parses_workspace_selection_and_lock_flags() {
    let cli = Cli::try_parse_from([
        "sifr",
        "bridge",
        "check",
        "--workspace",
        "-p",
        "demo-app",
        "--exclude",
        "demo-tools",
        "--frozen",
    ])
    .expect("bridge check cli parses");

    let Some(Commands::Bridge {
        command:
            BridgeCommands::Check {
                workspace,
                packages,
                exclude,
                frozen,
                ..
            },
    }) = cli.command
    else {
        panic!("expected bridge check command");
    };

    assert!(workspace);
    assert_eq!(packages, ["demo-app"]);
    assert_eq!(exclude, ["demo-tools"]);
    assert!(frozen);
}

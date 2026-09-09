use super::DeferredArgs;
use crate::cli_model_and_entrypoint::Cli;
use crate::formatter_cli::FmtArgs;
use crate::lint_cli::LintArgs;
use crate::python_cli::PythonArgs;
use crate::self_update_cli::SelfArgs;
use clap::{Arg, ArgAction, ArgMatches, Args, Command, CommandFactory, FromArgMatches};
use std::sync::atomic::{AtomicUsize, Ordering};

fn root(name: &'static str, operation: Command) -> Command {
    Command::new("sifr")
        .arg(
            Arg::new("config")
                .long("config")
                .global(true)
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("isolated")
                .long("isolated")
                .global(true)
                .action(ArgAction::SetTrue),
        )
        .arg(Arg::new("sysroot").long("sysroot").global(true).hide(true))
        .subcommand(operation.name(name))
}

fn equivalent<T: Args>(name: &'static str, cases: &[&[&str]]) {
    let eager = root(name, T::augment_args(Command::new(name)));
    let deferred = root(name, DeferredArgs::<T>::augment_args(Command::new(name)));
    for args in cases {
        let before = eager.clone().try_get_matches_from(*args);
        let after = deferred.clone().try_get_matches_from(*args);
        match (before, after) {
            (Ok(mut before), Ok(mut after)) => {
                assert_eq!(before, after, "{args:?}");
                let (_, mut before) = before.remove_subcommand().unwrap();
                let (_, mut after) = after.remove_subcommand().unwrap();
                assert_eq!(
                    T::from_arg_matches(&before).is_ok(),
                    DeferredArgs::<T>::from_arg_matches(&after).is_ok()
                );
                assert!(T::from_arg_matches_mut(&mut before).is_ok(), "{args:?}");
                assert!(
                    DeferredArgs::<T>::from_arg_matches_mut(&mut after).is_ok(),
                    "{args:?}"
                );
                assert_eq!(before, after, "consumed matches: {args:?}");
            }
            (Err(before), Err(after)) => {
                assert_eq!(before.kind(), after.kind(), "{args:?}");
                assert_eq!(before.to_string(), after.to_string(), "{args:?}");
                assert_eq!(before.exit_code(), after.exit_code());
            }
            (before, after) => {
                panic!("different parser outcomes for {args:?}: {before:?}, {after:?}")
            }
        }
    }
}

#[test]
fn deferred_cli_args_preserve_formatter_and_lint_contracts() {
    equivalent::<FmtArgs>(
        "fmt",
        &[
            &["sifr", "fmt", "--check", "--no-cache", "main.sifr"],
            &[
                "sifr",
                "--config",
                "a.toml",
                "fmt",
                "--config",
                "b.toml",
                "--exclude",
                "one,two",
                "a.sifr",
                "b.sifr",
            ],
            &["sifr", "fmt", "--preview", "--no-preview"],
            &["sifr", "fmt", "--line-length", "0"],
            &["sifr", "fmt", "--check", "--diff"],
            &["sifr", "fmt", "--unknown"],
            &["sifr", "fmt", "--help"],
        ],
    );
    equivalent::<LintArgs>(
        "lint",
        &[
            &["sifr", "lint", "--select", "all", "main.sifr"],
            &["sifr", "--config", "a.toml", "lint", "--config", "b.toml"],
            &["sifr", "lint", "--unknown"],
            &["sifr", "lint", "--help"],
        ],
    );
}

#[test]
fn deferred_cli_args_preserve_nested_help_and_required_arguments() {
    equivalent::<PythonArgs>(
        "python",
        &[
            &["sifr", "python"],
            &["sifr", "python", "check", "--json"],
            &["sifr", "python", "doctor", "--json"],
            &["sifr", "python", "bind", "--help"],
            &[
                "sifr",
                "python",
                "certify",
                "arrow",
                "module.func",
                "--fixture",
                "fixture.py",
            ],
            &["sifr", "python", "certify", "arrow", "module.func"],
            &[
                "sifr",
                "python",
                "certify",
                "dlpack",
                "module.func",
                "--fixture",
                "fixture.py",
            ],
            &[
                "sifr", "--config", "a.toml", "python", "certify", "--check", "--config", "b.toml",
            ],
            &["sifr", "python", "certify", "--help"],
            &["sifr", "help", "python", "certify", "arrow"],
            &["sifr", "python", "--unknown"],
            &["sifr", "python", "--help"],
        ],
    );
    equivalent::<SelfArgs>(
        "self",
        &[
            &["sifr", "self"],
            &["sifr", "self", "version", "--short"],
            &["sifr", "self", "update", "--channel", "stable", "--dry-run"],
            &["sifr", "self", "update", "--format", "invalid"],
            &["sifr", "self", "version", "--help"],
            &["sifr", "help", "self", "update"],
        ],
    );
}

#[derive(Debug, PartialEq, Args)]
#[group(required = true, multiple = false)]
struct UpdateContract {
    #[arg(long)]
    left: Option<String>,
    #[arg(long)]
    right: Option<String>,
}

#[test]
fn deferred_cli_args_preserve_groups_and_update_semantics() {
    equivalent::<UpdateContract>(
        "operation",
        &[
            &["sifr", "operation"],
            &["sifr", "operation", "--left", "one"],
            &["sifr", "operation", "--left", "one", "--right", "two"],
        ],
    );
    assert_eq!(
        DeferredArgs::<UpdateContract>::group_id(),
        UpdateContract::group_id()
    );
    for mutable in [false, true] {
        let mut before = UpdateContract {
            left: Some("one".into()),
            right: None,
        };
        let mut after = DeferredArgs(UpdateContract {
            left: Some("one".into()),
            right: None,
        });
        let mut eager_matches = UpdateContract::augment_args_for_update(Command::new("update"))
            .try_get_matches_from(["update", "--left", "two"])
            .unwrap();
        let mut deferred_matches =
            DeferredArgs::<UpdateContract>::augment_args_for_update(Command::new("update"))
                .try_get_matches_from(["update", "--left", "two"])
                .unwrap();
        assert_eq!(eager_matches, deferred_matches);
        if mutable {
            before
                .update_from_arg_matches_mut(&mut eager_matches)
                .unwrap();
            after
                .update_from_arg_matches_mut(&mut deferred_matches)
                .unwrap();
        } else {
            before.update_from_arg_matches(&eager_matches).unwrap();
            after.update_from_arg_matches(&deferred_matches).unwrap();
        }
        assert_eq!(before.left.as_deref(), Some("two"));
        assert_eq!(before, after.0);
        assert_eq!(eager_matches, deferred_matches);
    }
}

static AUGMENTATIONS: AtomicUsize = AtomicUsize::new(0);

struct ConstructionProbe {}

impl FromArgMatches for ConstructionProbe {
    fn from_arg_matches(_matches: &ArgMatches) -> Result<Self, clap::Error> {
        Ok(Self {})
    }

    fn update_from_arg_matches(&mut self, _matches: &ArgMatches) -> Result<(), clap::Error> {
        Ok(())
    }
}

impl Args for ConstructionProbe {
    fn augment_args(command: Command) -> Command {
        AUGMENTATIONS.fetch_add(1, Ordering::SeqCst);
        command
    }

    fn augment_args_for_update(command: Command) -> Command {
        Self::augment_args(command)
    }
}

#[test]
fn deferred_cli_args_do_not_construct_unselected_commands() {
    let command = Command::new("sifr")
        .subcommand(Command::new("selected"))
        .subcommand(DeferredArgs::<ConstructionProbe>::augment_args(
            Command::new("other"),
        ));
    assert_eq!(AUGMENTATIONS.load(Ordering::SeqCst), 0);
    command
        .clone()
        .try_get_matches_from(["sifr", "selected"])
        .unwrap();
    assert_eq!(AUGMENTATIONS.load(Ordering::SeqCst), 0);
    command.try_get_matches_from(["sifr", "other"]).unwrap();
    assert_eq!(AUGMENTATIONS.load(Ordering::SeqCst), 1);
}

#[test]
fn deferred_cli_args_complete_cli_model_is_consistent() {
    Cli::command().debug_assert();
}

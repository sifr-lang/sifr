//! Keep argument construction with its owner and delay it until command use.

use clap::{ArgMatches, Args, Command, Error, FromArgMatches, Id};

pub(crate) struct DeferredArgs<T>(pub(crate) T);

impl<T: Args> Args for DeferredArgs<T> {
    fn group_id() -> Option<Id> {
        T::group_id()
    }

    fn augment_args(command: Command) -> Command {
        command.defer(|command| augment_before_inherited_globals(command, T::augment_args))
    }

    fn augment_args_for_update(command: Command) -> Command {
        command
            .defer(|command| augment_before_inherited_globals(command, T::augment_args_for_update))
    }
}

fn augment_before_inherited_globals(command: Command, augment: fn(Command) -> Command) -> Command {
    // Clap propagates parent globals before invoking a deferred child builder.
    // Eager Args are built before that propagation. Keep their declaration order
    // for help, diagnostics, default-value indices and match introspection.
    let inherited = command
        .get_arguments()
        .filter(|arg| arg.is_global_set())
        .map(|arg| arg.get_id().clone())
        .collect::<Vec<_>>();
    let mut command = augment(command);
    for id in inherited {
        command = command.mut_arg(id, std::convert::identity);
    }
    command
}

impl<T: FromArgMatches> FromArgMatches for DeferredArgs<T> {
    fn from_arg_matches(matches: &ArgMatches) -> Result<Self, Error> {
        T::from_arg_matches(matches).map(Self)
    }

    fn from_arg_matches_mut(matches: &mut ArgMatches) -> Result<Self, Error> {
        T::from_arg_matches_mut(matches).map(Self)
    }

    fn update_from_arg_matches(&mut self, matches: &ArgMatches) -> Result<(), Error> {
        self.0.update_from_arg_matches(matches)
    }

    fn update_from_arg_matches_mut(&mut self, matches: &mut ArgMatches) -> Result<(), Error> {
        self.0.update_from_arg_matches_mut(matches)
    }
}

#[cfg(test)]
#[path = "deferred_cli_args_tests.rs"]
mod tests;

//! Keep argument construction with its owner and delay it until command use.

use clap::{ArgMatches, Args, Command, Error, FromArgMatches, Id};

pub(crate) struct DeferredArgs<T>(pub(crate) T);

impl<T: Args> Args for DeferredArgs<T> {
    fn group_id() -> Option<Id> {
        T::group_id()
    }

    fn augment_args(command: Command) -> Command {
        command.defer(T::augment_args)
    }

    fn augment_args_for_update(command: Command) -> Command {
        command.defer(T::augment_args_for_update)
    }
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

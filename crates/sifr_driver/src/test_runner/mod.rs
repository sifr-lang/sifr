mod artifacts;
mod execution;
mod orchestrator;

pub use orchestrator::run_tests;

#[cfg(test)]
pub(crate) use artifacts::{compose_test_runner_lib, generate_test_runner_cargo_toml};
#[cfg(test)]
pub(crate) use execution::execute_test_runner_project;
#[cfg(test)]
pub(crate) use orchestrator::build_test_runner_project;

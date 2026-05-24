//! Intrinsic registry and dispatch for incremental IR rollout.

mod registry;
pub(crate) use registry::*;

#[cfg(test)]
mod registry_core_tests;
#[cfg(test)]
mod registry_extended_tests;

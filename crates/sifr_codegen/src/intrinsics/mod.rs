//! Intrinsic registry and dispatch for incremental IR rollout.

mod registry;
pub(crate) use registry::*;

#[cfg(test)]
#[path = "../../tests/unit/intrinsics/registry_core_tests.rs"]
mod registry_core_tests;
#[cfg(test)]
#[path = "../../tests/unit/intrinsics/registry_extended_tests.rs"]
mod registry_extended_tests;

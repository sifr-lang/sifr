//! Intrinsic registry and dispatch for incremental IR rollout.

include!("registry.rs");

#[cfg(test)]
mod tests {
    include!("registry_core_tests.rs");
    include!("registry_extended_tests.rs");
}

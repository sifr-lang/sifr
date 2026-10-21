//! Project-cache regressions exercise the shared production lease owner.
use sifr_cache_storage::ProcessLease;
use std::{fs::File, io};

#[cfg(unix)]
#[path = "process_lease_tests.rs"]
mod tests;

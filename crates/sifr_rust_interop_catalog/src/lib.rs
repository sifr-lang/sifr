//! Lockfile and cache catalog for the Rust-interop certification matrix.
//!
//! The dependencies are intentionally optional. Their presence in this
//! package keeps every matrix crate and exact version in the workspace
//! `Cargo.lock`, while ordinary compiler builds do not compile the deferred
//! ecosystem graph.

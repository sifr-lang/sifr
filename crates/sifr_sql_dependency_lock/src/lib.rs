//! Qualification-only dependency root for the schema-first SQL platform.
//!
//! This crate has no runtime API. Its optional dependency edges make every
//! approved direct dependency part of the root workspace lockfile before the
//! provider crates exist. Provider implementations consume the same workspace pins.

/// The schema version of the checked-in SQL dependency baseline.
pub const BASELINE_SCHEMA_VERSION: u32 = 1;

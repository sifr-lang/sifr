//! Tier-2 Rust scaffold for the Sifr negative fixtures in this folder.
//!
//! The paired Sifr programs are rejected because these import forms must not
//! trigger project-mode resolution: bare relative imports, multi-level relative
//! imports, and plain `import helper` statements are all explicitly unsupported
//! in single-file resolution and still surface the reachable unresolved-name
//! follow-on diagnostics.
//!
//! This folder exists to preserve the resolver resolver-trigger rules. The
//! important behavior is not a generic Rust import failure, but Sifr's promise
//! that unsupported local import syntax does not silently switch compilation
//! modes. This file therefore records the frontend rules directly.

fn main() {}

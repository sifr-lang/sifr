//! Tier-2 Rust scaffold for the Sifr negative fixtures in this folder.
//!
//! The paired Sifr programs are rejected because bare relative imports,
//! multi-level relative imports, and plain `import helper` statements are
//! explicitly unsupported inside the declared workspace.
//!
//! This folder preserves explicit diagnostics for unsupported import syntax.
//! This file records the frontend rules directly rather than modeling a generic
//! Rust import failure.

fn main() {}

//! Tier-2 Rust scaffold for the Sifr negative fixtures in this folder.
//!
//! The paired Sifr programs are rejected because user code may not use bare
//! relative imports, multi-level relative imports, or plain `import helper`
//! statements. Each fixture also leaves the imported name unresolved, so the
//! diagnostics include the reachable `undefined variable` or `undefined function`
//! follow-on error alongside the explicit unsupported-import message.
//!
//! This folder exists to preserve the import-form import-form diagnostics:
//! unsupported bare relative imports must point users at `from <module> import`,
//! multi-level relative imports must report the rejected level explicitly, and
//! plain `import ...` statements must remain unsupported. The Rust-side analogue
//! is module resolution policy owned by Sifr, so this file documents the
//! rules instead of inventing a Rust compile-fail sample.

fn main() {}

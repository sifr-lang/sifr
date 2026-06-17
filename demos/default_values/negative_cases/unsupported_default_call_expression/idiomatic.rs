//! Tier-2 Rust scaffold for the Sifr negative fixture in this folder.
//!
//! The paired Sifr program is rejected because `pick(x: int = seed())` uses a
//! call expression as a default argument, which Sifr does not allow. Because the
//! default is rejected, the later `pick()` call also surfaces the deterministic
//! follow-on diagnostic that `x` is still required.
//!
//! This fixture exists to preserve the default-argument default-argument restriction and
//! its missing-argument follow-on behavior. The Rust-side analogue is frontend
//! policy rather than a direct Rust compile error, so this file records the
//! rules instead of inventing a synthetic sample.

fn main() {}

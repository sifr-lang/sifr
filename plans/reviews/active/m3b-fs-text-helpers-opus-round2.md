Blocking finding:

**Stale retained intrinsic signatures for `_sifr.io`** — `crates/sifr_retained_intrinsics/src/io_json.rs:47-81` still registers `read_text`, `write_text`, `exists`, and `read_lines` under `intrinsic_io()`. The M3b diff removed these from `sys_fs.rs::intrinsic_fs()` and `stdlib/_sifr/io.sifr` is now an empty stub, so these retained signatures are dead surface that contradicts the milestone goal of fully removing the helpers from retained intrinsic registrations, and they leave a shadow resolution path competing with the new compiled `_sifr.fs` declarations.

Fix: delete the four `functions.insert(...)` blocks at `crates/sifr_retained_intrinsics/src/io_json.rs:47-81` so `intrinsic_io()` returns an empty `functions` map, matching the empty `stdlib/_sifr/io.sifr`. Re-run `check_stdlib_retained_intrinsic_*` guardrails and the retained-intrinsics unit tests.

Everything else reviewed (registry removal, `_sifr/fs.sifr` decls, wrapper ownership, feature-`fs` inference and Cargo plumbing in normal + e2e batch paths, closure guardrail, harness fixtures, retained TOML) is clean.

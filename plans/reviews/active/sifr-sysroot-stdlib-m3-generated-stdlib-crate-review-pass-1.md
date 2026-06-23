I have enough to assess. Let me compose findings.

## Findings

### Blocking

**1. Feature-gated wrapper tests never run in any CI profile** — `verification/profiles/{create-pr,merge,nightly,release}.json` only register `cargo test -p sifr_stdlib` with default features. The crate's `default = []` (`crates/sifr_stdlib/Cargo.toml:14`), and every behavior test in `crates/sifr_stdlib/tests/api_behavior.rs:8-36` is `#[cfg(feature = "...")]` gated (json/unicode/i18n/http). Result: only the trivial `crate_identity_is_generated_program_stdlib` actually executes under any profile mode. A future regression in `json::validate_integer_digit_limit`, `unicode::normalize`, `i18n::canonicalize_locale`, or `http::HeaderName::new` would land green. The validation log records that you exercised these combos manually, but the M3 scope item "Add direct Rust API behavior tests and feature-combination evidence" needs that evidence encoded as a durable gate (compare with the existing `sifr_runtime_http` entry in `verification/profiles/*.json:54` that pins `--features http`). At minimum, profiles should run `cargo test -p sifr_stdlib --features json,unicode,i18n,http` (and a `--no-default-features --all-features` variant for nightly to cover the `python` path).

### Non-blocking but worth addressing in the same PR

**2. Cargo tree snapshots are not regenerated or diffed by any verification script.** `verification/areas/stdlib_parity/data/sifr_stdlib_feature_tree_snapshots/` is referenced only by its own README; no script under `verification/` or `scripts/` regenerates or asserts these files. They will drift silently the first time the runtime or stdlib feature graph changes. Either add a regeneration step to the stdlib_parity area or note explicitly in the README that they are point-in-time evidence (not regression baselines).

**3. `planned_sifr_stdlib_features` duplicates the module→feature topology already encoded in `features_for_stdlib_module`** — `crates/sifr_stdlib_model/src/features.rs:601-679` and `crates/sifr_stdlib_model/src/features/generated_stdlib_features.rs:24-47` both map module names. They use overlapping but non-identical coverage (the new one omits `sifr.time`, `sifr.datetime`, `sifr.random`, `sifr.encoding`, `sifr.ipc`, `sifr.parallel`, `_bigint`; existing one omits `sifr.process`, `sifr.io`, `sifr.os`, `sifr.shutil`, `sifr.tempfile`, `sifr.signal`). These two paths will diverge under maintenance. Consider a single source-of-truth table that both surfaces (the cargo-dep planner and the new sifr_stdlib feature planner) project from, or at least a test that asserts every module name appears in both tables or in a known "skip" set.

**4. `zipfile = ["gzip"]` in `crates/sifr_stdlib/Cargo.toml:23` violates the "leaf feature" framing.** The M3 doc says these are leaves; `zipfile` is documented as a leaf in `feature_contract::LEAF_FEATURES`, but its definition implicitly enables `gzip`. If this is intentional (zip archives' deflate path), say so in a comment; otherwise split them.

**5. Exported APIs that no test exercises** — `default_integer_digit_limit` (`crates/sifr_stdlib/src/json.rs:7`), `unicode::is_normalized` / `unicode::data_version` (`crates/sifr_stdlib/src/unicode.rs:5,10`), `i18n::format_number` (`crates/sifr_stdlib/src/i18n.rs:5`), `runtime_observability::emit_diagnostic` (`crates/sifr_stdlib/src/runtime_observability.rs:1`). The "owned wrappers" claim in the milestone scope is undertested. `HeaderName::new` also has only one invalid-name case (`"bad header"` in `tests/api_behavior.rs:35`) — empty-string and non-ASCII cases are not covered.

**6. Marker-only modules (`base64.rs`, `fs.rs`, `gzip.rs`, `hash.rs`, `net.rs`, `process.rs`, `python.rs`, `regex.rs`, `signals.rs`, `tls.rs`, `toml.rs`, `url.rs`, `uuid.rs`, `zipfile.rs`)** each contain a single `feature_name()` const string with no caller. Either (a) reference them from `feature_contract` to assert each leaf has a present module under cfg, or (b) drop them until the M4+ milestone wires real APIs. As-is, they're dead surface and have no compile-time meaning beyond proving the feature key resolves a `mod`. The `lib.rs:9-46` `#[cfg(feature = "...")] pub mod X;` lines already provide that proof.

**7. Workspace `sifr_runtime = { default-features = false }` (Cargo.toml:54) is a workspace-wide change.** Today only `sifr_stdlib` consumes the workspace dep, so it's effectively a no-op. But every future crate that adds a `sifr_runtime = { workspace = true }` entry will silently inherit `default-features = false` — possibly surprising. Consider scoping the override to the `sifr_stdlib` crate's own dep entry only (it already restates `default-features = false` at `crates/sifr_stdlib/Cargo.toml:10`). Minor.

### Verified-clean items

- `internal_docs/architecture.md:53-56`, `sifr_sysroot_and_stdlib_architecture.md:30-32,385-391`, and `plans/issues/active/ad-hoc-sifr-sysroot-stdlib-toolchain.md:11-22` consistently advance M3 status and reflect the new crate.
- `scripts/check_source_crate_dependency_direction.py:34-95,132-141,290` correctly classifies `sifr_stdlib` as generated-program-only and bans compiler/parser deps in both directions.
- `crates/sifr_runtime/src/python/resource_ops.rs:6` PyModule import removal is genuinely unused (no remaining `PyModule` reference in the file body).
- `crates/sifr_stdlib_model/src/features_tests.rs:160-205` correctly asserts no umbrella features leak into the planner output and that representative modules stay minimal.

---

review-blocked

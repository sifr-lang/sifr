I have the full picture. Let me write up the review.

# M10 Wave 10 Datetime Interop Migration — Follow-up Review (Pass 4)

## Verdict: **PASS**

The pass-3 actionable E2E fixture harness gap is **fully resolved**. Batch grouped-fixture manifests now correctly request `sifr_stdlib` with the `time` feature for datetime consumers and `gzip`/`zipfile` for compression consumers, and dedicated tests lock in the behavior. No new blocking issues surface in the delta since pass 3.

## Pass-3 concern resolution

**Concern (pass 3 §1):** `needs_sifr_stdlib_module_dependency` did not list `sifr.datetime`/`_sifr.datetime` (and had latent gaps for `sifr.gzip`/`sifr.zipfile`/`_sifr.compress` from wave 9); `sifr_stdlib_dependency_spec_for_modules` had no arms to emit `"time"`, `"gzip"`, or `"zipfile"`.

**Resolved:**
- `crates/sifr/tests/e2e_support/fixture_cargo_toml.rs:39` — datetime removed from the raw-chrono arm; only `sifr.time`/`_sifr.time` still request direct `chrono` (correct — `sifr.time` bootstrap remains on active intrinsics).
- `crates/sifr/tests/e2e_support/fixture_cargo_toml.rs:324-328` — `needs_sifr_stdlib_module_dependency` now enumerates `sifr.datetime`, `_sifr.datetime`, `sifr.gzip`, `sifr.zipfile`, `_sifr.compress`, closing both the wave-10 gap and the retroactive wave-9 gap.
- `crates/sifr/tests/e2e_support/fixture_dependency_paths.rs:89-97` — new arms emit `"time"`, `"gzip"`, `"zipfile"` when the corresponding modules are present. Feature ordering is stable and deterministic (`gzip` before `zipfile` for `_sifr.compress`), and combined with existing arms produces `features = ["gzip", "zipfile"]` for the private compress leaf.
- `crates/sifr/tests/e2e_support/stateless_sysroot_cargo_toml_tests.rs:38-58` — four new test cases lock in: `sifr.datetime` → `features = ["time"]` with no raw `chrono`; `sifr.gzip` → `["gzip"]` with no raw `flate2`; `sifr.zipfile` → `["zipfile"]` with no raw `zip`; `_sifr.compress` → `["gzip", "zipfile"]`. The existing combined assertion (line 60-62) is unaffected since the datetime/compression modules aren't in `combined_modules`.

**Cross-check:** For a mixed batch fingerprint like `{sifr.datetime, _sifr.datetime, _sifr.time, sifr.test}`, `_sifr.time` still contributes direct `chrono` via the retained arm (needed by unmigrated `sifr.time` bootstrap intrinsics), and `sifr.datetime`/`_sifr.datetime` layer on `sifr_stdlib` with `"time"`. Both entries coexist without conflict. The generated Rust from the migrated datetime declarations calls `sifr_stdlib::time::*`, which pulls chrono transitively through the feature — no direct chrono dependency required.

## Behaviour and adapter policy (unchanged since pass 3)

- `stdlib/_sifr/datetime.sifr` — all four declarations bind directly to `@rust(sifr_stdlib.time.*, panic=trusted_no_panic)`; no `@rust.via`, `bridge.`, or converter/pipeline metadata. Adapter-policy fleet test (`crates/sifr_driver/src/stdlib/stateless_private_adapter_policy_tests.rs:31-33`) now includes `_sifr.datetime`.
- `crates/sifr_codegen/src/intrinsics/registry.rs` — datetime intrinsic dispatch removed; `registry/datetime.rs` deleted; `registry_extended_tests.rs:220` now negatively asserts that the four datetime intrinsic names are owned by private stdlib declarations.
- `crates/sifr_stdlib/src/time.rs:34` — `datetime_from_timestamp` returns `Err(io::Error::other("invalid timestamp"))`; `datetime_format` still returns `""` on parse failure.
- `crates/sifr_stdlib/Cargo.toml:92` — `time = ["dep:chrono", "dep:sifr_runtime"]` (sifr_runtime needed for `SifrIntBridge`).
- `crates/sifr_stdlib_model/src/features.rs:644` — `sifr.datetime`/`_sifr.datetime` correctly drop direct `Chrono` (feature flows via the sifr_stdlib `time` feature).

## Non-blocking notes carried forward from pass 3

These remain unresolved but are cosmetic / optional and do not block PR:

1. **`ts as i64` saturating cast** (`crates/sifr_stdlib/src/time.rs:35`) — NaN silently maps to epoch instead of `invalid timestamp`. Not a regression from pre-migration behaviour.
2. **`std::io::Error` as adapter error type** — semantically awkward (no I/O); bridge only uses `.to_string()`, so runtime behaviour is unaffected.
3. **Bootstrap intrinsic signatures retained** (`crates/sifr_stdlib_model/src/platform_misc.rs:64`) — consistent with the documented "keep until final registry sweep" pattern.
4. **`internal_docs/stdlib_native_surface_ownership.toml`** — the `_sifr.datetime` surface entry itself is updated to reflect the migration (lines 122-129), but `_sifr.time`'s `public_modules` at line 133 still lists `sifr.datetime`. Technically still true (`stdlib/sifr/datetime.sifr:3` still imports `_sifr.time.time_now`), so leaving it is defensible.

## Required before PR

Only what `AGENTS.md` already mandates: `scripts/run_all_tests.sh --profile create-pr`. The focused validation the wave already ran (formatting, guardrails, the two grouped cargo-toml tests, plus the datetime demo runs) covers the pass-3 concern directly; the batch e2e path should now pass on datetime-only buckets rather than depending on batch-fingerprint mixing to mask the gap.

Ready for create-pr validation and PR.

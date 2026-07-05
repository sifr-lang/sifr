Based on my review of the datetime interop migration, here is my assessment:

# M10 Wave 10 Datetime Interop Migration — Code Review

## Verdict: **PASS WITH NON-BLOCKING NOTES**

The adapter policy is followed, the public API surface is preserved, the compiler intrinsic dispatch is cleanly removed, and the required error contracts (`ValueError("invalid timestamp")`, empty-string-on-format-mismatch) are preserved by both the adapter unit tests and the driver codegen test. However, one gap in the e2e batch harness mirrors an unfixed pattern from wave 9 that warrants verification before PR.

## Strengths

- **Adapter policy adherence** (`stdlib/_sifr/datetime.sifr`, `stateless_private_adapter_policy_tests.rs:64`): all four declarations bind directly to `@rust(sifr_stdlib.time.*, panic=trusted_no_panic)`. No `@rust.via`, `bridge.*`, or converter/pipeline metadata. The new adapter-policy fleet test now includes `_sifr.datetime`.
- **Registry cleanup**: `crates/sifr_codegen/src/intrinsics/registry.rs` drops all four datetime lowerers, `registry/datetime.rs` is deleted, and `registry_extended_tests.rs:220` now asserts (not lowers) that these names are owned by the private stdlib declarations.
- **Error preservation** (`crates/sifr_stdlib/src/time.rs:34`): `datetime_from_timestamp` returns `Err(io::Error::other("invalid timestamp"))`, and the codegen test at `stateless_private_codegen_tests.rs:730` asserts the bridge emits `map_err(|__sifr_bridge_error| ValueError { message: __sifr_bridge_error.to_string() })`. Verified end-to-end by `edge_case_validation.sifr` expecting `"caught ValueError: invalid timestamp"`.
- **`datetime_format` behavior** (`time.rs:28`): still returns `""` on parse failure via `unwrap_or_default()`, matching the review's explicit acceptance criterion.
- **Feature planning**: `sifr_stdlib`'s `time` feature now gates `dep:chrono` and `dep:sifr_runtime` (needed for `SifrIntBridge`); driver-side `features_for_stdlib_module` drops direct `Chrono` from datetime and the `time` feature is already wired via `generated_stdlib_features.rs:52`. Fixture harness no longer emits raw `chrono` for datetime modules.
- **Public wrappers Sifr-owned**: `stdlib/sifr/datetime.sifr` is untouched beyond nothing; datetime/date/time/timedelta/timezone all remain Sifr code that just calls the three private leaves.

## Non-blocking concerns

1. **E2E fixture harness gap (probably latent, matches wave 9)** — `crates/sifr/tests/e2e_support/fixture_cargo_toml.rs:300` — `needs_sifr_stdlib_module_dependency` does **not** list `sifr.datetime` / `_sifr.datetime` (nor did it list `sifr.gzip` / `_sifr.compress` after wave 9), and `sifr_stdlib_dependency_spec_for_modules` in `fixture_dependency_paths.rs:40` has no arm to emit the `time` feature. For a batch group whose dependency fingerprint is exactly `{sifr.datetime, _sifr.datetime, _sifr.time, sifr.test}`, the generated `Cargo.toml` would include only `chrono` (from the `_sifr.time` arm at line 39) and no `sifr_stdlib` — but the emitted Rust calls `sifr_stdlib::time::datetime_now(...)`. The wave's focused validation only exercised `cargo run -p sifr -- run …` (driver path, which is correct) and did not run `scripts/run_all_tests.sh --profile create-pr`. Since wave 9 shipped the same-shape gap without breaking create-pr, it is likely masked by batch fingerprint mixing, but re-running the required create-pr gate before PR is the cheapest confirmation. Recommend a follow-up that adds `sifr.datetime`/`_sifr.datetime` (and the missed `sifr.gzip`/`sifr.zipfile`/`_sifr.compress`) to both helpers.

2. **`ts as i64` saturating cast** (`time.rs:35`) — matches previous intrinsic behavior, but NaN silently maps to epoch (1970-01-01) instead of `invalid timestamp`. Not tested, not a regression, but easy to tighten with `if !ts.is_finite()` → `Err`.

3. **`std::io::Error` as the adapter error type** — semantically awkward (no I/O involved). Bridge only uses `.to_string()`, so behavior is fine; a lightweight local error type or `chrono::Error` would communicate intent better. Cosmetic.

4. **Bootstrap intrinsic signatures retained** — `crates/sifr_stdlib_model/src/platform_misc.rs:64` still declares the datetime function signatures. Consistent with the documented "keep until final registry sweep" pattern used by earlier waves; not a bug, but worth calling out in the wave's evidence text (as wave 1 does for platform/html).

5. **Stale doc note** — `internal_docs/stdlib_native_surface_ownership.toml:133` still lists `sifr.datetime` in `_sifr.time`'s `public_modules`. Technically still true because `sifr.datetime` continues to import `_sifr.time.time_now`, but re-reading it post-migration reads as if datetime hadn't migrated. Optional cleanup.

## Behavior regression check

None found. The Rust surface change from `Option<String>`-via-intrinsic to `Result<String, io::Error>`-via-bridge for `datetime_from_timestamp` still yields `ValueError { message: "invalid timestamp" }` at the Sifr level; `datetime_format` still returns empty string on mismatch; `datetime_now` / `datetime_now_struct` unchanged in shape.

## Required before PR

- Run `scripts/run_all_tests.sh --profile create-pr` (already required by `AGENTS.md`) to confirm the batch e2e path is unaffected. If it fails at a datetime-only bucket, addressing note #1 becomes blocking.

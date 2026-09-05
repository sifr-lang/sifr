## Code review — M10 compression interop migration (pass 2)

**Verdict: PASS.** All pass-1 blocking-adjacent observations that had actionable fixes were addressed; the migration is adapter-policy compliant, preserves the public Sifr API, cleanly removes the compression registry, narrows the workspace zip dependency, and passes the local create-pr gate with only the warm wall-time advisory.

### Delta against pass 1

Pass 1 issued NON-BLOCKING APPROVAL with six flagged observations. The changes since pass 1 explicitly resolve the three that were actionable within this migration:

- **`zip_create` now produces a valid empty archive.** `crates/sifr_stdlib/src/zipfile.rs:10-14` calls `ZipWriter::new(file).finish().map_err(zip_error)?` so `is_zipfile(path)` on a fresh empty archive returns `True` and `zip_namelist` returns an empty vector rather than erroring. Confirmed by the new `zip_namelist(&archive).expect("empty archive should be readable").is_empty()` assertion in the unit round-trip test.
- **`zip_read_file_bytes` unit coverage.** `zipfile_adapter_round_trips_text_and_names` (`zipfile.rs:82-104`) now writes with `zip_add_file_bytes` and reads back with `zip_read_file_bytes`, closing the pass-1 defense-in-depth gap while the empty-archive assertion covers the new `zip_create` behavior.
- **Historical infallible-gzip invariant documented.** `gzip.rs:9-19` carries a comment explaining that `GzEncoder<Vec<u8>>` cannot fail in practice and that returning `Vec::new()` on the theoretical error branches preserves the non-`Result` public signature.

The remaining pass-1 observations (`ZipFile.compression` silently defaulting, `_sifr.compress` coupling gzip and zipfile in feature planning, and `sifr.zipfile.zip_namelist`-style re-export leakage) are pre-existing behaviors that this migration explicitly preserves. All three are called out again in the non-blocking follow-up section below.

### Adapter policy adherence

Adapter policy (`internal_docs/sifr_sysroot_and_stdlib_architecture.md:119-135`) is satisfied:

- Every `@rust(...)` in `stdlib/_sifr/compress.sifr:3-40` binds directly to `sifr_stdlib.gzip.*` (2) or `sifr_stdlib.zipfile.*` (6) with `panic=trusted_no_panic`. No `@rust.via`, no `bridge.`, no `converter`/`pipeline` metadata.
- Every Rust adapter signature shape-matches its Sifr declaration: `str↔&str`, `bytes↔&[u8]/Vec<u8>`, `list[str]↔Vec<String>`, `Result[X, IOError]↔Result<X, std::io::Error>`, `Result[None, IOError]↔Result<(), std::io::Error>`. No compiler-side conversion or normalization is introduced.
- Adapter panic safety holds: `crates/sifr_stdlib/src/zipfile.rs` propagates every `std::io::Error` via `?` and every `zip_8_6::result::ZipError` via `map_err(zip_error)`; `gzip.rs` uses `if …is_err()` for the invariant Vec-backed IO paths and `if let Ok(bytes) = encoder.finish()` — no `.unwrap()` or `.expect()` outside `#[cfg(test)]`.
- `completed_private_declarations_follow_adapter_policy_syntax` (`stateless_private_codegen_tests.rs:28-31, 63-87`) now covers `_sifr.compress` and asserts `sifr_stdlib.` binding + `panic=trusted_no_panic` on every `@rust(...)` line — a regression that reintroduces callee injection or bridge routing will fail this test.

### Public API preservation

- `sifr.gzip.compress(data: str) -> list[int]` and `decompress(data: list[int]) -> Result[str, IOError]` keep their original public shapes. Internally, the Sifr wrappers round-trip through `bytes.to_ints()` (`stdlib/sifr/gzip.sifr:6`) and `bytes.from_ints(data)` (`stdlib/sifr/gzip.sifr:10-13`) so the byte-native private declarations do not leak.
- The previous imports (`from _sifr.compress import gzip_compress, gzip_decompress`) are replaced by `_`-prefixed helpers (`_gzip_compress_bytes_impl`, `_gzip_decompress_bytes_impl`) which are correctly hidden from public re-export.
- `stdlib/sifr/zipfile.sifr` is unchanged; `zip_create/zip_add_file/zip_add_file_bytes/zip_read_file/zip_read_file_bytes/zip_namelist` resolve through the same import names, now backed by real declarations. `is_zipfile`'s "unreadable → False" contract is preserved and now returns `False` correctly for a `zip_create()`-produced empty archive too (empty namelist without error → `True`, matching CPython parity).
- Public constants (`ZIP_STORED`, `ZIP_DEFLATED`) and public class shapes (`ZipInfo`, `ZipReadHandle`, `ZipFile.__init__`/`__enter__`/`__exit__`) are byte-identical.

### Registry removal

- `crates/sifr_codegen/src/intrinsics/registry.rs` drops `mod gzip;` and `mod zipfile;` and removes all eight lowering entries. `intrinsics/registry/gzip.rs` and `intrinsics/registry/zipfile.rs` are deleted at the filesystem level.
- `compression_intrinsics_are_owned_by_compiled_stdlib_declarations` (`registry_extended_tests.rs:357-380`) positively asserts `lower_intrinsic` returns `None` for all eight names, so any regression that reinstates a registry entry breaks a test.
- `compression_private_declarations_codegen_through_sifr_stdlib` (`stateless_private_codegen_tests.rs:708-790`) verifies routed emission of `sifr_stdlib::{gzip,zipfile}::…` calls and the `IOError { message, kind }` map_err shape, plus that `sifr.gzip` / `sifr.zipfile` have empty intrinsic sets and `_sifr.compress` in transitive deps.
- No residual HIR/codegen references to `gzip_compress`/`gzip_decompress`/`zip_*` intrinsic names outside the type-signature-owning `intrinsic_compress()` in `crates/sifr_stdlib_model/src/platform_misc.rs:174-255` (renamed to match the new `_impl` names) and the tests already noted.

### Dependency planning and lockfile narrowing

- Workspace `Cargo.toml:144` narrows `zip_8_6` to `{ default-features = false, features = ["deflate"] }` — the minimum surface needed for the compressed archive round-trip.
- `features_for_stdlib_module` (`crates/sifr_stdlib_model/src/features.rs:645`) returns `&[]` for the compression triple, so `retained_direct_dependencies` (`features/dependency_plan.rs:212`) no longer emits direct `flate2`/`zip` deps into generated user Cargo.
- `planned_sifr_stdlib_features` (`features/generated_stdlib_features.rs:37-39`) maps `sifr.gzip → ["gzip"]`, `sifr.zipfile → ["zipfile"]`, `_sifr.compress → ["gzip","zipfile"]`. `sifr_stdlib/Cargo.toml` gates `gzip = ["dep:flate2"]` and `zipfile = ["dep:zip_8_6","gzip"]`, keeping the leaves narrow.
- `fixture_cargo_toml.rs:93-96` drops the direct `flate2` / `zip = "8.6.0"` deps for the compression triple; generated fixtures now depend on `sifr_stdlib` with the correct feature flags only.
- `Cargo.lock` diff (~165 lines removed) drops **all** native-build-script-heavy or crypto sub-graphs pulled by the zip crate's default features: `aes`, `bzip2`, `cipher`, `cmov`, `cpubits`, `constant_time_eq`, `ctutils`, `deflate64`, `hmac`, `inout`, `libbz2-rs-sys`, `lzma-rust2`, `pbkdf2`, `pkg-config`, `ppmd-rust`, `zstd`, `zstd-safe`, `zstd-sys`. Nothing in the retained graph regressed. `time`/`js-sys` transitive-feature reductions came along for free.
- `crates/sifr/tests/e2e_support/harness_model.rs:433-438` still contains `flate2::`/`zip::` sniffers. Since generated user Rust now routes through `sifr_stdlib::{gzip,zipfile}::…` and no longer emits raw `flate2::` or `zip::` paths, these sniffers act as guards rather than dead code — worth keeping.

### Validation sufficiency

- Adapter-level unit tests exercise the round-trip and error paths for both gzip (`gzip_adapter_round_trips_text`, `gzip_adapter_reports_invalid_data`) and zipfile (`zipfile_adapter_round_trips_text_and_names` now covering `zip_read_file_bytes` and empty-archive `zip_namelist`, plus `zipfile_adapter_reports_missing_file`).
- Compiler-side coverage is dual-anchored: the codegen registry test asserts `None` lowering for the eight names, and the driver test asserts routed `sifr_stdlib::{gzip,zipfile}::…` emission plus the `IOError { message, kind }` shape.
- Feature-model coverage runs through `sifr_stdlib_model` `features_tests` and the adapter-policy syntax test (`completed_private_declarations_follow_adapter_policy_syntax`) which now includes `_sifr.compress`.
- Six CLI fixtures cover the stdlib gzip/zipfile round-trip, the cpython subset parity, and mixed filesystem/tempfile+archive integration.
- Full create-pr gate (`SIFR_LSP_COMMAND=… CARGO_TARGET_DIR=target/m10-compression-create-pr CARGO_BUILD_JOBS=1 scripts/run_all_tests.sh --profile create-pr`) reported zero failures with only the warm wall-time budget advisory; `target/validation_lane_reports/create-pr.latest.json` is the recorded evidence.

### Non-blocking follow-ups (unchanged from pass 1; not gating this PR)

1. **`ZipFile.compression` is silently ignored on write** (`stdlib/sifr/zipfile.sifr:82-103` + `crates/sifr_stdlib/src/zipfile.rs:58-67`). `SimpleFileOptions::default()` picks the crate default, and the `compression: int` argument threads through Sifr without ever reaching the adapter. Pre-existing across the registry → adapter port. If ever exercised, a `compression_method: i64` parameter through both layers would fix it.
2. **`_sifr.compress` couples `gzip` and `zipfile` in generated Cargo features** (`generated_stdlib_features.rs:38`). Any project importing only `sifr.gzip` still pulls the zip crate through the transitive `_sifr.compress` dependency. Pre-existing shape; a `_sifr.gzip`/`_sifr.zipfile` split would enable finer feature slicing but is out of scope for this wave.
3. **`sifr.zipfile.zip_namelist` (and siblings) remain re-exported** (`stdlib/sifr/zipfile.sifr:2`). This is unchanged behavior and even asserted in the driver test (`stateless_private_codegen_tests.rs:789`). Public parity with CPython is preserved through `is_zipfile`/`ZipFile`, but the raw helpers can still be called directly. A future public-surface tightening pass could hide them behind `_`-prefixed names in `sifr.zipfile` the same way `sifr.gzip` now does.
4. **`gzip_compress_bytes` swallows `Vec<u8>` write/finish errors and returns an empty payload** (`crates/sifr_stdlib/src/gzip.rs:9-19`). The invariant is documented, no callers can distinguish this from "input compressed to zero bytes" — but if the invariant ever changes (e.g. buffered writer swapped in), silent zero-byte output would only surface on later decompression. Optional hardening: `debug_assert!(compressed_ok)` in the theoretical error branches.

Nothing above is blocking. Recommend PR proceed.

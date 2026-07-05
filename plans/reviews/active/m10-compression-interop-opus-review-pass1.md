## Code review — M10 compression interop migration

**Verdict: NON-BLOCKING APPROVAL.** The migration is clean, adapter-policy compliant, and the lockfile evidence backs the "no zstd/native build script" claim.

### What was verified

- **Adapter policy compliance** — Every `@rust(...)` in `stdlib/_sifr/compress.sifr` binds to `sifr_stdlib.gzip.*` / `sifr_stdlib.zipfile.*` with `panic=trusted_no_panic`. No `bridge.*`, no `@rust.via`. The `completed_private_declarations_follow_adapter_policy_syntax` test already enforces this for the added module.
- **Public API preservation** — `sifr.gzip.compress` / `.decompress` keep their `str → list[int]` / `list[int] → Result[str, IOError]` signatures via `bytes.to_ints()` / `bytes.from_ints()`. `sifr/zipfile.sifr` is byte-for-byte unchanged; `ZipFile.create/write/write_bytes/read/read_bytes/namelist` and `is_zipfile` still resolve through the same import names (now backed by real declarations instead of intrinsic-registry symbols).
- **Registry removal** — `registry.rs` drops all 8 lowerers; `registry_extended_tests.rs` positively asserts each name now returns `None` from `lower_intrinsic`, so any regression that reinstates a registry entry breaks.
- **Codegen routing** — `compression_private_declarations_codegen_through_sifr_stdlib` verifies emission of `sifr_stdlib::{gzip,zipfile}::…(…)` for all 8 functions plus the `IOError { message, kind }` map_err shape, and that `sifr.gzip` / `sifr.zipfile` have empty intrinsic sets and `_sifr.compress` in transitive deps.
- **Feature planning** — `sifr_stdlib_model::features_for_stdlib_module` now returns `&[]` for the compression modules (compiler-emitted Cargo planning is off). `generated_stdlib_features::features_for_module` maps `sifr.gzip → ["gzip"]`, `sifr.zipfile → ["zipfile"]`, `_sifr.compress → ["gzip","zipfile"]`, and `sifr_stdlib/Cargo.toml` has `gzip = ["dep:flate2"]`, `zipfile = ["dep:zip_8_6","gzip"]`. Fixture generator no longer adds direct `flate2`/`zip` deps for the compression triple, and `harness_model.rs`' `flate2::` / `zip::` sniffers no longer match user-facing generated code (calls go through `sifr_stdlib::…`).
- **Dependency narrowing** — Workspace `zip_8_6 = { default-features=false, features=["deflate"] }` is correct. `Cargo.lock` diff confirms removal of `zstd`, `zstd-safe`, `zstd-sys`, `bzip2`, `libbz2-rs-sys`, `deflate64`, `lzma-rust2`, `ppmd-rust`, `aes`, `cipher`, `cmov`, `ctutils`, `cpubits`, `constant_time_eq`, `hmac`, `inout`, `pbkdf2`, and `pkg-config`. That's the ~165-line reduction claimed.
- **Panic-safety** — Rust adapter functions only propagate `std::io::Error` via `?` / `map_err(zip_error)`; no `.unwrap()` / `.expect()` outside `#[cfg(test)]`. The `trusted_no_panic` claim holds under review.

### Non-blocking observations (worth noting; none warrant a block)

1. **`gzip_compress_bytes` silently returns `Vec::new()` on encoder errors** (`crates/sifr_stdlib/src/gzip.rs:8-16`). Writes to `Vec<u8>` and `finish()` on it don't fail in practice, so both `is_err()` branches are effectively dead. This is defensible because the Sifr signature is non-`Result`, but if either branch ever fires (e.g. an OOM-adjacent path), the caller silently gets zero bytes and only detects failure on later `decompress`. Since the previous compiler-lowered version used `unwrap_or(())` / `unwrap_or_default()` with the same effect, this is behavior-preserving. Consider `debug_assert!` or a comment noting the Vec-backed IO invariant.

2. **`zip_create` writes a 0-byte file, not a valid empty archive** (`zipfile.rs:9-12`). `ZipWriter::new(file)` followed by `drop` never flushes an end-of-central-directory record, so `is_zipfile(path)` on a freshly-`create()`-d empty archive returns `False`, and any tool that expects `zipfile.ZipFile(path, 'w').close()` (Python parity) to produce a valid empty archive will diverge. The old registry lowerer had identical behavior, so this is a preserved quirk, not a regression — but the migration was an opportunity to fix it with a single `.finish()?` and wasn't taken.

3. **`ZipFile.compression` is silently ignored on write.** The Sifr surface exposes `ZIP_STORED=0` / `ZIP_DEFLATED=8` and stores `self.compression`, but the adapter always calls `SimpleFileOptions::default()` (Stored) regardless. Pre-existing across the intrinsic → adapter port; worth a follow-up because it means the workspace pulled the zip `deflate` feature (and flate2 through it) exclusively for *read* decompression of externally-produced archives — writes are always stored.

4. **`_sifr.compress` maps to `["gzip","zipfile"]` in `generated_stdlib_features`** (`generated_stdlib_features.rs:38`). Any project that imports only `sifr.gzip` still pulls `zip_8_6` (via the transitive `_sifr.compress` dep). Pre-existing shape (`_sifr.compress` is one module for both), not introduced here, but the migration would have been the moment to split into `_sifr.gzip` / `_sifr.zipfile` if finer feature slicing is desired.

5. **`_gzip_compress_bytes_impl` shadows leaked names** — the old `stdlib/sifr/gzip.sifr` did `from _sifr.compress import gzip_compress, gzip_decompress`, so `sifr.gzip.gzip_compress` was accessible via Python-import-leak. That leak is now closed (private helpers are `_`-prefixed). `stdlib/sifr/zipfile.sifr` is unchanged, so `sifr.zipfile.zip_namelist` etc. remain leaked (this is unchanged behavior and even asserted in the driver test at lines 785-786). Not a regression — flagging for parity awareness.

6. **Unit-test coverage gap in `zipfile::tests`** — `zip_read_file_bytes` is not exercised by the sifr_stdlib unit test (`zipfile_adapter_round_trips_text_and_names` only round-trips text via `zip_read_file`); `zip_add_file_bytes` writes but is never read back. E2E `tempfile_and_zipfile.sifr:65-68` does cover the bytes path, so this is a defense-in-depth gap, not a real hole.

### Nothing found blocking

- No panic-introducing paths.
- No API surface regression against the migration's preservation goals.
- No adapter-policy violations.
- Lockfile trims are consistent with narrowed workspace deps and no build-script-heavy crates re-enter.

Recommend proceed to PR.

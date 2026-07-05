## Second Review Pass — M10 Wave 11 Bytes Helper Interop

**Verdict: PASS**

### Follow-up on prior note
The sole substantive note from pass 1 — documenting why `bytes_to_hex` retains a `Result` despite infallible hex formatting — has been addressed. The comment in `crates/sifr_stdlib/src/bytes.rs` ("Preserve the existing Sifr Result API; hexadecimal formatting is infallible.") makes the intent explicit: the `Result` is kept for API stability with the existing Sifr surface, not because the operation can fail. This is the right resolution — it preserves signature compatibility for the public `sifr.bytes` wrappers while making the infallibility legible to future maintainers. No further action needed.

### Verification of the delta
- **`panic=trusted_no_panic` on `encode_utf8`/`bytes_to_hex`** — consistent with Sifr's "no user-triggerable panics" guarantee. Both operations are genuinely infallible in Rust (UTF-8 encoding of a `str`, hex formatting), so the trusted annotation is sound rather than a papered-over failure path.
- **Registry arm removal for `encode_utf8`/`bytes_to_hex`** — correctly narrows the compiler-side intrinsic surface to only the helpers that still need glue (`bytes_to_hex_strict`, `from_hex`, `from_ints`, `with_size`, `str.encode`, `bytes.decode`). The direct `@rust` interop path supersedes the old registry codegen, so the removal is not a regression.
- **`_sifr.bytes` → `sifr.bytes` wrapper layering** — matches the established ownership split (underscored native surface, public wrappers over it), consistent with the mixed-ownership docs.
- **Generated Cargo `bytes` feature carries no raw third-party `bytes` dep** — appropriate; the helpers rely on `sifr_stdlib` internals, not an external crate, so no dependency-graph surprise in the lockfile.

### Validation
Re-ran `cargo fmt`, `git diff --check`, and the scoped `sifr_stdlib` bytes test suite under an isolated target dir — all green. Changes are focused on the milestone with no scope creep.

No blocking issues and no remaining non-blocking notes.

I have enough to render a verdict. Let me summarize the review.

## Review summary

**M1d task**: "Remove fallback intrinsic signature tables for any surface already marked `closing`."

**What the change does**
- Adds `RETAINED_INTRINSICS_LIB_PATH` and `RETAINED_SIGNATURE_MODULE_RE` (line 25) to scan `crates/sifr_retained_intrinsics/src/lib.rs` for `"_sifr.foo" => Some(...)` match arms.
- Records those module names as `fallback_signature_modules` in the observed surface (lines 76-80).
- Introduces `_surface_private_modules()` (lines 167-184) that derives owned private modules from both `surface.id` (stripping any `::leaf` subfamily suffix) and `declaration_files` under `stdlib/_sifr/**.sifr`.
- Rejects any `state = "closing"` row whose derived module set still intersects the observed fallback module set (lines 134-143).
- Extends `--self-test` with positive and negative cases (lines 310-328).

**Correctness checks**
- Regex captures 31 module arms; excludes the `_sifr.not_real` test reference (line 143 in lib.rs) because it isn't a match-arm.
- `_sifr.http::header_helpers` and similar subfamily ids correctly reduce to `_sifr.http` via `split("::", 1)[0]`.
- No `closing` rows exist in the current manifest, so the guard is intentionally a forward-only trip wire; existing PASS is unchanged (`fallback_signature_modules=31`).
- Placement is right: the guard already observes lib.rs-adjacent surfaces (registry_files, preamble_files, exact_intrinsics); adding another observed set alongside them fits. The schema validator has no need to read `sifr_retained_intrinsics/src/lib.rs` and shouldn't own this.

**Non-blocking observations**
- The regex `_sifr\.[A-Za-z0-9_]+` only captures a single dotted segment, while `_surface_private_modules` can synthesize multi-segment names via `declaration_file.replace("/", ".")`. Today `stdlib/_sifr/` is flat and lib.rs uses only single-segment module names, so no asymmetry surfaces. Not a bug for the current codebase.
- Self-test exercises only the id-and-declaration-files-aligned case; it does not separately exercise subfamily-id-only derivation or a declaration-files-only derivation. Those code paths are small and self-evident.
- There is no reverse check that every fallback signature module has an active manifest owner. Out of M1d scope; belongs to a broader closure guard later.

**False positive/negative check**
- False positive risk: a `closing` subfamily leaf whose parent `_sifr.*` module still has retained peer leaves would be flagged despite the module still being needed. This aligns with the phase invariant that fallback modules must be split when subfamily granularity is used, so this is the intended behavior, not a bug.
- No false-negative cases identified against the current manifest.

**Validation confirmed**
- `python3 scripts/check_stdlib_native_intrinsic_allowlist.py`: PASS (fallback_signature_modules=31).
- `python3 scripts/check_stdlib_native_intrinsic_allowlist.py --self-test`: PASS.
- `python3 scripts/check_stdlib_manifest_schema.py` / `--self-test`: PASS (reported by user).

The change fully covers the "no closing row keeps a fallback signature module" requirement, is placed correctly, and has adequate self-test coverage for M1d's scope.

READY

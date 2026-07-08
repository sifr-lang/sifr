Round 3 review complete. The post-round-2 change (classification row) is the correct and minimal fix.

**Verification of the coverage-matrix classification**

The added row
`{"name": "logging", "profile_assignment": "merge", "classification": "first_party_runtime"}`
matches peers with the same shape:
- `logging = ["dep:sifr_runtime"]` mirrors `calendar = ["dep:sifr_runtime"]` and `math = ["dep:sifr_runtime"]`, both classified `merge` / `first_party_runtime` (lines 139, 148).
- `sifr_runtime` itself is `first_party_runtime` / `merge` (line 115), so declaring a runtime-linking leaf as `first_party_runtime` is consistent.
- `validate_features` (`coverage_matrix.py:529`) requires every `sifr_stdlib` Cargo feature to appear in this list; the new `logging` feature would otherwise fail readiness (which is exactly what happened).
- No corresponding row exists in `profile_assignment_matrix.json`; that file governs verification suites, not stdlib leaves, so no other coverage-matrix wiring is needed.

**No regressions from the post-round-2 change**

- The classification file diff is a single-line addition; no other rows moved, no schema changes.
- All the other changes I re-audited match round-2 READY:
  - `crates/sifr_codegen/src/stdlib_filter/implementation.rs:471` — round-1 residual `__SIFR_GLOBAL_LOG_LEVEL` allowlist entry has been removed.
  - No stray references to `set_global_level`/`get_global_level`/`LoggingState`/`build_logging_items`/`intrinsic_logging`/`__SIFR_GLOBAL_LOG_LEVEL` remain in `crates/`.
  - `_sifr.logging` transitions `retained → closing` with `declaration_files` evidence; schema guard adds a metadata-only-closing self-test; allowlist guard adds a matching self-test; closure guard adds both retired names.
  - `sifr_stdlib::logging` uses `LazyLock<Mutex<i64>>` with `PoisonError::into_inner`; `api_behavior` test saves/restores the process-global level (safe against the file's other tests, which don't touch it).
  - Manifest `logging_module_emits_only_sysroot_stdlib_dependency` mirrors the peer `unicode_module_...` shape and asserts `features = ["logging"]` with no `sifr_runtime` direct dep.

READY.

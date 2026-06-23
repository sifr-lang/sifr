Delta review confirmed clean. The change is correctly scoped:

- `dependency_line` at lines 154-171 emits `default-features = false` only when `dependency_name == "sifr_stdlib"`, in both the no-features and with-features branches — matching the shape that generated manifests produce.
- `sifr_runtime` injection at line 149 is unchanged: still `sifr_runtime = { path = ... }` with no `default-features` flag.
- The two updated tests (lines 499-511 and 526-537) assert the exact stricter string, including `default-features = false` in the correct position relative to `features = [...]`.
- The unrelated tests (`sysroot_runtime_probe_manifest_does_not_duplicate_runtime_dependency`, the two `sysroot_stdlib_probe_features_*` tests, and `sysroot_probe_vendor_args_use_invocation_scoped_config`) are untouched and continue to pass.

Validation results:
- `cargo fmt --check` — clean (no output).
- `cargo test -p sifr_driver sysroot_probe_manifest --locked` — 2 passed, 0 failed.
- `cargo test -p sifr_driver sysroot_stdlib_probe_features --locked` — 2 passed, 0 failed.

**VERDICT: PASS**

No blockers. No new non-blocking follow-ups for M9 wave 1.

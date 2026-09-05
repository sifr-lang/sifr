## M11 Python Interop Adapters - Closeout Review

**Verdict: READY**

The current pending diff (`plans/issues/active/ad-hoc-stdlib-native-boundary-completion.md`) correctly flips M11 from `in progress` -> `merged` and appends the M11i evidence (PR #2912 - sha=0d963d45). All boundary claims verify against the tree.

### What checks out

- **All M11 sub-milestones landed.** `git log` shows M11a (#2896) through M11i (#2912) merged into `main`. Each has agent review evidence captured in the M11 cell. The current merge sha `0d963d45a` matches the evidence claim.
- **Python compiler surface is closed.** `crates/sifr_codegen/src/intrinsics/registry.rs` and `crates/sifr_codegen/src/intrinsics/registry/*` contain zero `py_`/`python` entries. `crates/sifr_retained_intrinsics/src/lib.rs` has no `_sifr.python` fallback signature module. Both the migration closure guard and the native intrinsic allowlist guard PASS.
- **Guard counts in evidence match the tree.** `check_stdlib_native_intrinsic_allowlist.py` reports `exact_intrinsics=31, fallback_signature_modules=21`; `check_stdlib_migration_closure.py` reports `retired_intrinsics=366` - exactly the numbers the M11i evidence cell claims.
- **Manifest state is correct.** `_sifr.python` is `state = "closing"` with no `registry_files`, `preamble_files`, or `exact_intrinsics` fields (`internal_docs/stdlib_retained_compiler_intrinsics.toml:212-224`). This mirrors the M4/M6/M8/M10 closure pattern where `closing` rows persist for audit traceability until M13 deletes them.
- **Callback helpers routed through source.** All five callback helpers (`local_callback`, `threadsafe_callback`, `py_local_callback_echo`, `py_threadsafe_callback_echo`, `py_close_callback`) are declared in `stdlib/_sifr/python.sifr:350-374` behind `@rust(sifr_stdlib.python.py_*)` and their `sifr_stdlib::python::*` targets exist (`crates/sifr_stdlib/src/python.rs:592-614`).
- **Certification rows are live.** `_sifr.python`'s three declared certification rows (`opaque_resource_matrix`, `callbacks_call_scoped`, `callback_subscription_ecosystem`) all exist in `verification/areas/rust_interop/data/rust_interop_compatibility_matrix.json`.

### Non-blocking notes

1. **Empty closeout review artifact.** `plans/reviews/active/m11-python-interop-closeout-agent-round1.md` is a 0-line placeholder. Populate it with the round-1 closeout verdict (this review) before merging the closeout PR, and mirror M10's convention of appending "M11 closeout review satisfied in round N" to the M11 evidence cell after the round settles.
2. **`_sifr.python` `certification_rows` still lists `opaque_resource_matrix`.** M10 round 1 flagged this as possibly stale post-M3 split; M10 round 2 verified it live. The row is still present in the matrix, so this is benign. It can stay to be revisited during M13 final closure.
3. **M11 Tasks section under-specified.** Lines 1012-1025 name only M11a explicitly; unlike M4/M7/M8 which list every sub-slice as a bullet, M11b-M11i are captured only in the evidence cell. Consistent with M9's style, and the evidence cell is the operative record - no action needed for this closeout, but worth a light refactor before the phase-level M13 audit.
4. **`raw[4]` `kind` field consumption (from M11i round 2).** Both `_local_callback_from_raw` and `_threadsafe_callback_from_raw` in `stdlib/sifr/python_core.sifr:66-81` now correctly consume `raw[4]` instead of the previous hard-coded strings - an M11i round 2 follow-up that landed cleanly.

### Bottom line

The Python adapter native boundary is closed exactly as M11's acceptance criteria require: adapter behavior source-declared and stdlib-owned, CPython substrate retained in the runtime, `_sifr.python` in the `closing` audit-only state, and no migrated adapter operation remaining in intrinsic dispatch. The closeout diff faithfully records this. Populate the round-1 closeout review file (with this content or equivalent), and M11 is PR-ready.

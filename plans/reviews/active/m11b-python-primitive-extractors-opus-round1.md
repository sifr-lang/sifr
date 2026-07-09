I have completed a thorough review. Let me consolidate the findings.

## Code Review Verdict: **PR-ready** [ok]

No blockers. The M11b migration is correct, scope-clean, and follows the established M11a pattern precisely.

### Findings (all non-blocking)

**Scope - verified exact.** All 8 files touch exactly the 16 named primitive extractors and nothing else:
- `stdlib/_sifr/python.sifr`: +16 `@rust` declarations
- `sifr_stdlib/src/python.rs`: +16 native wrappers
- `registry/python.rs`: -16 match arms, -16 `lower_py_to_*` helpers
- `sifr_retained_intrinsics/src/python.rs`: -16 inserts
- `stdlib_retained_compiler_intrinsics.toml`: -16 entries
- `check_stdlib_migration_closure.py`: +16 to `RETIRED_INTRINSICS`
- Both test files add matching 16-name coverage.

The retained surfaces stay retained: `py_copy_list/tuple/dict_str_*`, `py_copy_record_fields`, `py_from_list/tuple/dict_str/record`, and all object/module/call/callback/buffer/arrow/dlpack/context/coroutine intrinsics are untouched in both `registry/python.rs` and the retained module.

**Type/bridge correctness - verified.** Every Sifr return type maps correctly to the native signature (`None->()`, `int->SifrIntBridge`, `int8..usize->i8..usize`, `float->f64`, `str->String`, `bytes->Vec<u8>`), matching the old `Type::*` retained signatures exactly. `object_handle()` reconstructs the runtime `ObjectHandle = (i64,i64)` via `to_i64_saturating()`, mirroring M11a's `py_from_int`. `py_to_int`'s `.map(SifrIntBridge::from)` lines up with the compiler bridge's `__sifr_bridge_ok.to_i64_saturating()` return convention (same as `calendar_weekday`, `set_len`, etc.). No unused-import fallout - `FixedIntType` still used by the collection loop.

**No panic / no fallback.** `to_i64_saturating` saturates (no unwrap), handles originate as i64 so the round-trip is lossless, and all runtime `to_*` return `Result` (invalid handle / out-of-range int -> `Err`, never panic). No fallback paths introduced.

**Tests are adequate.** Codegen test asserts all 16 no longer lower as intrinsics; driver test asserts generated private Rust routes to `sifr_stdlib::python::py_to_*(`, carries the bridge conversions, and drops them from `intrinsic_names`. The obsolete `py_to_i32` assertion was correctly removed from the feature-metadata test, which still validates a retained intrinsic (`copy_list_u8`).

### Minor observations (worth noting, not gating)
1. **Closure guard is documentary for `py_*` names.** `check_stdlib_migration_closure.py` scans only `registry.rs`, which dispatches Python via a `starts_with("py_")` guard (stripped by the `if`-split), so no `py_*` literal ever appears as "active." The `RETIRED_INTRINSICS` additions therefore don't actively block re-adding an arm in `registry/python.rs` - real enforcement is the codegen `is_none()` + driver `intrinsic_names` tests. This exactly matches the already-merged `py_from_*` precedent, so it's consistent; just be aware the guard here is mostly bookkeeping.
2. **Runtime-execution coverage is `to_int`-only.** The `primitive_roundtrip.sifr` fixture exercises `to_int` end-to-end; the other 15 extractors are covered structurally (codegen/type) plus the runtime-layer tests in `sifr_runtime::python`. Same coverage shape as the M11a constructors - acceptable, but a future wave could add an execution fixture touching the full extractor set.
3. `py_to_int` does a lossless `i64 -> SifrIntBridge -> i64` round-trip (native wraps, bridge unwraps); harmless and dictated by the bridge convention.

Validation you ran (fmt, targeted codegen/driver/stdlib/retained tests, `check`+`emit` on the roundtrip fixture, and all five closure/allowlist/schema/certification/file-size guards) is the right set and covers the migration's integration points.

I ran the full `test_e2e_pass` suite to verify hypotheses. It fails with 10+ JSON-related test failures — two independent root causes.

## VERDICT: FAIL

### Blocker 1 — Batch e2e Cargo.toml is missing `sifr_stdlib` for `sifr.json` / `_sifr.json`

**Files:** `crates/sifr/tests/e2e_support/fixture_cargo_toml.rs:35, 324-350`; `crates/sifr/tests/e2e_support/fixture_dependency_paths.rs:46-84`

`fixture_cargo_toml.rs:35` drops `sifr.json | _sifr.json` from the `SERDE_JSON_DEP` arm — correct for eliminating the direct dep — but nothing else in the fixture path adds `sifr_stdlib` for those modules:

- `needs_sifr_stdlib_module_dependency` (line 324) omits `sifr.json` / `_sifr.json`.
- `sifr_stdlib_dependency_spec_for_modules` in `fixture_dependency_paths.rs:46` has no branch that appends `"json"` to the features list.
- `infer_dependencies` in `harness_model.rs:398` does not scan for `sifr_stdlib::json`.

`emit` output for `cpython_json.sifr` confirms the generated Rust calls `sifr_stdlib::json::json_load_tokens(text)`, `sifr_stdlib::json::json_validate_integer_digit_limits(text)`, `sifr_stdlib::json::json_dump_tokens*(tokens)`, and `sifr_stdlib::json::json_dump_tokens_web(tokens)`. The batch harness (`batch_execution.rs:65`) uses `generate_cargo_toml`, so groups whose fingerprint reduces to `{sifr.json}` receive no `sifr_stdlib` line at all, and mixed groups that already have `sifr_stdlib` still don't get the `"json"` feature.

**Repro (from this branch, uncommitted diff):**
```
SIFR_E2E_DISABLE_CACHE=1 cargo test -p sifr --test e2e test_e2e_pass -- --nocapture
```
Fails with Rust compilation errors on group `06916de4f0cd399e` (cpython_json, cpython_json_subset, json_integer_profiles, stdlib_json_consolidated) plus `error_subclass_handling`, `json_and_datetime`, `panic_free_stdlib_errors`, `parse_safety_error_paths`, `parsers_and_encoders`, `structured_data_formats`. The `sifr run` path passes because `crates/sifr_driver/src/build/cargo_manifest.rs:32` uses `try_sysroot_dependency_plan`, which does emit the sysroot crate with `features = ["json"]`.

**Fix:** Add `sifr.json | _sifr.json` to both `needs_sifr_stdlib_module_dependency` in `fixture_cargo_toml.rs:324` and `sifr_stdlib_dependency_spec_for_modules` in `fixture_dependency_paths.rs:46` so the emitted line is `sifr_stdlib = { path = ..., default-features = false, features = ["json"] }`. Consider extending `fixture_cargo_toml_tests.rs` (mirror of `stateless_sysroot_cargo_toml_tests.rs:4`) with a JSON-only assertion so the batch path is guarded.

### Blocker 2 — `json_dumps` public surface regression: `Decimal` / `BigDecimal` no longer accepted

**File:** `stdlib/sifr/json.sifr:427`

The new wrapper narrows to `JsonValue | bool | int | float | str`:

```
def json_dumps(own value: JsonValue | bool | int | float | str) -> str:
```

Two existing pass fixtures depend on the previous broader intrinsic surface:

- `crates/sifr/tests/e2e/pass/decimal_conversions.sifr:25` — `assert json_dumps(Decimal("1.2300")) == '"1.2300"'`
- `crates/sifr/tests/e2e/pass/decimal_conversions.sifr:26` — `assert json_dumps(BigDecimal("1.2300")) == '"1.2300"'`
- `crates/sifr/tests/e2e/pass/decimal_runtime_operations.sifr:41-42` — same pattern.

Full-suite run above emits:
```
argument 1 ('value') of function 'json_dumps': expected 'bool | int | float | str | JsonValue', got 'decimal'
argument 1 ('value') of function 'json_dumps': expected 'bool | int | float | str | JsonValue', got 'bigdecimal'
```
Sifr type-check failure, so `test_e2e_pass` also fails at compile stage for these fixtures.

**Fix:** Extend the union in `stdlib/sifr/json.sifr:427` to include `decimal` and `bigdecimal` and add matching `isinstance` branches (or route Decimal/BigDecimal through `from_str(str(value))`), or update the two decimal e2e fixtures to serialize explicitly through a `str(...)`/JsonValue conversion. Whichever direction is chosen, add a Wave-5 test that pins the accepted union so this doesn't drift again.

### Non-blocking observations

- `crates/sifr_codegen/src/intrinsics/registry/requirements.rs:31-35` still names `json_loads | json_validate_integer_digit_limits | json_dumps_value_exact | json_dumps_value_web | json_dumps_value_string_ints`, but `lower_intrinsic_rendered` in `registry.rs:667` returns `None` for those names before `additional_required_features` is consulted, so this arm is dead code. Safe to delete during cleanup.
- JSON-specific `bridge_error_expr` arms in `crates/sifr_codegen/src/rust_interop_direct.rs:129-155` hardcode class names (`JSONDecodeError`, `JsonLimitError`, `JsonIntegerRangeError`). Not a policy violation given each error has a non-`str` field, but this is the first user of typed multi-field error interop and every additional class will need its own arm — worth revisiting as more surfaces migrate.
- JSON runtime errors (`crates/sifr_stdlib/src/json.rs:53-57, 89-95`) collapse `JsonLimitError` → `JSONDecodeError` at (line=0, column=0). This matches previous intrinsic behavior; keep in mind for any future work on preserving source locations for digit-limit failures.

Once Blockers 1 and 2 are resolved, please re-run `scripts/run_all_tests.sh` (or at minimum `cargo test -p sifr --test e2e test_e2e_pass`) end-to-end before considering Wave 5 validated — the focused validation set in the plan document doesn't exercise the batch harness.

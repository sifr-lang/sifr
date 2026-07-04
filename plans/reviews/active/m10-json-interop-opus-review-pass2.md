## VERDICT: PASS

The M10 wave 5 JSON interop migration is architecturally sound and Pass 1 blockers are correctly resolved. The uncached full-corpus e2e (651/651) plus focused test suites give strong signal, and the adapter policy is followed (private `_sifr.json` declarations, no active JSON intrinsic registry, sifr_stdlib feature-gated behind `json`, typed error bridge codegen verified).

### Pass 1 blocker resolution — all verified

1. **Batch E2E json feature planning** — `fixture_cargo_toml.rs:35` moves `sifr.json`/`_sifr.json` out of the `serde_json` branch and into the empty-deps branch; `fixture_cargo_toml.rs:338` and `fixture_dependency_paths.rs:77` add the `json` feature; new `test_generate_cargo_toml_json_uses_stdlib_json_feature` at `harness_behavior_tests.rs:470` locks the harness output.
2. **`json_dumps` Decimal/BigDecimal preservation** — `stdlib/sifr/json.sifr:427` union signature preserves the primitive+decimal surface; assertions `json_dumps(Decimal("1.2300")) == '"1.2300"'` and BigDecimal variant now pass via `from_str(str(value))` fallback matching prior `serde_json::to_string` textual output.
3. **Stale requirements.rs entries removed** — retired JSON names dropped from `additional_required_features`; ownership test `json_intrinsics_are_owned_by_compiled_stdlib_declarations` (registry_core_tests.rs:53) asserts no active lowering registrations remain.
4. **`toml_loads` disambiguation** — `stdlib/sifr/tomllib.sifr:228` exposes `toml_loads`, `loads` retained as alias; three mixed JSON/TOML fixtures updated. Full uncached run (651 fixtures) confirms no residual collisions.

### Architecture / codegen correctness spot-checks

- Direct interop bridge (`rust_interop_direct.rs:129-155`) correctly emits struct init for the three JSON error types with matching accessor signatures; string-match assertions in `stateless_private_codegen_tests.rs:496-500` pin down the emitted shape.
- Token bridge (`crates/sifr_stdlib/src/json.rs`) uses `serde_json` with workspace `preserve_order`; Rust load side surfaces line/column via `JsonDecodeBridgeError`, and the SifrInt digit-limit is enforced pre-parse.
- `sifr_stdlib_model/src/io_json.rs` type signatures updated to the token-based `list[str]` bridge and match `_sifr.json` declarations.
- Ownership registry (`stdlib_native_surface_ownership.toml:145`) and architecture doc (`internal_docs/sifr_sysroot_and_stdlib_architecture.md:92`) reflect the new state; migration blocker language updated.

### Non-blocking observations

1. **Silent `"null"` fallback on token-stream corruption** — `crates/sifr_stdlib/src/json.rs:64,73,171-177`: if `SifrInt::parse_decimal` fails on a malformed integer token or trailing tokens exist, dump silently emits `"null"` instead of surfacing the structural error. Only reachable via a malformed Sifr-side bridge stream, but masks logic bugs in future edits. Consider emitting a debug-time assert or dedicated `BridgeCorruptedError` return path.
2. **Name-keyed bridge error dispatch** — `crates/sifr_codegen/src/rust_interop_direct.rs:136,144,152`: match arms key off bare class names. If future stdlib work introduces a class named `JSONDecodeError`/`JsonLimitError`/`JsonIntegerRangeError` under a different Rust adapter with different accessor names, generated code would break silently at rustc time. Adapter policy currently makes this safe; worth a doc note or explicit qualifier check.
3. **`JsonLimitError` mapped through `JsonDecodeBridgeError` loses `limit`** — `crates/sifr_stdlib/src/json.rs:93-95`: when the digit-limit trips inside `json_load_tokens`, the limit value is dropped (only message survives). If any consumer wants the limit for diagnostics on load-time overflow, that context is now unrecoverable. Consider expanding `JsonDecodeBridgeError` with an optional `limit` or plumbing a separate result variant.
4. **`JsonIntegerRangeError.path` ambiguity** — `crates/sifr_stdlib/src/json.rs:273`: object keys with `.` or `[` produce ambiguous JSONPath strings. Existing behavior, but flagged for future ergonomics.
5. **Public surface duplication in `sifr.json`** — `stdlib/sifr/json.sifr:377,442-455`: both `loads`/`json_loads` and `dumps`/`json_dumps_value*` are exported. Intentional for backward compat with pre-migration wrappers; may want a deprecation plan for the `json_*_value*` set post-M10 wave stability.
6. **`json_dumps` public type narrowing** — `stdlib/sifr/json.sifr:427-429`: the union `JsonValue | bool | int | float | str | decimal | bigdecimal` replaces the previous `Type::Any`. This is a stricter public contract (positive), but is a technically observable narrowing that should be called out in the M10 wave 5 PR summary and, ideally, the acceptance-criteria doc if user-facing docs claim `Any`.
7. **Test coverage for `JsonLimitError`/`JsonIntegerRangeError` codegen shape only asserts string presence** — `stateless_private_codegen_tests.rs:497-499` matches on exact rendered text. A behavioral test that actually invokes the compiled `_sifr.json` interop and observes the typed error at runtime would give stronger coverage than substring matching; the e2e `panic_free_stdlib_errors`/`parse_safety_error_paths` fixtures partially cover this for decode-error but not for limit/integer-range paths.

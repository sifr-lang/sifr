## Review Verdict: PASS

The slice does what it claims and the documentation disclaimers match the actual diff scope. The wiring from `validate_require_serializable_call` into `extract_ipc_schema_type` runs the path in real lowering (with the result discarded via `let _schema = ...`), while the two unit tests are the only behavioral assertions — that's an honest "compiler-internal" surface.

## Blocking findings

None.

## Non-blocking polish

- `crates/sifr_lowering/src/lower/ipc_schema_extraction.rs:9` — `Type::None | Type::Never` are collapsed to `IpcSchemaType::None`. `Never` as the bottom type is semantically distinct from the unit-shaped `None`; mapping is conservative (a `Never` payload should never materialize) but worth tracking if peer-schema exchange later needs to distinguish them.
- `crates/sifr_lowering/src/lower/ipc_schema_extraction.rs:21` — `Type::Dict(_, value)` discards the key without comment. It's safe because `non_ipc_serializable_reason` rejects non-`str` keys upstream and `IpcSchemaType::DictStr` already names that invariant, but this is one of the rare cases where a one-line `// why` would aid future readers (no need to add unless others raise it).
- `crates/sifr_lowering/src/lower/ipc_schema_extraction.rs:38` — `Type::Enum` variants always lower to `payload: None`. That matches the current Sifr `Enum` shape `(String, Option<i64>)` (discriminant, not payload), so it's accurate — just flagging it so a future variant-payload extension doesn't silently keep returning `None`.
- `crates/sifr_lowering/src/lower/ipc_schema_extraction.rs:5` — `extract_ipc_schema_type_inner` re-runs `non_ipc_serializable_reason` at every recursion depth. Correct but O(n²)-ish on deep graphs; only matters if compile-time cost shows up, otherwise leave it.
- Test 1 mixes lowering-side extraction with `sifr_stdlib::canonical_schema_descriptor`. Good coverage as an integration check, but the canonical-string assertion is brittle — any whitespace/format change in the stdlib formatter will fail this test. Consider whether the canonical-format assertion belongs in `sifr_stdlib` and the lowering test should assert just the `IpcSchemaType` tree.
- Test 2's `PipeReader` rejection relies on the existing name-based heuristic inside `non_ipc_serializable_reason_inner`. It's the same coupling the diagnostics tests already accept, so not a new risk — just worth noting it's now load-bearing for schema-extraction evidence too.

## Scope and documentation honesty

Honest. The disclaimers in `verification/platform/supported_host_matrix.md`, `verification/stdlib/concurrency_runtime_m6_typed_ipc_design.md`, and the merge-ledger entry consistently exclude:
- public worker/connection APIs,
- generated worker integration,
- runtime peer schema exchange,
- Windows process-pipe fixture support.

The diff matches: no new public surfaces, no runtime wiring, no fixture changes. The `let _schema = …` at `crates/sifr_lowering/src/lower/ipc_payload_calls.rs:21` makes the "compiler-erased, marker preserved" claim true at the call site.

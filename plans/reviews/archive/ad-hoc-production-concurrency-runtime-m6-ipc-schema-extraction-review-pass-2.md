## Review Verdict: PASS

(Pass 2 confirms pass 1 on the rebased branch — no scope drift, no new code beyond what was already approved.)

## Blocking findings

None.

## Non-blocking polish

All carry over from pass 1; nothing newly introduced by the rebase:

- `ipc_schema_extraction.rs:9` — `Type::None | Type::Never` still collapse to `IpcSchemaType::None`. Semantically conservative (a `Never` payload can't materialize), but worth distinguishing if peer-schema exchange ever lands.
- `ipc_schema_extraction.rs:21` — `Type::Dict(_, value)` discards the key. Safe because `non_ipc_serializable_reason` rejects non-`str` keys upstream and `IpcSchemaType::DictStr` names the invariant; no comment required, but flagged.
- `ipc_schema_extraction.rs:38` — `Type::Enum` variants always lower `payload: None`. Matches today's `(String, Option<i64>)` enum shape (discriminant, not payload), so accurate now; if variant payloads are added later, this branch must be revisited so it doesn't silently keep returning `None`.
- `ipc_schema_extraction.rs:5` — `non_ipc_serializable_reason` is re-invoked at every recursion depth (O(n²)-ish on deep graphs). Fine until it shows up in compile-time profiles.
- Test 1 still asserts against the full `canonical_schema_descriptor` string. That couples the lowering test to the `sifr_stdlib` formatter; a whitespace change in the formatter will fail it. Consider keeping the canonical-format assertion in `sifr_stdlib` and asserting only the `IpcSchemaType` tree from lowering.
- Test 2's `PipeReader` rejection relies on the existing name-based heuristic in `non_ipc_serializable_reason_inner`. Same coupling the payload diagnostics tests already accept — just noting it's now load-bearing for schema-extraction evidence too.

## CPython-shape ledger preservation and claim honesty

**Preserved.** Commits `47beb9914` (CPython-shape merge ledger) and `e9f49b0e8` (CPython-shape diagnostics) sit upstream of this slice's `1224c1a31`. The issues-file edit appends the schema-extraction ledger entry below the existing CPython-shape merge-ledger section without touching it. `supported_host_matrix.md` and `concurrency_runtime_m6_typed_ipc_design.md` only edit payload-eligibility rows to cross-reference schema extraction; the CPython-shape rows are untouched.

**Claims honest.** Disclaimers across the matrix, design doc, and merge-ledger entry consistently exclude generated worker integration, public connection/worker APIs, runtime peer schema exchange, and Windows process-pipe fixtures. The diff matches: helper is `pub(in crate::lower)`, no codegen edits, no runtime wiring, no fixture changes. The `let _schema = …` at `ipc_payload_calls.rs:22` keeps the "compiler-erased marker preserved" claim true at the only call site.

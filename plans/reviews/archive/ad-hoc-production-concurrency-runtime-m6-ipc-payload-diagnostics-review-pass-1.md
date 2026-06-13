PASS

## Findings

### Acceptance criteria
1. ✅ `require_serializable` is name-intercepted in `lower_regular_call` (regular_calls.rs:202–212) before type-checking flows, and `lib/sifr/ipc.sifr` defines it as a no-op `def require_serializable[T](value: T) -> None`. No worker/connection runtime surface added.
2. ✅ `SIFR-OWN-0013` wired in `DiagnosticCode` const, `ACTIVE_DIAGNOSTIC_CODES`, the registry entry in `calls_flow_and_protocols.rs`, and `docs/errors/SIFR-OWN-0013.md` / `docs/errors/diagnostic-codes.md` / `internal_docs/diagnostic_codes.md`.
3. ✅ Accept/reject set in `ipc_payload_calls.rs` is conservative: primitives + None + Option (2-member union w/ None) + list/tuple/`dict[str, T]` + record/Result/newtype/alias accepted; resources/sync endpoints/guards/tasks/coroutines/callables/iterators/Range/Any/Unknown/TypeVar/Protocol/Intersection/Set/BigInt/Decimal/non-str dict keys/non-Option unions rejected. `visiting` HashSet guards class cycles.
4. ✅ Three fixtures: `ipc_payload_require_serializable_basic.sifr` (accepted), `ipc_payload_process_resource_rejected.sifr` (PipeReader), `ipc_payload_sync_endpoint_rejected.sifr` (ChannelSender). Accepted fixture added to both create-pr and merge manifests.
5. ✅ `verification/stdlib/concurrency_runtime_m6_typed_ipc_design.md`, `verification/platform/supported_host_matrix.md`, and the ledger entry all explicitly keep generated schema extraction, generated worker integration, public connection/worker APIs, and Windows process-pipe fixtures as follow-up.

### Non-blocking follow-ups
- `Type::Enum { .. } => None` accepts all enums unconditionally. If Sifr enums can carry variant payload data, variants are not walked here — a non-IPC-serializable variant field could slip through. Worth a targeted unit test or comment confirming variants are constrained to IPC-eligible types upstream.
- Lowering replaces the call with `HirExpr::BoolLiteral(true)` even though the declared return is `None`. This is a small type drift between the HIR's typed return and the literal emitted; reads as a placeholder erasure. If a `NoneLiteral` (or equivalent unit/none HIR node) exists, prefer it for clarity. Currently safe because the result is statement-discarded in the fixture; flag as a tidy-up.
- `func_name == "require_serializable"` is a bare-name string match. Any user-defined function named `require_serializable` in scope would be erased too. Matches the existing `parallel_calls`/`task_scope_calls` interception pattern, so consistent — but worth a brief comment noting this is name-resolution-shadowed by the `sifr.ipc` import in practice.
- Diagnostic message template `typed IPC payload cannot transfer {value} of type {type_name}` declares a third arg `reason (json-only)`, but the rendered message in `non_ipc_serializable_payload` interpolates the reason inline plus an additional suffix (`; pass owned schema data instead of process-local resources`). This matches the prior OWN-0012 pattern (template summarizes, rendered string adds detail), so it's consistent — noting for future contract tightening if templates ever start being used for structured rendering.

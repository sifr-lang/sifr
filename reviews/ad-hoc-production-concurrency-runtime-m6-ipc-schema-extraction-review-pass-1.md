# Review: M6 typed IPC compiler schema extraction — **PASS**

## Real schema extraction (not a fallback/path placeholder)
Verified. `crates/sifr_lowering/src/lower/ipc_schema_extraction.rs:67-153` actually walks the concrete `sifr_type_system::Type` graph and lowers each variant into an `IpcSchemaType`:
- Primitives, literal-int/str/bool, fixed-width ints → primitive variants
- `List`, `Tuple`, `Result`, `Dict(Str, _)` → recursive container variants (recurses through aliases via `resolve_alias`)
- 2-member `Union` containing `None` → `Option`
- `Newtype` → `Record { name, fields: [{value, inner}] }`
- `Class` → `Record` with cycle detection through a `visiting: HashSet<String>`
- `Enum` → `IpcSchemaType::Enum` (variant names only; Sifr enums currently carry integer tags, not typed payloads — that's a deliberate v1 choice tied to `protocol_schema_version=1`)

The output then goes through `sifr_stdlib::validate_ipc_payload_type`, the same runtime validator the host-independent payload eligibility tests exercise. Not a string match, not a path lookup.

## Unsupported payloads rejected before wire compatibility
Verified. `validate_descriptor_payloads` (lines 34-41) runs `validate_ipc_payload_type` over request/response/error before returning a descriptor. Any `IpcSchemaType::Unsupported` sentinel anywhere in the graph fails extraction with `IpcPayloadEligibilityError::UnsupportedPayload`. Rejected at extraction time:
- Process-local resources by name (`PipeReader`, `Channel*`, `Lock*`, `Context*`, `Child`, `Notify`, `Shared`, guards, permits)
- `Dict` with non-`Str` keys
- General `Union` (not Option[T])
- `Type::Set`, `Type::Any`, `Type::Function`/`Callable`, and everything else falling into the `_ => unsupported(ty)` arm
- Recursive class cycles (visiting check returns Unsupported instead of looping)

Tests at lines 285-340 cover dict-int-key, Set, multi-arm Union, Any, and Function rejection, plus PipeReader nested inside a record.

## Existing SIFR-OWN-0013 diagnostics remain precise
Verified. `ipc_payload_calls.rs:29-30` keeps `non_ipc_serializable_reason(arg.ty())` as the **primary** source, with the extractor error only used via `.or_else(...)`. Since `non_ipc_serializable_reason_inner` already covers process-local resources with specific labels ("process pipe handle", "channel endpoint", etc. — `ipc_payload_calls.rs:171-185`), the user-visible wording for SIFR-OWN-0013 on `PipeReader`/`ChannelSender` is unchanged. The extractor's generic fallback "`<T>` does not have a generated IPC schema" only fires for types the existing reason path returns `None` for — in the current Type universe that's effectively defense-in-depth, which is fine.

## No overclaiming of public worker APIs
Verified.
- `verification/platform/supported_host_matrix.md:45-47`: the new row is scoped to "compiler-side extraction from concrete Sifr `Type` graphs into canonical IPC schema descriptors", and the process-pipe row still flags Windows host-limited plus "Generated worker integration remains M6-owned follow-up work".
- `verification/stdlib/concurrency_runtime_m6_typed_ipc_design.md:5`: status still says "Generated worker integration remains M6 implementation work; Windows child-process fixture evidence remains host-limited follow-up work."
- Ledger entry mirrors the same scope; the design table replaces "missing" with the extractor row but does not claim the public worker/connection API.

## Minor observations (non-blocking, not required changes)
- `module_path="__sifr.ipc"` / `schema_name="RequireSerializablePayload"` at `ipc_payload_calls.rs:22-23` are placeholders used only because the marker discards the descriptor. When the same extractor is reused by generated worker codegen, real module/schema names will need to be threaded in — worth noting in the M6 follow-up scope.
- Newtype → `Record{name, fields:[value:T]}` is indistinguishable on the wire from a real one-field class. Acceptable under `compatible_version_min/max=1`; if newtypes ever need wire distinction, that's a v2 schema bump.
- The schema-extraction error path is effectively unreachable today because `non_ipc_serializable_reason` is comprehensive. That's fine for now (belt-and-braces), but if you later remove a branch from the reason path, this fallback would silently swallow the regression with a less precise message.

No required changes.

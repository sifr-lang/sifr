Follow-up review complete. Writing the verdict.

# PASS

Reviewer verified that the post-pass-1 polish addresses every non-blocking item from reviews/ad-hoc-production-concurrency-runtime-m6-ipc-payload-eligibility-review-pass-1.md without introducing new blockers, that the validator contract remains correct, that documentation still does not overclaim compiler diagnostics / generated schema extraction / child-process transport / public APIs, and that the execution ledger's updated validation and line counts are honest.

## What changed since pass 1

**1. `None` unit type explicitly enumerated in the design's accepted families (verification/stdlib/concurrency_runtime_m6_typed_ipc_design.md:191):**

- Pass-1 item 1 flagged that the validator accepts `IpcSchemaType::None` (ipc_payload.rs:28) but the design's "Initially accepted payload families" list did not name it. The new bullet `- the \`None\` unit type used by generated option/result schemas,` lands between `bytes` and `Option[T]`, and the bullet explicitly ties it to its purpose (so `Option[T]` and `Result[T, None]` can be expressed). The wording does not overclaim — it says "used by generated option/result schemas" rather than asserting standalone first-class semantics, which matches how the test at ipc_payload.rs:92–98 actually uses it.
- The validator and the design list are now in lock-step.

**2. `unsupported(<type_name>)` canonical descriptor explicitly framed as rejected-type evidence only (verification/stdlib/concurrency_runtime_m6_typed_ipc_design.md:211):**

- Pass-1 item 3 asked for either (a) documentation that the descriptor format intentionally carries `unsupported(...)` for evidence carriage or (b) a typed gate. The follow-up takes option (a) — the new paragraph reads: "The canonical schema descriptor may render `unsupported(<type_name>)` only as rejected-type evidence so generated peers and tests can preserve diagnostics without panicking. Payload eligibility validation must reject any schema graph containing that sentinel before a payload is encoded or treated as wire-compatible."
- This makes the contract explicit: the descriptor representation exists for diagnostics, NOT as a claim that the sentinel can be encoded on the wire, and the validator gate (ipc_payload.rs:51–55) is the authoritative reject path. The phrasing "must reject" reads as a contract obligation for future code that consumes the descriptor, not a statement that the descriptor itself blocks encoding.
- This does not raise the slice's claim surface — it narrows it, by writing the existing semantics down.

**3. Recursive-container rejection coverage added (crates/sifr_stdlib/src/ipc_payload.rs:152–190):**

- Pass-1 item 2 noted that the dispatch-coverage matrix was incomplete: top-level `Unsupported`, plus `Unsupported` inside `Option` / `List` / `DictStr` / `Result.ok` / `Result.err` / `Tuple` value positions were not directly covered. The new test `rejects_unsupported_payloads_through_recursive_containers` (ipc_payload.rs:152–190) closes exactly that gap with a per-arm matrix:
  - Top-level `Unsupported` (line 155);
  - `Option(Unsupported)` (line 158);
  - `List(Unsupported)` (line 161);
  - `DictStr(Unsupported)` (line 164);
  - `Result(Unsupported, None)` exercises the ok-side (line 167);
  - `Result(None, Unsupported)` exercises the err-side (line 173);
  - `Tuple([Unsupported])` exercises the slice walk (line 179).
- `Record` field and `Enum` variant-payload nesting were already covered by `rejects_unsupported_process_resource_payloads_inside_records` (line 107) and `rejects_unsupported_task_payloads_inside_enum_variants` (line 127), so combined the matrix now hits every recursive arm of `validate_ipc_payload_type` (ipc_payload.rs:21–57).
- The assertion uses `matches!(..., Err(IpcPayloadEligibilityError::UnsupportedPayload { .. }))` rather than pinning the rendered text, which is appropriate here — the rendered-text contract is already pinned by `eligibility_errors_do_not_render_payload_values` (line 192), so this test correctly stays focused on the dispatch contract.

## Validator contract still correct

- `validate_ipc_payload_type` (ipc_payload.rs:21–57) is byte-identical to the pass-1 version. The arms are unchanged, the Unsupported arm still returns `IpcPayloadEligibilityError::UnsupportedPayload { type_name: type_name.clone() }`, and no new accept paths were added.
- `IpcSchemaType::Unsupported` (ipc_schema.rs:49–51, 162–166) is unchanged — still a `type_name: String` sentinel, still rendered through `push_escaped` so the type name cannot break the descriptor grammar.
- `schema_hash_v1_is_stable_and_sensitive_to_shape` (ipc_schema.rs:255–272) still produces `4733c89fb23a40ecb5f3bcda99fb34da` for the sample descriptor, confirming the existing canonical descriptor format was not perturbed by either the polish doc edits or the new test.
- No `unwrap`, `expect`, `panic!`, `assert!`, `unreachable!`, or `todo!` in the validator or the Unsupported arm — panic-freedom preserved.

## Documentation honesty (no new overclaim)

- verification/stdlib/concurrency_runtime_m6_typed_ipc_design.md:5 status line still keeps `Child-process fixture transport, full connection negotiation, compiler diagnostics, and generated worker integration` as M6 follow-ups; the polish edits did not silently graduate any of those.
- verification/stdlib/concurrency_runtime_m6_typed_ipc_design.md:34 evidence row narrows to "recursively validates the initially accepted IpcSerializable schema families and returns typed UnsupportedPayload evidence for unsupported process/task/resource-like shapes without rendering payload values" and disclaims `Compiler diagnostics, generated schema extraction, and runtime foreign-peer payload handling` — unchanged scope claim, no overclaim added by the new sentence on line 211.
- verification/stdlib/concurrency_runtime_m6_typed_ipc_design.md:209 still appends "compiler diagnostic wiring is still follow-up work" to the diagnostics paragraph.
- verification/stdlib/concurrency_runtime_m6_typed_ipc_design.md:211 (new) constrains the descriptor's `unsupported(...)` rendering to "rejected-type evidence" and requires "Payload eligibility validation must reject any schema graph containing that sentinel before a payload is encoded or treated as wire-compatible." This is a tightening, not an overclaim — it explicitly forbids the sentinel from being treated as wire-compatible.
- verification/platform/supported_host_matrix.md:43 still marks only the host-independent payload-eligibility validator as supported and disclaims `compiler diagnostics, child-process fixture transport, generated schema extraction, or public connection/worker APIs`. The "Typed IPC frames over process pipes" row (line 44) remains `blocked-on-concurrency-runtime-m6` — i.e., transport row is not silently flipped.
- issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md:1136–1149 keeps scope, sentinel intent, accepted/rejected families, follow-up boundary, and per-file line counts honest. The new sentence (line 1140) about the `Unsupported` sentinel adds "without pretending the type is encodable" — consistent with the new design-doc paragraph, no claim drift.
- issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md:462 still keeps "M6 typed IPC payload eligibility: pending PR" rather than fabricating a PR number.

## Execution-ledger honesty (validation and line counts)

Locally verified during this review:

- `cargo test -p sifr_stdlib ipc_payload -- --nocapture` -> `5 passed; 0 failed`. The ledger now claims 5 tests covering accepted families, record-nested Unsupported, enum-variant-nested Unsupported, recursive container dispatch paths, and redacted error text — matches what ran, including the new `rejects_unsupported_payloads_through_recursive_containers`.
- `cargo test -p sifr_stdlib ipc_schema -- --nocapture` -> `2 passed; 0 failed`. The descriptor / hash stability tests still pass against the unchanged FNV hash and canonical-descriptor format.
- `cargo clippy -p sifr_stdlib -- -D warnings` -> green.
- `cargo fmt --check` -> green; `git diff --check` -> green.
- `python3 scripts/check_file_size_guardrails.py` -> `PASS (2255 files, limit 900 lines)` — matches the ledger.
- `wc -l` on the touched files matches the ledger's updated counts exactly: `crates/sifr_stdlib/src/ipc_payload.rs` `203`, `crates/sifr_stdlib/src/ipc_schema.rs` `273`, `crates/sifr_stdlib/src/lib.rs` `442`, `verification/stdlib/concurrency_runtime_m6_typed_ipc_design.md` `250`, `verification/platform/supported_host_matrix.md` `46`. The ipc_payload.rs delta (`163` -> `203`, +40 lines) is consistent with the 39-line recursive-container test plus the trailing newline boundary; the design-doc delta (`247` -> `250`, +3 lines) is consistent with the new `None` bullet plus the new two-line paragraph (blank line + the new sentence). No discrepancies between claimed and observed line counts.
- The ledger continues to disclose the `create-pr` warm wall-time advisory at `649.93s` against the `<=2m` target rather than burying it. The text `report_signature=530c89bb7012eeb0`, `e2e_pass_suite` slowest step `252162ms`, and `124 passed, 0 failed` are stated as run output rather than as a green claim — honest framing.

## No blockers

The three pass-1 non-blocking items are now addressed, no new overclaim was introduced, the validator contract and the canonical-descriptor format are unchanged, and the ledger's updated counts match local runs. Returning PASS.

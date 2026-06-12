Review complete. Writing the verdict.

# PASS

Reviewer verified the M6 typed IPC payload eligibility slice against scope, validator correctness, sentinel honesty, redaction, doc honesty, and validation evidence. No blocking findings.

## What was verified

**Scope alignment (issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md:1136–1149 and issues/ad-hoc-production-concurrency-runtime-platform-substrate.md M6 narrative):**

- The slice ships only the internal host-independent payload-eligibility validator and a sentinel variant on `IpcSchemaType`. Public worker/connection APIs, compiler diagnostics, generated schema extraction, and child-process fixture transport are explicitly left to follow-up work. This matches the review prompt's scope and the design's "first implementation may expose only fixture-oriented internal helpers" boundary (verification/stdlib/concurrency_runtime_m6_typed_ipc_design.md:139).

**Correctness — `validate_ipc_payload_type` (crates/sifr_stdlib/src/ipc_payload.rs:21–57):**

- Primitive accept arm (lines 23–28) covers `Bool`, `Int`, `Float`, `Str`, `Bytes`, and `None`. The first five exactly match the design's initially accepted primitive families (verification/stdlib/concurrency_runtime_m6_typed_ipc_design.md:187–190). `None` is the unit type the schema needs so `Option[T]` and `Result[T, None]` can be expressed in the existing `IpcSchemaType` model — used by the test at lines 92–98 as the `err` half of `Result(Tuple(Bool,Float), None)`.
- Single-arg recursive arms (lines 29–31) wire `Option`, `List`, and `DictStr` to the same validator — the design's `Option[T]`, `list[T]`, and `dict[str, T]` rules (verification/stdlib/concurrency_runtime_m6_typed_ipc_design.md:191, 194) require the contained `T` to itself be `IpcSerializable`, which falls out naturally from the recursion.
- `Result` arm (lines 32–35) validates both `ok` and `err`, matching "Result[T, E] when both sides are IpcSerializable".
- `Tuple` arm (line 36) goes through `validate_all` (lines 59–64), which fails on the first ineligible element. Empty tuples are vacuously eligible — consistent with "fixed-shape tuples when every element is IpcSerializable".
- `Record` arm (lines 37–42) iterates `fields` and fails-fast on the first ineligible field. Empty records are accepted, which matches "generated records ... with stable schemas".
- `Enum` arm (lines 43–50) iterates `variants` and validates each variant's optional payload. Payload-less variants and empty enums pass — consistent with the design's enum eligibility rule.
- `Unsupported` arm (lines 51–55) is the only error path, returning `IpcPayloadEligibilityError::UnsupportedPayload { type_name: type_name.clone() }`. This is structural: any `Unsupported` reachable anywhere in the type graph rejects the whole payload, exactly as the test pair `rejects_unsupported_process_resource_payloads_inside_records` (lines 107–124) and `rejects_unsupported_task_payloads_inside_enum_variants` (lines 126–150) exercise — the rejection survives wrapping by `Record` and by `Enum` variant payload.

**Sentinel honesty — `IpcSchemaType::Unsupported` (crates/sifr_stdlib/src/ipc_schema.rs:49–51, 162–166):**

- The new variant only carries a `type_name: String` — no payload value, no host handle, no encoded bytes. It is a marker for a rejected type *evidence*, not an encodable payload family.
- In `canonical_schema_descriptor`, `push_type` renders the variant as `unsupported(<escaped_type_name>)` (lines 162–166). The escape pass through `push_escaped` is the same one applied to record/enum names, so the type name cannot break the descriptor grammar.
- This appears in the descriptor (and therefore in `schema_hash_v1`), but the validator gates encoding — any schema containing `Unsupported` is rejected before bytes are produced, so the descriptor's representation is evidence carriage rather than a claim that the payload can be encoded on the wire. The implementation has exactly the shape the review prompt asks for: sentinel, not encodable claim.
- The retained `schema_hash_v1_is_stable_and_sensitive_to_shape` test (ipc_schema.rs:255–272) still produces `4733c89fb23a40ecb5f3bcda99fb34da` after the enum got a new variant, confirming the added arm did not perturb the canonical descriptor format for the existing `Bool/Int/Float/Str/Bytes/None/Option/Result/List/DictStr/Tuple/Record/Enum` families.

**Redaction (crates/sifr_stdlib/src/ipc_payload.rs:9–17, 153–162):**

- `Display` for `IpcPayloadEligibilityError::UnsupportedPayload` writes only `"unsupported IPC payload type {type_name}"`. No payload bytes, no field values, no record/enum field/variant names, no host paths.
- The test `eligibility_errors_do_not_render_payload_values` (lines 152–162) pins the exact rendered text against `"unsupported IPC payload type sifr.process.PipeReader"`, locking in the no-value contract.

**Panic-freedom:** no `unwrap`, `expect`, `panic!`, `assert!`, `unreachable!`, or `todo!` in `crates/sifr_stdlib/src/ipc_payload.rs` or the added `Unsupported` arm in `crates/sifr_stdlib/src/ipc_schema.rs`.

**Module wiring (crates/sifr_stdlib/src/lib.rs:16, 44):**

- `mod ipc_payload;` declared alphabetically next to existing `ipc_*` modules. `pub use ipc_payload::{validate_ipc_payload_type, IpcPayloadEligibilityError};` re-exports the new surface; naming is consistent with the `ipc_frame`, `ipc_request_tracker`, `ipc_schema`, `ipc_transport` siblings. The new `Unsupported` variant flows out through the existing `pub use ipc_schema::IpcSchemaType` re-export — no additional surface added.

**Documentation honesty (no overclaim):**

- verification/stdlib/concurrency_runtime_m6_typed_ipc_design.md:5 status line was extended only to add "validate host-independent payload eligibility with unsupported-payload evidence" while keeping `Child-process fixture transport, full connection negotiation, compiler diagnostics, and generated worker integration` as M6 follow-ups.
- verification/stdlib/concurrency_runtime_m6_typed_ipc_design.md:34 evidence row uses `cargo test -p sifr_stdlib ipc_payload -- --nocapture` as evidence and the row narrows the claim to "recursively validates the initially accepted IpcSerializable schema families and returns typed UnsupportedPayload evidence for unsupported process/task/resource-like shapes without rendering payload values", explicitly disclaiming `Compiler diagnostics, generated schema extraction, and runtime foreign-peer payload handling`.
- verification/stdlib/concurrency_runtime_m6_typed_ipc_design.md:208 amends the Payload Eligibility section's diagnostics paragraph with a precise scope sentence: "The internal `ipc_payload` helper now validates the host-independent schema shape used by generated peers; compiler diagnostic wiring is still follow-up work."
- verification/platform/supported_host_matrix.md:43 adds one row marking macOS/Linux/Windows `supported` for the host-independent value-model surface only. The row explicitly disclaims `compiler diagnostics, child-process fixture transport, generated schema extraction, or public connection/worker APIs`. The existing row at line 44 (`Typed IPC frames over process pipes`) remains `blocked-on-concurrency-runtime-m6` — i.e., the slice does not silently flip the transport row.
- issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md:1136–1149 records scope, sentinel intent, accepted/rejected family enumeration, follow-up boundary, fmt/clippy/diff/guardrails pass with 2255 files and 900-line limit, and per-file line counts.
- The PR list at issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md:462 keeps "M6 typed IPC payload eligibility: pending PR" rather than fabricating a PR number — honest about implementation/PR ordering.

**Validation evidence (locally verified during this review):**

- `cargo test -p sifr_stdlib ipc_payload -- --nocapture` -> 4 passed, 0 failed, matching the ledger's claim.
- `cargo test -p sifr_stdlib ipc_schema -- --nocapture` -> 2 passed, 0 failed, confirming existing descriptor/hash shape stayed stable after adding the `Unsupported` variant.
- `cargo clippy -p sifr_stdlib -- -D warnings` -> green.
- `cargo fmt --check` -> green; `git diff --check` -> green.
- `python3 scripts/check_file_size_guardrails.py` -> `PASS (2255 files, limit 900 lines)` — matches the ledger.
- `wc -l` on the touched files matches the ledger exactly: ipc_payload.rs `163`, ipc_schema.rs `273`, lib.rs `442`, concurrency_runtime_m6_typed_ipc_design.md `247`, supported_host_matrix.md `46`.

## Non-blocking follow-ups

1. `IpcSchemaType::None` is accepted by `validate_ipc_payload_type` (ipc_payload.rs:28) but is not explicitly enumerated in the design's "Initially accepted payload families" list (concurrency_runtime_m6_typed_ipc_design.md:187–195). It is structurally required so `Option[T]` and `Result[T, None]` can be expressed and the test at ipc_payload.rs:92–98 already exercises it transitively, but the design list could explicitly mention "the `None` unit type" to keep the validator and the design in lock-step.
2. The four payload tests cover record-nested and enum-variant-nested `Unsupported`, plus the rendered error text. They do not directly cover (a) top-level `Unsupported`, (b) `Unsupported` inside `Option`/`List`/`DictStr` value position, (c) `Unsupported` inside `Result`'s ok/err halves, or (d) `Unsupported` inside `Tuple`. The recursive structure is uniform, but a tiny matrix-style test would close the dispatch-coverage gap cheaply.
3. `canonical_schema_descriptor` will happily serialize a schema containing `Unsupported` (ipc_schema.rs:162–166) and `schema_hash_v1` will produce a stable hash for it. The validator is the only guard. Consider either (a) documenting that the descriptor format intentionally carries `unsupported(...)` for evidence so generated peers can render rejected schemas without panicking, or (b) adding a `debug_assert!` / typed gate so schemas that reach `canonical_schema_descriptor` with an `Unsupported` arm cannot silently get a wire-stable hash. Either is fine; today nothing in the slice claims this is forbidden.
4. Recursion in `validate_ipc_payload_type` is unbounded. For generated schemas this is fine, but if the validator ever ingests untrusted descriptors (e.g., for runtime peer schema negotiation) a depth limit would be wise. Not in scope for this slice.
5. The ledger entry at issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md:1149 reports the `create-pr` lane warm wall-time advisory at `649.93s` against a `<=2m` target — well over the prior tracker slice's `552.38s`. Worth a follow-up note on whether the new tests materially moved the warm budget or whether this is environmental noise.

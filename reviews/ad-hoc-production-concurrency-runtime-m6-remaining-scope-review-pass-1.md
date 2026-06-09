## M6 milestone-scope review

### Q1 — Does current evidence satisfy DoD except for Windows fixtures?

**FAIL (narrowly).** Almost every DoD bullet has a named evidence row in `verification/stdlib/concurrency_runtime_m6_typed_ipc_design.md:24-39` and is honest:

- safe/typed/versioned/panic-free — `ipc_frame`, `ipc_transport`, `ipc_request_tracker`, `ipc_connection` ✓
- layered over accepted transport with deterministic close, backpressure, cancellation, malformed-frame — Unix `ipc_process_pipe_fixture` covers all six ✓
- compile-time diagnostics for unsupported payloads — `SIFR-OWN-0013` via `require_serializable(...)` ✓
- payload eligibility fixtures — `ipc_payload_*` ✓
- CPython family classification — eight `ipc_*_unsupported` fixtures plus matrix ✓
- no public worker pool / future worker not default — phase scope respected ✓

The one unmet seam beyond Windows: the phase scope says *"IPC compatibility is generated from Sifr IPC schema definitions, not inferred dynamically from arbitrary runtime values."* The schema extractor (PR 2464) and the Unix process-pipe fixture (PR 2455/2458) each exist independently, but **nothing tests that a compiler-extracted `IpcSchemaType` composes with the runtime substrate** (bootstrap negotiation, frame exchange) — the fixture and connection-state tests use hand-rolled descriptors. That is exactly what the design's own status line acknowledges: *"Generated worker integration remains M6 implementation work."*

### Q2 — Is this a true M6 blocker or honestly deferrable?

**True M6 blocker, but not as a public worker pool.** Two reasons it cannot be deferred:

1. The design doc (`verification/stdlib/concurrency_runtime_m6_typed_ipc_design.md:5`) — authored under M6 ownership — explicitly calls this *"M6 implementation work,"* not deferred. Deferring it would contradict the design owner's own status.
2. The phase forbids only a **public** process-worker pool (*"Do not ship a public process worker pool in this phase"*, line 749). An internal boundary proof that the extractor composes with the substrate is **not** a public worker API and does not violate that constraint.

The "future process-worker API boundary" language defers the public API, not the internal compose proof.

### Q3 — Narrowest implementation evidence

A single internal integration test (no new public surface), ideally inside `sifr_stdlib`'s `__test_fixture` feature or as a new `sifr_lowering`↔`sifr_stdlib` integration test:

1. Define a representative Sifr fixture type (record + enum + container).
2. Lower it through `sifr_lowering::lower::ipc_schema_extraction` to `IpcSchemaType`.
3. Feed the extracted descriptor into `sifr_stdlib::ipc_schema` to compute the canonical descriptor and FNV‑1a‑128 hash.
4. Drive a `Hello`/`Ready` bootstrap exchange via `sifr_stdlib::ipc_connection` over the existing Unix `ipc_process_pipe_fixture` child binary, using that extracted hash/range.
5. Round-trip one `Run`/`Completed` carrying a payload whose Postcard envelope is built from the extracted schema, then a `Shutdown`/`Terminating` close.

This is purely a compose proof. No `sifr.ipc.Worker`, no `ipc.Connection[Req,Res,Err]` public binding, no public worker pool. Windows fixtures stay host-limited.

### Q4 — Should M6 stay pending?

**Yes, pending.** The honest closure path is:

1. Land the narrow boundary test in Q3 as one more M6 PR (slot 8 in the design's implementation waves matches this — *"generated worker boundaries when those are accepted"*).
2. Add the corresponding evidence row to `concurrency_runtime_m6_typed_ipc_design.md`, the host matrix, and the execution ledger.
3. Update the design's status sentence to drop *"Generated worker integration remains M6 implementation work"* and leave only the Windows host-limited follow-up.
4. Then mark `milestone_concurrency_runtime_6` complete; `milestone_concurrency_runtime_7` (docs/demos/inventory closeout) is the natural next slice and is not blocked by this gap once the compose proof lands.

Closing M6 now would either (a) require amending the design doc to reclassify the integration gap as deferred — which contradicts the phase-level *"IPC compatibility is generated from Sifr IPC schema definitions"* clause — or (b) leave the design's own status text out of sync with the milestone state.

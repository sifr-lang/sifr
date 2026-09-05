I now have complete ground-truth. Here is the review.

---

# Independent Architecture Review — Complete Declaration-First Python Interop (agent High, Pass 1)

**Scope reviewed:** `python_interop_declaration_architecture.md`, `python_interop_protocol_architecture.md`, `plans/issues/active/ad-hoc-declaration-first-python-interop.md`, against the implemented baseline (`python_interop_architecture.md`), the async model (`async_concurrency_model.md`), and current crates (`sifr_runtime/src/python/*`, `sifr_package/src/python/*`, `sifr_lowering`, `sifr_ir`, `sifr_codegen`, `verification/areas/python_interop`).

**Overall:** The direction is coherent and the governing constraints (one end state, no dual authorities, no `py.Object` fallback, affine protocol resources, one sealed handle) are respected in spirit throughout. The implementation substrate is more favorable than expected: `@rust` already provides a structured dotted-path decorator grammar with a static value grammar, effect classification, ellipsis-stub bodies, and `Self`-root method targets (`crates/sifr_lowering/src/lower/rust_interop.rs`), and `own self` affine consumption is fully real and enforced (`sifr_type_system/src/types/definitions.rs:290-389`, `sifr_lowering/src/lower/ownership_diagnostics.rs`). So the Python declaration layer is greenfield but has a proven template.

However, several contracts are specified by name without a definition, and two of them collide with the sealed async/cancellation model. As written, independent implementers would build observably different systems. Verdict is REQUEST CHANGES; the blocking items are all resolvable now without deferring design.

---

## BLOCKING

### B1 — `ExitCause`, `ExitDecision`, `SifrBoundaryError` are used but never defined; `AsyncExitCause` reuse is unspecified
`python_interop_protocol_architecture.md` → *Synchronous Context Managers* and *Asynchronous Context Managers*; `python_interop_declaration_architecture.md` → *Decorator Grammar* (`@python.context.*`).

The exit wrappers return `Result[ExitDecision, PythonError]`, take `cause: ExitCause` / `cause: AsyncExitCause`, and synthesize `SifrBoundaryError`. Ground truth: `ExitCause`, `ExitDecision`, and `SifrBoundaryError` **do not exist anywhere in the codebase**; `AsyncExitCause` exists only as a *bare marker class with no fields or variants* (`crates/sifr_lowering/src/lower/async_with.rs:78-85`), even though `async_concurrency_model.md` (*Async Resource Protocols*) describes it as a variant enum. Failure mode: two implementers produce incompatible cause/decision types, and neither matches the async model.

**Required correction:** Define all four normatively in the protocol doc, e.g.:
- `enum ExitCause: Normal | Return | OrdinaryError(Error) | PythonError(PythonError)` (sync path has no cancellation/timeout/runtime-fault variant — see B2/B3).
- `enum AsyncExitCause: Normal | Return | OrdinaryError(Error) | PythonError(PythonError) | Timeout(TimeoutError) | Cancellation(CancellationError) | RuntimeFault(...)` — and explicitly state this *replaces* today's fieldless marker and is the same type used by native `async with`.
- `enum ExitDecision: Suppress | Propagate`.
- `SifrBoundaryError`: a generated Python exception carrying redacted structured cause metadata; specify which fields survive redaction.
State that `PythonError` must be a cause variant carrying the **live** originating exception (see B4).

### B2 — Two incompatible `with` semantics; dispatch is ambiguous
`python_interop_protocol_architecture.md` → *Synchronous Context Managers*.

Native Sifr sync `with` uses an **argless** `__exit__(self) -> None` with Drop-style semantics and **no suppression** (`crates/sifr_lowering/src/lower/protocol_diagnostics.rs:22-42`; e2e `with_basic.sifr`, `with_break.sifr`). The proposed Python context manager uses `__exit__(own self, cause: ExitCause) -> Result[ExitDecision, PythonError]` *with* suppression, and the doc says "Sifr `with` lowering honors suppression explicitly." Native `with` has no such lowering. Failure mode: it is undefined whether `with py_ctx:` selects native or Python lowering, and whether native `with` gains suppression/cause.

**Required correction:** State explicitly that a value whose type is a `@python.opaque(cleanup=context)` class is lowered through a **dedicated Python-context lowering** distinct from native `with`, keyed on the opaque declaration, and that native user `with` is unchanged (argless, Drop-style, no suppression). One sentence removes the ambiguity.

### B3 — Python suppression is allowed to swallow Sifr cancellation / Sifr errors
`python_interop_protocol_architecture.md` → *Synchronous Context Managers* ("Python's truthy `__exit__` result becomes `ExitDecision.suppress`") and *Asynchronous Context Managers*; general statement "Suppression consumes the active exit cause and continues after the block."

This contradicts `async_concurrency_model.md` invariants 2, 3, and 20: active cancellation is scope-exit semantics, is not catchable, and suppression/shielding are out of the model. The async paragraph even self-contradicts by stating "the original cancellation resumes." Failure mode: foreign Python `__aexit__` returning truthy while the body is cancelled would suppress Sifr cancellation — a core-guarantee violation — and could equally swallow an ordinary Sifr `Err(E)` based on foreign code.

**Required correction:** Specify that `ExitDecision.Suppress` is honored **only** when the active cause is a Python-originating exception (`PythonError(...)`). When the cause is `Cancellation`, `Timeout`, `RuntimeFault`, or (decide explicitly and state) an ordinary Sifr `OrdinaryError`, a truthy Python exit result is recorded as ignored cleanup evidence and the cause resumes unchanged. Make this a normative table row per cause × decision.

### B4 — "Preserved exception triple" requires retaining the live Python exception; not specified
`python_interop_protocol_architecture.md` → *Synchronous Context Managers* / *Asynchronous Context Managers* ("an originating Python error to its preserved exception triple").

To feed `__exit__(type, value, tb)` faithfully (so managers like `contextlib.suppress`, DB transaction handlers, and `except SpecificError` behave correctly), the wrapper must replay the **original** exception object, not a synthetic one. That means `PythonError` must retain the live `Py<PyAny>` exception (a non-send ref with a defined lifetime). Today `PythonError` is a structured value produced at the boundary (`crates/sifr_runtime/src/python/object_ops.rs` error mapping); the docs do not require it to hold the live exception. Failure mode: silent behavioral divergence from CPython for any type-sensitive context manager, or an unbuildable contract if implementers assume a synthetic triple.

**Required correction:** State that a Python-originating `PythonError` retains the live exception triple until the enclosing `with`/`async with` exit consumes it, that this retained reference is non-send and released on exit, and that a Sifr-originating cause yields `SifrBoundaryError` with no real Python traceback. If full fidelity is intentionally *not* provided, say so explicitly and enumerate the lost behaviors — do not leave it implicit.

### B5 — Arrow zero-copy certification has no defined authoring/discovery mechanism → hidden allowlist
`python_interop_protocol_architecture.md` → *Arrow C Data Interface* ("accepted only when the exact producer target and distribution fingerprint has executable zero-copy certification recorded in the binding evidence").

No part of either doc defines how a package author *produces* or *records* this certification, what artifact stores it, or how the compiler *discovers* it at declaration-check time. The current implementation decides copy-vs-zero-copy at **runtime**, per capsule (`crates/sifr_runtime/src/python/arrow_ops.rs`; verification `copy_possible_cases` marking pandas/unknown producers `copy_possible:true`). Failure mode: as written, `@python.arrow` compiles only for whatever producers happen to have pre-recorded evidence — a fixed allowlist that no downstream author can extend — which is exactly a reduced version wearing a gate.

**Required correction:** Define the certification artifact concretely: e.g., a per-binding fingerprint file (producer target path + distribution SOABI/version) plus a named executable certification fixture that asserts pointer identity and exact-once release, whose recorded pass is referenced by the binding contract digest and re-checked by `sifr python bind --check`. State the exact authoring command/flow that lets a package author certify a new producer. Without this, B5 is unbuildable and violates the "no reduced version" direction.

### B6 — `@python.async` ellipsis declarations will be rejected by the async-effect seal
`python_interop_protocol_architecture.md` → *Async Python Calls*; `python_interop_declaration_architecture.md` → *Blocking And Async*.

`async_concurrency_model.md` invariant 15 and `SIFR-ASYNC-0001` reject any `async def` whose transitive suspension summary is `NoSuspend`. An ellipsis-only `@python.async` body has no visible `await`, so under current lowering it is a "fake async" function and is rejected (`crates/sifr_lowering/src/lower/workload_annotations.rs`, async-effect diagnostics). Neither doc grants the Python async declaration an `AsyncIo`-equivalent summary. Failure mode: the primary async binding form does not compile.

**Required correction:** State that `@python.async` declarations carry a synthesized `AsyncIo` suspension summary (mirroring `RustInteropEffect::Async` in `crates/sifr_ir/src/rust_interop.rs:30-37`) and are exempt from `SIFR-ASYNC-0001`, and that sync `@python` declarations carry the `blocking_io` effect via the same interop-effect channel rather than the bare-name workload annotation.

### B7 — `**record` kwargs behavior on non-introspectable targets is contradictory
`python_interop_declaration_architecture.md` → *Argument Passing*.

The record-expansion bullet says field names are "checked against an inspectable target," while the paragraph immediately below says non-introspectable targets "remain runtime-checked." It is undefined whether `**record` is *rejected* when the target can't be introspected (C extensions expose no signature — confirmed as a real case in `crates/sifr_package/src/cargo/python_probe.rs`) or *accepted as runtime-checked*. Failure mode: divergent implementations; a legitimate C-extension binding either works or is a hard error depending on the implementer.

**Required correction:** Pick one and state it. Recommended: `**record` requires an introspectable target and is rejected (`SIFR-PYCALL-*`) otherwise, because record→kwargs field-name checking is the entire value of the form; heterogeneous/uninspectable kwargs go through a bridge. Say this explicitly.

### B8 — DLPack `stream=consumer` has no stream source, and `device=cuda` is untestable
`python_interop_protocol_architecture.md` → *DLPack*; plan M10.

`stream=consumer` "states whether the consumer supplies the synchronization stream," but the acquisition signature (`tensor(self) -> Result[python.DlpackTensor[float32]]`) has no stream parameter and the design defines no stream/device-context value type. Separately, the current runtime is CPU-only and *rejects* non-CPU devices (`crates/sifr_runtime/src/python/dlpack_ops.rs:110-112`), so `device=cuda` + stream synchronization is wholly new and unverifiable in CI (no GPU). Failure mode: the `stream=consumer` contract cannot be implemented as specified, and `device=cuda` can never reach "supported" evidence.

**Required correction:** Define where the consumer stream comes from (an explicit declared parameter, or a runtime device-context object passed to `__dlpack__(stream=...)`), and pin the exact `max_version`. State the host/hardware requirement for CUDA certification (or restrict the shipped end state to `device=cpu|any` with CUDA gated behind the same certification-host mechanism as B5, named explicitly rather than implied).

---

## NON-BLOCKING (should be resolved in this pass, not deferred)

### N1 — Non-send opaque handoff to the loop thread needs an explicit statement
`python_interop_protocol_architecture.md` → *Event-Loop Ownership* / *Cancellation And Shutdown*. Async methods take non-send `self` yet convert inputs and execute on a *different* OS thread (the owned loop). This is safe only because handles are `(i64,i64)` object-store ids, not raw pointers (`crates/sifr_runtime/src/python/object_ops.rs:15`), resolved under the GIL on the loop thread. Say so explicitly, and state that the caller's borrow of `self` is frozen across the await and cannot be closed concurrently (relevant to `cleanup=async_close`, which consumes `self`). Otherwise an implementer may wrongly impose a `Send` bound and conclude the whole async-method surface is impossible.

### N2 — Sync opaque Python objects are unusable from async code; document the intended pattern
Sync `@python` methods are `blocking_io` and take/return non-send opaque values; async code must offload via `task.spawn_blocking`, which requires owned+sendable+static captures (`async_concurrency_model.md`, *Ownership And Borrowing* table). A non-send Python object can neither be captured into nor returned from `spawn_blocking`. Net effect: from async code you can only touch a sync-only Python object inside a single self-contained `spawn_blocking` closure that returns sendable values. That is defensible, but it must be stated as the intended pattern or users hit a silent dead-end.

### N3 — Per-call `asyncio.run` must be explicitly retired
Current raw coroutine support is `run_coroutine_blocking` = `asyncio.run(coro)` per call (`crates/sifr_runtime/src/python/coroutine_ops.rs:5-17`). The protocol doc bans per-call loops and mandates one implementation path, and *Raw API Relationship* says the raw API reuses the same resources. State that the raw coroutine path is re-routed through the one owned loop and the per-call `asyncio.run` path is removed, so two async execution paths do not coexist.

### N4 — `dispatch=asyncio` callback concurrency is unspecified
`python_interop_protocol_architecture.md` → *Dispatch Modes*. `foreign` requires `concurrency=serial|parallel`; `current` and `asyncio` state none. On one loop, multiple tasks can invoke an asyncio-dispatched callback with interleaved awaits. Define the concurrency/reentrancy contract for `asyncio` dispatch (e.g., serialized per owner by the loop, reentrancy rejected with the same `SifrCallbackReentrancyError` rule).

### N5 — Confirm union error channels are real
`python_interop_protocol_architecture.md` → *Callback Declarations* returns `Result[list[Row], PythonError | HandlerError]`. The async/error model uses a single `E`. Confirm Sifr supports a `A | B` union as an ordinary `try`-propagatable error channel; if not, specify the generated concrete error enum rather than a bare union, or the callback examples do not compile.

### N6 — `[python].requires-imports` + derived requirements is a residual second source
`python_interop_declaration_architecture.md` → *Environment And Trust* (manifest table). Derived (declaration/bridge) roots and manual `requires-imports` both feed one inventory. This is defensible for genuinely underivable dynamic imports, but define dedup/precedence when a manual root also appears via a declaration, so it is demonstrably one authority and not a silent second one. Current fields: `PythonConfig.requires_imports` (`crates/sifr_package/src/manifest/sifr.rs:56`), aggregated with the (removed) allow-list in `trust_policy.rs:6-32`.

### N7 — M1 conflates independent concerns into one oversized merge unit
Plan → *M1*. It bundles the sealed-handle substrate + pending-release queue (additive runtime changes) with uv discovery, requirement inference, the atomic `[python].allow-imports` removal, and the PYTRUST renumber. Only the authority swap needs atomicity per the delivery rule. Split the additive runtime substrate from the allow-imports removal to keep PRs reviewable per `AGENTS.md`, without weakening the atomic-removal requirement. This also respects the 900-line file guardrail, since sealed handles touch `sifr_runtime/src/python.rs` (already large).

### N8 — Live certification is a wholesale harness replacement; say so, and state host requirements
Plan → M12 and *Verification Policy*; declaration doc *Verification Contract*. Today no compiled Sifr binary touches the live containers: live cases run Python clients and only `sifr check` (typecheck) the `_live_roundtrip.sifr` sources (`verification/areas/python_interop/runner/live_examples.py`); only in-process-fake `*_full_example.sifr` cases build+run. The new requirement (compiled `sifr run` against Redis/Postgres/Kafka/SQS/SNS/async/zero-copy) is correct but is a full replacement of the live lane, and Arrow pointer-identity + DLPack CUDA need hardware CI lacks. State the replacement explicitly and name the certification-host requirement, so the capability matrix cannot mark those rows "supported" without matching executable evidence.

### N9 — `AsyncExitCause` shape mismatch across doc/model/implementation
Tied to B1. `async_concurrency_model.md` describes a variant enum; the implementation is a fieldless marker (`crates/sifr_lowering/src/lower/async_with.rs:78-85`). The protocol doc reuses it and needs it to carry the originating cause including a live-Python-exception variant. Reconcile all three: specify the concrete variant set (with a `PythonError` variant) as the single source of truth and note the implementation must be upgraded from the marker class.

### N10 — Baseline diagnostic reservations omit `PYASYNC`/`PYCTX`
`python_interop_architecture.md` → *Diagnostic Evidence* reserves only `PYIMP/PYCALL/PYCONV/PYRES/PYZC/PYCB`; the new docs and plan M0 add `PYASYNC`/`PYCTX`. Add these to the baseline reservation list in the same merge unit that locks M0 families, so no family gap exists (`crates/sifr_diagnostics/src/codes/registry.rs:64-80`).

---

## Optional taste (not findings)

- **T1.** `python.omit` cannot be produced from a runtime variable (no conditional omission at a call site). This is by design (matches Python) but worth one explanatory sentence. Note also that the omission-tracking mechanism is entirely new — nothing like it exists today (no sentinel/`Missing` machinery in `sifr_lowering`); the doc's "call lowering preserves whether the caller supplied that argument" is a sufficient spec of intent, but a one-line note on the lowering (compile-time provided-set, not a runtime sentinel) would prevent a wrong implementation.
- **T2.** `python.Buffer[T]` carries no ndim/shape in the type; document the runtime shape/stride/suboffset accessors so consumers of multi-dimensional buffers know the surface.
- **T3.** The cleanup-policy list appears in both docs (declaration *Ownership And Cleanup* and protocol *Cleanup Policies*). Pick one as normative and cross-reference, to prevent future drift.

---

## Verdict

**REQUEST CHANGES.**

The end-state architecture is sound and largely feasible on the existing `@rust`-interop template, sealed-handle store, and `own`-based affine model. But eight blocking items are contracts named without definitions or in direct conflict with the sealed async/cancellation model (B1–B4, B6), an unbuildable/hidden-allowlist gate (B5), an ambiguous call shape (B7), and an unspecified DLPack stream/device contract (B8). Each is resolvable now with the exact language proposed above; none requires deferring design. Resolve the blocking set and fold in N1–N10, then this is approvable.

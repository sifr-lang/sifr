I now have airtight ground truth. The Ruff-fork parser's `parse_identifier` (`third_party/ruff/crates/ruff_python_parser/src/parser/expression.rs:520-528`) explicitly rejects a non-soft keyword in attribute-name position, and `async` is a hard keyword token (`ruff_python_ast/src/token.rs:282`). The proven `@rust` async template never uses a `.async` attribute — it derives async-ness from `async def` (`verification/areas/rust_interop/.../stdlib_async_resource_lifecycle.sifr:12`). This confirms a blocking, previously-unexamined defect. Here is the final audit.

---

# Final Approval Audit — Complete Declaration-First Python Interop (agent, Pass 3)

**Scope:** `python_interop_architecture.md`, `python_interop_declaration_architecture.md`, `python_interop_protocol_architecture.md`, `plans/issues/active/ad-hoc-declaration-first-python-interop.md`, prior passes 1–2, re-grounded against the Ruff-fork parser, `sifr_lowering`/`sifr_type_system` ownership, the native `AsyncExitCause`/async-effect substrate, and `@rust` interop fixtures.

## Recheck of pass-1 and pass-2 findings — all resolved
Pass-1 B1–B8 and N1–N10, and pass-2 B-N1 and N-1…N-5, are all genuinely incorporated in the current text. Specifically the six pass-2 corrections the user named are present: async `ExitCause` constructed directly from the concrete body outcome before native cause erasure (`protocol` 258-269; plan M8 384-388); live triple retained behind a sendable private store id with explicit lifetime/cost (`protocol` 172-186); Arrow records within-run pointer-identity assertion results, never absolute addresses (`protocol` 421-423, 438-439); `from(name)` and `**record` declared as new productions (`protocol` 478-482; `decl` 204-206; plan M3 224, M12 507-508); DLPack source capsule renamed to the used sentinel before Sifr assumes deleter ownership (`protocol` 497-508); and the milestone split of conversion/opaque from sync contexts (M4/M5) and of check-doctor/bind-certify/LSP/raw/ecosystem (M13–M17). No regressions found in those areas.

But prior passes audited semantics and substrate feasibility; neither examined the **concrete decorator surface syntax against the actual Python front end**. That is where the remaining blocking defect lives.

---

## BLOCKING

### P3-B1 — Three decorator forms use reserved Python keywords the Ruff-fork front end cannot parse
Sifr source (including decorators) is parsed by the Ruff-fork Python grammar (`AGENTS.md:61`). Decorators are Python expressions, so every decorator attribute must be an identifier and every keyword-argument value must be a valid expression. `parse_identifier` rejects a non-soft keyword in identifier position: *"Expected an identifier, but found a keyword … that cannot be used here"* (`third_party/ruff/crates/ruff_python_parser/src/parser/expression.rs:520-528`). `async`, `from`, and `return` are hard keywords (`ruff_python_ast/src/token.rs:282` etc.), not soft keywords. Three surface forms therefore fail to parse *before* any interop lowering runs:

1. **`@python.async(...)`** — the primary coroutine boundary. `python.async` lowers to `Attribute(value=Name('python'), attr='async')`; `attr` goes through `parse_identifier`, which hits the hard-keyword branch → parse error. This is pervasive: `protocol` 35, 40, 48, 52, 67, 77, 345; `decl` 131, 292, 456, 466; plan 30, 348. The proven `@rust` template never uses a `.async` attribute — async-ness comes from `async def` (`verification/areas/rust_interop/fixtures/async_runtime_core/positive/stdlib_async_resource_lifecycle.sifr:12`), which is exactly why this collision was never exercised before.
   - *Failure mode:* every `@python.async` declaration in every example is a hard syntax error; the primary async binding form cannot be written.
   - *Correction:* rename `.async` to a non-keyword attribute, e.g. `@python.coroutine(path)` (or `@python.aio(path)`), and update all three docs and the plan.

2. **`stream=from(name)`** — DLPack stream source (`protocol` 462-463, 475, 478; plan 505, 507). `from` is a hard keyword; `from(consumer_stream)` cannot begin an atom/call, and it also collides with `from … import`. Plan 507 bills this as "a decorator-argument parser/HIR production" without acknowledging that it requires special-casing a reserved keyword in the tokenizer.
   - *Failure mode:* the DLPack stream-provenance decorator cannot be parsed; the entire consumer-stream contract (M12) is unwritable as specified.
   - *Correction:* choose a non-keyword spelling, e.g. `stream=of(name)`, `stream=param(name)`, or `stream=bind(name)`; keep the same "resolves to a same-declaration keyword-only `python.DlpackStream` parameter" semantics.

3. **`lifetime=return`** — callback lifetime atom (`protocol` 293, 304; plan 412 `lifetime=call | return | Self`). `return` is a hard keyword; `@python.callback(handler, lifetime=return, …)` is a syntax error because a keyword-argument value must be an expression and `return` cannot appear in expression position.
   - *Failure mode:* the `lifetime=return` (retained-into-return-value) callback declaration cannot be written; retained callbacks bound to the return value are unexpressible.
   - *Correction:* rename the atom to a non-keyword, e.g. `lifetime=retained` or `lifetime=owner`.

All three are one root cause and one fix pattern (do not spell decorator attributes/atoms with Python hard keywords). None requires deferring or reducing the design — only renaming the surface tokens across the three docs and the plan in the same merge unit that locks the grammar (M0). Because the chosen syntax is literally unparseable and the plan does not budget a tokenizer change (nor should it, given `async def` needs `async` to stay a hard keyword), this blocks.

---

## NON-BLOCKING (resolve in this pass; do not defer)

### P3-N1 — Linear "must-use" enforcement is net-new and is neither articulated as a task nor covered by a negative test
`protocol` *Cleanup Policies* line 137 requires: "Ownership analysis rejects paths that can abandon a value requiring semantic close," and lines 129/131 require `cleanup=context`/`async_context` values to be *consumed by* `with`/`async with`. Ground truth: Sifr ownership is **affine, not linear** — `OwnershipKind::{Copy, Move}` (`crates/sifr_type_system/src/types/definitions.rs:382-389`); the only ownership diagnostic is use-after-move (`SIFR-OWN-0001`), and **nothing** rejects an owned value that leaves scope unconsumed. The single precedent for a must-consume obligation is the bespoke JoinSet function-exit liveness check (`crates/sifr_lowering/src/lower/typing_and_functions/annotations_and_function_lowering.rs:747-766`) — a hard-wired side-table, not a general facility.

The architecture is correct and feasible (that precedent is the implementation template), but the plan under-specifies it: M4 says only "Enforce … consuming synchronous semantic close" (satisfiable by ordinary affine move alone) and its validation lists "double close, poison, use-after-close" (plan 273-276) with **no** "value requiring semantic close/context exit is abandoned → rejected" fixture. M5, M7 (`async_close`), and M8 (`async_context`) have the same omission.
- *Failure mode:* an implementer builds affine-only cleanup, passes the stated M4/M5/M7/M8 validation, and ships silently-skipped semantic cleanup (a Redis client dropped without `close`) — the exact lifecycle leak the "never replaces … rejects paths that can abandon" contract forbids.
- *Correction:* add an explicit task to M4 — "implement net-new linear must-use analysis (liveness side-table + scope/function-exit check, mirroring the JoinSet precedent) that rejects abandonment of any `cleanup=close|async_close|context|async_context` value" — and add an abandonment-rejection negative fixture to M4 (sync close), M7 (async close), M5 (sync context never entered), and M8 (async context never entered).

### P3-N2 — The "synthesized AsyncIo suspension summary" mechanism does not match the substrate
`protocol` 77-82 states the interop HIR "assigns every `@python.async` declaration a synthesized `AsyncIo` suspension summary" to escape `SIFR-ASYNC-0001`. Ground truth: `AsyncSuspensionSummary` has exactly `{ NoSuspend, Suspends }` with no `AsyncIo` variant (`crates/sifr_lowering/src/lower/async_effects.rs:4-8`), and `@rust` interop stubs escape the NoSuspend gate not via any summary but via the `!stub_body.skips_normal_body_lowering()` guard for `Bodyless` stubs (`annotations_and_function_lowering.rs:565-580`, `rust_interop.rs:25-49`). `RustInteropEffect::Async` is a separate effect concept, not a suspension summary. The goal (ellipsis `@python.async` not rejected as fake-async) is trivially achievable — the `@rust` ellipsis path already does exactly this — but the doc names a mechanism that doesn't exist.
- *Failure mode:* an implementer searches for a nonexistent `AsyncIo` summary state; wording drift between "suspension summary" and "interop effect."
- *Correction:* reword `protocol` 77-82 and plan M3 (216-217)/M7 (352-355) to say `@python.async` ellipsis declarations skip the NoSuspend gate through the interop stub-body path (as `@rust` `Bodyless` stubs do) and carry the async interop effect — not a new `AsyncSuspensionSummary` variant.

### P3-N3 — Verification-ownership gap for device/CUDA Arrow (and unnamed CPU DLPack)
The design lists `python.ArrowDeviceArray`/`ArrowDeviceStream` as supported return types with device-metadata validation (`protocol` 402-405; plan M11 474), but the runner-ownership assignment names only "CPU runners own Arrow pointer evidence, and labeled CUDA runners own CUDA DLPack rows" (`decl` 596-598; plan M17 623-626). No runner owns CUDA/device **Arrow** certification, and CPU DLPack (e.g. torch CPU tensors) is not named in the triad though M12 implies it.
- *Failure mode:* a supported capability row (`ArrowDeviceArray` on GPU) can never obtain executable evidence and thus can never be promoted — a "complete verification ownership" hole.
- *Correction:* extend the ownership statement so labeled CUDA runners own CUDA Arrow device rows as well as CUDA DLPack, and name CPU runners as owners of CPU DLPack rows; or explicitly scope `ArrowDeviceArray`/`ArrowDeviceStream` to CPU-device-only in the shipped end state.

### P3-N4 — Milestone dependency annotations are inconsistent (ordering is sound; annotations are not)
M5/M6/M8 carry explicit "Depends on" lines, but real prerequisites are unstated for M3 (needs M1 sealed identity + M2 trust/probe for opaque wrappers), M4 (needs M3), M7 (needs M4 for `cleanup=async_close` opaque lifecycle), M9 (asyncio dispatch needs M7's owned loop; retained owners need M4), and M10/M11/M12 (need M1 affine substrate + M4 conversion). Numeric ordering satisfies every dependency — there is **no** backward gap — so this is annotation completeness only.
- *Correction:* add "Depends on" lines to M3, M4, M7, M9, M10, M11, M12 matching the stated style, so no milestone reads as prerequisite-free.

### P3-N5 — DLPack "legacy capsule" support vs. always passing `max_version`/`copy` needs a no-fallback clause
`protocol` 493-496 says acquisition always passes `copy=False` and `max_version=(1,0)` yet "validates legacy or versioned capsule names." A versioned-signature producer may legitimately return a v0 (`dltensor`) capsule, but a genuinely legacy-*signature* producer (no `max_version`/`copy` kwargs, pre-DLPack-1.0) raises `TypeError` when called with them, and the "one path, no fallback" rule (`protocol` 28-29) forbids a retry-without-kwargs.
- *Failure mode:* ambiguity about whether legacy-signature producers are bindable; risk of an implementer adding a hidden retry-without-kwargs fallback.
- *Correction:* state that "legacy capsule name" support means a versioned-signature producer that emits a v0 capsule, and that old-signature producers (which reject `max_version`) are not directly bindable and must go through a package bridge — no silent fallback path exists.

### P3-N6 (minor) — Make callback shutdown's non-reentrancy explicit
`protocol` 352-356 shutdown "waits for accepted invocations to finish." Reentrant *invocation* is explicitly rejected (`protocol` 340-348), but reentrant *semantic close* from within an accepted invocation is not addressed. It is unreachable by construction (foreign dispatch forbids opaque captures; `lifetime=return` owners don't exist at handler-construction time), so no deadlock actually occurs — but one sentence stating that semantic owner close cannot be invoked from within an accepted invocation would make the non-deadlock property explicit, parallel to the reentrant-invocation rejection.

## Confirmations (no action)
- The pass-2 B-N1 resolution is correct **and** reinforced by ground truth: native async-with does not even run `__aexit__` on error paths and reduces nothing to a `PythonException` variant (`AsyncExitCause` has none), so the design's dedicated Python async-context lowering (constructing `python.ExitCause` directly and running exit on all paths, M8) is the only correct route — it must not reuse native lowering, which the docs already require.
- Union error channels (`Result[T, PythonError | HandlerError]`) are real, `?`-propagatable, and codegen'd as named enums (verified against passing `Result[…, ArrowError | RustPanicError]` fixtures).
- Sync decision-table `timeout`/`cancellation`/`runtime-fault` rows are an unreachable-but-harmless superset for purely synchronous blocks. No change.
- Exception-replay lifetime (borrow across nested exits, exactly-once final release via the detach-before-decref queue, `Send` preserved via integer store ids) is internally consistent and leak-free.

---

## Verdict

**REQUEST CHANGES.**

All pass-1 and pass-2 findings are genuinely resolved and the design's semantics (exception replay, async cause classification, cancellation/shutdown, Arrow/DLPack ownership, buffer affinity, trust atomicity) are coherent and feasible on the proven `@rust` substrate. But one previously-unexamined class of defect blocks: **P3-B1** — the concrete decorator surface (`@python.async`, `stream=from(...)`, `lifetime=return`) spells three forms with Python **hard keywords** that the Ruff-fork front end rejects at `parse_identifier` before any interop lowering runs, making the primary async binding form and two protocol atoms literally unparseable. It is a pure rename fix (no capability reduction, no deferral), but as written the syntax cannot compile. Fold in P3-N1 (net-new must-use analysis + abandonment tests), P3-N2 (correct the async-effect mechanism wording), P3-N3 (device/CUDA Arrow evidence ownership), P3-N4 (dependency annotations), and P3-N5 (DLPack no-fallback clause) in the same pass. With P3-B1 corrected and the non-blocking items folded in, this is approvable.

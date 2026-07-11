I have complete ground truth. Here is pass 2.

---

# Independent Architecture Review — Complete Declaration-First Python Interop (Opus High, Pass 2)

**Scope reviewed:** the revised `python_interop_declaration_architecture.md`, `python_interop_protocol_architecture.md`, `plans/issues/active/ad-hoc-declaration-first-python-interop.md`, the updated baseline `python_interop_architecture.md`, and pass-1 (`…-complete-opus-high-pass-1.md`), re-grounded against `crates/sifr_codegen/src/preamble/types_and_errors.rs`, `sifr_lowering/src/lower/async_with.rs`, `sifr_codegen/src/stmt_support_emitter/async_cleanup.rs`, the union-error fixtures in `verification/areas/rust_interop`, and `sifr_diagnostics`.

## Pass-1 recheck — all eight blockers and ten non-blockers are resolved

- **B1** (ExitCause/Decision/SifrBoundaryError undefined) — resolved. Protocol *Exit Cause And Decision Types* now defines `python.ExitCauseKind`, `python.ExitDecision`, sealed `ExitCause` with redacted metadata, and `SifrBoundaryError` fields.
- **B2** (two `with` semantics) — resolved. Protocol *Synchronous Context Managers*, line 234-238: dedicated Python-context lowering keyed on `@python.opaque(cleanup=context)`; native `with` explicitly unchanged.
- **B3** (suppression swallowing cancellation/Sifr error) — resolved. The normative decision table (lines 189-200) plus the closing sentence “a foreign truthy return can never swallow Sifr cancellation or an ordinary Sifr error.”
- **B4** (live exception triple) — resolved for the sync path. *Exit Cause And Decision Types*, lines 170-185: compiler-private replay capability, unforgeable object-store identity, GIL-only resolution, released once. (New consequence surfaced below — N-2.)
- **B5** (Arrow certification authoring) — resolved. *Arrow C Data Interface*, lines 407-430: `src/python_certifications/<name>.json`, `sifr python certify arrow`, `certify --check`, “Any package author can certify a new producer.” (Integrity wording issue below — N-1.)
- **B6** (async-effect seal rejects ellipsis) — resolved. Protocol lines 77-82 + plan M3/M6: synthesized `AsyncIo` summary, `blocking_io` via the interop-effect channel.
- **B7** (`**record` on uninspectable target) — resolved. Declaration *Argument Passing*, lines 200-202: requires inspectable target, `SIFR-PYCALL-*` otherwise.
- **B8** (DLPack stream source / CUDA) — resolved. Protocol *DLPack*: `stream=from(parameter)`, keyword-only `python.DlpackStream`, `@python.dlpack.stream`, `max_version=(1,0)`, labeled CUDA runners.
- **N1–N10** — all folded in: non-send handoff stated (proto 83-89); sync-from-async pattern (decl 459-464); per-call `asyncio.run` retired (proto 113-116, M6); asyncio callback concurrency (proto 326-339); union error channels confirmed real against `verification/areas/rust_interop/.../arrow_schema_identity.sifr` using `Result[T, A | RustPanicError]` with `?`; requirement provenance/dedup (decl 431-434, M2); M1 split from trust cutover; live-lane host requirements (M13); `PYASYNC`/`PYCTX` now reserved in the baseline (`python_interop_architecture.md:130-133`). T1/T2/T3 taste items also folded in (omit bitset decl 208-210; buffer accessors proto 376-381; cleanup-policy normativity decl 280-282).

The correction of N9 — decoupling `python.ExitCause` from the native `AsyncExitCause` — is where the one new cross-contract contradiction was introduced.

---

## BLOCKING

### B-N1 — Async context-manager Python-exception replay is unreachable through the specified `AsyncExitCause` mapping
`python_interop_protocol_architecture.md` → *Asynchronous Context Managers*, lines 254-256 (“Dedicated Python async-context lowering **maps native `AsyncExitCause` into `python.ExitCause`** and follows the same decision table”); plan `M7`, lines 362-364 (“**Map native `AsyncExitCause` into `python.ExitCause`**, replay original Python exception triples…”).

Ground truth: the native runtime `AsyncExitCause` enum (`crates/sifr_codegen/src/preamble/types_and_errors.rs:598-644`) has variants `Normal | Return | OrdinaryError(String) | Timeout | Cancellation | RuntimeFault(String)`. It has **no `PythonException` variant**, and a propagated `Err(PythonError)` from an async-with body is reduced to `OrdinaryError(String)` (Display), discarding the live exception — `async_cleanup.rs:97-106` passes `AsyncExitCause::{variant}` by reference with no capability payload. The protocol doc itself states (lines 143-144) that `python.ExitCause` “does not reuse or redefine the native `AsyncExitCause` protocol type,” i.e. native `AsyncExitCause` is *not* being extended.

So the two clauses are mutually exclusive: you cannot both (a) leave native `AsyncExitCause` unchanged and (b) recover the original live Python triple by *mapping from it*. As written, an implementer routing through native `AsyncExitCause` classifies a Python-originating body error as `OrdinaryError` → feeds `__aexit__` a synthetic `SifrBoundaryError` instead of the real `(type, value, tb)`.

**Failure mode:** `async with aiohttp/asyncpg-style manager:` where the body returns `Err(PythonError)` originating from a Python call — `__aexit__` receives `SifrBoundaryError` rather than the live exception. `contextlib.suppress(SpecificError)`, async DB `rollback`-on-specific-exception, and `except TypeError` inside the manager all misbehave. This is exactly the B4 failure mode the replay machinery was built to prevent, reappearing on the async path, and it directly contradicts M7’s own acceptance “replay original Python exception triples.” Two implementers (one routing through native `AsyncExitCause`, one constructing directly like the sync path) build observably different systems.

**Required correction (resolve now, one clause):** Specify that dedicated Python **async**-context lowering constructs `python.ExitCause` **directly** from the async-with body’s control-flow outcome and the concrete propagated value — inspecting the terminal `Err` to detect a concrete `PythonError` and populate `PythonException` with its replay capability, using the runtime’s cancellation/timeout signals for those causes — exactly as the sync path does (`Synchronous Context Managers`, lines 220-222). State that native `AsyncExitCause` remains solely the *native* `async with` protocol type and is **not** the cause-classification source for Python contexts. Replace “maps native `AsyncExitCause` into `python.ExitCause`” in proto:256 and M7:363 accordingly.

---

## NON-BLOCKING (resolve in this pass, do not defer)

### N-1 — Arrow certification artifact records absolute pointer identities; `certify --check` would always fail
`python_interop_protocol_architecture.md` → *Arrow C Data Interface*, lines 411-413 (“…SOABI, …fixture/source digest, compiler certification version, **observed buffer identities**, and exact release counts”) and 429 (“fails on environment, source, **pointer**, or release-count drift”).

Absolute buffer addresses are not stable across runs (ASLR, allocator state). If the checked-in JSON records literal pointer values and `--check` re-compares them, every recheck fails spuriously; if it records nothing verifiable, “pointer drift” is undefined. Zero-copy is proven by *within-run* identity (producer buffer address == consumer-observed address), not by a cross-run-stable address.

**Correction:** State that the artifact records the *assertion outcomes* — “within-run producer/consumer pointer identity held” (boolean) and “exact release/deleter count = N” — plus the environment/source/SOABI fingerprint; `certify --check` **reruns the fixture and re-derives** those assertions rather than diffing absolute addresses. Reword “observed buffer identities” → “observed within-run pointer-identity results” and “pointer drift” → “pointer-identity-assertion failure.”

### N-2 — Retained replay triple makes `PythonError` sendability and retention scope implicit
`python_interop_protocol_architecture.md` → *Exit Cause And Decision Types*, lines 172-185; declaration doc *Blocking And Async*, lines 459-463.

Every `PythonError` now “stores that capability beside its public structured fields.” `PythonError` must cross the `task.spawn_blocking` return boundary from a GIL/blocking-pool thread back to the async executor (decl 461-463), so `PythonError` must be `Send`. That holds only if the retained triple is an object-store id (integers) released via the detach-before-decref queue on a later attach — never a live `Py<PyAny>` pointer. Two things are left implicit: (a) that `PythonError` remains `Send` under this change; (b) whether *every* boundary error retains a full live Python traceback until drop, which pins arbitrary frame/local memory for the ubiquitous error path.

**Correction:** State that `PythonError` remains `Send` because the replay capability is a compiler-private object-store id (not a live pointer) released through the pending-release queue; and pin retention scope — either the capability is populated for all `PythonException`-caused errors (Python-parity, accept the traceback cost) or only for errors that can reach a context exit — explicitly, so the cost model and the `Send` guarantee are not left to implementers.

### N-3 — M4 (and M12) are oversized merge units, same critique that split M1
Plan → `M4. Recursive Conversion, Opaque APIs, And Sync Contexts` (lines 245-284) and `M12` (lines 505-539).

M4 bundles three independently large, independently reviewable subsystems: the full recursive conversion matrix; the opaque object model (isinstance/borrow/move/poison/semantic close); and the entire sync context machinery (`ExitCause`, live-exception replay capabilities, `SifrBoundaryError`, the decision table, secondary cleanup evidence). This is the same reviewability/900-line concern that justified splitting M1 from the trust cutover (pass-1 N7); it touches conversion, opaque, and context lowering plus `sifr_runtime/src/python` all at once.

**Correction:** Split M4 into M4a (recursive conversion + opaque model + semantic close) and M4b (sync context managers + `ExitCause`/replay/`SifrBoundaryError`/decision table). Note M12 similarly bundles check + doctor + bind + certify + LSP + raw-API; at minimum flag `certify` (shared with M10) and the raw-API rework as separable. This preserves the atomic-cutover rule (which only M2 requires) while keeping PRs reviewable per `AGENTS.md`.

### N-4 — `from(parameter)` and `**record` are new grammar productions, not static-literal atoms; pin their resolution
`python_interop_protocol_architecture.md` → *DLPack*, lines 449-466 (`stream=from(consumer_stream)`); declaration doc *Argument Passing*, lines 200-202 (`**record`).

Pass 1 correctly noted `@rust` supplies a dotted-path decorator grammar with a *static value* grammar. `from(name)` (a decorator-argument form referencing a declaration parameter) and call-site `**record` spread are genuinely new syntactic productions beyond static literals, and `from(name)` needs a defined resolution scope.

**Correction:** State that `from(name)` is a decorator-argument production (not a literal atom) whose `name` must resolve to a **keyword-only `python.DlpackStream` parameter of the same declaration** of the matching device family/id, diagnosed under `SIFR-PYZC-*`/`SIFR-PYCALL-*` otherwise; and that `**record` is a new call-site expansion requiring an inspectable target (already stated, decl 202). Add both to M3/M11 as explicit parser/HIR additions so they are not mistaken for existing `@rust` grammar.

### N-5 (taste) — DLPack legacy-capsule acquisition must rename the source capsule to the used sentinel
`python_interop_protocol_architecture.md` → *DLPack*, lines 480-491.

The one-shot rules cover Sifr’s own deleter invocation, but not the source `PyCapsule`’s own destructor: if Sifr acquires a legacy `dltensor` capsule and drops the tensor (invoking the deleter) without renaming the capsule to `used_dltensor`, Python’s capsule destructor may *also* invoke the deleter → double free.

**Correction:** State that acquisition renames the source capsule to `used_dltensor` (or the versioned equivalent) at the moment ownership transfers to the affine `python.DlpackTensor`, so Python’s own capsule destructor becomes a no-op and the deleter is invoked exactly once by Sifr.

---

## Confirmations (no action)

- Union error channels (`Result[T, PythonError | HandlerError]`, proto 273-314) are real and `?`-propagatable — verified against existing `Result[…, ArrowError | RustPanicError]` fixtures. N5 is correctly closed.
- The sync decision-table rows `timeout`/`cancellation`/`runtime fault` (proto 194-196) are unreachable in a purely synchronous block (Sifr cancellation is await-point scope-exit, not mid-sync preemption); the shared table is a harmless superset. No change needed.
- Nested replay + suppression semantics (proto 181-185) are internally consistent: the live triple is borrowed by each stacked exit, released on suppression/replacement/exit-from-chain, owned finally by the propagated `PythonError`. Correct.

---

## Verdict

**REQUEST CHANGES.**

Pass-1’s eight blockers and ten non-blockers are all genuinely resolved against ground truth. One new cross-contract contradiction was introduced by the N9 correction: **B-N1** — async context-manager Python-exception replay is specified to flow through the native `AsyncExitCause` enum, which lacks a `PythonException` variant and reduces Python errors to `OrdinaryError(String)`, making M7’s own “replay original Python exception triples” acceptance unreachable and reintroducing the B4 failure mode on the async path. It is a one-clause fix (construct `python.ExitCause` directly, as the sync path already does), but as written it forces divergent implementations and a real behavioral defect, so it blocks. Fold in N-1 through N-5 in the same pass; with B-N1 corrected this is approvable.

I have complete ground truth. All pass-1/2/3 findings are genuinely incorporated, and I verified the load-bearing substrate claims directly against the code. Here is the pass-4 audit.

---

# Final Approval Audit — Complete Declaration-First Python Interop (Opus High, Pass 4)

**Scope:** `python_interop_architecture.md`, `python_interop_declaration_architecture.md`, `python_interop_protocol_architecture.md`, `plans/issues/active/ad-hoc-declaration-first-python-interop.md`, and passes 1–3, re-grounded against the Ruff-fork tokenizer (`third_party/ruff/crates/ruff_python_ast/src/token.rs`), `async_effects.rs`, the `Bodyless` interop-stub path, the JoinSet exit-liveness precedent, and ownership/diagnostics substrate.

## Recheck of all prior findings — resolved and verified against ground truth

**Pass 3 surface-token fix (P3-B1).** Confirmed clean. `grep` across all three docs and the plan finds **zero** occurrences of `@python.async`, `lifetime=return`, or `stream=from(`. The replacements are in place and every one tokenizes as a `Name`, not a hard keyword: `Async`/`From`/`Return` are distinct `TokenKind` variants (`token.rs:282,295,308`), while `coroutine`, `result`, and `parameter` are absent from the token enum. `parse_identifier`'s hard-keyword rejection therefore never fires on the new surface. The remaining atoms are all safe: `async_close`/`async_context` are single identifiers (not the `async` keyword), `none`/`any` are lowercase (not the `None` singleton keyword), `Self` is not a keyword.

**Pass 3 non-blocking items — all folded in and verified:**
- **P3-N1 (net-new linear must-use):** Protocol *Cleanup Policies* (141–149) now specifies the liveness side table, obligation transfer through moves/returns/aggregates/joins, and scope/function-exit checks, explicitly citing the JoinSet precedent as a general (not hard-coded-type) mechanism. Plan M4 (266–268) adds the task; abandonment negatives exist in M4 (284), M5 (315), M7 (387), M8 (419). Ground truth confirms the precedent is real: `reject_live_join_sets_at_function_exit` (`annotations_and_function_lowering.rs:747–766`) is exactly a `live_*_bindings` side-table filtered on `scope.is_moved` and checked at exit, hard-wired to `JoinSet` — precisely what the doc says it generalizes.
- **P3-N2 (async-effect mechanism):** Protocol 77–82 now says coroutine ellipsis uses the interop `Bodyless` stub path and "does not add a new `AsyncSuspensionSummary` variant." Verified: `AsyncSuspensionSummary` has exactly `{NoSuspend, Suspends}` (`async_effects.rs:5–7`), and the `SIFR-ASYNC-0001`/NoSuspend gate is guarded by `!stub_body.skips_normal_body_lowering()` (`annotations_and_function_lowering.rs:565–580`), which `Bodyless` returns true for (`rust_interop.rs:25–26`). The mechanism named in the doc is the one that exists. No `AsyncIo` variant references remain in any doc.
- **P3-N3 (device/CUDA evidence ownership):** `decl` 596–598 and plan M17 663–667 now assign CPU runners CPU-Arrow + CPU-DLPack and labeled CUDA runners Arrow-device-interface + CUDA-DLPack. The prior hole (no owner for device Arrow / unnamed CPU DLPack) is closed.
- **P3-N4 (dependency annotations):** Every milestone M3–M17 now carries an explicit "Depends on" line; ordering remains backward-gap-free.
- **P3-N5 (DLPack no-fallback):** Protocol 511–516 and plan M12 539–540 state legacy-*name* support means a versioned-signature producer emitting a v0 capsule; old-*signature* producers are bridge-only; "Generated code never catches `TypeError` and retries."
- **P3-N6 (callback close reentrancy):** Protocol 373–375 and plan M9 445–446 reject semantic owner-close from within an accepted invocation, statically where visible and via a runtime guard otherwise.

**Pass 1/2 semantic fixes still intact:** async `python.ExitCause` is constructed directly from the concrete body outcome; native `AsyncExitCause` "remains solely the native `async with` protocol type and is not the classification source" (protocol 155, 275) — the B-N1 contradiction stays resolved. Arrow records within-run pointer-identity assertion results, never absolute addresses (protocol 421–423). `[python].allow-imports` appears only in atomic-removal instructions.

No regressions in any previously-resolved area. No unparseable syntax, no dual authority, no `py.Object` degradation path, no alternate lowering path, no backward-compatibility period.

---

## NON-BLOCKING (resolve in this pass; neither defers nor reduces a capability)

### P4-N1 — DLPack `device=any` has no defined stream policy; as written it has no valid `stream=` completion
`python_interop_protocol_architecture.md` → *DLPack*, lines 486–491 (grammar also in plan M12:531–532).

The grammar admits `device=cpu | cuda | any`, but the stream rule partitions only two ways: "`stream=none` is valid only for **CPU**" (488) and "A **non-CPU** declaration uses `stream=parameter(...)`, and that named keyword-only parameter must be `python.DlpackStream` for the **same device family and id**" (489–490). `device=any` fits neither bucket cleanly: it is not `cpu`, so `stream=none` is excluded; and it has no fixed device family, so the `stream=parameter` "same device family and id" match rule has nothing static to check against. **Failure mode:** an author writing `@python.dlpack(Self, device=any, ...)` has no spec-valid `stream=` value — one advertised enum value is unspecifiable — and two implementers resolve it differently (one rejects `any`, one accepts `stream=parameter` with runtime-only family validation). **Correction:** state the `device=any` rule explicitly — either (a) `device=any` requires `stream=parameter(...)` and lowering defers the stream's device-family/id match to a runtime check against the producer's reported device (with `stream=none` still CPU-only), or (b) drop `any` from the DLPack `device` set and keep only `cpu | cuda`. Either keeps the one-path, no-copy, no-cross-device contract intact.

### P4-N2 — Async-interop-effect wording names only `@python.coroutine`; `.aenter`/`.aexit` and asyncio callbacks rely on the general rule implicitly
`python_interop_protocol_architecture.md` → *Async Python Calls*, lines 77–82, vs *Asynchronous Context Managers* (259–267) and *Dispatch Modes* (`dispatch=asyncio`).

Line 77 states "**Every `@python.coroutine` declaration** carries the async interop effect. Its ellipsis body uses the interop `Bodyless` stub path…". Async context methods (`@python.context.aenter`/`.aexit` on `async def`) and asyncio-dispatched callbacks are also async interop ellipsis declarations that must skip the `NoSuspend` gate and carry the async interop effect, but the effect paragraph is phrased coroutine-specifically. This is **already covered** by the general rule that all interop ellipsis bodies lower through `Bodyless` (verified: the `skips_normal_body_lowering()` guard is decorator-agnostic), so an M8 implementer building `.aenter`/`.aexit` as interop stubs inherits the correct behavior automatically. **This is a wording-completeness refinement, not a defect.** **Correction (optional):** generalize line 77 to "Every async Python interop declaration (`@python.coroutine`, `@python.context.aenter`/`.aexit`, and `dispatch=asyncio` handlers) carries the async interop effect through the same `Bodyless` interop-stub path," so no reader infers the escape applies only to coroutines.

---

## Confirmations (no action)
- New surface tokens verified non-keyword against `token.rs`; `parse_identifier` hard-keyword rejection cannot fire on any decorator attribute or atom in the current grammar.
- `Bodyless`/`skips_normal_body_lowering()`, the two-variant `AsyncSuspensionSummary`, and the JoinSet function-exit liveness side-table all exist exactly as the docs now describe — the resolved-finding mechanisms are grounded, not aspirational.
- The pass-2 B-N1 async-cause resolution and pass-1 B3/B4 suppression/replay semantics remain internally consistent; native `AsyncExitCause` is correctly excluded as the Python classification source.
- Milestone ordering satisfies every stated dependency with no backward gap.

---

## Verdict

**APPROVE WITH NON-BLOCKING REFINEMENTS.**

No blocking findings remain. Every pass-1/2/3 blocker and non-blocker is genuinely incorporated and independently verified against the Ruff-fork parser, lowering, ownership, async, runtime, package, and diagnostics substrate: the surface tokens are parser-safe, the net-new linear must-use analysis and its abandonment tests are specified with a real precedent, the async-effect and `Bodyless` mechanisms match the code, CPU/CUDA Arrow and DLPack evidence ownership is complete, milestone dependencies are annotated, and the DLPack old-signature and callback-close-reentrancy rules are closed. Two non-blocking refinements remain — P4-N1 (define or drop the `device=any` DLPack stream policy, currently unsatisfiable as written) and P4-N2 (generalize the async-interop-effect wording beyond `@python.coroutine`) — both resolvable in this pass with a spec clarification and neither deferring, reducing, or dual-pathing any capability.

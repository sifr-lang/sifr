## Audit against pass-1 blockers

| # | Pass-1 blocker | Resolution | Verdict |
|---|---|---|---|
| 1 | `MustUseKind` split so `cleanup=context` cannot be discharged by return | R1 introduces typed kinds `CloseLike`/`ContextOnly`/`AsyncContextOnly`; `ContextOnly` only discharged by successful `PythonWith` transfer/exit — never return/aggregate | Resolved |
| 2 | `PythonError` manual PartialEq/Eq to avoid `Arc<Replay>` breaking equality | R2 excludes replay from manual impls; public structured fields retain equality | Resolved |
| 3 | `SifrBoundaryError` install site owned by runtime init | R3 puts registration + all sync exit APIs in new `context_ops.rs`, invoked from Python runtime init | Resolved |
| 4 | Sysroot-only `ExitCause`/`ExitDecision` construction + direct `__exit__` call rejection | R4 makes `ExitCause` compiler-only/sysroot-private; `ExitDecision` is compiler-generated from bodyless wrapper truthiness; direct calls to `@python.context.exit` methods rejected — stronger than construction-only gating | Resolved |
| 5 | Closure + body-outcome enum as exclusive codegen shape | R5 mandates closure + explicit body-outcome enum; no fallible cleanup in Drop | Resolved |
| 6 | Secondary evidence routing without changing `Result` payload | R6 appends redacted exit failure to `PythonError.context` for Python primary; ordinary primary uses a runtime cleanup evidence sink exposed only for diagnostics/tests; no `Result` payload changes | Resolved |
| 7 | Module splits in the same PRs that would exceed the 900-line cap | R7 lands `python_interop/context.rs` (W1), `python_error.rs` + `context_ops.rs` (W2), `lower/python_with.rs` (W3), `stmt_support_emitter/python_context.rs` (W4) with the responsibility they add | Resolved |

Cross-check against substrate: 877/874/758-line files confirmed; 62 `HirStmt::With`/`AsyncWith` matcher sites confirmed (W3 must sweep these when adding `PythonWith`); `must_use_obligation_for_type` at `mod_context.rs:216` returns uniform `Option<String>` today, so R1's typed split is a real code change not a documentation touch; `reserved_cleanup(ctx, "context", …)` at `python_interop.rs:337` is the exact activation point W1 flips.

## Wave dependency / independent buildability

- **W1** (types/diagnostics/decorator/signature/entered-policy/`ContextOnly` obligation): standalone. `ExitCause`/`ExitDecision`/`SifrBoundaryError` declared sysroot-private; decorator accepts `cleanup=context` producing `PythonCleanupPolicy::Context`; `ContextOnly` obligation rejects any non-`PythonWith` use. Between W1 and W3, `cleanup=context` values are effectively unusable (any use — including a `with` block still lowering to `HirStmt::With` — fails the obligation). That is the intended "recognized but not yet activated" contract, consistent with the phase's `SIFR-PYRES-0002` reservation model.
- **W2** (replay/runtime/boundary error/exit APIs/evidence sink): standalone. Runtime additions with no compiler callers yet. Manual `PartialEq/Eq` on `PythonError` unblocks existing `object_ops_tests.rs:143` regressions before any consumer wires replay in.
- **W3** (HIR variant + ownership + traversal consumers): depends on W1 (obligation kinds) but not W2. Adds `HirStmt::PythonWith`, updates the 62 matcher sites, marks the entered value as a non-escaping/non-moveable borrow. Codegen for the new variant must gate to a clear "not yet activated" path (reserved-until-W4) — this is the one wave where the reserved-activation contract does real work.
- **W4** (closure-outcome codegen + decision table): depends on W1/W2/W3. Activates. Also fits the new `stmt_support_emitter/python_context.rs` inside its own wave per R7.
- **W5** (matrices + sqlite live demo + combined M5 review + tracker closure): depends on W1–W4.

Each wave has local validation + Opus-to-SATISFIED before merge, and W5 does the combined M5 review — a proper end-of-milestone gate rather than "review once at the end and pray."

No remaining blocking decision. The only implementation risk worth flagging (non-blocking, W3-internal): the reserved-until-W4 gate for `HirStmt::PythonWith` codegen must land as a hard diagnostic, not a silent fallthrough to `With` codegen — otherwise the ownership guarantees stated in W3 would be quietly nullified for two waves. W3's own local validation is the right place to prove that.

SATISFIED

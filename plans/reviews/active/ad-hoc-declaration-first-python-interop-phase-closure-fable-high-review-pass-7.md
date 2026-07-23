All verification is complete and probes cleaned up. Here is the consolidated final report.

---

# Whole-Phase Closure Review — Declaration-First Python Interop (PR #3000), Pass 7

**Fresh read-only review of committed HEAD `21bed5ac4`** (branch `codex/m17-phase-closure`, base `origin/main`, 128 files, +2110/−500), covering the complete M0–M17 diff, all pass-5/pass-6 remediation conclusions, and the final closure unit. Method: full audit of the closure commits (`485749d91` bookkeeping, `21bed5ac4` closure docs — verified docs-only); byte-level cross-check of the plan's gate claims against `target/validation_lane_reports/merge.latest.json`/`.log`/`.time`; independent re-verification of every load-bearing pass-5/pass-6 code claim against source; live compile/run/check probes with a debug compiler current at this tree (run from `/tmp`, removed after); focused unit suites plus the complete 238-test python-feature runtime suite re-run; a scripted resolution check of every relative link in the archived plan and authoritative docs; file-size and HIR-maintainability guardrails re-run. No subagents. No repository file was modified.

## 1. Validation evidence is represented honestly (required check 3)

Every quantitative claim in the plan, roadmap, phase index, and architecture summary was cross-checked against the on-disk artifacts, not the prompt:

- **Gate identity**: `merge.latest.log` stamps `built_by_compiler_commit: 485749d91c47d4dd90ab0673798090f7ffa5711d`; the report finished 11:42, between `485749d91` (10:29) and `21bed5ac4` (11:44). `git show --stat 21bed5ac4` confirms the later commit touches only `internal_docs/architecture.md`, the plan archive move, `plans/phases/index.md`, and `plans/roadmap.md` — closure documentation only, exactly as claimed.
- **Wall time**: `merge.latest.time` reads `4316.43 real` ✓. All **21 lane steps pass** in the JSON.
- **Python interop 25/25**: the JSON's `case_timings` contain exactly 25 `python_interop` cases, all `pass` (self-test, scaffold, env, readonly-check-doctor, binding-authoring, lsp-declaration-authoring, tier1–4, callbacks, six example suites, cloud-boto3, async-declaration/context examples, and three cpython311 compatibility suites) ✓.
- **Codegen 890/890**: the `crate_tests` step's `sifr_codegen` suite reports `890 passed; 0 failed` ✓.
- **Runtime-platform 30/0/3**: `runtime platform verification ok: variants=30, failures=0, blocking_failures=0, skipped=3` (one capability-blocked golden fixture plus two sanitizer-tooling skips, each with a structured reason) ✓.
- **E2E 674/674, signature `1f8b1cadc4f48ec8`**: `[sifr-e2e] report_signature=1f8b1cadc4f48ec8` and `674 pass tests completed (674 passed, 0 failed)` ✓.
- **Hardening 261/0**: `hardening_summary: variants 261, failures 0` ✓.
- The only advisories are the warm wall-time budget and group-skew notice — non-blocking, exactly as the plan states. PR #3000 exists, is OPEN, head `codex/m17-phase-closure`, base `main`.

## 2. Pass-6 IR remediation independently rechecked (required check 4, part 1)

- `plain_call_target_for_ir` (`crates/sifr_codegen/src/stmt_support_emitter/expr_call_metadata.rs:14-20`) splits `::` into `RustExpr::Path`, with its unit test; all five routed sites confirmed at the exact lines pass 6 cited (`plain_call_args.rs:290`, `recursive_exprs.rs:68`, `leaves_and_plain_calls.rs:445`, `await_and_async_comprehension.rs:41`, `iterators_and_callables.rs:397`).
- Strict validation (`ir_validate.rs:257-269`) rejects any non-plain `Ident` and syn-parses `Verbatim` nodes; the former string-smuggling preamble site is now fully structured IR (`io_file_handles.rs` `io_error_kind_expr()` reviewed in the diff).
- **Live probes at this tree**: the three merge-gate fixtures (`classmethod_basic`, `staticmethod_basic`, `cls_constructor`) compile and run with exit 0, and `emit` shows structured `Point::origin()`. A fresh async analogue (async staticmethod awaited plainly, async classmethod inside `async with task.timeout(...)`, and `scope.spawn(Calc.double(5))`) compiles through strict validation and prints `4`, `9`, `Ok(10)` — covering all three routed async/task routes. `890/890` codegen tests re-confirmed via focused runs.

## 3. Pass-5 conclusions and phase-wide claims re-verified (required check 4, part 2)

- **Declaration validation**: nested `@python` defs hard-error at `statement_dispatch.rs:684-692`; `is_python_rooted_decorator_expr` (`stub_syntax.rs:26-34`) recurses Attribute/Call/Subscript. **Live**: `@python(math.sqrt)()` and `@python(math.sqrt)[0]` each fail with exactly one `SIFR-PYCALL-0001`; the plain ellipsis control with `from sifr.python import PythonError` passes `check` with **zero errors** — no over- or under-rejection.
- **Canonical nominal identities**: `cause_variant` (`outcome.rs:3-19`) and `classify_cause_kind` (`sync.rs:732-748`) match only canonical `sifr.builtin.*`/`sifr.parallel.*` identities; all five worker-error synthesis sites stamp `Some("sifr.parallel.{name}")`; `GLOBAL_RUST_NOMINAL_IDENTITIES` additions verified. Focused lowering suites (validation 9/9, canonical-identity 2/2, wrapped-decorator 1/1) pass at HEAD.
- **Callback reentrancy**: `current.rs:175+` copies the target pointer out of the `RefCell` before invoking user code, ending the registry borrow so a handler can create/close another current-thread callback.
- **DLPack ownership**: `attach_finalize` disarms the entry owner before `attach` and re-arms on the loop thread; `argument_capsule` disarms across `store_object` so an attach-failure cannot double-invoke the deleter. Diff reviewed line-by-line; the gate ran all 11 `dlpack_ops::declaration_tests` green.
- **Runtime suite, full width**: I ran the complete python-feature `sifr_runtime` unit suite myself — **238/238 pass** (34.3s), including the new reentrancy test and every callback/context/async/shutdown module.
- **Authoring/certification/LSP**: `normalize_direct_type` (`python_binding.rs:255-315`) strips only the bound module's own prefix and recurses Option/list/tuple/dict; `validate_binding_distributions` fails closed on version drift and every interpreter spawn passes `-I -B`; `hash_runnable_app_entries` remains keyed into the LSP fingerprint (`python_declarations.rs:607,621`). All new `unwrap`/`expect` in the diff are inside `#[cfg(test)]` modules — no new panics in user paths.
- **Hygiene**: file-size guardrail passes globally (2818 files, 900-line limit), HIR maintainability guardrail passes, no TODO/FIXME/`unimplemented!` introduced.

## 4. Closure-document audit (required checks 1–2)

- The plan was renamed `plans/issues/active/` → `plans/issues/archive/ad-hoc-declaration-first-python-interop.md` in `21bed5ac4` (R099); no copy remains at the active path. **All 107 unique relative links in the archived plan resolve** (scripted check). No unchecked `[ ]` remains anywhere in the plan; M17 Wave 4 is `[x]` at line 2457.
- The plan's closure paragraph (lines 2686–2698) states exactly the gate evidence I verified above — commit, time, and all five counts match the artifacts byte-for-byte. Passes 3/4 remain honestly disqualified as non-evidence.
- `plans/roadmap.md:129` and `plans/phases/index.md:55` both now say "complete through PR #3000" and link the **archive** path; `internal_docs/architecture.md:58` adds an accurate closure sentence; `python_interop_declaration_architecture.md` status honestly extends to ecosystem certification; the shutdown-slot wording in `docs/python-interop.mdx` and the protocol doc is precise ("reserved slot, currently no independent registrations"). The updated line-number pointers in `typescript_go_architecture_transfer_guardrails.md` were spot-verified against the actual CLI sources — all land on the described probe sites. PYRES-0002/PYCONV-0001 recode is consistent across registry, generated docs, catalog JSON, and the blessed baseline. Review artifacts pass-1 through pass-6 are committed; the plan's per-pass history matches their contents.
- `third_party/ruff` dirt confirmed **whitespace-only** (a token-identical line join in `parser/expression.rs`) — semantically incapable of affecting the gate; ignored as instructed.

## 5. Non-actionable observations (required check 5 — stated separately, none withhold satisfaction)

1. **Python-feature runtime unit tests are not wired into any gate lane except three filtered cpython311 suites** (buffer release-evidence, arrow, dlpack). The phase-added callback reentrancy test (`current_tests.rs`) therefore runs in no authoritative lane — it passes when run directly (I ran it, and the full 238-test suite, green), and the behavior is additionally exercised by the compiled callback-examples suite. This is the pre-existing lane design shared by ~15 older python-gated test modules, not a phase regression; worth filing as a validation-coverage follow-up.
2. Pre-existing broken link on `origin/main` at `plans/roadmap.md:55` (`verification/areas/stdlib_parity/reports/milestone_psp_7_parity_governance_inventory.md` does not exist) — unrelated Phase 31.5 row, untouched by this diff.
3. `value_place` (`python_interop_direct_helpers.rs:11-15`) has an unreachable empty-`Ident` fallback (`str::split` always yields a first segment) — dead-code polish only; strict IR validation would catch it anyway.
4. PY-1V is marked complete but its plan file still lives in `issues/active/` (links resolve) — pre-existing, different phase.
5. Carried pass-6 notes remain accurate: the pre-existing core-language keyword-method rename gap, `MethodCall.method` turbofish outside `ir_validate` scope (observed unchanged, e.g. `downcast_ref::<std::io::Error>`), and the try/finally divergence — all out-of-phase.
6. The untracked empty `…pass-7.md` is this review's own artifact slot, mirroring the pass-5/6 convention.

## Conclusion

The closure unit is complete and truthful: the plan is archived with every link resolving, M17 Wave 4 and all M0–M17 items are checked, roadmap/index/architecture all point to the archive path with accurate "complete through PR #3000" status, and every number in the recorded gate evidence matches the on-disk merge-gate artifacts at the stated candidate commit. Every pass-5/pass-6 remediation was independently re-verified at source level and by live compile/run/check probes; the full python-feature runtime suite, focused lowering/codegen suites, and both guardrails pass at this tree. No M0–M17 acceptance item or prior review finding remains actionable; the only residual step is the merge of PR #3000 itself, which is the act this review authorizes.

VERDICT: SATISFIED

All verification is complete. Here is the consolidated pass-5 report.

---

# Whole-Phase Closure Review — Declaration-First Python Interop (PR #3000), Pass 5

**Fresh independent review of the committed tree at `c375d48ea`** (branch `codex/m17-phase-closure`, base `origin/main`), covering the complete M0–M17 implementation, the full `origin/main...HEAD` closure diff (122 files, +1891/−483), the phase plan `plans/issues/active/ad-hoc-declaration-first-python-interop.md`, and all four prior whole-phase review artifacts. Method: every prior actionable finding re-verified directly against source, diffs, and tests; focused unit suites re-run at HEAD; wrapped-decorator rejections live-reproduced with a debug compiler built at this exact commit; the authoritative create-PR gate report corroborated on disk. No repository files were modified; all probes ran in `/tmp`.

## Status of the prior review artifacts

Pass 1 (76 lines) and pass 2 (100 lines) are complete reports: pass 1 found B1 + M1–M8, pass 2 verified those closed and found NB-1, NM-1–NM-4, AM-1–AM-10. **Passes 3 and 4 are incomplete interim fragments** (3 and 7 lines) and are correctly recorded as non-satisfaction evidence in the plan (lines 2652–2668); their confirmed content — the NM-2 scope/join-set residual, NF-1 wrapped decorators, and the `RuntimeFault` basename shadow — is what commit `c375d48ea` remediates. Pass-3's un-itemized "six actionable minors" were never persisted; my independent sweep below stands in for them and found no surviving actionable minor.

## Gate corroboration (accepted)

`target/validation_lane_reports/create-pr.latest.json`/`.log` confirm the plan's claims byte-for-byte for this tree (report finished 09:05–09:06, the commit minute of `c375d48ea`): wall `1086.53s`, all 21 lane steps pass including `python_interop` (481.5s), `crate_tests` (208.7s), `core_guardrails` (file-size/maintainability), E2E `131/131` with signature `7c39b8c1dd4fec7c`, runtime-platform zero failures with one declared capability skip, hardening `6/6`. The only advisory is the non-blocking warm wall-time budget. My own runs at HEAD: lowering python-interop **72/72**, codegen python-context **17/17**, and both new `fallback_worker_errors_keep_canonical_parallel_identity` tests pass. The full **merge-profile gate has not yet run at this exact tree** — that is an explicitly pending closure-unit step (see Procedural section), consistent with the plan's own instruction.

## Itemized closure table

### Pass-2 findings (required check 1)

| Finding | Status | Evidence |
|---|---|---|
| **NB-1** nested `@python` silently discarded | **CLOSED** | `statement_dispatch.rs:684-692` hard-errors `SIFR-PYCALL-0001` ("nested Python declarations are not supported…") before any body lowering; test `nested_python_declarations_are_rejected_without_discarding_decorators` covers real-body and ellipsis forms; live probe at HEAD reproduces the rejection. |
| **NM-1** M7 fix broke same-module class-typed binding | **CLOSED** | `python_binding.rs:255-315` `normalize_direct_type` strips the bound module's own prefix (`{module}.`) before the class-name check and normalizes through Option/list/tuple/dict recursion; unit test `normalizes_same_module_qualified_class_annotations_in_scaffolds` (`python_binding.rs:559`); foreign-qualifier rejection (`other.Client`) retained with its e2e fixture (`binding_authoring.py:314-330`). |
| **NM-2** `WorkerRuntimeError` fallbacks with `identity: None` | **CLOSED** (fully, incl. pass-3/4 residual — check 2) | All four synthesis sites stamp `Some("sifr.parallel.{name}")`: `parallel_calls.rs:278`, `task_calls.rs:216` (commit `a803b4ddc`), `task_scope_offload_calls.rs:291`, `task_join_set_calls.rs:471` (commit `c375d48ea`), covering **both `WorkerError` and `WorkerRuntimeError`**; per-file unit tests assert the canonical identity for both names and pass at HEAD. |
| **NM-3** class-level non-opaque python decorators ignored | **CLOSED** | `python_interop.rs:460-495` rejects any python-rooted non-`python.opaque` class decorator with `SIFR-PYCALL-0001`; test `python_class_decorators_require_the_opaque_declaration_form`; live probe (indexed variant) reproduces the rejection. |
| **NM-4** closure record not closure-grade | **CLOSED** | Plan returned to `issues/active/` (archive-status violation gone); PR #3000 linked in Status (line 6), roadmap (`plans/roadmap.md:129`), and phases index (`plans/phases/index.md:55`); the false completeness sentence is replaced by an honest per-pass account; an **explicit executable at-merge instruction** exists at plan lines 2670-2674 (record artifacts/evidence, check Wave 4, flip to `completed`, update PY-2/index/architecture naming PR #3000, archive before merge). |
| **AM-1** missing lockfile-less `certify --check` fixture | **CLOSED** | `binding_authoring.py:24-33` deletes `Cargo.lock`, asserts `SIFR-PACKAGE-0101` on `certify --check` (exit 1), and byte-snapshots non-mutation. |
| **AM-2** `__pycache__` assertion blind to `.venv` symlink | **CLOSED** | `bytecode_snapshot` (`binding_authoring.py:569-576`) calls `.resolve(strict=True)` before `rglob("*.pyc")`, descending the symlinked venv; before/after comparison spans all authoring probes. |
| **AM-3** probe leaf-reduction of foreign scalars/generics | **CLOSED** | `python_binding_probe.py:50-97` matches fully qualified names exactly (`builtins.int`, `typing.List`, …) instead of leaf-stripping; rejection fixtures for `conf.int` and `mymod.List[int]` assert fail-closed plus non-mutation. |
| **AM-4** case-sensitive reserved artifact filename | **CLOSED** | `binding_authoring.rs:202-205` uses `eq_ignore_ascii_case`; fixture drives `--output SIFR.PYTHON-BINDINGS.JSON` to exit 2 with non-mutation; unit coverage in `binding_authoring_tests.rs`. |
| **AM-5** stale diagnostic-evidence artifacts | **CLOSED** | `code_catalog.json` PYCONV-0001 representative fixture repointed to `python_interop_validation_tests.rs`; the blessed compact baseline now renders "unsupported Python resource declaration"; a repo-wide sweep finds no "lowering is not active"/"belongs to a later phase" wording anywhere. |
| **AM-6** DLPack attach-failure double-free path | **CLOSED** | `dlpack_ops/argument.rs` `attach_finalize` disarms the entry owner before `attach` and re-arms inside the loop-thread closure, so a stopped runtime cannot GIL-lessly release while the producer-named capsule still owns the deleter; coverage added in `declaration_tests.rs`. |
| **AM-7** no executable fixture for async truthy-exit/no-replay `PythonError` | **CLOSED** | `aiosqlite_session.sifr` adds `unsuppressible_python_error_case` (Sifr-origin `PythonError` inside truthy-`__aexit__` `async with` must propagate); runner marker extended to `python-error=unsuppressed` and evidence counts 7→8. |
| **AM-8** capture diagnostic names factory `Result[…]`; missing in-`try` test | **CLOSED** | `ownership_diagnostics.rs:119-151` `must_use_resource_type_name` unwraps Result/List/Tuple/Union/Dict to the captured resource; new in-`try` nested-def double-close test asserts `type 'Client'` and absence of `Result[`. |
| **AM-9** Ident-payload IR hygiene half-done | **CLOSED** | `ir_validate.rs:245-269` strictly validates `RustExpr::Ident` as a plain identifier and syn-validates a new `RustExpr::Verbatim`; former smuggle sites (`callback_frame.rs`, `async_context.rs`, io/task/join-set preambles) converted to structured IR (e.g. `io_file_handles.rs` `io_error_kind_expr()`); unit tests reject raw syntax in Ident nodes; E2E signature unchanged (`7c39b8c1dd4fec7c`) evidencing behavior parity. |
| **AM-10** Verification Policy overstatement | **CLOSED** | Policy now reads: value-roundtrip fixtures combine with "dedicated sibling cases that assert the protocol-specific close, drain, cancellation, and reconciliation evidence; the ledger records which cases own each cleanup claim" (plan lines 2729-2733). |

### Pass-3/4 residuals (required checks 2–4)

| Finding | Status | Evidence |
|---|---|---|
| **NM-2 residual** — scope/TaskGroup and join-set fallbacks | **CLOSED** | See NM-2 above: `task_scope_offload_calls.rs:291` and `task_join_set_calls.rs:471` stamp canonical `sifr.parallel.WorkerError`/`sifr.parallel.WorkerRuntimeError`; both unit tests pass at HEAD. |
| **NF-1** — wrapped python-rooted decorators discarded | **CLOSED, live-verified on every surface** | `is_python_rooted_decorator_expr` (`stub_syntax.rs:26-34`) recurses through Attribute/Call/Subscript to the `python` root; `classify_decorator` (`python_interop.rs:822-843`) rejects non-call python-rooted forms and wrapped call targets ("cannot be called, indexed, or accessed after the declaration call"); class path rejects via `collect_python_opaque_declaration`; nested defs hard-error. Test `wrapped_python_rooted_decorators_are_rejected_on_every_declaration_surface` covers `@python(...)()` (module fn + nested def), `@python(...).extra`, `@python.opaque(...)()` (class), `@python(Self.read)()` (opaque method). **My live probes at HEAD**: `@python(math.sqrt)[0]` (indexed), `@python(math.sqrt)()` with a real body, `@python(math.sqrt).extra`, `@python.opaque(...)[0]` on a class, and a nested wrapped form all fail `SIFR-PYCALL-0001`; the plain `@python(math.sqrt)` ellipsis control still passes `check` (no over-rejection). |
| **RuntimeFault shadow** — user class named `RuntimeFault` misclassified | **CLOSED, sync and async** | The `identity: None, name == "RuntimeFault"` arms are removed from both classifiers; `cause_variant` (`outcome.rs:3-19`, used by the async path at `async_context.rs:107`) and `classify_cause_kind` (`sync.rs:732-748`) now match **only** canonical identities. Tests assert user `RuntimeFault` → `OrdinaryError` on both classifiers, user-basename `WorkerRuntimeError`/`TimeoutError`/`CancellationError` → `OrdinaryError`, and canonical `sifr.parallel.WorkerRuntimeError` → `RuntimeFault` (`python_context_tests.rs:276-296`). No other classification source exists (grep-verified), and no remaining site synthesizes canonical worker errors with `identity: None`. |

### Pass-1 findings

All pass-1 findings (B1, M1–M8) were verified closed by pass 2 with live reproduction; I spot-re-verified the load-bearing ones at HEAD (stub-body classifier has no fall-through; `current.rs` copies the target pointer out of the `RefCell` before handler invocation with `current_tests.rs` passing; PYRES-0002 has exactly one emission site — the terminal, documented, fixture-backed bridge-opaque rejection at `python_interop.rs:597`; `certify --check` uses Frozen; all interpreter spawns pass `-B`). Nothing regressed.

## Independent sweep (required check 5)

Beyond finding-closure verification, I swept the full closure diff and the high-risk domains: no new blocker, major, or actionable minor found.

- **Safety/no-panic**: the only new runtime-touching changes (current-callback pointer copy, DLPack disarm/re-arm) shrink panic/double-free surface; no new `unwrap`/`expect`/borrow-across-user-code introduced.
- **Ownership**: must-use capture rejection now spans all four cleanup policies including the in-`try` closure shape, verified by test.
- **Diagnostics**: PYRES-0002 re-key is complete and consistent across registry, generated docs (`docs/errors/SIFR-PYRES-0002.md`), catalog, and the blessed baseline; wrapped/nested/class misuse all land on stable `SIFR-PYCALL-0001`.
- **Authoring non-mutation**: byte-level snapshot fixtures now cover lockfile-less recheck, case-variant reserved outputs, and venv-resolved bytecode.
- **LSP freshness**: `hash_runnable_app_entries` (`python_declarations.rs:607,621`) keys app-entrypoint presence into the fingerprint.
- **Evidence integrity**: gate JSON corroborates every Status claim byte-for-byte; docs' shutdown-phase wording now honestly describes the reserved async-cleanup slot.
- **Docs/status**: roadmap, phases index, and architecture summary are accurate for the pending-closure state; the plan's per-pass history is truthful, including disqualifying passes 3/4 as non-evidence.
- **File-size/IR hygiene**: no touched source file exceeds 900 lines; `ir_validate` now enforces Ident/Verbatim hygiene structurally.

## Pending procedural closure-unit steps (not defects — required check 6)

The plan carries an executable closure instruction (lines 2670-2674, plus Status lines 5-10). Remaining steps, all explicitly planned:

1. Persist this satisfied pass-5 report as the committed review artifact (the untracked empty `…pass-5.md` placeholder is this review's own working file) and record it plus exact merge-gate evidence in the plan.
2. Run the authoritative **merge-profile** gate on the exact final tree (the last full merge gate ran at `453e50eaa`; create-PR gate passes at HEAD).
3. Check M17 Wave 4, flip status to `completed`, update PY-2 roadmap / phase index / architecture summary to name PR #3000 and final evidence, move the plan file to `issues/archive/`, then merge PR #3000.
4. Working-tree hygiene before the exact-tree gate: the `third_party/ruff` submodule carries an uncommitted **whitespace-only** reformat of one line in `parser/expression.rs` (semantically inert) — revert or fold it so the gate tree is exactly the committed candidate.

## Optional / out-of-scope observations (non-blocking — required check 7)

- A user who defines their **own** class literally named `WorkerRuntimeError` in scope and then uses `spawn_cpu` would have genuine worker faults typed as the user class via the `ctx.class_types` lookup and labeled `ordinary-error` at context exits — evidence-label precision only; both causes are Sifr-origin and unsuppressible either way.
- The committed pass-1/pass-2 artifacts each open with a short leftover working-notes preamble line; cosmetic.
- The M17 milestone checkbox (line 241) is `[x]` while Wave 4 (line 2457) is `[ ]`; the Status paragraph explicitly disambiguates (implementation merged through #2999, Wave 4 procedural), and the at-merge instruction resolves it.
- Pre-existing, out-of-phase: the core-language check/build divergence (`return` in `try/except` nested in `try/finally`) recorded by pass 2 remains worth filing separately.
- The NF-1 unit test does not include a subscript-indexed case; the shared recursive helper covers it (live-reproduced above), so this is coverage polish only.

## Conclusion

Every pass-2 finding (NB-1, NM-1–NM-4, AM-1–AM-10) is closed with exact code and test evidence; the pass-3/4 residuals (scope/join-set canonical worker identities, wrapped python-rooted decorator rejection across all five surfaces including indexed variants, and the `RuntimeFault` basename shadow on both sync and async context exits) are closed and live-verified at HEAD; the authoritative create-PR gate passes on this exact tree and the focused suites pass under my own runs. The fresh sweep found no remaining blocker, major, or actionable minor. What remains is exclusively the explicitly planned closure-unit procedure: exact-tree merge gate, bookkeeping flip, archival, and merge.

VERDICT: SATISFIED

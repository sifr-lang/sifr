All verification is complete. Here is the review.

---

# M16 Final Frozen-Ledger Closure Review — Raw API Ergonomics On Shared Ownership (Pass 6)

**Reviewer:** Independent frozen-ledger closure review (agent High, fresh pass)
**Range:** `3f974f33b` → `b88c7e274` (8 commits), PR #2996 — HEAD, local branch, and `origin/codex/m16-python-raw-api-ergonomics` all confirmed at `b88c7e2743898a5be57637dbd0bd93bfd02881d2`, and `gh pr view 2996` confirms the PR is OPEN against `main` with `headRefOid` exactly that hash.
**Scope reviewed:** the complete whole-range diff read end to end (all compiler/runtime/stdlib product files, driver and LSP tests, all four fixtures, demo package, `docs/python-interop.mdx`, both `internal_docs` guardrail inventories, `architecture.md`, `performance_budgets.md`, the full performance remediation, interop verification metadata/runner, and the plan-ledger changes); AGENTS.md; the M15/M16 sections of `plans/issues/active/ad-hoc-declaration-first-python-interop.md` at head; all five prior M16 review artifacts re-read and re-verified, not taken on trust; `target/validation_lane_reports/merge.latest.json`, `.log`, and `.time` audited claim by claim. No files were modified.

## Closure-ledger audit (every claim checked against the pushed head and the validation report)

**Gate provenance and freeze discipline — confirmed.**
- The merge gate finished at 21:36:36 (log/time-file mtime) with `real_seconds = 4410.10`, so it started at ≈ 20:23:06 — eight seconds *after* commit `1a727b1e9` ("record satisfied M16 closure review", 20:22:58), which itself is docs-only (+66/−1: plan status sentence plus the checked-in pass-5 artifact). The gate therefore ran on the tree at `1a727b1e9`.
- The only commit after the gate is `b88c7e274` (21:37:42), and its full diff is exactly one file — the issue-plan ledger (+21/−8): the status paragraph, the M16 milestone checkbox with PR link, the Wave 5 checkbox, and the two merge-profile evidence paragraphs. **No product, test, or verification file changed after the gate ran.** No unreviewed product change exists: relative to the pass-5-satisfied head `403d71020`, the entire remainder of the range is those two docs commits.

**Every merge-profile number in the ledger reproduces from the report/log:**

| Ledger claim | Evidence |
|---|---|
| `4410.10s`, every blocking lane passes | `time.real_seconds = 4410.1`; all 21 `lane_steps` status `pass`; 554/556 case timings pass, 2 skips both non-blocking |
| Python interop `25/25` | report bucket: 25 cases, all pass; log: "python interop verification ok: variants=25, failures=0" |
| LSP `72/72`, package `139/139` | log: `sifr_lsp` suite "72 passed; 0 failed"; `sifr_package` "139 passed; 0 failed" |
| driver `383/383` plus `34/34` generated builds | log line 3779: "383 passed; 0 failed; 34 ignored" (`sifr_driver_lib`), line 3822: "34 passed; 0 failed; 383 filtered out" (`sifr_driver_generated_builds`) |
| representative performance `8/8` | log: "performance verification ok: variants=8, failures=0" |
| runtime-platform `30` variants, three capability/tooling skips | log line 4933: "runtime platform verification ok: variants=30, failures=0, … skipped=3" — 1 capability-blocked golden (`blocked_until_capabilities=text_i18n_core_unicode,network_http_async_network`) + 2 sanitizer tooling skips (missing `llvm-symbolizer`/nightly). "Capability/tooling" is exact |
| E2E `674/674`, signature `1f8b1cadc4f48ec8` | log line 5032: "674 pass tests completed (674 passed, 0 failed)"; line 5031: "report_signature=1f8b1cadc4f48ec8" |
| hardening `261` variants, zero failures | `hardening_summary`: variants 261, failures 0, blocking_failures 0, skipped 0 |
| cold E2E cache `0/178` | `e2e.cache_hits = 0`, `groups = 178` |
| advisories non-blocking | `advisories` = exactly ["warm wall-time budget exceeded", "group skew is high…"]; `within_warm_budget: false` is the budget advisory; no blocking failure anywhere |

The second M16 paragraph's shorthand "cold-cache wall-time … non-blocking advisories" is a fair compression — the fuller paragraph immediately above states the advisory set precisely (`0/178` cold cache, aggregate warm wall-time, group skew). Not an overclaim.

**Critically, both M16 end-to-end acceptance evidences executed green *inside* the authoritative gate itself** (not only in prior local runs): `typed_raw_api_builds_runs_and_releases_ordinary_objects` and `raw_coroutine_api_builds_and_runs_on_the_owned_loop` appear as `ok` in the `sifr_driver_generated_builds` pass (log lines 3795, 3799), building and running native CPython binaries with the `live_objects` equality and `:released` assertions.

**Ledger text audit — no overclaim, no stale pending text, no missing documentation.**
- Status paragraph ("M16 is implemented, authoritatively validated, repeatedly reviewed to satisfaction, and closure-approved on PR #2996. M17 is not yet implemented."), the `[x]` M16 milestone entry with PR link, and the `[x]` Wave 5 entry are each exactly supported by the evidence above and by passes 2 and 5 returning SATISFIED. The review-history narrative (pass 1 two minors → pass 2 satisfied → merge-gate benchmark defect → pass 3 two majors → pass 4 two minors → pass 5 satisfied) matches the artifacts verbatim. All previous "remain before Wave 5 can close" sentences were replaced in `b88c7e274`; the only remaining "pending" strings in the plan sit inside M10's historical chronology, each immediately resolved by the following sentence. Per the M15 precedent (`afa4c9686` → `841d0641e`), recording *this* frozen-ledger pass is a follow-up docs commit after the review — its absence from the frozen ledger is expected, not a defect.

## Prior-findings independent re-verification (all five passes)

- **Pass-1 MINOR-1/MINOR-2:** closed. Whole-range diff shows both async fixtures use canonical `await task.sleep(0.0)` with no bogus import, and `internal_docs/architecture.md:54` carries the full M16 contract sentences (read in the diff and at head).
- **Pass-3 MAJOR-1/MAJOR-2:** closed. The whole-range diff contains **zero** changes to `crates/sifr_lsp/src/python_declarations.rs` — the anti-conservative gate cannot exist at head — and both discriminating regressions are present in the diff (`workspace_member_python_requirements_are_validated` now writes a pure root `sifr.toml`; new `misplaced_bridge_root_is_reported_for_an_otherwise_pure_package` pins `SIFR-PYIMP-0002`). Both ran green inside the gate's `sifr_lsp` 72/72.
- **Pass-4 MINOR-1/MINOR-2:** closed. `benchmark_source` (`lsp_query_bench.py:283-289`) short-circuits `lsp.did_open_diagnostics` before any temp package is created while mode validation still fires first; `internal_docs/performance_budgets.md:83-91` documents the required `workspace_mode` contract, the two-way manifest cross-check, the isolated temp-package semantics, and the didOpen exception with rationale. The ledger's qualified isolation sentence matches the shipped code exactly.

## Implementation and acceptance criteria (independently reassessed from the complete diff)

- **"Raw ergonomics improve without a second ownership or conversion model."** Holds. The three intrinsics validate exclusively through the declaration predicate `is_direct_type` (`direct_validation.rs:1001-1030` of the diff; visibility widened, predicate unchanged); codegen (`python_raw_api_codegen.rs`) generates only through the existing declaration converters `input_conversion`/`output_value_expr` and the existing `__sifr_declaration_object_argument`/`__sifr_declaration_object_result` helpers; the registry entries are `(None, None)` so no alternate lowering exists. The four raw `Object` methods are a closed compiler-known set gated on `is_python_object_contract()`, hard-requiring the five-field `PythonError` contract, lowered as `FunctionType::all_borrow` (borrowed receivers/arguments → ordinary automatic drop), and routed to `py_call_keyed`/`py_call_attr_keyed`, which delegate to the same `python::call_object`/`call_attr` over the same tracked store. `kwarg` erases only to the pre-existing `(str, Object)` tuple. No new runtime type, handle field, or parallel converter anywhere in the diff.
- **"Raw coroutine execution uses the owned loop and no per-call event loop remains."** Holds. `run_coroutine_blocking` is untouched product-side and routes through `async_runtime::ensure_started`; the diff *adds* owned-loop failure coverage (`raw_coroutine_python_failure_returns_checked_error_on_owned_loop`) and the package test asserts loop identity plus automatic release, executed green inside the merge gate. Success/failure/concurrency/cancellation/shutdown coverage confirmed in the diff and gate log.
- Supporting fixes are root-cause, not shims: the `record_field` dict-branch fix makes mapping keys win over dict attribute names with a targeted regression; the Result-ok type-var fallback and return-position contextual typing are general guarded inference improvements exercised by the full E2E gate. The `methods_lambdas_and_comprehensions.rs` refactor into `method_call_arguments.rs` is behavior-preserving extraction (verified line by line) that *reduced* the file to 864 lines. All touched hand-maintained files are under the 900-line cap (largest: 864). Guardrail inventories cover exactly the three intrinsics and both keyed adapters; capability metadata upgrades raw-api evidence to concrete passing owners including cancellation; docs and demo match shipped behavior.

## Validation/inspection actually performed (this pass)

Read the complete `3f974f33b..b88c7e274` diff (61 files, +1661/−103) and the surrounding implementation at head; audited `merge.latest.json` (all top-level fields, all 556 case timings, both skips), `merge.latest.log` (crate counts, runtime-platform variant/skip lines, E2E signature and totals, both native package tests), and `merge.latest.time`; reconciled every ledger number as tabulated above; verified gate/commit timing from file mtimes and commit timestamps; verified `b88c7e274` is ledger-only and `1a727b1e9` docs-only; confirmed branch, origin tip, and PR #2996 head OID all equal `b88c7e274`; recomputed `sha256(benchmark_manifest.json)` = `f2e180e4…` matching `trend/current.json`; ran the file-size guardrail check on all touched files; and independently compiled all four fixtures with the existing debug compiler at head — `raw_typed_ergonomics.sifr` clean, and each fail fixture emitting exactly one diagnostic with the annotated code (`SIFR-PYCONV-0001` naming `set[int]`; `SIFR-ASYNC-0003` for `from_value` and for `Object.get_attr`). I did not re-run the heavy native suites myself; both executed green inside the audited authoritative gate at the product-identical tree, which is the stronger evidence.

## Findings

None. No BLOCKER, MAJOR, or MINOR findings.

## Non-findings (observations, no action required)

- Working-tree bookkeeping outside the frozen diff: an empty untracked pass-6 artifact placeholder (`plans/reviews/active/…-pass-6.md`) and a content-dirty `third_party/ruff` submodule (gitlink unchanged across the range). Per the freeze discipline I left both untouched; recording this pass-6 result in the ledger is the established follow-up docs commit (M15 precedent).
- The ledger's "all Python declaration LSP tests pass 24/24" is the declaration suites proper; the filter matches 26 including 2 pre-existing fingerprint unit tests — a scoping nuance passes 4–5 already documented, not an evidence error.

## Verdict

The frozen ledger is exact: every count, timing, signature, skip, and advisory statement reproduces from the authoritative merge report and log; the gate demonstrably ran on the tree at `1a727b1e9`; the sole post-gate commit is the ledger update itself; all seven prior findings across five review passes are independently confirmed closed at the pushed head; both M16 acceptance criteria hold under fresh whole-diff inspection with the decisive end-to-end evidence executed inside the gate; and PR #2996 points at exactly the reviewed head.

VERDICT: SATISFIED

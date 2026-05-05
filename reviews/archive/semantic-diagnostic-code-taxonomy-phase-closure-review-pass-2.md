# Phase-Closure Review (Pass 2): Ad-Hoc Semantic Diagnostic Code Taxonomy and Structured HIR Diagnostics

**Phase:** 31.7
**Review date:** 2026-05-04
**Branch:** `main` (post-merge full-phase review)
**Closure commits:** `41592db6` (Close semantic diagnostic taxonomy phase, PR #1785), `fb45ddb1` (Add issue and review artifacts, PR #1786)
**Pass-1 reviewer artifact:** `reviews/semantic-diagnostic-code-taxonomy-phase-closure-review-pass-1.md`

---

## TL;DR

The structural goals of the phase landed well: raw `ctx.error(...)` HIR transport is gone, `LoweringError` is renamed and guarded, the registry/docs/schema/docs URL machinery is wired and synced, and six guardrail scripts run clean and are wired into `scripts/run_all_tests.sh`. The end-to-end picture matches the phase's "Phase Definition of Done" except for the issues called out below.

However, this pass uncovers two blocking findings the pass-1 review did not catch:

1. **Compiler panic on user input.** Three keyword-argument call patterns introduced during this phase's HIR migration crash the compiler with `index out of bounds` instead of emitting a structured diagnostic. One has a checked-in unit test that is **currently failing on `main`**.
2. **Authoritative validation gate does not exercise `sifr_hir` lib tests.** `scripts/run_all_tests.sh` (the documented merge gate) never runs `cargo test -p sifr_hir`, which is where the new HIR diagnostic regression tests live (~414 tests). This is the structural reason finding (1) made it through the closure run.

A few smaller doc-drift items round out the findings.

---

## What's correct on `main`

These contract goals verify clean against the current tree:

- **Raw HIR error transport is gone.** `rg -n "ctx\.error\(" crates/sifr_hir/src -g '*.rs'` returns no matches. `LowerCtx::error(String)` no longer exists.
- **`LoweringError`, `TypeError`, `TypeErrorKind` symbols are deleted.** `rg -n "LoweringError|TypeErrorKind|sifr_type_system::TypeError" crates --type rust` finds matches only inside [scripts/check_diagnostic_transport_cleanup.py](scripts/check_diagnostic_transport_cleanup.py) (the guardrail itself) and historical issue text.
- **`SIFR-TYPE-0001` is not active and not emitted as a catch-all.** Registry has no active entry for it, and `scripts/check_diagnostic_baseline_hygiene.py` rejects it from fail fixtures and verification baselines.
- **All six diagnostic guardrails pass and are wired into `scripts/run_all_tests.sh`** at lines 99–115.
  - `check_diagnostic_schema_sync.py`
  - `check_diagnostic_docs_sync.py`
  - `check_diagnostic_code_coverage.py`
  - `check_diagnostic_baseline_hygiene.py`
  - `check_diagnostic_cancel_usage.py`
  - `check_diagnostic_transport_cleanup.py`
- **Registry / docs / active codes are aligned.** `crates/sifr_diagnostics/src/codes.rs` has 99 `active_entry!` rows, 99 `pub const ...: Self = Self::new("SIFR-...")` `DiagnosticCode` constants, and `docs/errors/` carries 99 per-code pages plus the index page. `cargo run -q -p sifr_diagnostics --bin gen-error-docs -- --check` runs clean.
- **HIR maintainability guardrails pass.** `python3 scripts/check_hir_maintainability_guardrails.py` reports `PASS` against the 82 files now under `crates/sifr_hir/src/lower/`.
- **Roadmap/architecture docs updated.** `internal_docs/roadmap.md:57` records phase 31.7 as `completed` on 2026-05-03 with a summary that matches the implemented contract. `internal_docs/architecture.md:709` and `internal_docs/phases/27_diagnostics_error_recovery_and_stability_contract.md:14` both record the corrective amendment.
- **`SIFR-INTERNAL-0002` recovery-cap omission summaries** and **`SIFR-TYPE-0902` reveal-type notes** / **`SIFR-TYPE-0901`/`SIFR-FLOW-0901` warnings** are populated as registry entries with docs, matching `milestone_diag_10` DoD.

---

## Blocking findings

### Finding 1 — Compiler panics on three keyword-argument method patterns (correctness regression)

The HIR migration of method-call diagnostics introduced direct `arg_ranges[N]` indexing in argument-validation paths but did not thread keyword-argument source ranges into `resolved_method_arg_ranges`. When a method accepts a keyword that `normalize_*_method_args` pushes into the normalized `args` vector, the validator's `args.first()` / `args.get(N)` returns `Some(...)` while `arg_ranges` stays shorter than `args`, so the indexing panics.

Reproductions on `main`, all from a `cargo run -q -p sifr -- check` invocation against tiny `.sifr` files:

| User input | Crash location | Symptom |
| --- | --- | --- |
| `nums.sort(reverse=1)` | [crates/sifr_hir/src/lower/expressions.rs:2430](crates/sifr_hir/src/lower/expressions.rs:2430) | `panicked at ... 'index out of bounds: the len is 0 but the index is 0'` |
| `d.get(0, default="bad")` (with `d: dict[int, int]`) | [crates/sifr_hir/src/lower/expressions.rs:2687](crates/sifr_hir/src/lower/expressions.rs:2687) | `panicked at ... 'index out of bounds: the len is 1 but the index is 1'` |
| `d.setdefault(0, default="bad")` | [crates/sifr_hir/src/lower/expressions.rs:2772](crates/sifr_hir/src/lower/expressions.rs:2772) | `panicked at ... 'index out of bounds: the len is 1 but the index is 1'` |

In every case, the CLI prints `internal compiler error: internal compiler panic during check command execution: index out of bounds...` to stderr — a user-visible compiler panic on a small valid-syntax input that should produce a structured `SIFR-TYPE-0002` type-mismatch diagnostic.

**Why this matters.** The phase preamble explicitly states (in `AGENTS.md` and the issue) that `if it compiles, it works` and that there are no user-triggerable runtime panics. These panics happen in the compiler itself on legal input, on the very ranged-emission paths this phase introduced.

**Root cause.** [`resolved_method_arg_ranges`](crates/sifr_hir/src/lower/method_call_args.rs:53) at `crates/sifr_hir/src/lower/method_call_args.rs:53-103` only special-cases `dict.update`, `str.split`, `str.replace` for keyword-range threading. The list `sort` (`reverse=`), dict `get` / `setdefault` (`default=`), and (via `normalize_dict_method_args`) dict `pop` (`default=`) keyword paths are not covered. Meanwhile `normalize_list_method_args` at lines 449-468 and `normalize_dict_method_args` at lines 470+ push the keyword's value into `args`, so the validator believes it has an arg and indexes `arg_ranges[i]` blindly.

**Failing test on main.** `cargo test -p sifr_hir -- test_list_sort_rejects_non_bool_reverse_keyword`:

```
thread 'lower::expressions_tests::test_list_sort_rejects_non_bool_reverse_keyword'
panicked at crates/sifr_hir/src/lower/expressions.rs:2430:29:
index out of bounds: the len is 0 but the index is 0
test result: FAILED. 0 passed; 1 failed; 0 ignored
```

The test is at [crates/sifr_hir/src/lower/expressions_tests.rs:3181](crates/sifr_hir/src/lower/expressions_tests.rs:3181). It was added by commit `2d8bfb830` (2026-04-07, pre-phase). The migration commit `76727af7` ("Migrate list method diagnostics", PR #1779, 2026-05-03) replaced the prior `ctx.error(...)` call with `expression_diagnostics::type_mismatch(ctx, msg, arg_ranges[0])` without extending `resolved_method_arg_ranges` to thread the `reverse=` keyword range. The migration's new tests (e.g., `test_list_method_type_mismatch_has_type_code`) all use positional args, so the keyword regression fell through.

The other two crashes (`dict.get` / `dict.setdefault` with `default=`) appear to have been introduced by `879f9ae7` ("Migrate dict method diagnostics", PR #1780). They have no direct unit-test coverage today.

**Recommended remediation.**
1. Extend [`resolved_method_arg_ranges`](crates/sifr_hir/src/lower/method_call_args.rs:53) so list `sort` (`reverse=`), dict `get` / `pop` / `setdefault` (`default=`) thread the keyword's `value.range()` into the returned `Vec<TextRange>` in the same slot the normalizer fills in `args`.
2. Add HIR regression tests for the two uncovered patterns (`d.get(key, default=<wrong-type>)`, `d.setdefault(key, default=<wrong-type>)`).
3. After fixing, re-grep all 29 `arg_ranges\[` sites in `crates/sifr_hir/src/lower/` for the same shape — direct indexing past arity guards — and assert one of: (a) the prior arity check guarantees the index, or (b) the normalizer threads the corresponding keyword range. The current count comes from `rg -n "arg_ranges\[" crates/sifr_hir/src --type rust | wc -l`.

### Finding 2 — `scripts/run_all_tests.sh` does not run `sifr_hir` lib tests

This is the structural reason Finding 1 slipped through closure validation.

`scripts/run_all_tests.sh` lines 120-127 run only:

```
cargo test -p sifr_diagnostics
cargo test -p sifr -- --skip test_e2e_pass
cargo test -p sifr_driver --lib
```

There is no `cargo test -p sifr_hir`. The HIR crate hosts ~414 lib tests including the new diagnostic-code/range regression tests every `milestone_diag_*` slice added (`diagnostic_transport_tests`, `expressions_tests`, `match_diagnostics_tests`, `name_import_diagnostics_tests`, `protocol_diagnostics`, `result_diagnostics_tests`, `statement_diagnostics_tests`, `type_alias_tests`, etc.). None of these guard `main`.

The phase's own validation evidence repeatedly cites `cargo test -p sifr_hir -- --skip test_e2e_pass` as part of slice-level validation, but the closure entry at issue line 45 says "full `scripts/run_all_tests.sh`" — and that script does not include the HIR tests. So a regression that fails a HIR unit test passes the documented merge gate.

The phase issue's `Hard Rules` and `Required guardrails` (issue lines 1359-1392) speak of decidable enforcement and full local validation passing. The intent is that the merge gate exercises the HIR regression tests this phase invested heavily in. The script does not match that intent today.

**Recommended remediation.** Either:
- Add `cargo test -p sifr_hir` (with the same `--skip test_e2e_pass` filter pattern used for `sifr`) to `scripts/run_all_tests.sh`, alongside the existing `sifr_diagnostics` / `sifr` / `sifr_driver` invocations; or
- Replace those four targeted invocations with `cargo test --workspace -- --skip test_e2e_pass` so future crates are auto-included.

Either fix surfaces Finding 1 immediately on PR.

---

## Non-blocking findings (documentation drift)

These are not regressions but they make `internal_docs/diagnostic_emission_inventory.md` an unreliable reference now that the phase has closed.

### Finding 3 — Inventory still lists removed codes as planned active codes

`internal_docs/diagnostic_emission_inventory.md:348` and `:350` still list:

| Code | Status referenced |
| --- | --- |
| `SIFR-STDLIB-0002` | "stdlib method/argument type mismatch ... stdlib wrong type/count fixtures" |
| `SIFR-CODEGEN-0002` | "codegen panic boundary/internal backend failure ... panic boundary tests" |

Both registry entries were intentionally removed by `milestone_diag_11` guardrail-audit slice (PR #1753) because they had no non-test compiler emission path. The inventory's headline "Closure snapshot from May 3, 2026" at lines 12-16 was updated, but the bottom-of-file "Target Code And Fixture Plan" was not. A future contributor reading the inventory would think these are active codes to emit against.

Recommended fix: either drop the two rows or annotate them with the same "Removed in `milestone_diag_11` guardrail audit; no compiler emission path" note used for `SIFR-PARSE-0001` at line 143.

### Finding 4 — Inventory missing `SIFR-TYPE-0010`

The active registry has `SIFR-TYPE-0010` (TypeVar constraint not satisfied), with docs page [docs/errors/SIFR-TYPE-0010.md](docs/errors/SIFR-TYPE-0010.md) and fixture [crates/sifr/tests/e2e/fail/typevar_constraints_violation.sifr](crates/sifr/tests/e2e/fail/typevar_constraints_violation.sifr). The "Target Code And Fixture Plan" table in the inventory lists `SIFR-TYPE-0008`, `SIFR-TYPE-0009`, then jumps to `SIFR-TYPE-0011` — there is no row for `0010`. Same pattern: the inventory's bottom-of-file plan was not synced when the registry was finalized.

### Finding 5 — Phase issue body retains aspirational phrasing

`issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1041` says

> `LoweringError` becomes private transitional plumbing only, is removed from user-facing paths in `milestone_diag_4a`, and is fully deleted by residual cleanup in `milestone_diag_11`.

The deletion happened (good) but the phrasing reads as plan, not state. Combined with other "should/will" statements throughout the body (e.g., `:822` "This inventory is not a compatibility table"), the issue file is hard to use as a closure record. Consider a short "Phase Closure Summary" preamble at the top above `Execution Status` linking to the closure review and stating the contract is satisfied — the `Execution Status` checklist is comprehensive but expects the reader to scan ~250 line items to reconstruct that.

This is editorial and lower priority than 3 and 4.

---

## Verification commands run for this review

All run from repo root on `main` at `fb45ddb1`:

```
git status --short --branch                                  # clean
rg -n "ctx\.error\(" crates/sifr_hir/src -g'*.rs'            # no matches
rg -n "LoweringError|TypeErrorKind|sifr_type_system::TypeError" crates --type rust  # only in guardrail script
python3 scripts/check_diagnostic_transport_cleanup.py        # exit 0, no output
python3 scripts/check_diagnostic_code_coverage.py            # exit 0
python3 scripts/check_diagnostic_baseline_hygiene.py         # exit 0
python3 scripts/check_diagnostic_cancel_usage.py             # exit 0
python3 scripts/check_diagnostic_schema_sync.py              # exit 0
python3 scripts/check_diagnostic_docs_sync.py                # exit 0
python3 scripts/check_hir_maintainability_guardrails.py      # PASS
cargo test -p sifr_diagnostics --lib                         # 31 passed
cargo run -q -p sifr_diagnostics --bin gen-error-docs -- --check  # exit 0, no output
cargo test -p sifr_driver --lib                              # 113 passed
cargo test -p sifr_hir --lib                                 # 413 passed; 1 failed (test_list_sort_rejects_non_bool_reverse_keyword)
cargo run -q -p sifr -- check /tmp/sort_reverse_kw.sifr      # internal compiler panic (Finding 1)
cargo run -q -p sifr -- check /tmp/dict_get_kw.sifr          # internal compiler panic (Finding 1)
cargo run -q -p sifr -- check /tmp/dict_sd_kw.sifr           # internal compiler panic (Finding 1)
cargo run -q -p sifr -- check /tmp/sort_pos.sifr             # clean diagnostic (positional path works)
```

---

## Summary

The structural deliverables of the phase — registry, schema, docs, ranged HIR diagnostics, retired transport symbols, guardrail scripts, ordering policy, recovery-cap summaries, structured warnings/notes — are present and consistent on `main`. The pass-1 reviewer's contract verification holds.

Two findings block calling the closure complete:

1. The compiler panics on three reachable user-input patterns introduced by this phase's migration of list/dict method diagnostics, with one currently failing checked-in unit test (`test_list_sort_rejects_non_bool_reverse_keyword`). Root cause is in [crates/sifr_hir/src/lower/method_call_args.rs:53](crates/sifr_hir/src/lower/method_call_args.rs:53) failing to thread `list.sort(reverse=)`, `dict.get(default=)`, and `dict.setdefault(default=)` keyword ranges into `resolved_method_arg_ranges`.
2. The authoritative `scripts/run_all_tests.sh` gate does not run `cargo test -p sifr_hir`, leaving ~414 HIR regression tests — most of them added by this phase — unguarded against `main`. This is the structural cause of (1) reaching closure undetected.

Findings 3-5 are documentation drift and editorial; they should be cleaned up but do not change runtime correctness.

Recommend: re-open the phase or open a fast follow-up to land the `resolved_method_arg_ranges` fix, the two missing keyword-default regression tests, and the `sifr_hir` invocation in `scripts/run_all_tests.sh`. The inventory and issue-file edits can land in the same PR or a separate doc-only PR.

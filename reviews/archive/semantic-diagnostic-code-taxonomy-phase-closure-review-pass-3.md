# Phase-Closure Review (Pass 3): Ad-Hoc Semantic Diagnostic Code Taxonomy and Structured HIR Diagnostics

**Phase:** 31.7
**Review date:** 2026-05-05
**Branch:** `codex/diag-phase-review-fixes`
**Base commit:** `fb45ddb1` (same as `main`; all changes are uncommitted working-tree edits)
**Pass-2 reviewer artifact:** [reviews/semantic-diagnostic-code-taxonomy-phase-closure-review-pass-2.md](reviews/semantic-diagnostic-code-taxonomy-phase-closure-review-pass-2.md)

---

## TL;DR

All pass-2 blocking findings and documentation-drift items are resolved. The compiler no longer panics on the four keyword-default method patterns called out in pass-2; the authoritative validation gate now exercises `sifr_hir` lib tests; and the inventory + issue-file drift is cleaned up. I have no blocking findings. The diff is small (5 files, +88/-7) and surgical — no scope creep, no collateral edits.

Recommend landing as-is.

---

## Findings status

### Pass-2 Finding 1 — Compiler panics on keyword-default method patterns

**Status: Fixed.**

The fix in [crates/sifr_hir/src/lower/method_call_args.rs:60-83](crates/sifr_hir/src/lower/method_call_args.rs:60) adds two new arms to `resolved_method_arg_ranges` so the keyword value range is threaded into the same slot the normalizer fills in `args`:

- `Type::List(_) if method == "sort"` pushes `reverse.value.range()` when `ranges.is_empty()` — mirrors `normalize_list_method_args` at [crates/sifr_hir/src/lower/method_call_args.rs:480-489](crates/sifr_hir/src/lower/method_call_args.rs:480), which only pushes the keyword into `args` when `args.is_empty()`.
- `Type::Dict(_, _) if matches!(method, "get" | "pop" | "setdefault")` pushes `default.value.range()` when `ranges.len() == 1` — mirrors `normalize_dict_method_args` at [crates/sifr_hir/src/lower/method_call_args.rs:500-512](crates/sifr_hir/src/lower/method_call_args.rs:500), which only pushes the keyword into `args` when `args.len() == 1`.

Both predicates exactly track the normalizer's invariants, so `args.len()` and `ranges.len()` stay in lockstep on every reachable path. I checked for the inverse (range pushed but normalizer rejects) and found none — when the normalizer returns `duplicate_argument_error` (e.g., `list.sort(False, reverse=True)`), the lowering returns `None` before `resolved_method_arg_ranges` is consulted in the validator, so the over-pushed range is never indexed.

The four pass-2 CLI repros now emit structured diagnostics:

```
$ cargo run -q -p sifr -- check /tmp/test_review_sort_kw.sifr
type error: list.sort() argument 'reverse' must be 'bool', got 'int'
exit=1
```

Same shape for `dict.get(0, default="bad")`, `dict.pop(0, default="bad")`, `dict.setdefault(0, default="bad")` — all return exit 1 with `type error: ...`, no panic.

The HIR regression tests in [crates/sifr_hir/src/lower/expressions_tests.rs:3181-3422](crates/sifr_hir/src/lower/expressions_tests.rs:3181) lock this in. The existing `test_list_sort_rejects_non_bool_reverse_keyword` is strengthened from a message-only check to also assert `code == TYPE_MISMATCH` and `primary_range == range_for_after_anchor(source, "reverse=", "1")`. Three new tests cover `dict.get`, `dict.pop`, and `dict.setdefault` with the same code+range assertions. All four pass:

```
test test_list_sort_rejects_non_bool_reverse_keyword ... ok
test test_dict_get_default_keyword_type_mismatch_has_type_code_and_range ... ok
test test_dict_pop_default_keyword_type_mismatch_has_type_code_and_range ... ok
test test_dict_setdefault_keyword_type_mismatch_has_type_code_and_range ... ok
```

**Residual `arg_ranges[N]` audit.** I re-grepped all 29 `arg_ranges\[` sites in `crates/sifr_hir/src/lower/`. Every one is one of:

- Gated by an explicit arity check that bounds `args.len()` (e.g., dict.update at [expressions.rs:2599-2615](crates/sifr_hir/src/lower/expressions.rs:2599), set.union at [expressions.rs:2851-2853](crates/sifr_hir/src/lower/expressions.rs:2851)) **and** `resolved_method_arg_ranges` either accepts only positional args for that method or has a matching keyword-threading arm.
- Reached only after a normalizer that rejects keywords entirely (e.g., decimal methods at [decimal_methods.rs:285-312](crates/sifr_hir/src/lower/decimal_methods.rs:285) — `lower_method_call_args` routes non-collection types through `reject_remaining_keywords`, so `args == positional` and lengths match by construction).
- Already using the safe `arg_ranges.get(i).copied().unwrap_or(method_range)` pattern (e.g., list.index at [expressions.rs:2551](crates/sifr_hir/src/lower/expressions.rs:2551), tuple.index at [expressions.rs:3096](crates/sifr_hir/src/lower/expressions.rs:3096)).

No remaining direct-indexing site can be reached with `arg_ranges` shorter than `args`.

### Pass-2 Finding 2 — `scripts/run_all_tests.sh` doesn't run `sifr_hir` lib tests

**Status: Fixed.**

[scripts/run_all_tests.sh:123-124](scripts/run_all_tests.sh:123) adds `cargo test -p sifr_hir -- --skip test_e2e_pass`, placed alongside the existing `sifr_diagnostics`, `sifr`, and `sifr_driver` invocations. The `--skip test_e2e_pass` filter is defensive — `sifr_hir` has no `test_e2e_pass`-named test today (verified via `grep -rn 'test_e2e_pass\|fn test_e2e' crates/sifr_hir/`) — but matches the convention used for the `sifr` invocation and prevents accidental future coupling.

This is the targeted fix (option (a) from pass-2's recommendation). The alternative — replacing the four individual invocations with `cargo test --workspace -- --skip test_e2e_pass` — would have been broader and risked pulling in slow integration tests from other crates. The targeted choice is appropriate given the script's existing structure.

Validation evidence: `cargo test -p sifr_hir --lib -- --skip test_e2e_pass` reports `417 passed; 0 failed; 1 ignored`, matching the user-reported numbers and meaning a future regression like pass-2's `test_list_sort_rejects_non_bool_reverse_keyword` failure would now block the merge gate immediately.

### Pass-2 Finding 3 — Inventory still lists removed codes as planned active codes

**Status: Fixed.**

[internal_docs/diagnostic_emission_inventory.md:349-350](internal_docs/diagnostic_emission_inventory.md:349) replaces the descriptions for `SIFR-STDLIB-0002` and `SIFR-CODEGEN-0002` with `removed in milestone_diag_11 guardrail audit; no compiler emission path` and `n/a` for the column-3 / column-4 cells. This matches the existing annotation pattern used for `SIFR-PARSE-0001` at line 143. Verified the registry source has no `active_entry!` for either code (`grep 'SIFR-(STDLIB-0002|CODEGEN-0002)' crates/sifr_diagnostics/src/codes.rs` → no matches) and no `docs/errors/SIFR-STDLIB-0002.md` / `docs/errors/SIFR-CODEGEN-0002.md` exists.

### Pass-2 Finding 4 — Inventory missing `SIFR-TYPE-0010`

**Status: Fixed.**

[internal_docs/diagnostic_emission_inventory.md:308](internal_docs/diagnostic_emission_inventory.md:308) inserts the missing row between `SIFR-TYPE-0009` and `SIFR-TYPE-0011`:

```
| `SIFR-TYPE-0010` | TypeVar constraint not satisfied by inferred concrete type | type variable constraint checking | `crates/sifr/tests/e2e/fail/typevar_constraints_violation.sifr` |
```

Verified the registry has `SIFR-TYPE-0010` (`grep 'SIFR-TYPE-0010' crates/sifr_diagnostics/src/codes.rs` returns lines 42 and 680), the docs page exists at `docs/errors/SIFR-TYPE-0010.md`, and the referenced fixture exists.

### Pass-2 Finding 5 — Phase issue body retains aspirational phrasing

**Status: Editorially addressed.**

[issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:9-11](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:9) adds a "Phase Closure Summary" preamble that frames phase 31.7 as closed on 2026-05-03, names the post-closure review pass that found the hardening blockers, and points to the follow-up slice. The new execution-status entry at line 17 records the hardening slice with full local validation evidence, including the `report_signature` and `wall_time` from `scripts/run_all_tests.sh --profile quick`. The remaining "should/will" language elsewhere in the issue body is left intact — the new preamble plus checklist log line gives a reader the closure context they need without a wholesale rewrite, which matches pass-2's "lower priority, editorial" framing.

---

## What I verified independently

```
git diff --stat main                                         # 5 files, +88/-7
cargo test -p sifr_hir --lib -- \
    test_list_sort_rejects_non_bool_reverse_keyword \
    test_dict_get_default_keyword_type_mismatch_has_type_code_and_range \
    test_dict_pop_default_keyword_type_mismatch_has_type_code_and_range \
    test_dict_setdefault_keyword_type_mismatch_has_type_code_and_range
                                                              # 4 passed
cargo test -p sifr_hir --lib -- --skip test_e2e_pass         # 417 passed; 1 ignored
cargo run -q -p sifr -- check /tmp/test_review_sort_kw.sifr   # structured type error, exit 1
cargo run -q -p sifr -- check /tmp/test_review_dict_get_kw.sifr   # structured type error
cargo run -q -p sifr -- check /tmp/test_review_dict_pop_kw.sifr   # structured type error
cargo run -q -p sifr -- check /tmp/test_review_dict_sd_kw.sifr    # structured type error
grep -rn 'arg_ranges\[' crates/sifr_hir/src/lower/ | wc -l    # 29 — audited each site
ls docs/errors/SIFR-{STDLIB-0002,CODEGEN-0002,TYPE-0010}.md   # only TYPE-0010 exists, as intended
grep -nE 'SIFR-(STDLIB-0002|CODEGEN-0002|TYPE-0010)' \
    crates/sifr_diagnostics/src/codes.rs                      # only TYPE-0010 active, as intended
```

I did not re-run the full `scripts/run_all_tests.sh` profile because the user already reported `--profile quick` clean with `report_signature=e1bf653aaa770517, wall_time=76.13s`, and the targeted regressions are the load-bearing evidence for the pass-2 blocker fixes.

---

## Non-blocking observations (for future hardening, not for this PR)

These are worth keeping on a backlog but do not block closure of the pass-2 hardening slice.

1. **`list.index` / `tuple.index` accept `start=` / `stop=` keywords without keyword-range threading.** [crates/sifr_hir/src/lower/method_call_args.rs:455-470](crates/sifr_hir/src/lower/method_call_args.rs:455) (`append_start_stop_args`) inflates `args` past `positional` when `start=` / `stop=` keywords are supplied, but `resolved_method_arg_ranges` has no arm for `index`. The validator at [expressions.rs:2543-2554](crates/sifr_hir/src/lower/expressions.rs:2543) and [expressions.rs:3088-3099](crates/sifr_hir/src/lower/expressions.rs:3088) uses the safe `arg_ranges.get(bound_index).copied().unwrap_or(method_range)` pattern, so this is **not** a panic risk — but a type-mismatch on `list.index(x, start="bad")` will point its primary span at the `.index` method range rather than at `"bad"`. Diagnostic-quality nit only.

2. **`SIFR-TYPE-0010` row placement.** The new row is correctly placed between `SIFR-TYPE-0009` and `SIFR-TYPE-0011`. Worth noting that this completes the contiguous `0001..0011` block, so a future contributor adding `SIFR-TYPE-0012+` can extend without re-sorting.

---

## Summary

The structural deliverables of phase 31.7 plus the pass-2 hardening slice are all on the branch and verified:

- `resolved_method_arg_ranges` now threads keyword-default ranges for `list.sort`, `dict.get`, `dict.pop`, `dict.setdefault` — exactly the gap pass-2 identified, and exactly the four patterns covered. The threading predicates match the normalizer's `args` push conditions, so the lengths stay in lockstep.
- All four pass-2 CLI panic repros are gone; in their place is a single-line `type error: ...` structured diagnostic, exit 1.
- `scripts/run_all_tests.sh` now runs `sifr_hir` lib tests, closing the structural gap that let pass-2's failing test reach `main`.
- Inventory drift on `SIFR-STDLIB-0002` / `SIFR-CODEGEN-0002` / missing `SIFR-TYPE-0010` is cleaned up to match the post-`milestone_diag_11` registry state.
- Issue body has a closure preamble and a hardening-slice checklist entry with validation evidence.

I have **no blocking findings**. The pass-2 closure-hardening slice is complete and ready to ship.

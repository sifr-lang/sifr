## Definitive Post-Rebase Review — Wave 1, PR #3068

**Head reviewed:** `024a9d5cf731b5dd35c8fe9858f5bf8dfc6f9573`
**Base:** `afd25c3920a646fb0eea273c6899010baa7e94b7` (confirmed merge-base — linear, no stale base)
**Diff:** 8 files, +199/−3 (4 code/test/fixture files, 4 docs)

Every check below ran to completion in this invocation. No background work, no monitors, nothing pending. No files, refs, or GitHub state were modified.

### Production change

`crates/sifr_lowering/src/lower/type_bounds.rs:220` — one line: `Type::List(element) => supports_total_order(element)`.

Verified sound end-to-end:
- Lists lower to `Vec<T>` unconditionally (`preamble/types_and_errors.rs:14`, `generic_bounds_helpers.rs:143`), and `Vec<T>: Ord ⟺ T: Ord`. The recursion is exactly the trait rule — no over-claim.
- `list.sort()` emits `Vec::sort()` (`methods/list.rs:139-171`); keyless `sorted` emits `.sort()`, keyed `sorted` emits `sort_by(|l,r| key(l).cmp(&key(r)))` (`intrinsic_method_emitters/builtin_core_methods.rs:461-513`). A `list`-returning key yields `Vec<i64>::cmp` — satisfied.
- Recursion terminates: `Type::List` is an owned finite tree, and the branch never re-enters class types (`Class` falls to `_ => false`), so no visiting-set guard is needed here, unlike the equality/hash queries in the same file.
- Blast radius is exactly two call sites (`method_type_collections.rs:43`, `expression_sum_sorted.rs:301`). `min`/`max` (`expressions/call_builtins.rs:610+`) never consult this query, so they are provably not widened — matching the issue's scope statement. `.values()` guard untouched.
- Sets and dicts stay excluded (`_ => false`), preserving the issue's semantic requirement that non-total-order languages semantics are not admitted regardless of incidental generated representation.

### Recursive generated-Rust total-order semantics (native probes)

Built the compiler from this head and ran probes outside the repo. `Vec` lexicographic `Ord` matches CPython list comparison in every case I exercised — nested `bytes`, nested `tuple[int, str]`, four-deep `list[list[list[list[int]]]]`, nested `bool`, and **ragged** prefix ordering (`[[1,2],[1],[1,2,3]] → [[1],[1,2],[1,2,3]]`). The identical CPython program agrees on all five.

### Diagnostics and exact ranges

- `list.sort()` rejection: `SIFR-TYPE-0002`, message unchanged, anchored on the method-name range (`method_range`) — confirmed both by the new test's `range_for(source, "sort")` assertion and by a live diagnostic render pointing at column of `sort`.
- `sorted()` rejection: message includes the offending type; range is `key_keyword.map_or(iterable_range, …)`, and the new test asserts the iterable range via `range_for_after_anchor(source, "sorted(", "values")` — correct for the no-key case.

### Coverage

`expressions_tests/algorithmic_corpus_regressions.rs` (new focused module, 52 lines — satisfies the issue's explicit "new focused module" requirement and keeps `expressions_tests.rs` at 23 lines):
- Positive: nested `list.sort()`, triple-nested `sorted`, and a `list`-returning `sorted` key (the pass-1 request).
- Negative: nested `float`, nested `set`, nested `dict`, nested user class — all four asserted on code + exact message + exact range, not merely `is_err()`.
- Negative for `sorted` with the iterable-range anchor.

`crates/sifr/tests/e2e/pass/recursive_list_total_order.sifr` matches the directory's assert-based convention (no expectation file is required; fixtures are directory-discovered by the merge profile). Its three assertions are CPython-exact — I ran the equivalent Python: `sorted(key=…)` on `[21,12,11]` → `[11,21,12]` and `reverse=True` on nested string lists → descending, both confirmed.

### Native behavior and all six target fixtures

All six checked, built, and ran natively, and **stdout matched the CPython reference byte-for-byte** for each:

`0056_merge_intervals`, `0252_meeting_rooms`, `0435_non_overlapping_intervals`, `0452_minimum_number_of_arrows_to_burst_balloons`, `1383_maximum_performance_of_a_team`, `2402_meeting_rooms_iii` — `check=0 build=0 run=0 diff=match` for all six.

### Independent regression sweep

Full 411-fixture `sifr check --isolated` sweep on this head: **397 pass / 14 fail**. The 14 are exactly the documented 20 minus the six Wave-1 targets — set-identical, no new failures, no fixture silently changed category. This independently confirms the PR body's 397/14 claim and rules out regression from the widened gate. (A widened capability gate can only admit programs, and the query never influences inference, so previously-`BUILD_PASS` fixtures are structurally unaffected.)

### Gates re-run here

`cargo fmt --check` PASS · `cargo clippy --workspace -- -D warnings` PASS (project's documented gate) · `check_hir_maintainability_guardrails.py` PASS · `check_file_size_guardrails.py` PASS (2980 files, limit 900) · `cargo test -p sifr_lowering` 882 passed / 1 ignored / 0 failed · new focused module 3/3.

### Scope, maintainability, evidence integrity

Scope is minimal and correct: one production line, one test module, one e2e fixture, one ledger row. No corpus fixture edits (correctly — Wave 1 needs none), no baselines, exclusions, fallbacks, profile/matrix/pin changes. Ledger diff replaces the placeholder row with a Wave-1 row citing passes 1–3 and adds a `Waves 2-8 pending` row, which together with the separate closeout row covers waves 2–9 without gaps. The separately-tracked-findings expansion is accurate and does not broaden the wave. Acceptance checkboxes correctly remain unchecked. Passes 1–3 are tracked in the commit; pass-4 (747 B, self-declared mid-run) and pass-5 (0 B) are untracked and correctly not cited as approval evidence anywhere in the diff. Working tree carries nothing pushable: submodule pointers unchanged (`git submodule status` shows no `+`), the ` M` entries are only `.DS_Store`-class untracked content inside submodules.

### Non-blocking observations (not actionable findings)

1. The PR body's `42/42 cache hits` is from the pre-rebase run; the rebased run was `0/42` cold-cache in 418928/600000 ms. The substantive claims (exit 0, 131/131) hold on this head, and a cold run is strictly stronger evidence, so this is a stale incidental figure with no correctness weight.
2. The create-pr manifest (`verification/areas/core_language/data/create_pr_e2e_manifest.json`) is a curated 131-fixture list and does not include the new fixture — so the 131/131 figure did not exercise it. The merge profile discovers it by directory, and I ran it natively here (exit 0).
3. `cargo clippy --workspace --all-targets` fails in `sifr_ipc` test code (`expect_used`) — pre-existing on main, untouched by this diff, and outside the project's documented clippy gate.
4. `crates/sifr/tests/e2e/pass/Untitled` (3 bytes, content `sin`) is stray junk tracked since `c9e5aba729` on main — pre-existing, outside this diff's scope.

Zero actionable findings. The implementation is correct, minimally scoped, correctly gated in both directions, natively verified against CPython on all six target fixtures plus independent nested probes, and free of corpus regression at 397/14.

**APPROVED**

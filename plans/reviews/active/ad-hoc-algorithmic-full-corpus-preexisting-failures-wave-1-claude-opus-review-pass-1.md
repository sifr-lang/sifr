I completed an independent review, including building the compiler in this tree and empirically probing the widened predicate.

## What I verified

**Core change** — `crates/sifr_lowering/src/lower/type_bounds.rs:220`: `Type::List(element) => supports_total_order(element)`.

This is exactly right against the generated representation. `Vec<Vec<T>>: Ord` holds iff `Vec<T>: Ord` holds iff `T: Ord`, so recursing into the element with the same predicate is the precise condition — no wider, no narrower. The accept-arm at `type_bounds.rs:208-219` is unchanged, so no new leaf type is admitted.

Empirically confirmed with `target/debug/sifr` (built from this tree, exit codes checked):

- Accepts + runs correctly: `list[list[int]].sort()`, `list[list[list[str]]]` via `sorted(..., reverse=True)`, `list[list[bytes]].sort()`, `list[list[bool]].sort()`, `list[tuple[list[int], int]].sort()`, nested sort on a class field, `sorted(..., key=fn -> list[int])`.
- Rejection preserved with the clean Sifr diagnostic: nested `float`, `set`, `dict`, class, and `Any` (`list[list[Any]].sort()` → SIFR-TYPE-0002); `sorted(list[list[float]])` and `sorted(list[list[dict[str, int]]])` likewise. `Type::TypeVar` still returns `false` at `type_bounds.rs:222`.
- **No widening of min/max**: `crates/sifr_lowering/src/lower/min_max_validation.rs` has no reference to `supports_total_order*`; the only two call sites are `expression_sum_sorted.rs:301` (`sorted`) and `expressions/method_type_collections.rs:43` (`list.sort`), so `sort`/`sorted` move consistently and nothing else does.
- **No widening of sets/dicts**: the hash gate (`supports_hash_key_in_context`, `type_bounds.rs:151`) is untouched; `set[list[int]]` and `dict[list[int], int]` are still rejected as non-hashable.
- No cycle-guard risk: a self-referential alias (`type Rows = list[Rows]`) never resolves to a `List`, so the guardless recursion cannot diverge; `Tuple` already recursed the same way.

**Corpus and gates**: all six fixtures (`0056`, `0252`, `0435`, `0452`, `1383`, `2402`) build and run exit 0 in `verification/areas/algorithmic_compatibility/corpora/leetcode/src/`, and each contains `assert`s, so exit 0 is semantic evidence, not just compilation. All six use `list.sort()` on `list[list[int]]` — root cause matched, no fixture edited, no submodule pin moved (`git submodule status` shows no staged pointer change; the `M` entries are only untracked `.DS_Store`, out of scope and not in the change set). The two new tests pass, `check_hir_maintainability_guardrails.py` PASS, `check_file_size_guardrails.py` PASS (2961 files, limit 900); the touched files are 361/23/34 lines. The e2e fixture is capability-named with no phase number, uses runtime `assert`s, and matches the convention of all 675 pass fixtures (none use `# expect-stdout`; 532 use bare `assert`). `mod` insertion at `expressions_tests.rs:13` keeps alphabetical order.

## Findings

**1. (Medium-low) Newly-enabled `sorted` key-ordering and `sorted` rejection paths for nested lists are untested.**
`crates/sifr_lowering/src/lower/expression_sum_sorted.rs:300-311` — Wave 1 makes `ordering_ty` of `list[...]` acceptable, so `sorted(values, key=fn_returning_list)` is newly legal and exercises a distinct codegen surface (comparison on key output, plus the `supports_derived_clone` branch at `expression_sum_sorted.rs:285`). Nothing covers it: the new module's positive test (`algorithmic_corpus_regressions.rs:6`) only covers keyless `sorted`, and the negative test (`algorithmic_corpus_regressions.rs:14-19`) asserts only the `list.sort()` message. The pre-existing `sorted` negative test at `expressions_tests/minmax_sorted_sum.rs:137-153` covers class element/key types, never nested lists. I manually confirmed the key path builds and produces correct output, so this is a coverage gap rather than a defect — but the issue's own acceptance criterion ("focused e2e tests build and run every corrected generated-Rust surface") is not met for it. Add a `sorted`-with-list-returning-key positive case and a `sorted(list[list[float]])` negative case with the `sorted()`-specific message plus its range (which differs: `key_keyword.map_or(iterable_range, …)` at `expression_sum_sorted.rs:309`).

**2. (Low) The change set carries a zero-byte review artifact and a stale status row.**
`plans/reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-1-claude-opus-review-pass-1.md` is 0 lines. Committing an empty review document is a diff-hygiene defect. Relatedly, `plans/issues/active/ad-hoc-algorithmic-full-corpus-preexisting-failures.md:296` still reads "implemented; validation and review pending" although the full local validation is already complete; the row should record the passing evidence, or the placeholder should be excluded until it is written.

## Non-blocking observations (pre-existing on `main`, not caused by Wave 1)

I isolated both against flat lists, so neither is a Wave 1 regression, but both are genuine "if it compiles, it works" violations worth adding to the issue's *Separately Tracked Findings* (`plans/issues/active/ad-hoc-algorithmic-full-corpus-preexisting-failures.md:259-273`), which currently records only the reverse-sort stability widening:

- `sorted(x, key=lambda …)` passes lowering and then fails in generated Rust with `E0425: cannot find function 'sorted' in this scope` — reproduced with flat `list[int]` and a `lambda x: x` key, so it is independent of nesting. Wave 1 widens the element types that can reach it. A named key function works.
- `list[None]` and `list[list[None]]` literals emit `E0308: expected '()' found 'Option<_>'`; reproduced with no `sort` call at all.

The compiler change itself is minimal, correct, root-cause-shaped, and scope-disciplined; the two findings above are about test coverage and artifact hygiene, not the predicate.

VERDICT: CHANGES REQUESTED

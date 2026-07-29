Independent re-review of Wave 1, pass 2. I inspected the complete working-tree diff and re-verified everything empirically with a compiler built from this tree; I did not modify any tracked file, ref, submodule, or GitHub state.

## Pass-1 finding 1 — closed

**(a) Positive named list-returning key.** Present in both required places and change-sensitive:
- `crates/sifr_lowering/src/lower/expressions_tests/algorithmic_corpus_regressions.rs:4-10` — `nested_key(value: int) -> list[int]` plus `sorted([21, 12, 11], key=nested_key)`, alongside keyless `list.sort()` and triple-nested `sorted()`.
- `crates/sifr/tests/e2e/pass/recursive_list_total_order.sifr:1-14` — same named key with a **discriminating** runtime ordering assertion: `assert keyed == [11, 21, 12]` (line 14). Natural int order would be `[11, 12, 21]`, so the assertion only holds if lexicographic `Vec` comparison on the key output is actually performed. The other two asserts are equally discriminating: line 7 forces a first-element tie (`[1,1]` vs `[1,2]`), line 11 exercises triple nesting under `reverse=True`.

I confirmed this path is genuinely the widened one: a key returning `list[float]` is rejected with `sorted() requires an element or key type with generated Rust total Ord support, unavailable for 'list[float]'`, proving `ordering_ty` is the key's return type flowing through `expression_sum_sorted.rs:300-311`. Pre-change that gate rejected any `Type::List`, so the positive test fails without the diff.

**(b) Nested-float `sorted()` negative.** `algorithmic_corpus_regressions.rs:36-52` asserts the exact `TYPE_MISMATCH` message string and `range_for_after_anchor(source, "sorted(", "values")`. I verified against `expression_sum_sorted.rs:184-194` that `iterable_range` is the first positional argument's range, so the assertion pins the `key_keyword.map_or(iterable_range, …)` *iterable* branch at `expression_sum_sorted.rs:309` — distinct from `method_type_collections.rs:43` (`list.sort()`, message and `sort` range asserted at `algorithmic_corpus_regressions.rs:24-32`) and from the pre-existing key-range coverage at `minmax_sorted_sum.rs:137-153`. Both `sorted` call sites and the `list.sort` call site are now covered.

**E2E succeeds in generated Rust:** `sifr run --isolated crates/sifr/tests/e2e/pass/recursive_list_total_order.sifr` → exit 0 with all three asserts live. Discovery is directory-based (`e2e_entrypoints.rs:21`, `read_dir_file_paths_sorted`); `report_signature` is only printed, so no snapshot or registration needs updating.

## Pass-1 finding 2 — closed

`…wave-1-claude-opus-review-pass-1.md` is now 36 lines, untracked and available for inclusion. `plans/issues/active/ad-hoc-algorithmic-full-corpus-preexisting-failures.md:299` replaces the stale row with a Wave 1 row reading "implemented and validated; review changes addressed", and the evidence cell accurately names the capability, the added `sorted` key coverage, the e2e, the six fixtures, and the pass-1 origin.

## Full reassessment

- **Core change** (`type_bounds.rs:220`) — one line, exactly the right predicate: `Vec<Vec<T>>: Ord` iff `T: Ord`. The accept arm (`208-219`) and `TypeVar => false` (`222`) are unchanged, so no new leaf type is admitted. Recursion terminates unconditionally because `Type` is a finite `Box` tree; `type Rows = list[Rows]` never resolves to a `List` (rejects with `SIFR-STDLIB-0001`), and `Tuple` already recursed identically.
- **Boundaries I re-probed myself** — accepted and correct at runtime: nested `bool`, nested `bytes`, nested `decimal`, nested `bigint`, `list[tuple[list[int], int]]`. Still rejected: nested `float`, `set`, `dict`, class, and `int | None` elements. Only two call sites consume the predicate (`expression_sum_sorted.rs:301`, `method_type_collections.rs:43`); `min_max_validation.rs` uses `type_check_comparison` and contains no reference to it, so `min`/`max`, the hash gate, and set/dict membership are provably unaffected.
- **All six corpus fixtures** (`0056`, `0252`, `0435`, `0452`, `1383`, `2402`) — `build` exit 0 and generated-binary `run` exit 0, each with 1–3 live `assert`s, so exit 0 is semantic evidence. No fixture source edited; both `M` submodule entries are untracked `.DS_Store` only, with no pointer change (`git submodule status`, empty `--submodule=diff`).
- **Structure and gates** — new focused module per the plan's wave-1 requirement, `mod` inserted alphabetically (`expressions_tests.rs:13`); files 52/361/23/14 lines. `cargo fmt --check` clean; file-size guardrail PASS (2961 files, limit 900); HIR maintainability PASS; project clippy gate (`-p sifr_lowering -p sifr -- -D warnings`) clean. Full `sifr_lowering` suite: 865 passed / 1 ignored — consistent with 864 before the single added test.
- **Separately tracked observations** — I reproduced both exactly as newly worded at lines 264-266: `sorted(…, key=lambda …)` on flat `list[int]` → generated Rust `E0425`; `list[None]` with no sorting → `E0308`. Wave 1's own coverage correctly uses a *named* key function so it does not depend on the broken lambda path.
- **Scope discipline** — one production line, one test module, one e2e fixture, five doc lines. No baselines, exclusions, fallbacks, profile registration, crate pins, or Rust-interop matrix changes.

## Non-blocking notes (no action required for this wave)

- `…wave-1-claude-opus-review-pass-2.md` is currently 0 bytes; it is the destination for this report and must be populated before it is committed.
- Wave 1's reviews are referenced in prose but not linked as markdown artifacts the way diagnosis passes 4-14 are (`…failures.md:194-215`); worth adding at wave closeout for continuity.
- Pre-existing and untouched by this diff: `min`/`max` over a list whose element lacks generated `Ord` lowers and then fails generated Rust (`min(list[float])` → `E0618`, `min(list[list[float]])` → `E0277`), so line 267's "min/max list ordering remains intentionally narrower than `list.sort()`" understates it. `min`/`max` share no call site with the widened predicate, so this is not Wave 1's to fix. Nested non-`int` list literals still infer `list[list[Any]]` in equality — precisely Wave 2's scope.

Zero actionable findings.

VERDICT: APPROVED

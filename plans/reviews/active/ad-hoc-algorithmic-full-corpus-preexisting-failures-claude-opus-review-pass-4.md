I reproduced all 20 failures and probed each hypothesis against the compiler with the debug binary (no repo files modified; scratch probes lived in `/tmp/ownprobe`).

## Verdict: **Approved with conditions.** The failure grouping is accurate, but two of the six claims are wrong about mechanism, and one wave as written produces a green gate over code that does not build.

### Reproduction
All 20 fixtures reproduce exactly the stated diagnostics at `649334330c` (`target/debug/sifr check`). Family counts match: 6 × `list.sort()` total-Ord, 6 × empty-list-literal structural equality (incl. nested in 1489), 4 × defaultdict(int) `Any` key, 1 × defaultdict(set) membership, 2 × mutable-borrow representation, 1 × missing annotations. Note diagnostics are first-error-per-function, so several fixtures (0036, 0767, 0002) will surface follow-on errors after a wave lands — each wave must verify whole-fixture cleanliness, not diagnostic disappearance.

### Claim-by-claim

**1) Recursive list total Ord — CONFIRMED.** `Type::List` → `Vec<T>` (`crates/sifr_codegen/src/preamble/types_and_errors.rs:14`), `sort()` lowers to `Vec::sort` (`crates/sifr_codegen/src/methods/list.rs:139`), and Rust's `Vec<T>: Ord` is lexicographic — matching Python. Add the arm in `crates/sifr_lowering/src/lower/type_bounds.rs:206`. Existing negative tests use classes, not containers, so they won't regress. `Set`/`Dict` must stay excluded on *semantic* grounds, not representation: sets appear to lower to `Vec` in places (`_new_set_impl() -> Vec<i64>`), so "the repr is Ord" is not a sufficient argument — keep the justification semantic.

**2) Contextual empty-list specialization — CONFIRMED, with a hard constraint.** The gate that fires is `crates/sifr_type_system/src/check.rs:397`, *before* the existing `equality_comparable` Any-tolerance at line 431. Do **not** fix it by relaxing line 397: `list[Any]` would then reach codegen and emit `Vec<i64> == Vec<AnyRepr>`, which rustc rejects. The literal's HIR type must actually change, in lowering. The insertion point is symmetric with existing precedent: `crates/sifr_lowering/src/lower/expression_operators.rs:598-600`, next to `refine_empty_dict_index_comparison_expr`. I verified the annotated-equivalent already compiles cleanly, nested included (`empty: list[int] = []`, `nested: list[list[int]] = [[], [0,1,2,3]]` → `no errors found`), so only contextual typing is missing. Restrict it to *literal* nodes (never variables — their Rust type is fixed), recurse only into list literals whose element type is `Any`/`Unknown`, and keep negative coverage that `list[int] == ["a"]` still fails.

**3) defaultdict(int) refinement "persists after the loop" — CHALLENGED; premise is wrong.** There is no refinement to persist. Probe: `c = defaultdict(int); for n in nums: c[n] += 1` then `.items()` *inside the same loop body, immediately after the augassign* still fails identically. The augassign path never refines the key: `crates/sifr_lowering/src/lower/container_literal_specialization.rs:244-253` rebuilds the defaultdict alias with the original `Any` key. That line is the root cause and the fix site. Codegen is already correct and needs no change (it emits untyped `HashMap::new()` + `entry(n).or_insert(0)`, which Rust infers as `HashMap<i64,i64>`) — this is purely a HIR typing gap.

**4) defaultdict(set) refinement "should persist" — CHALLENGED; persistence already works.** Probe: `s = defaultdict(set); for i in range(3): s[i].add("x")` followed by `"x" in s[0]` **after** the loop type-checks today. 0036 fails for a different reason: `cell in rows[r]` appears *textually before* `rows[r].add(cell)` in the same iteration, and lowering is a single forward pass. Reordering is not available — the membership test is the algorithm's duplicate check. So claim 4's remedy does not fix 0036.

**5) 0002/0086 need ownership, not mutable borrow — CONFIRMED as diagnosis.** `nodeNext(own node)` consumes, so a mutable borrow cannot be moved out; the diagnostic is correct, and 20 sibling fixtures already use `own` / `own mut`. I applied `own mut l1/l2` and `own mut head` to `/tmp` copies: both reach `no errors found`. **But see Finding H1 — that is not sufficient.**

**6) 0377 remove dead code — CONFIRMED.** The unannotated `dfs` sits after `return`, is pure upstream porting residue, and the iterative DP above it is the fixture's algorithm. Removal is right; annotating it to satisfy the checker would preserve dead code and a compiler-ignores-invalid-code precedent. Note the `.py` sibling carries the same dead block — state the parity policy in the PR (I'd leave the `.py` alone and say so).

### Findings by severity

**H1 — HIGH. The corpus lane is `check`-only, so the 0002/0086 wave would go green over code that does not compile.** `runner.py` (`run_leetcode_full`, `run_fixture`) invokes only `target/debug/sifr check`; nothing builds or runs. With `own mut` applied, both fixtures pass `check` and then fail `sifr run` with rustc `E0596`. The bug is in the generated helper: `nodeNext` emits `let Some(node) = node else {...}; node.next.take()` — `.take()` needs `&mut`, and the early-return destructure of an owned param omits `mut` (the `if let Some(mut cur)` path in the same file gets it right). This is pre-existing and *already latent in the passing corpus*: `0206_reverse_linked_list`, not on the failure list, fails `sifr run` at `HEAD` for exactly this reason. Wave 5 must include the codegen fix (mark the owned destructured binding `mut`, or emit a partial move instead of `.take()`), and the acceptance criteria should require build+run for the touched fixtures — otherwise criterion "every listed fixture passes" is satisfiable without the fixtures working. This is a direct "if it compiles, it works" violation.

**H2 — HIGH. Do not use a `dict[...]` annotation as the defaultdict fixture workaround; it silently produces wrong results.** `c: dict[int,int] = defaultdict(int)` type-checks and emits `if let Some(__elem) = c.get_mut(&n) { *__elem += 1 }` — the missing-key insert is gone, so a counting loop yields 0. The `entry().or_insert()` lowering only survives when the alias is retained. Separately, this is a real latent defect worth its own issue: plain `c: dict[int,int] = {}; c[1] += 1` silently no-ops where Python raises `KeyError` — a wrong-answer divergence, not a panic.

**M1 — MEDIUM. Wave 3 should be one declaration-site inference, not two flow patches.** Given F3/F4, I recommend replacing claims 3+4 with a single bounded, syntactic pre-scan at the `x = defaultdict(...)` binding site: collect the element/key shapes implied by `x[...] += v`, `x[...].add(v)`, `x[...].append(v)` within the enclosing function, and specialize the binding at declaration. That is order-independent (fixes 0036), subsumes the augassign gap (fixes 0350/0621/0767/1481), and avoids a fixpoint. Conflicting sites must emit the existing deterministic `TYPE_CONTAINER_ELEMENT_CONFLICT`. If that design is contentious, split: 3a = augassign key refinement at `container_literal_specialization.rs:244` (4 fixtures, small and safe), 3b = order-independent pre-scan (0036 only) as its own reviewable PR.

**M2 — MEDIUM. Annotation is not an escape hatch for 0036 either.** `s: dict[int, set[str]] = defaultdict(set)` then `s[i].add(...)` fails with `type 'None | set[str]' has no method 'add'`. 0036 has no clean fixture-side fix; it is genuinely compiler-side. Don't let it get bundled into a "fixture violations" wave.

**L1 — LOW. Test placement will breach the 900-line cap.** `expressions_tests/minmax_sorted_sum.rs` is at 859 lines and `sifr_type_system/src/check.rs` at 876. Put wave 1/2 tests in new modules; wave 2 should not touch `check.rs` at all (it shouldn't need to).

**L2 — LOW. `sort(reverse=True)` lowers to `sort()` then `reverse()`**, which inverts equal-element order versus Python's stable reverse sort (`methods/list.rs:146-168`). Pre-existing, not triggered by these six fixtures (all use bare `sort()`), but wave 1 widens the element types that reach it.

**L3 — LOW. `min()`/`max()` will stay asymmetric with `sort()` after wave 1** — `type_satisfies_comparable_bound` (`type_bounds.rs:97`) excludes lists, so `min(list_of_lists)` remains rejected while `sort()` is accepted. Decide and document; don't fix silently in the same PR.

**L4 — LOW. `.values()` never hashes keys**, yet the blanket gate at `expressions/method_type_collections.rs:305` rejects every dict method except `len`/`clear` on an `Any` key — that alone accounts for 0621 and 1481. Relaxing it is *not* the recommended vehicle (it would let `Any` keys escape through `.items()` into user code), but note it as a known over-broad guard.

**L5 — LOW. No unreachable-code diagnostic exists** — 0377's dead block is fully type-checked. Worth a separate follow-up; out of scope here.

### Recommended wave boundaries

| Wave | Change | Fixtures | Files |
| --- | --- | --- | --- |
| W1 | `Type::List` recursion in `supports_total_order` | 0056, 0252, 0435, 0452, 1383, 2402 | `type_bounds.rs` |
| W2 | empty-list-literal contextual specialization in `==`/`!=` | 0094, 0144, 0145, 0442, 1203, 1489 | `expression_operators.rs` + new refinement module |
| W3a | defaultdict key refinement at subscript-augassign | 0350, 0621, 0767, 1481 | `container_literal_specialization.rs:244` |
| W3b | order-independent defaultdict declaration-site inference | 0036 | new pre-scan module |
| W4 | fixture: remove 0377 dead `dfs` | 0377 | fixture only |
| W5 | fixture `own mut` **+ codegen `mut`/partial-move fix for owned recursive-field reads** | 0002, 0086 (+ latent 0206 class) | fixture + codegen |
| W6 | closeout: restore `leetcode-full` to `verification/profiles/release.json`, regenerate taxonomy/delta artifacts, update tracker | — | verification + plans |

W1–W5 touch disjoint files and can land in any order; W6 last.

### Required tests per wave
- **Unit (lowering):** positive `list[list[int]]` sort/sorted; negatives for `list[float]`, `list[set[int]]`, `list[dict[str,int]]`, `list[Class]` (W1). Positive nested empty-literal comparison; negatives for non-empty mismatched literals and variable operands (W2). Refinement asserted on both branches and after the loop, plus a conflicting-write conflict diagnostic (W3).
- **E2E `pass` fixtures (mandatory, not optional):** `crates/sifr/tests/e2e/pass/` builds *and runs* fixtures — this is the only gate that would have caught H1. One per wave, asserting runtime output: nested-list sort ordering; empty-list equality incl. nested; defaultdict counting result (must be non-zero — guards H2); linked-list traversal through an `own` helper.
- **E2E `fail` fixtures:** preserve the rejections for genuinely non-Ord elements and genuinely mismatched literals.
- **Corpus:** full `algorithmic-leetcode-full` after each wave, plus regenerated taxonomy/delta at closeout, plus `nightly` profile locally.
- **Gates:** `scripts/run_all_tests.sh --profile create-pr` per PR, full run at closeout, clippy/fmt/file-size/`check_hir_maintainability_guardrails.py`.

### Traps to call out in the PR descriptions
- No baselines, exclusions, or annotation workarounds (H2 makes the annotation route actively wrong).
- W2 must not weaken `check.rs`'s capability gate; the literal's type must change in HIR. Verify with `sifr emit` that the empty literal materializes as the concrete `Vec<T>`.
- W3 refinement must not retroactively retype a binding whose generated Rust type is already fixed elsewhere in the function; confirm via `emit` that `HashMap` key inference stays consistent.
- W5's codegen change touches every owned optional-class destructure — check the e2e linked-list fixtures broadly, not just 0002/0086.

One housekeeping note: `plans/reviews/active/ad-hoc-algorithmic-corpus-diagnosis-claude-opus-round-1.md` exists but is empty (0 bytes); I left it untouched. The established convention in that directory is `ad-hoc-algorithmic-full-corpus-preexisting-failures-claude-opus-review-pass-4.md`.

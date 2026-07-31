# Wave 8 Correction Review (Pass 2)

**Target:** branch `codex/algorithmic-recursive-option-constructor`, exact head `a1633ea4f`, base `4c867d1cda`, draft PR #3089
**Scope:** entire PR diff (10 files, +381/−126). Working tree unchanged throughout — `git status` at end is byte-identical to session start; no files modified.

## Verdict: REQUEST CHANGES — 4 actionable findings (0 correctness, 1 coverage, 2 documentation/workflow, 1 simplification)

All five pass-1 findings are resolved. I could not produce a single regression; a base-vs-head differential over 9 hand-built probes shows head strictly improves or is byte-identical everywhere.

---

## Pass-1 findings: independently reproduced and confirmed resolved

I built both compilers (`cargo build -p sifr` at head; `git archive 4c867d1cda` + submodule symlink → `/tmp/w8base`, `cargo build -p sifr`) and diffed full emitted Rust for each probe.

**Findings 1 + 2 (double box; `E0507` on the nested path) — RESOLVED.**
```
$ ./target/debug/sifr emit /tmp/w8r2/p1.sifr
fn wrap(node: &Option<TreeNode>) -> TreeNode {
    TreeNode::new(5_i64, (node).clone().map(|__sifr_option_value| Box::new(__sifr_option_value)))
fn wrapNested(node: &Option<TreeNode>) -> Vec<TreeNode> {
    nodes.push(TreeNode::new(6_i64, (node).clone().map(|__sifr_option_value| Box::new(__sifr_option_value))));
fn wrapKw(node: &Option<TreeNode>) -> TreeNode {
    TreeNode::new(7_i64, (node).clone().map(|__sifr_option_value| Box::new(__sifr_option_value)))
fn wrapNone() -> TreeNode { TreeNode::new(8_i64, None) }
```
Clone precedes `map`, exactly one box layer, keyword form identical, `None` untouched. Builds and runs (`sifr run p1.sifr` → release build finished, assertions pass). Base emitted `TreeNode::new(6_i64, (node).clone())` here — `E0308`.

**Finding 3 (idempotency-by-recognizer / four divergent sites) — RESOLVED.**
`is_option_box_map_expr_for_ir` is gone (`grep` across `crates/` returns nothing). `registry_is_some_ctor`, `registry_is_some_expr`, `registry_ensure_some_box_inner` deleted. One adapter, three call sites, and the routes are mutually exclusive by early `return`:
- `plain_call_args.rs:119` (strict registry) → caller at `expr_call_and_literal_helpers.rs:264` now `continue`s for option params, so no second application.
- `call_args_and_returns.rs:82` (nested/print path, reached via `print_calls.rs:88`, `expr_call_and_literal_helpers.rs:738`, `recursive_exprs.rs:275`).
- `expr_call_and_literal_helpers.rs:335` (constructor fallback, only reached after the registry path returns `None` at line 281).

I hand-verified all eight base branch combinations (option/non-option arg × `needs_box_inner` × `None` × owned/borrowed) against head: behaviour-equivalent except that the duplicate application and the clone-ordering bug are gone.

**Finding 4 (coverage) — PARTIALLY RESOLVED** — see finding 1 below.
**Finding 5 (ledger) — RESOLVED**: the overstated idempotence sentence is gone; the status cell now reads `parent [PR #3089](…) in review`.

## Base-vs-head differential (the decisive evidence)

Every probe, full emit diff. `m4` and `m6` are byte-identical; the other seven change only in the direction of correctness:

| probe | shape | base | head |
|---|---|---|---|
| p1 | borrowed option → nested ctor | `(node).clone()` **E0308** | `+ .map(Box::new)` |
| m1 | `node.left` → nested ctor | `(node.left).as_deref().cloned()` **E0308** | `+ .map(Box::new)` |
| m1 | `own` option param → nested ctor | `node` **E0308** | `node.map(Box::new)` |
| m1/m7 | non-option arg → direct ctor | `Some(Box::new((Some(Box::new((child).clone()))).clone()))` **double box** | `Some(Box::new((child).clone()))` |
| m2 | mutually recursive → nested | `(e).clone()` **E0308** | `+ .map(Box::new)` |
| m3 | generic recursive class → nested | `(n).clone()` **E0308** | `+ .map(Box::new)` |
| m5 | defaults + kwargs → nested | `(n).clone()` **E0308** | `+ .map(Box::new)` |
| m8 | option arg alongside recursive container | `(p).clone()` **E0308** | `+ .map(Box::new)` |
| m4 | ctor param order ≠ field order | — | **identical** (pre-existing bug, unchanged) |
| m6 | method/dict/list-literal/print, non-recursive `plainFn` | — | **identical** |

Non-recursive options and `None` are untouched at head (`Plain::new(a, (b).clone())`, `Plain::new(None, None)`, `plainFn(&c)`). All probes build and run at head.

## Validation I ran

| Check | Result |
|---|---|
| `cargo test -p sifr_codegen` | **964 passed, 0 failed** |
| `verification/runner/e2e/run_e2e_pass.sh` | **686/686 passed**, `report_signature=96d2681cf0c5ac5c` (686 = all `e2e/pass/*.sifr`, so the new fixture ran) |
| `cargo clippy --workspace -- -D warnings` | pass (exit 0) |
| `cargo fmt --check` | pass |
| `python3 scripts/check_hir_maintainability_guardrails.py` | `PASS` |
| `python3 scripts/check_file_size_guardrails.py` | `PASS (3066 files, limit 900)` |
| `python3 scripts/check_submodule_ownership.py` | `PASS` |
| `verification/areas/diagnostics/checks/baseline_hygiene.py` | exit 0 |
| `0894_all_possible_full_binary_trees` | check/build/run exit 0; both ctor sites emit exactly one box layer |
| 30-fixture recursive corpus build sweep (`ListNode`/`TreeNode`/`TrieNode`) | **30/30 exit 0** |

Note: `cargo clippy -p sifr_codegen --all-targets` fails with 14 errors, but every one is in a file this PR does not touch (`rust_interop_direct.rs`, `builtin_core_methods.rs`, `python_interop_direct_tests.rs`, …). `--all-targets` is not the project gate; not attributable.

---

## Actionable findings

### 1. MEDIUM — coverage: three shapes this PR demonstrably fixes are pinned by no test
`crates/sifr_codegen/src/lib_codegen_tests/recursive_node_codegen_tests.rs:168`

The new focused test covers exactly two shapes: a local bound from a call (`left_copy`) and a shared-borrowed parameter, each direct + nested. The differential above shows this PR *also* changes emit for three more, none of which is pinned anywhere in `crates/` or `verification/`:

- **Non-option recursive-class argument.** Base emitted `Some(Box::new((Some(Box::new((child).clone()))).clone()))` — a genuine `Option<Box<Box<T>>>`, i.e. the exact defect class this wave exists to eliminate. Head emits `Some(Box::new((child).clone()))`. `grep -rn "Some(Box::new((" crates/sifr_codegen/src/lib_codegen_tests/` → no hits.
- **`own`-convention optional recursive parameter into a nested constructor.** Base `TreeNode::new(4_i64, node, None)` → `E0308`; head `node.map(|__sifr_option_value| Box::new(…))`.
- **Recursive optional field access into a nested constructor.** Base `(node.left).as_deref().cloned()` → `E0308`; head adds the map. `grep -rn "as_deref().cloned().map(|__sifr_option_value|" crates/…/lib_codegen_tests/` → no hits.

Additionally, the **keyword-argument form** — which commit `f546f563b` is literally named after ("Coerce named recursive option constructor args") and which pass-1 finding 2 called out by name — is exercised by neither the focused tests nor the e2e fixture. It works (verified in p1/m5) but is unpinned.

Failure scenario: a future refactor of `recursive_constructor_args.rs` that drops the `!clone_before_adaptation` guard in the non-option branch reintroduces base's `Option<Box<Box<T>>>` on `TreeNode(5, child)` while all 964 codegen tests and 686 e2e fixtures stay green — the same blind spot that let pass-1's regression ship.

Secondary, same file, line 230: the negative
```rust
!rust_code.contains("node.map(|__sifr_option_value| Box::new(__sifr_option_value))")
```
asserts the absence of a string that is the **correct** output for an `own` optional parameter named `node` (see m1 `fromOwned`). It cannot coexist with `own`-convention coverage in that test and should be narrowed to the borrowed shape it means to guard.

### 2. LOW — documentation: ledger row omits full native e2e evidence
`plans/issues/active/ad-hoc-algorithmic-full-corpus-preexisting-failures.md:334`

The row claims "the capability e2e checks, builds, and runs …" and "`0894…` checks, builds …, and runs", but no complete-suite figure. Every prior wave row cites one (wave 4: "the complete native e2e suite passes 678/678"; wave 7: "679/679 with signature `b0887ad6eb81c080`"). Wave 8 *adds* an e2e fixture and, per the differential, changes emitted code for essentially every recursive-constructor optional argument — the broadest blast radius of any wave in this issue. Single-fixture corpus evidence understates it. Substantiating figures: **686/686, `report_signature=96d2681cf0c5ac5c`**, and **30/30 recursive corpus fixtures build**.

### 3. LOW — workflow: PR #3089 body still describes the design pass 1 rejected
`gh pr view 3089` summary bullet 2 reads *"make the shared option-box adapter structurally idempotent across repeated constructor adaptation"* — the syntactic self-recognizer that pass-1 finding 3 rejected and that `a1633ea4f` deleted. The body describes `f546f563b`, not the head under review. Its Validation list also omits the new capability e2e fixture and the full native e2e suite. The PR body is what a merging reviewer reads first.

### 4. LOW — simplification: `consumed_owned_borrowed_name` carries no information
`crates/sifr_codegen/src/stmt_support_emitter/recursive_constructor_args.rs:89`

The flag is returned as `context.convention.is_owned() && context.borrowed_name_arg` — both operands are already in scope at both call sites, which then guard with `convention.is_owned() && borrowed_name_arg && !consumed_owned_borrowed_name` (`plain_call_args.rs:224`, `call_args_and_returns.rs:152`). That reduces to *always false*: the terminal clone is unconditionally suppressed whenever the adapter fires. The `NoneLiteral` early return sets it `false`, but `borrowed_name_arg` only matches `HirExpr::Name`, so it is already `false` there. The contract is "the adapter owns ownership adaptation for this argument" — state that directly (or drop the field and have callers skip the terminal clone when the adapter returns `Some`) rather than round-tripping a value the caller computed.

---

## Pre-existing, out of scope (verified byte-identical between `4c867d1cda` and `a1633ea4f`)

- **Constructor parameter order ≠ field declaration order** mis-indexes `class_field_order`: for `class T { val: int; left: T|None }` with `__init__(self, left, val)`, both compilers emit `T::new((n).clone(), Box::new(1_i64))` — the recursive option arg unboxed, an unrelated `i64` wrongly boxed. The adapter inherits base's positional lookup verbatim. Full emit diff for this probe is empty.
- **Recursive *container* parameter through the nested-call path loses its `Box::new`**: `Outer::new(2_i64, (ks).clone(), …)` for `kids: Box<Vec<Outer>>` → confirmed `E0308` at build. `call_args_and_returns.rs` has no `is_recursive_container_param` post-pass; the direct path does. Identical at base; only the *option* argument on that line differs between base and head.
- `crates/sifr/tests/e2e/pass/Untitled` — stray 3-byte tracked file from `c9e5aba729`, unrelated.

## Requested changes to reach APPROVE

1. Add focused codegen coverage for the non-option recursive argument, the `own`-convention nested forward, the field-access nested forward, and the keyword form; narrow the line-230 negative.
2. Add the complete native e2e count + signature (and a corpus sweep broader than 0894) to the wave-8 ledger row.
3. Rewrite the PR #3089 body to describe `a1633ea4f`'s shared post-ownership adapter and list the actual validation.
4. Collapse `consumed_owned_borrowed_name` into a contract the callers can read directly.

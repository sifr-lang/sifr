## Findings

### 1. Incomplete root cause — `Box::new(vec![])` for empty recursive containers is still emitted from the constructor-argument boxing sites (actionable)

The correction patches exactly one of three recursive-boxing emission points. `crates/sifr_codegen/src/class_method_emitter.rs:121-129` now special-cases an empty `RustExpr::Vec` in `wrap_recursive_constructor_field_value`, but the sibling logic that boxes recursive **constructor arguments** at the call site is untouched and still wraps unconditionally:

- `crates/sifr_codegen/src/stmt_support_emitter/expr_call_and_literal_helpers.rs:267-277` (keyword/param-matched ctor path)
- `crates/sifr_codegen/src/stmt_support_emitter/expr_call_and_literal_helpers.rs:352-368` (positional ctor path)

Both build `RustExpr::FnCall { func: Path(["Box","new"]), args: vec![lowered_arg] }` with no empty-collection check, so the exact lint that blocked nightly pass 2 is still reachable from ordinary user source.

Repro against the **fixed** working tree (`target/debug/sifr` rebuilt from this diff):

```python
class Tree:
    children: list[Tree]
    left: Tree | None

    def __init__(self, children: list[Tree], left: Tree | None):
        self.children = children
        self.left = left

def build() -> Tree:
    return Tree([], None)
```

`sifr emit` line 19 → `Tree::new(Box::new(vec![]), None)`, and `clippy-driver 0.1.94` on that emitted file reports:

```
warning: `Box::new(_)` of default value
  --> e.rs:19:15
19 |     Tree::new(Box::new(vec![]), None)
   |               ^^^^^^^^^^^^^^^^ help: try: `Box::default()`
   = note: `#[warn(clippy::box_default)]` on by default
```

`clippy::box_default` is **not** in the generated-code-quality allow list (`verification/areas/generated_code_quality/generated_code_quality.py:100-237`; only `clippy::box_collection` is allowed there), so this form fails the same `-D warnings` gate that demo-004 failed.

Test adequacy is affected by the same gap: the new test's negative assertion `!rust_code.contains("Box::new(vec![])")` (`crates/sifr_codegen/src/lib_codegen_tests/recursive_node_codegen_tests.rs:182-186`) is scoped to a single source that has no recursive-container constructor parameter, so it cannot detect the argument-position site.

Severity is bounded but real: this is latent rather than currently gate-failing — I re-emitted all 10 `demos-required` entries against the fixed compiler and none contains `Box::new(vec![])`, `Box::new(Vec::new())`, `Box::new(HashMap::new())` or `Box::new(String::new())`, and demo-004 now emits `Box::default()` at `demos/dependency_manifest/main.sifr` emit lines 2610 and 2613. So the reported nightly evidence is consistent; the closeout claim that the box_default root cause is resolved is not.

The structural fix is a shared lint-clean boxing helper used by all three recursive-boxing sites rather than a per-site `matches!` guard.

## Verified correct (no findings)

- **Type inference at the patched site.** The only caller that can reach the new branch is `class_method_emitter.rs:363-385`, where `temp_ty` is `field_ty.map(|ty| self.class_struct_field_rust_type(...))` and the branch itself only runs when `field_ty` is `Some`, so the emitted `let` always carries an explicit annotation. Confirmed: `let __sifr_field_init_0: Box<Vec<Tree>> = Box::default();` and, for nested containers, `let __sifr_field_init_0: Box<Vec<Vec<Nest>>> = Box::default();`. The second caller (`class_method_emitter.rs:535-546`) passes `RustExpr::Ident`, which can never match `RustExpr::Vec`.
- **Semantics.** `Box::<Vec<T>>::default()` == `Box::new(Vec::new())`; no behavioral change.
- **Non-empty behavior preserved.** `self.children = [Tree("x")]` still emits `Box::new(vec![Tree::new(...)])`.
- **Guard ordering.** The new branch sits after the `is_box_new_call_expr` (`:117`) and Option (`:107-115`) early-outs, so it cannot shadow them.
- **Related empty-default forms that are already lint-clean** (checked, not gaps): recursive `dict` fields emit `Box::new(HashMap::from([]))` and recursive `Optional[list[T]]` fields emit `Some(Box::new(vec![]))` — `clippy-driver 1.94` reports nothing on either form.
- **Scope / hygiene.** Tracked diff vs `13709187f` is exactly 2 files, +31/-0. `third_party/ruff` and `verification/areas/algorithmic_compatibility/corpora/leetcode` show as modified in porcelain but produce an empty `git diff --submodule=short` (untracked content only) — no pointer drift.
- **File-size guardrail.** 824 and 662 lines, both under 900.
- **Focused test.** `cargo test -p sifr_codegen -- test_empty_recursive_container_field_uses_box_default` → 1 passed, 967 filtered (consistent with the claimed 968-test suite).
- **Prior pass-3 satisfaction.** The diff touches only recursive-constructor boxing and adds one test; nothing in the Wave 10 corpus/release selection surface is modified.

## Observation (not counted as a finding)

`crates/sifr_codegen/src/stmt_support_emitter/field_assignment.rs:229-236` emits `self.<recursive_field> = Box::new(...)` for method-level reassignment. On Rust 1.94 that pattern trips `clippy::replace_box` (also absent from the allow list) for **every** value, empty or not — I confirmed it fires equally on `Box::new(vec![other])` and `Box::new(vec![])`. That is a pre-existing latent gate exposure independent of this correction's emptiness concern, not a regression from this diff.

## NOT SATISFIED

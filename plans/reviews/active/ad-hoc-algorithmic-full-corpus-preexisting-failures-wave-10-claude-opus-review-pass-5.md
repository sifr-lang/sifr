## Findings

**None.** Zero actionable findings at any severity.

## Verification performed (read-only, this working tree)

Toolchain used for lint reproduction: `clippy-driver 0.1.94 (4a4ef493e3 2026-03-02)` — the same Rust 1.94 line that produced the nightly pass-2 failure.

**1. Both pass-4 cited call sites are now fixed, via the single shared helper.**
- Helper: `crates/sifr_codegen/src/stmt_support_emitter/print_calls.rs:388-409` — preserves already-boxed input (`is_box_new_call_expr_for_ir`, `print_calls.rs:379-386`), emits `Box::default()` only for `RustExpr::Vec` with `items.is_empty()`, else `Box::new(value)`.
- Registry/keyword-matched ctor site: `expr_call_and_literal_helpers.rs:267-270` now delegates.
- Positional/fallback ctor site: `expr_call_and_literal_helpers.rs:350-352` now delegates.
- Field-init site: `class_method_emitter.rs:117` now delegates.
- The two removed inline guards were `&& !is_box_new_call_expr_for_ir(...)` conjuncts; the helper's own first branch is logically identical, and `is_box_new_call_expr` (`class_method_emitter.rs:33-40`) and `is_box_new_call_expr_for_ir` are byte-equivalent predicates (both match `Path(["Box","new"])` or `Ident("Box::new")`), so replacing the former with the latter at the field-init site is behavior-preserving.

**2. Pass-4 reviewer repro no longer reproduces.** Rebuilt `target/debug/sifr` from this tree; emitting pass-4's exact source gives `Tree::new(Box::default(), None)` (positional) and the keyword form `Tree(children=[], left=None)` also gives `Tree::new(Box::default(), None)`. The program compiles and runs (`sifr run` → `0`,`0`), and `clippy-driver -D warnings -A clippy::box_collection` on the emitted file reports **no** `box_default` (only the allow-listed `box_collection` plus unrelated `cast_possible_wrap`).

**3. Type inference at the argument position holds.** `Tree::new` is emitted as `fn new(children: Box<Vec<Tree>>, left: Option<Box<Tree>>)`, so `Box::default()` in argument position resolves concretely; the program builds with rustc. At the initializer position both demo-004 sites carry explicit annotations, including the wrapped multi-line one: `demos/dependency_manifest/main.sifr` emit lines 2610 (`let __sifr_field_init_6: Box<Vec<…TomlValue>> = Box::default();`) and 2611-2613 (`let __sifr_field_init_7: Box<Vec<(String, …TomlValue)>> = Box::default();`).

**4. Non-empty and already-boxed behavior preserved.** `Tree([child], None)` → `Tree::new(Box::new(vec![c]), None)`; `Tree([[inner]])`-style non-empty nesting → `Box::new(vec![vec![inner]])`; no double-boxing anywhere in the 969-test suite (which includes the Wave 8 double-box rejection tests).

**5. Non-`Vec` empty forms at the patched sites are lint-clean, so the narrow `RustExpr::Vec` guard is sufficient.** Recursive `dict` field/arg → `Box::new(HashMap::from([]))`; `Optional[list[T]]` field → `Some(Box::new(vec![]))`; both produce zero clippy 1.94 diagnostics (verified on minimal reproductions). `list()` also lowers to the empty-`Vec` form and gets `Box::default()`.

**6. The exact nightly failure form is what fires, and it is gone.** `let x: Box<Vec<i64>> = Box::new(vec![]);` fires `clippy::box_default`; `J::new(Box::new(vec![]))` fires it; struct-literal field position `J { items: Box::new(vec![]) }` does **not** (explains why the 209 pre-existing struct-literal occurrences in checked-in `demos/*/emitted.rs` artifacts never failed the gate). Sweeping every tracked file: zero `let`-position and zero call-argument-position `Box::new(vec![])` / `Box::new(Vec::new())` / `Box::new(HashMap::new())` / `Box::new(String::new())` occurrences remain. `clippy::box_default` and `clippy::replace_box` are both absent from `GENERATED_CLIPPY_ARGS` (`verification/areas/generated_code_quality/generated_code_quality.py:98-237`; only `clippy::box_collection` is allowed).

**7. Claimed evidence checks out.**
- `cargo test -p sifr_codegen -- test_empty_recursive` → **2 passed, 967 filtered out**.
- `cargo test -p sifr_codegen` → **969 passed, 0 failed**.
- `cargo clippy -p sifr_codegen -- -D warnings` → clean.
- `cargo fmt -p sifr_codegen --check` → exit 0; `git diff --check HEAD` → clean.
- `scripts/check_file_size_guardrails.py` → PASS (3078 files, limit 900); touched files 807/477/758/695 lines.
- `scripts/check_submodule_ownership.py` → PASS; `git diff --submodule=short third_party/ruff verification/.../leetcode` is empty (untracked content only, no pointer drift).
- `target/sifr_generated_code_quality/evidence/clippy-1785536600-48931.json` (`run_id` matches the claim) contains exactly 10 `demos-required` records, all `"status": "passed"`, including `demo-004-dependency-manifest`. Its `source_sha256` is a digest over the generated crate's Rust files (`generated_code_quality.py:643-653`), and those crate roots have since been cleaned, so it cannot be recomputed; the record set is internally consistent and its timestamp (2026-07-31T22:23:20Z) postdates the sibling run.
- Scope: working tree touches exactly the 4 claimed files; no test/baseline/snapshot in the repo pins a now-changed `Box::new(vec![])` form.

Per instruction I did not run nightly/release/merge. Note also that `demo-004`'s emitted output is a multi-module crate, so I could not re-run crate-level clippy on it standalone (single-file compile fails with `E0583`/`E0282`); I verified instead that its emitted text contains no `box_default`- or `replace_box`-triggering form.

## Observations (not counted as findings)

- **Pre-existing, emptiness-independent: nested recursive constructor arguments are never boxed, producing non-compiling Rust.** `return Tree([Tree([], None)], None)` emits `Tree::new(Box::new(vec![Tree::new(vec![], None)]), None)` → rustc `E0308` (`expected Box<Vec<Tree>>, found Vec<_>`). The non-empty variant behaves identically (`Tree::new(vec![c], None)`, and `Nest::new(vec![vec![leaf()]])`), so this route never boxes at all and is untouched by this diff. Fails loudly at build time; not reached by the pinned corpus (nightly algorithmic 412/412).
- **Pre-existing, emptiness-independent: `clippy::replace_box` on method-level recursive field reassignment.** `field_assignment.rs:229-236` emits `self.children = Box::new(...)`; clippy 1.94 flags it for both `Box::new(vec![])` and `Box::new(vec![c.clone()])`. Confirms pass-4's observation verbatim. Related: for an `Optional[list[T]]` field, the same path emits `self.kids = Box::new(vec![])` without the `Some(...)` wrapper → rustc `E0308`; also emptiness-independent (`optional_recursive_class_name` returns `None` for `Option<list>`, taking the un-wrapped else-branch).
- **Pre-existing, unrelated:** a self-referential `set[T]` field is not registered as recursive, so it emits an unboxed `HashSet<S>` field and fails rustc with `E0369`.
- **Test coverage of the registry site is indirect.** User-defined classes take the fallback path, so the new `test_empty_recursive_constructor_argument_uses_box_default` pins the fallback site; the registry site is covered only by shared-helper construction plus demo-004's stdlib `TomlValue` field-init evidence. Low risk given the single helper, but the keyword/registry argument form has no unit pin — I verified it manually (item 2 above).
- **Ledger/docs still pending.** `plans/issues/active/ad-hoc-algorithmic-full-corpus-preexisting-failures.md` Wave 10 row ends at "Opus pass 3 … zero actionable findings"; the nightly `clippy::box_default` failure, this correction, and passes 4-5 are not yet recorded, and the two review artifacts remain untracked. Consistent with pass 4 (which also reviewed an uncommitted code-only response), so not counted; it must land before closeout.
- `cargo clippy -p sifr_codegen --all-targets -- -D warnings` reports 14 errors, all in pre-existing files unrelated to this diff (`rust_interop_direct.rs`, `registry_core_tests.rs`, `builtin_core_methods.rs`, `expr_call_metadata.rs`, `python_interop_direct_tests.rs`, `structured_lowering_codegen_tests.rs`); none in `recursive_node_codegen_tests.rs`, and the project gate is `cargo clippy --workspace -- -D warnings` without `--all-targets`.

## SATISFIED

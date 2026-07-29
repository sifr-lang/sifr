## CHANGES REQUIRED

Three actionable findings. The implementation is functionally correct and the wave's headline claims hold up under independent verification, but the nested left-side coverage is vacuous, the recursion has a real gap against the stated contract, and the new guard helper is a partial duplicate that already misbehaves on a reachable type shape.

### What I verified as good

- **Structural-equality gate untouched.** `git diff main` touches no file under `crates/sifr_type_system/` — `check.rs` is unmodified.
- **Variable-operand retyping is structurally impossible.** `specialize_empty_list_literal` matches only `HirExpr::ListLiteral` and is a pure function that never writes to `LowerCtx`, so no binding can be retyped. The negative test at `contextual_empty_list_equality.rs:52` confirms the diagnostic.
- **Recursion is type-safe.** I probed the widening hazard directly: `ff: list[list[float]]` vs `[[], [1]]` is correctly rejected (`is_assignable_to` treats list slots invariantly), `nested: list[list[int]]` vs `[[], ["x"]]` and `[[], [1.5]]` are both rejected, and `[[], [1.0]]` specializes to `Vec<f64>`.
- **Codegen honors the concrete type across element shapes.** Emitted `Vec<Option<i64>>`, `Vec<String>`, `Vec<(i64, String)>`, `Vec<HashMap<String, i64>>`, `Vec<HashSet<i64>>`; `ir_imports.rs:114,303` walk `RustStmt::Let{ty}` and `RustExpr::Block`, so the `HashMap`/`HashSet` imports are collected. Unresolved empties still lower to bare `vec![]` (`leaves_and_plain_calls.rs:345-349` returns `None` from `typed_empty_list_expr`), and the `Type::Bytes` branch at line 355 is unreachable-by-construction for the new path, so it is preserved.
- **Symmetry works at runtime.** `[[], [1]] != nested` emits the specialization block and runs green; the ordering at `expression_operators.rs:602-605` is safe in both literal/literal directions (a concrete literal can never be re-specialized by an `Any`-bearing expected type, because `type_contains_unknown_or_any(expected_element)` bails first).
- **Six corpus fixtures.** All six `check` clean, and all six `build`+`run` green in package context (`0094`, `0144`, `0145`, `0442`, `1203`, `1489`). `1489:134` is the genuine nested case (`== [[], [0, 1, 2, 3]]`).
- **Gates.** `cargo fmt --check` clean; `cargo clippy --workspace -- -D warnings` clean; `cargo test --workspace -- --skip test_e2e_pass` 0 failures across 60 result lines; `run_e2e_pass.sh` **676/676 pass**; file-size guardrail PASS (limit 900; largest touched file is `leaves_and_plain_calls.rs` at 881); HIR maintainability guardrail PASS.
- **Scope hygiene.** Nothing staged. The `third_party/ruff` and `leetcode` gitlinks are identical to `main`; both submodules are dirty only from untracked `.DS_Store` files. No unrelated changes in the diff.
- **Ledger.** The row at `plans/issues/.../ad-hoc-algorithmic-full-corpus-preexisting-failures.md:301` does not overstate: status is honestly "review and PR pending", the six-fixture check/build/run claim is true, and "Waves 3-8" matches the remaining wave list (item 9 is the separately-rowed closeout).

One caveat on validation: `scripts/run_all_tests.sh --profile create-pr` exited 124 on my first run, but purely on a **timing** budget (`rust_interop_checks 15506ms > 10000ms`) while my own compiler builds were loading the machine — every area reported `failures=0`. A clean re-run puts that step at 8497ms (pass) and has passed every step through `generated_code_quality_checks`; it is still finishing its tail steps. Nothing in either run is a functional failure.

---

### Finding 1 — the nested **left-side** case is vacuous; no test exercises it

`crates/sifr_lowering/src/lower/expressions_tests/contextual_empty_list_equality.rs:15` (and assertions at `:31-33`), plus `crates/sifr/tests/e2e/pass/contextual_empty_list_equality.sifr:11`.

Both use `[[1], []] == nested`. Sifr infers a list literal's element type from the first element, so `[[1], []]` is **already** `list[list[int]]` and `try_specialize_empty_list_literal` bails immediately at `contextual_list_literal_specialization.rs:23` on `!type_contains_unknown_or_any(ty)`.

Two independent proofs:
- `sifr emit` of the e2e fixture renders lines 10-11 as plain `assert!(nested == vec![vec![1_i64], vec![]]);` / `assert!(vec![vec![1_i64], vec![]] == nested);` — **no** `__sifr_empty_list_literal` block — while line 12 (`nested != [[], [1]]`) does emit the block. Had the literal been specialized, its inner `[]` element would carry `list[int]` and would have emitted the block.
- `assert [[1], []] == [[1], []]` type-checks clean with no operand able to donate a concrete type, whereas `assert [[], [1]] == [[], [1]]` fails with `'list[list[Any]]' and 'list[list[Any]]'`.

So `left_nested_left.ty() == list[list[int]]` holds identically on `main`. `right_nested` (`nested == [[], [1]]`) is the only genuine nested assertion; the left-side nested path — the `left = specialize_empty_list_literal(left, right.ty())` line at `expression_operators.rs:604` — has no nested coverage at all.

**Fix.** Use an empty-leading literal on the left. In the unit test: `left_nested: bool = [[], [1]] == nested`. In the e2e fixture: `assert [[], [1]] != nested`. I verified this emits the specialization block and runs green. Also assert the inner element type, not only the outer — e.g. that the specialized operand's first element has type `list[int]` — since the outer-type assertion alone cannot distinguish specialization from ordinary inference.

### Finding 2 — recursion does not descend into an already-concrete outer literal

`crates/sifr_lowering/src/lower/contextual_list_literal_specialization.rs:23`

The bail is keyed on the literal's *summary* type. For `[[1], []]` that type is concrete (`list[list[int]]`) even though the inner `[]` element is still `list[Any]`, so the inner empty is never specialized and codegen emits an untyped `vec![]` that compiles only because rustc infers from the sibling element. That contradicts the wave contract on two counts: literal HIR is not specialized *recursively* from the concrete opposite operand, and the native compiler does not preserve a specialized concrete empty-list type for that inner literal. It is latent rather than user-visible today (the sibling always pins the element type for rustc), but it is exactly what makes Finding 1's coverage vacuous.

**Fix — and a trap to avoid.** Do not simply drop the guard. Line 46 admits un-specialized elements via `is_assignable_to`, and `Int.is_assignable_to(Float)` is `true`, so `fl: list[float]; fl == [1]` would then be retyped to `list[float]` while codegen still emits `vec![1_i64]` — a generated-Rust type error. The safe form: enter the recursive branch when the literal is non-empty and *any element subtree* contains `Unknown`/`Any`, and for elements that were not themselves specialized require exact `element.ty().resolve_alias() == expected_element.resolve_alias()` instead of `is_assignable_to`.

### Finding 3 — the new `type_contains_unknown_or_any` helpers are partial and duplicated

`crates/sifr_lowering/src/lower/contextual_list_literal_specialization.rs:4-14` and `crates/sifr_codegen/src/lower_expr.rs:33-43` (byte-identical copies in two crates).

Both handle only `List`/`Set`/`Dict`/`Tuple`, omitting `Union`, `Iterable`, `Iterator`, `Result`/`Optional` carriers, `Callable`, and `Alias` bodies — all of which the existing `contains_any` inside `Type::is_assignable_to` (`crates/sifr_type_system/src/types/type_rendering.rs:413-455`) already enumerates. This is reachable, not theoretical: for `a: list[Any | None]`, `a == []` specializes the right operand to `list[None | Any]` (the diagnostic prints *both* operands as `list[None | Any]`, while an unspecialized `[]` is `list[Any]` — confirmed by `unresolved = []` lowering to `Vec<Box<dyn Any>>`). The guard that is supposed to refuse `Any`-bearing expected types simply does not fire, and only the untouched structural-equality gate catches it. That gate does not backstop the codegen copy, which applies to *every* empty list literal, not just comparison operands.

**Fix.** Promote one canonical `Type::contains_unknown_or_any` in `sifr_type_system` covering the same variant set as `contains_any`, and call it from both `sifr_lowering` and `sifr_codegen` instead of maintaining two partial copies.

---

### Non-blocking nits

- `typed_empty_list_expr` (`lower_expr.rs:44-64`) emits a `Block` + synthetic `let` where `RustExpr::FnCall` on `Vec::<T>::new()` would be a single expression with no binding. The block renders at column 0 inside nested contexts in `sifr emit` output, which is a user-facing surface. Purely cosmetic; correctness is fine.
- The codegen test at `leaves_and_compound_tests.rs:43` covers `Type::Any` as the unresolved case but not `Type::Unknown` or a nested-`Any` element such as `list[list[Any]]`.
- `plans/reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-2-claude-opus-review-pass-1.md` is currently a 0-byte untracked placeholder — noting only so it isn't committed empty.

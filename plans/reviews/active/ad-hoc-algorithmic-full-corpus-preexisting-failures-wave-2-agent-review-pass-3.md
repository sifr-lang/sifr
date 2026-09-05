# APPROVED

Wave 2 review pass 3 on the exact pushed head `211ec32fbc90050b99cead6344ae0581a82587c3` (verified `refs/pull/3074/head`) against base `1a90170db`. **Zero blocking findings.** No files modified.

## Head/scope verification

- `gh pr view 3074` → `headRefOid = 211ec32fbc9…`, base `main`; `git ls-remote` confirms `refs/pull/3074/head` is the same SHA. Two commits: `0002bf1b1` (fix) + `211ec32fb` (ledger).
- Diff is 15 files, all wave-relevant. Nothing staged. `third_party/ruff` and the leetcode corpus gitlinks are **byte-identical** to base (`git diff 1a90170db..211ec32fb` on those paths is empty); their `M` status is untracked `.DS_Store` only.
- **Structural-equality gate untouched**: no file matching `check` is in the diff; `crates/sifr_type_system` changes are only `types/type_rendering.rs` and its test file.

## Contract verification (independently reproduced)

| Requirement | Evidence |
|---|---|
| Recursive literal-only specialization from concrete opposite operand | `contextual_list_literal_specialization.rs:4-59`; `contains_unresolved_list_literal` (`:8`) recurses through literal elements so concrete-summary outers like `[[1], []]` are entered |
| No assignability widening | `has_exact_resolved_type` (`:11-13`) uses exact `resolve_alias()` equality. Probes: `fl: list[float] == [1]` → rejected; `nf: list[list[float]] == [[], [1]]` → rejected; `xs: list[list[int]] == [[], ["x"]]` → rejected |
| Reject named `list[Any]` variables | `a: list[Any \| None] == []` → `cannot compare values without structural equality 'list[None \| Any]' and 'list[Any]'` (pass-1 finding 3 genuinely fixed); `[] == []` still rejected |
| Both operand directions, nested leading **and** trailing | `sifr emit` of the new fixture shows `__sifr_empty_list_literal` blocks on **all 8** asserts, incl. `nested == [[1], []]`, `[[1], []] == nested`, `nested != [[], [1]]`, `[[], [1]] != nested` |
| Concrete empty lists typed in generated Rust | Verified `Vec<i64>`, `Vec<Vec<i64>>`, `Vec<String>`, `Vec<(i64,String)>`, `Vec<Vec<u8>>`, `Vec<HashMap<..>>`, `Vec<HashSet<..>>`, and `Vec<T>` inside a generic fn — all build and run |
| Files < 900 lines | Guardrail PASS (2987 files); largest touched `leaves_and_plain_calls.rs` 881, `recursive_exprs.rs` 862, `type_rendering.rs` 756 |

**`is_assignable_to` refactor is semantics-preserving.** I diffed the arm sets of the deleted nested `contains_any` against `contains_dynamic_slot`: identical variant coverage, and `Self::Unknown => include_unknown` keeps `contains_any` on the old `Unknown → false` behavior.

**Gates I ran myself:** full e2e pass suite **676/676, 0 failed** (exit 0, 873s); `cargo test --workspace -- --skip test_e2e_pass` → 0 failures in every result line; `cargo clippy --workspace -- -D warnings` clean (the `--all-targets` warnings that exist are all in untouched pre-existing files); `cargo fmt --check` clean; file-size + HIR maintainability guardrails PASS; all six fixtures (`0094`, `0144`, `0145`, `0442`, `1203`, `1489`) `run` green — `1489:134` is the genuine `== [[], [0,1,2,3]]` nested-leading case.

**Extra probes beyond the prior passes:** 3-level nesting (`list[list[list[int]]]` vs `[[[]], [[1]]]` and `[[], [[1]]]`) specializes at both depths and runs correctly; chained `a == [] == b` works; alias element (`type Ints = list[int]`) matches through `resolve_alias`; `RustExpr::Block` operands are valid in `return`, `if`, `while`, ternary, f-string, dict value, list element, and call-argument positions — all built and ran.

## Non-blocking suggestions

1. **Ledger wording — `plans/issues/active/ad-hoc-algorithmic-full-corpus-preexisting-failures.md:301`.** It calls the `create-pr` profile "authoritative"; `AGENTS.md` designates the **merge** profile as the authoritative gate and create-pr as "fast signal". Separately, create-pr's e2e is manifest-selected (`verification/areas/core_language/data/create_pr_e2e_manifest.json`, 131 names) and does **not** include `contextual_empty_list_equality`, so the cited 131/131 did not exercise the wave's own fixture. The full 676-fixture suite does — and I ran it green — so the capability claim holds; only the "authoritative" label is imprecise.
2. **Naming collision — three `Unknown`/`Any` queries coexist.** New canonical `Type::contains_unknown_or_any` (`type_rendering.rs:414`) plus untouched partial `type_contains_unknown_or_any` at `container_literal_specialization.rs:8` and `nested_function_inference/expression_inference.rs:869`. Correctly out of scope for this wave, but worth a consolidation follow-up.
3. **`contains_dynamic_slot` still bottoms out on type-carrying variants** (`type_rendering.rs:418`, `_ => false`): `PythonBuffer`, `PythonDlpackTensor`, `Newtype{inner}`, `Protocol`, `Enum`, `TypeVar`. Identical to pre-existing `contains_any`, so no regression, but the new doc comment ("transitively contains … `Any` slot") slightly overstates. Note if extending: `TypeVar` must stay "concrete" — I verified generic `list[T] == []` depends on that and emits a valid `let …: Vec<T> = vec![]`.
4. **Cosmetic (pass-1 nit stands).** The emitted block renders with a column-0 `let` in `sifr emit`; `Vec::<T>::new()` would be a single expression with no binding.
5. **Small coverage gaps.** No unit test for ≥3-level nesting (verified working by probe), none pinning the `==`/`!=`-only scoping at `expression_operators.rs:602`, and annotation-driven declarations (`nested: list[list[int]] = [[1], []]`) still emit a bare inner `vec![]` — outside the wave's equality scope, relying on rustc sibling inference.
6. **Literal-vs-literal remains one-directional** (`expression_operators.rs:602-605`): in `[[], [1]] == [[1], []]` the right literal's inner empty stays untyped. Compiles and runs correctly (verified).

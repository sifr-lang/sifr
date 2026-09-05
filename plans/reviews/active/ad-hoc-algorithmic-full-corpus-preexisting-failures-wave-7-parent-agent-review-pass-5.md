**Verdict: APPROVE — 1 low-severity, non-blocking finding; zero blocking findings.**

Reviewed exact head `10a4b9bc0` vs base `6f888ed327` (PR #3086). Nothing modified: repo state is unchanged (only pre-existing submodule dirt + this pass's own artifact path untracked); all probes ran in `/tmp/w7p5`.

### Incremental fix (`10a4b9bc0`) — verified

| Claim | Result |
|---|---|
| One canonical predicate now consumes the authoritative SCC registry | ✓ `should_force_mutable_binding(ty, recursive_fields)` (`expr_call_metadata.rs:159-201`) is the single definition; its inner recursive-class test is now registry membership (`:188-191`) instead of the self-name-only structural walk, i.e. the same source (`field_analysis_helpers.rs:128-155`) that drives `Box` wrapping and the narrowing helper `option_binding_mutability.rs:27-29`. Storage and mutability can no longer disagree. |
| Both paths pass the real registry | ✓ structured: `stmt_block.rs:104` → `&self.recursive_fields`; simple: `simple_dispatch_and_bindings.rs:60-63` → `bindings.recursive_fields`, threaded from `RustEmitter` via `stmt_block_helpers.rs:489` → `try_lower_simple_stmt_with_scope_result_and_bindings`. The three remaining `&HashSet::new()` entry points (`candidate_and_validation.rs:148/168/190`, `simple_dispatch_and_bindings.rs:34`) have no production callers — `try_lower_simple_stmt_with_ctx` is `#[cfg(test)]`-imported (`lower_stmt.rs:31-32`), and `structured_lowering_codegen_tests.rs:703` pins that `try_lower_simple_stmt_with_scope(` is absent from lib sources. |
| Pass-4's `Branch`/`Leaf` `local_branch` E0596 reproducer closes | ✓ pass-4's exact program now emits `let mut local_branch: Branch = b;`, builds, and runs (prints `12`, exit 0). Previously `E0596` → `SIFR-BUILD-0005`. |
| Regression test added | ✓ `test_mutually_recursive_local_binding_is_mutable_for_child_moves` asserts both the `let mut local_branch` binding and the `local_branch.leaf.take().map(…)` extraction — mutation-coupled to the fix, not just to emission. |
| Pre-existing forced-mutability reasons intact | ✓ the `Iterator` / `JoinSet` / `__sifr_defaultdict_*` / `__next__`-protocol arms are untouched (`:196-200`) and only the recursive-class arm changed. Live probe: `let mut counts: HashMap<String, i64>` (defaultdict), `let mut it: Box<dyn Iterator<Item = i64>>` (Iterator) still emitted; the `__next__`-protocol probe compiles and runs (`1 / 6 / 3`, exit 0). |
| Focused suite | ✓ `cargo test -p sifr_codegen recursive_node` → 13/13, and the run reports `945 filtered out`, arithmetically confirming the ledger's **958** total. |
| Gitlinks / hygiene | ✓ `third_party/ruff` identical to base (`e024f2a48`); leetcode advances `d50fa7350` → `9d71595347a369ef3a4f8d90a0a01508b591369a`; submodule dirt is untracked artifacts only. Incremental diff contains no fallback, waiver, `#[ignore]`, baseline, or new `unwrap`/`expect`. |

### Ledger accuracy (`plans/issues/active/ad-hoc-algorithmic-full-corpus-preexisting-failures.md:333`) — verified

- Pass 2: "returned only an interim progress sentence … neither is approval evidence and both unusable artifacts were discarded" ✓ (no pass-2 `.md` exists; only the `.agent.log`).
- Pass 3: "exceeded the 40-minute reviewer bound before returning `Execution error`" ✓ recorded as non-evidence, artifact discarded.
- Pass 4: "independently verified all six pass-1 corrections and approved them, then identified one low-severity pre-existing SCC mismatch in forced local mutability; the canonical predicate now consumes the authoritative recursive-field registry too, with a mutually recursive local regression test" ✓ matches the pass-4 artifact and this commit exactly.
- 958 codegen tests ✓ (confirmed above).

### Findings

**1. Low (non-blocking) — `class_has_recursive_option_field` no longer checks the Option shape, so forced-`mut` now fires for classes whose only recursive field is non-optional; the name is also now a misnomer**
`crates/sifr_codegen/src/stmt_support_emitter/expr_call_metadata.rs:183` (declaration), `:188-191` (registry membership test).
The registry is populated for *any* field satisfying `type_references_any_class` (`field_analysis_helpers.rs:144-149`), not just `T | None` unions, so the predicate is strictly broader than the structural version it replaces. Reproduced: `class Tree { value: int; children: list[Tree] }` with `local_tree: Tree = t` used read-only now emits `let mut local_tree: Tree = t;` and rustc warns `variable does not need to be mutable`; at base the field is not a `Union`, so no `mut` was emitted. Impact is warning-only — `unused_mut` is allowlisted (`verification/areas/generated_code_quality/generated_code_quality.py:114`) and it is the same type-based-not-use-based over-breadth already accepted in pass-1 finding 3, extended one shape further. *Suggested follow-up (not required for this PR):* rename to `class_has_recursive_field`, or gate on the option/`.take()`-eligible field shape.

### Non-actionable observations
- That `Tree` probe also fails with a raw `E0308` (`expected Box<Vec<Tree>>, found Vec<_>`) at the constructor call — a pre-existing recursive non-optional-field coercion leak, decided by the same unchanged registry/boxing code at base; unrelated to this diff and arguably Wave 8 territory.
- A `defaultdict` probe writing `counts["a"] = counts["a"] + 1` in one statement leaks raw `E0499`; splitting the read out compiles and runs. Untouched by this diff (defaultdict arm unchanged).
- `Counter2` (a `__next__`-protocol class) binds immutably at the local-let site in my probe yet iterates correctly; behavior identical at base since only the recursive arm changed.
- Per instruction, no corpus/demo/e2e/workspace sweeps were run; I executed only the focused `recursive_node` test file plus the prior reproducer and three small `/tmp` probes.

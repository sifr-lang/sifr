## Verdict: **APPROVE** — all six pass-1 findings closed; 1 non-blocking finding (Low, pre-existing, out of scope)

Reviewed exact head `a4b2feaf5` vs base `6f888ed32` (PR #3086). No files, branches, or PR state modified; probes ran in `/tmp/w7p4`.

### Pass-1 finding closure

| # | Pass-1 finding | Status |
|---|---|---|
| 1 | SCC/mutually-recursive owned options missed → raw rustc `E0596` | **Closed.** New `option_binding_mutability.rs:1-48` decides from the emitter's SCC-derived `recursive_fields` (`field_analysis_helpers.rs:128-155`) — the same registry that drives `Box` wrapping, so mutability and storage can no longer disagree. `SimpleStmtBindings` gains `recursive_fields` (`candidate_and_validation.rs:129`) and all six emitter call sites pass `&self.recursive_fields`. Pass-1's exact `Branch`/`Leaf` reproducer now builds and runs (`12`, exit 0); previously `E0596` → `SIFR-BUILD-0005`. |
| 2 | Two divergent predicates / precedence split | **Closed.** Both paths call the one helper; `option_binding_requires_mut_for_ir` deleted. Precedence is now uniformly borrow-exclusion → `mutated_vars` → type. The swap is safe on the structured path: borrowed/mut-borrowed narrowing lowers through `option_binding_value_expr_for_ir`'s `.as_ref()` (`condition_lowering.rs:129-142`), so the binding is `&T` and `mut` could never have enabled mutation. Third `should_force_mutable_binding` copy removed — one definition remains (`expr_call_metadata.rs:159`), used by both lowering paths. |
| 3 | Ledger overstated use-site precision | **Closed.** Wave 7 row now reads "emitting mutable Rust bindings for owned recursive class values so later child extraction can use `.take()`" — no false "only for … whose child extraction uses `.take()`" claim. |
| 4 | Nested params missed the Copy filter | **Closed.** `simple_dispatch_and_bindings.rs:505-522` now requires `ty.ownership() != OwnershipKind::Copy` on both sets, matching `function_like_lowering.rs`/`class_method_emitter.rs`; pinned by `test_nested_copy_parameter_is_not_registered_as_borrowed`. |
| 5 | Only one narrowing shape pinned | **Closed.** `test_recursive_option_mutability_covers_simple_narrowing_shapes` asserts if-let, and-chain, or-tuple let-else, truthiness, nested-function, and nested-block; plus a dedicated SCC test. 12/12 in `recursive_node_codegen_tests` pass on this head. |
| 6 | Nested blocks lowered blind | **Closed.** `try_lower_simple_stmt_block` takes the full `SimpleStmtBindings` (`try_tuple_flow.rs:250`); every caller in `condition_lowering.rs`, `loop_lowering.rs`, `with_yield_and_match.rs` threads it. `try_lower_simple_stmt_with_ctx` (the `HashSet::new()` shim) is now `#[cfg(test)]`; all remaining `&HashSet::new()` entry points are test-only. Nested-block reproducer builds and runs (`3`, exit 0). |

Also confirmed as requested: forced-let mutability duplicate consolidated to a single definition; `simple_dispatch_and_bindings.rs` is exactly **837** lines; new module is 48 lines.

### Requirements re-verified
- Gitlink = `9d71595347a369ef3a4f8d90a0a01508b591369a`; corpus diff vs `d50fa7350` is exactly 2 files / 2 lines (`own mut l1/l2`, `own mut head`). Helpers and all Python siblings untouched; `third_party/ruff` unchanged from base.
- No fallback, suppression, waiver, `#[ignore]`, baseline, or new `unwrap`/`expect` anywhere in the crate diff.
- Per instruction, corpus/demo/e2e/workspace sweeps not re-run; only the two prior reproducers plus the focused codegen test file were executed.

### Actionable findings (1, non-blocking)

**Low — SCC-blind forced-let mutability leaks raw rustc `E0596` on plain locals; not recorded in the ledger**
`crates/sifr_codegen/src/stmt_support_emitter/expr_call_metadata.rs:180` (`class_has_recursive_option_field`, self-name-only) vs the SCC-driven `.take()` extraction.

```
def sumTop(own b: Branch) -> int:
    local_branch: Branch = b
    next_leaf: Leaf | None = local_branch.leaf     # emits .take()
```
```
error[E0596]: cannot borrow `local_branch.leaf` as mutable, as `local_branch` is not declared as mutable
help: consider changing this to be mutable:  let mut local_branch: Branch = b;
error[SIFR-BUILD-0005]
```
Pre-existing and unchanged by this diff — the take-extraction path and this predicate are byte-identical to base, so `6f888ed32` fails the same way; the wave fixed the *narrowing* path only, and `test_local_recursive_node_binding_is_mutable_for_child_moves` covers only the self-recursive case. Not a regression and outside the PR's stated scope, so it does not block. *Suggested follow-up:* have `should_force_mutable_binding` consult `recursive_fields` (or record this explicitly in the ledger's pre-existing surface with an owning wave), per pass-1's deferral instruction.

### Non-actionable observations
- Simple-path narrowing no longer forces `mut` for `Iterator`/`JoinSet`/`__sifr_defaultdict_*`/`__next__`-protocol option inners. That is intentional convergence on the structured path and matches base behavior, so it is not a regression; the general forced-let path still handles those types.
- `register_external_class_fields` (`field_analysis_helpers.rs:158-188`) models imported recursive metadata as self/source-pair only, with an in-code note that mutually recursive imports should graduate to full SCC analysis. Sufficient for the corpus `ListNode` helper; worth knowing if imported SCC classes appear later.

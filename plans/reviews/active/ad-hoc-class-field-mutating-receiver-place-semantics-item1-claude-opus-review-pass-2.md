# Review Pass 2 — Item 1 (canonical receiver metadata and inference)

Scope: uncommitted working tree against `HEAD` (`cad0e8aaf`). Supplied validation
evidence taken as authoritative; no builds, tests, emitters, or probes were run.
Read-only `git diff` was used to inspect the change.

## Pass-1 findings: disposition

1. **Owned-local class-field clone suppression narrowed — RESOLVED.**
   `field_and_stdlib_rewrites.rs:40-63` now evaluates the class-typed-receiver
   branch *before* the root test, so `local.helper.bump()` still suppresses the
   field clone. `expr_render_helpers/tests.rs:107-143`
   (`borrowed_class_field_receiver_from_owned_local_does_not_clone`) pins the
   shape.

2. **`arg_ranges`/`args` misalignment — RESOLVED.**
   `resolved_method_arg_ranges` (`method_call_args.rs:54-90`) now takes the
   resolved signature and `arg_count`: it walks signature params, resolves each
   to its positional arg, then its matching keyword, then falls back to the
   callee range, and `resize`/`truncate` guarantee `len == args.len()` on both
   the signature and builtin paths. The verifier asserts the invariant
   (`method_call_verifier.rs:62-71`) and
   `method_receiver_analysis_tests.rs:285-308` covers `absorb(source=stock)`
   with a defaulted trailing parameter.

3. **Non-class receiver registry too narrow — RESOLVED.**
   `receiver_convention_for_non_class_method` (`mutating_methods.rs:6-52`) now
   covers `PythonBuffer`, `PythonArrow`, `PythonDlpackTensor`,
   `AsyncGenerator.aclose`, `Task`/`BlockingTask`, `JoinSet`,
   `TaskScope`/`TaskGroup`, plus the shared collection table, with `Owned` for
   the consuming surfaces. `Type::Iterator`/`AsyncIterator` have no reachable
   method-call arm in `resolve_method_type`, and `anext`/iterator `Next` keep
   their existing non-`MethodCall` paths in `collect_mutated_vars`. The
   `FileHandle.write`/`close` asymmetry is now coherent rather than a defect:
   the same convention drives both the emitted `&self`/`&mut self` signature and
   the `let mut` decision. Registry test at `mutating_methods.rs:230-262`.

4. **Protocol conformance vs. bridge emission disagreement — RESOLVED.**
   `refresh_protocol_implementations` (`method_receiver_analysis.rs:79-101`)
   recomputes `implements_protocols` from the post-inference class types, and
   `protocol_bridge_emitter.rs:20-27` now takes the bridge receiver from the
   *protocol* method (`&mut self` bridge legally reborrows into a `&self`
   implementation). Pinned by `receiver_codegen_tests.rs:106-143` and
   `method_receiver_analysis_tests.rs:220-251`.

5. **Invariant check was not a post-pass; reachable assert — RESOLVED.**
   `method_call_verifier::verify_module_method_calls` is a real pre-
   `LoweringResult` pass over every module function, class method, and operator
   impl (`mod_impl.rs:821-841`), checking convention presence, range alignment,
   and convention-vs-signature agreement, with compiler-authored malformed-HIR
   tests (`method_call_verifier.rs:94-191`). The `unwrap_or_else(|| assert!(…))`
   is gone; `instance.static_helper()` is now covered by
   `method_receiver_analysis_tests.rs:345-367`.

7. **Snapshot/receiver-metadata coverage — RESOLVED.**
   `hir_lowering_snapshot_matrix.json` gains
   `method_receiver_conventions_and_source_ranges` (schema v2, `binding_id`,
   `receiver`, `receiver_convention`, and all three source ranges), and
   `method_receiver_analysis_tests.rs` adds delegation fixed point, inheritance
   origin, shared-receiver non-escalation, generic specialization, protocol
   declaration, membership refresh, builtin conventions, keyword/default range
   alignment, and an `Owned` declaration-first case.

9. **`binding_id` universality — RESOLVED.** `module_constants_lowering.rs:219`
   now resolves the id. The only remaining production `None` is a function-value
   reference (`core_and_calls.rs:231`), which is not a storage binding.

10. **Named decomposition — RESOLVED for the grown file.**
    `recursive_method_calls.rs` extracts the recursive registry receiver/arg
    lowering; `while_loop.rs`, `method_calls.rs`, `literals.rs`,
    `class_type_helpers.rs`, `protocol_bridge_emitter.rs`, and
    `hir_snapshot_expr_projection.rs` also landed. All hand-maintained files
    inspected are under 900 (max 882, `mutability_and_clone_rewrites.rs`,
    untouched).

6 and 8 are carried, not closed, and remain acceptable for Item 1:
`rust_receiver_param` (`class_method_emitter.rs:12-15`) still `panic!`s on a
missing convention — every call site is now guarded to `Regular` non-`new`
methods or protocol methods, and every lowering path sets `Some` there, but no
lowering invariant or test proves `None` unreachable. Residual literal-`"self"`
keying persists where HIR carries no binding id (`class_semantics.rs:12-24`,
`helpers_impl.rs:638-651`, `field_and_stdlib_rewrites.rs:57`); the receiver-root
test that Item 2 depends on already uses `BindingKind::Receiver`
(`method_receiver_analysis.rs:358-372`).

## New observations (none blocking)

- **Invariant reporting deviates from the plan's stated ownership.** Plan §1
  requires an `assert!` with `sifr_driver::diagnostics` owning the rendering of
  `SIFR-INTERNAL-0001`; `mod_impl.rs:828-841` emits `INTERNAL_COMPILER_PANIC`
  directly from lowering. This is the safer behavior (no panic, precise range)
  and the code is already used across crates, but the plan text should be
  amended rather than left contradicted.
- **`body_contains_receiver_mutation` does not descend into
  `HirStmt::NestedFunction`** (`class_semantics.rs:8-72`), where the replaced
  `body_contains_field_assign_codegen` used `INCLUDE_NESTED_FUNCTIONS`. A bare
  `self.x = …` inside a nested function in a method would no longer force
  `&mut self`. No in-tree fixture nests a `def` inside a class method, and
  expression-level facts *are* collected through nested functions
  (`type_visit.rs:529`), so this is a latent gap rather than a live regression.
- **Clone suppression widened for borrowed-parameter roots.** With the
  `MUTATING_METHODS` name gate removed, a shared read such as
  `param.items.len()` on a `Type::List` field of a borrowed parameter now
  suppresses the field clone where it previously cloned. Strictly less cloning,
  but an unclaimed Item 1 behavior change with no direct test.
- **Symmetric narrowing for `Owned`/`None` conventions.** The gate returns
  `false` for `Owned`, so `self.pyobj.consume()` (a Python consuming method on a
  class field) now emits a field clone instead of suppressing. That shape was
  already semantically wrong and Item 2 owns it; buffer/arrow/dlpack `release`
  and Rust opaque cleanup all reject non-`Name` receivers at check time, so the
  reachable surface is small.
- **Protocol membership rule broadened beyond receivers.**
  `refresh_protocol_implementations` replaces the previous *name-only* rule
  (`class_body_lowering.rs:794-810`) with full structural `is_assignable_to`, so
  membership can now also shrink for param/return mismatches or a
  `@staticmethod` satisfying a protocol method name. The direction is safe
  (those bridges could not compile) and the receiver case is tested, but the
  wider rule change is neither noted in the plan nor covered by a test.
- **`field_and_stdlib_rewrites.rs` grew 869 → 870** without the plan-mandated
  extraction of field-storage re-rooting helpers, which Item 2 is supposed to
  reuse. Cap compliance is fine; the sequencing rule was not honored.
- **`body_contains_field_assign_codegen` survives as test-only code**
  (`helpers_impl.rs:594`, referenced solely by `helpers/tests.rs:518`, kept
  warn-free by the crate-level `allow(dead_code)`); plan §5 asks for its
  deletion.

## Scope

Item 1's declared boundaries hold: `pending_self_field_clone_suppression` and
both `*_needs_field_clone_suppression` helpers are retained at every site, no
`Place`/place emitter appears, and no `SIFR-OWN-0014`/`SIFR-PROTO-0005`/
`SIFR-PROTO-0006` work leaked forward. Cross-module receiver propagation is
carried through `bootstrap.rs:643-663`, `FunctionType.receiver` participates in
`union_identity`, and `substitute_type_vars` preserves it. Ephemeral
classification is applied at every iteration/comprehension/match-capture site
while `with`, exception, tuple-unpack, and chained-assignment targets stay
stable; because no consumer reads `BindingKind` beyond `is_parameter_binding`,
`is_inferred_local_binding`, and the new receiver-root test, the
reclassification is inert today, exactly as planned.

SATISFIED

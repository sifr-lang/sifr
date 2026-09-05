I completed an independent review of the full working-tree delta (150 changed/untracked paths) against `b3495318d`, re-read the issue plan and pass 1, read every new/modified lowering, codegen, diagnostics, fixture, and docs path, and ran read-only reproductions with a freshly built `target/debug/sifr`.

## Pass-1 remediation verification

| Claim | Verdict | Evidence |
|---|---|---|
| 1. Shared structural borrow, value reads still clone | **Confirmed** | `emit_shared_receiver_path` (`place_emitter.rs:31-40`) wired at `operator_rewrites.rs:132` and `place_emitter.rs:113-118,174-176`. Emitted: `self.items.len() as i64`, `self.lookup.get((key).as_str()).cloned()`, `let __sifr_index_list = &self.items`, `store.items.len() as i64`; `value = self.items` still `self.items.clone()`. Test restored to its original name with expanded negative guards (`collections_and_stdlib_codegen_tests.rs:4-44`). |
| 2. PROTO-0006 after convergence, incl. indirect | **Confirmed** | Fixed methods now seed `fixed_receivers` *and* `dependencies` (`method_receiver_analysis.rs:191-204`); validation runs post-fixed-point (`:61`). Reproduced: direct `self.bump()` in `__eq__`, delegated `self.helper.bump()` in `__str__`, 2-hop `self.outer()→inner()→helper.bump()` in `__lt__`, and inherited `self.touch()` in `__repr__` all report SIFR-PROTO-0006 at check time. |
| 3. OwnedTemporary semantic, conditionals rejected | **Confirmed** | `is_owned_temporary` recurses through `IfExpr` branches (`method_receiver_places.rs:543-547`). `(a if flag else b).bump()` → SIFR-OWN-0014. Verifier has two independent malformed-target tests (`method_call_verifier.rs:398-457`). |
| 4. Owned rvalues accepted | **Confirmed** (one gap, finding C) | `split(",").pop()`, `copy().pop()`, `(a+b).pop()`, `list(v).pop()`, comprehension `.pop()` all accepted; fixture covers all five plus fresh-branch conditional. |
| 5. Merge profile | **Not verifiable / not green** | No local gate artifact exists. Representative performance failed by the team's own account. |
| 6. Protocol conformance traverses operator_impls | **Confirmed** | `method_receiver_analysis.rs:260-264`. |
| 7. `Borrow { mutable: true }` restored | **Confirmed** | `flow_graph/effects.rs:518-531` via `argument_effects`, driven by checked `MutableArgumentTarget::Place`, applied to `Call`, `MethodCall`, `IteratorCall`. |
| 8. Optimizer table narrowed | **Partial** (finding F) | Table reduced to 12 names, protection test added — but `pop/insert/remove/clear/extend/push/take` are still source-visible lowered names, and genuinely compiler-generated names were dropped. |
| 9. Issue/ledger/architecture updated | **Confirmed** | Issue status + ledger entry present; pass-1 file now 11.6 KB; `internal_docs/architecture.md:446-483`. |
| 10. `open_write.sifr` restored | **Confirmed** | Unmodified vs base; `with` coverage lives in the new contexts fixture. |

Independent gates I ran: `cargo test -p sifr_lowering` 886 passed / 1 ignored; `cargo test -p sifr_codegen` 931 passed; **E2E pass suite 679/679**; E2E fail suite 556 fixtures passed; `cargo clippy --workspace -- -D warnings`, `cargo fmt --check`, `git diff --check`, docs error-link, HIR, and file-size guardrails all pass.

## Actionable findings

**1. (High) A shared receiver overlapping a mutable argument place is never checked, and leaks rustc `E0502`.**
`validate_call_overlaps` (`crates/sifr_lowering/src/lower/method_receiver_places.rs:188-246`) compares mutable-argument places only against the other **arguments** (`:218`); the receiver `object` is not passed into the function at all, and the receiver-side loops (`:197-210`) only run for `MutableReceiverTarget::Place`/`SpecializedIndexedStorage` — i.e. only for `MutableBorrow` receivers. A `SharedBorrow` receiver that overlaps a `mut` argument therefore escapes:

```python
def observe(self, mut other: Crate) -> int:   # &self + &mut Crate
    other.items.append(1)
    return len(self.items)
...
self.stock.observe(self.stock)   # check: "no errors found"
c.observe(c)                     # check: "no errors found"
```
Both emit `self.stock.observe(&mut self.stock)` / `c.observe(&mut c)` and fail `cargo build` with `E0502: cannot borrow ... as mutable because it is also borrowed as immutable`. The bare-name form is the "bare-root" case the plan says is closed, and the field form is the "field-projection" case. This violates acceptance criterion *"rejects every overlapping receiver/argument/read/move shape at Sifr check time"*, §4's *"closes receiver-versus-argument … with one rule"*, and the no-leaked-rustc-error core guarantee. The mutable-receiver control (`self.stock.absorb(self.stock)` → SIFR-OWN-0002) and the regular-function control (`f(owner.helper, owner.helper)` → SIFR-OWN-0002) both work, so the hole is narrow but real, and no fixture covers this row. Pre-existing on base, but Item 2 is the milestone chartered to close it.

**2. (Medium) Module-level bindings are accepted as mutable place roots, but each reference is re-materialized as a fresh temporary — silent lost mutation.**
`prove_mutable_place` accepts any `BindingKind::Local` root (`method_receiver_places.rs:352`), and module-level names are `Local`. Codegen substitutes every reference with `__const_NAME()` (`crates/sifr_codegen/src/lower_item/module_constants.rs:313,358`, `module_constants.rs:60`). Reproduced end-to-end:

```python
GLOBAL_HELPER: Helper = Helper()
def use_global() -> None: GLOBAL_HELPER.bump()
# emit: __const_GLOBAL_HELPER().bump();
# run:  prints []   <- mutation discarded

COUNTS: list[int] = []
def add() -> None: COUNTS.append(1)
# emit: __const_COUNTS().push(1_i64);   run: prints 0
```
`check` reports no errors. This is exactly the defect class the issue exists to close ("a program can compile and silently lose a source-visible mutation"), reached through an *accepted* checked place. It is pre-existing, but it contradicts the acceptance criterion *"Direct, nested, inherited, local, `self`, mutable-borrowed, and `own mut` field receiver mutations are observable after the call"*. Either module-level roots must be excluded from `prove_mutable_place` (SIFR-OWN-0014 or a dedicated immutability diagnostic), or the place emitter must reject the `__const_` rewrite.

**3. (Low/Medium) `Slice` and `WalrusExpr` receivers are neither places nor owned temporaries, so slice-rvalue mutation regressed to SIFR-OWN-0014.**
`is_owned_temporary` returns `false` for `Slice`/`WalrusExpr` (`method_receiver_places.rs:581-587`) while `extract_place` also rejects them (`:430`). `values[1:].pop()` → `SIFR-OWN-0014: mutable method receiver list[int] is not a supported storage place`, although a Sifr slice is a fresh owned list — semantically the same "fresh list temporary" §2.1 says must remain valid. Plan §3 does list "slice projections" among unsupported *places*, so a deliberate rejection is defensible; what is missing is the explicit decision plus a fixture. Nothing in `crates/`, `demos/`, or `stdlib/` uses this shape, hence the low severity.

**4. (Low) Unreachable `"pop"` arm retained, and `architecture.md` now documents it as supported.**
`specialized_indexed_storage_base` accepts `"pop"` on plain `Type::Dict` with a list value (`method_receiver_places.rs:389-398`), but plain dict indexing types as `T | None`, so both `d["a"].append(3)` and `d["a"].pop()` fail earlier with `SIFR-STDLIB-0001` (reproduced). The new architecture bullet asserts *"zero-argument `bucket[key].pop()` lowering … are compiler-owned exceptions"* — the documented capability does not exist for the plain-dict shape. This was pass-1 non-blocking suggestion #1, now also a doc-accuracy defect.

**5. (Low) `body_contains_field_assign_codegen` was not deleted.**
`crates/sifr_codegen/src/helpers/helpers_impl.rs:594` survives, with its only remaining reference being its own test (`helpers/tests.rs:484-522`). Plan §5: *"Delete the independent `body_contains_field_assign_codegen` and hard-coded immutable-self decisions at those sites."* No signature site consumes it any more, so this is dead code plus a test that pins dead behaviour.

**6. (Low) Optimizer narrowing went the wrong direction, and the pass is only reachable from test-module codegen.**
`remove_unneeded_mutability_in_items` has exactly one call site, `entrypoints.rs:159` in `generate_rust_test`; the production path (`generate_rust_with_stdlib`) never runs it. Within that one path, `COMPILER_GENERATED_MUTATING_METHODS` *kept* seven source-visible lowered names (`pop`, `insert`, `remove`, `clear`, `extend`, `push`, `take`) and *dropped* names that genuinely lack HIR provenance (`__sifr_join_all`, `__sifr_spawn_result`, `__aenter__`, `__aexit__`, `write_all`, `flush`, `seek`, `try_wait`, `writerow`, `anext`, `aclose`, `read_string`, …) — the inverse of §6's intent. I could not reproduce a resulting `E0596` (`sifr test` on iterator/list fixtures passes, and an unrelated pre-existing codegen break in `sifr test` blocks the `open()`/`try` shape), so this is Low, but the narrowing does not yet satisfy *"limited to compiler-generated Rust patterns that lack HIR provenance."*

**7. (Low, tracking) The authoritative merge gate is still not green.**
No local gate artifact exists in the tree to corroborate the run. The described outcome — every lane green up to representative performance, performance failing on calibrated medians — is not a passing `scripts/run_all_tests.sh`. I acknowledge the controlled detached base worktree at exact `b3495318dc` reproducing the same check/diagnostic median failures more severely (base ~1385-1388 ms vs implementation best ~1334-1363 ms) as credible evidence of host/baseline noise rather than an Item-2 regression, and I note that no budget, baseline, or waiver was changed. That is the right handling, but closure still requires one uncontended full-gate run that actually passes end to end.

## Observation requiring triage (not attributed to this delta)

The E2E fail lane prints a panic-hook message while still passing:

```
thread 'e2e_support::e2e_entrypoints::test_e2e_fail' panicked at crates/sifr_lowering/src/cfg.rs:300:9:
internal compiler error: invalid control-flow graph: branch terminator in block 2 is incomplete (1 target(s))
  556 fail tests completed
```

`crates/sifr_driver/src/diagnostics.rs:406-418` converts such panics into `INTERNAL_COMPILER_PANIC`, so the fixture's own expected code still matches and the lane reports green — the panic is masked. I could not reproduce it through `sifr check` on any of the 556 fixtures (including all 19 new ones), and the `cfg.rs` delta is test-constructor-only, so I cannot attribute it to Item 2. It should be attributed and either fixed or explicitly recorded as pre-existing before closure, since a green lane currently hides an internal compiler panic.

## Confirmed non-issues from pass 1's suggestion list

- `report_legacy_name_conflicts` widening: `bump(x, x)` with `mut a: int, b: int` still checks clean — scalar value-semantic params are not `is_mut_borrow`, so no regression.
- Suppression counter and all three helpers: zero references remain anywhere in `crates/`, `docs/`, `internal_docs/`.
- `class_method_mut_borrowed_field_argument.sifr` now asserts both receiver-side (`helper.items == [5]`) and argument-side (`stock.items == [.., 7]`) mutation on the same call, as the plan requires.
- `super().bump()` through an inherited mutable receiver mutates the original storage (verified by running).

## What blocks closure

Finding 1 leaks a rustc borrow error from a same-call overlap shape the Item 2 contract says is closed by one rule; finding 2 leaves a silent lost mutation reachable through an *accepted* checked place. Both are in-scope for the "no silent clone, unchecked receiver path" final criterion. Findings 3-6 are smaller contract/doc/dead-code deviations. A successful full `scripts/run_all_tests.sh` remains required for final closure.

NOT SATISFIED

I completed an independent review of the full working-tree delta (149 changed/untracked paths) against base `b3495318d`, read the issue plan, the new lowering/codegen modules, all new fixtures, diagnostics/docs/baseline governance, and ran read-only reproductions with a locally built `target/debug/sifr`.

The core defect **is** fixed: `self.helper.bump()`, `self.mid.inner.bump()`, inherited/`mut`/`own mut`/owned-local field receivers, generic receivers, expression/print/loop/return/f-string/dict-literal contexts, `with`/except/tuple-unpack/chained locals, protocol `own mut` conformance, and the `dict[K, list[V]]` indexed exception all emit the original storage with no clone and observably mutate. The suppression counter and its three helpers are gone from the tree. Diagnostics registry/catalog/baseline/docs governance for `SIFR-OWN-0014`/`PROTO-0005`/`PROTO-0006` is complete and the three guardrail scripts plus `check_docs_error_code_links.py` pass locally.

However, there are material findings.

## Actionable findings

**1. (High) Removing the suppression counter introduced pervasive deep clones on read-only `self`/borrowed-parameter field reads.**
`crates/sifr_codegen/src/expr_render_helpers/field_and_stdlib_rewrites.rs:111` now computes `needs_clone` unconditionally, and `operator_rewrites.rs:127-130` lost its arming for `self.field[...]`. Reproduced:

```
fn size(&self) -> i64 { self.items.clone().len() as i64 }        // len(self.items)
fn find(&self, key:&String) -> Option<i64> { self.lookup.clone().get((key).as_str()).cloned() }
fn first(&self) -> Option<i64> { let __sifr_index_list = &self.items.clone(); ... }  // self.items[0]
fn size(store: &Store) -> i64 { store.items.clone().len() as i64 } // borrowed param
```

Every read-only collection/string access through `self` or a borrowed parameter now deep-clones the container — an O(n) clone to call `.len()`. This is not incidental: the pre-existing regression test `method_calls_on_self_collection_fields_do_not_clone_for_read_only_receivers` was renamed to `read_only_receiver_calls_keep_ordinary_field_value_semantics` and its assertions **inverted** (`collections_and_stdlib_codegen_tests.rs:26-33,54`: `assert!(generated.contains("self.items.clone().len() as i64"))`, and the two `!contains("...clone()...")` guards deleted). That test's original assertions are proof the base tree did not clone here. This violates the plan's goal "Preserve existing value/clone behavior for ordinary field reads and shared-receiver method calls" and the acceptance criterion "ordinary field reads retain existing clone semantics." Item 2 was supposed to replace the counter for the *mutable* path only; the *shared* read path needs an equivalent non-cloning treatment (or the clone must be suppressed structurally), not accepted snapshots.

**2. (High) `SIFR-PROTO-0006` misses indirect receiver mutation, leaking rustc `E0596`.**
`method_receiver_analysis.rs:168-186` decides PROTO-0006 inside `collect_method_facts`, i.e. in the *seed* pass, using `HirExpr::MethodCall.receiver_convention` values that for user classes are not resolved until after the fixed point. So `call_mutates_receiver` is effectively always false for user-class method calls. Both of these compile clean (`check` → "no errors found") and then fail in `cargo build`:

```python
class Counter:
    value: int
    def bump(self) -> None: self.value += 1
    def __eq__(self, other: Counter) -> bool:
        self.bump()                      # -> fn eq(&self,..) { self.bump(); }  E0596
        return self.value == other.value
```
```python
class Counter:
    helper: Helper
    def __str__(self) -> str:
        self.helper.bump()               # -> fn fmt(&self,..) { self.helper.bump(); }  E0596
        return "counter"
```

`method_receiver_places.rs:26-29` explicitly relies on PROTO-0006 having rejected these bodies (`allow_fixed_receiver` lets a `Receiver` root prove a place regardless of the final convention, `method_receiver_places.rs:366-374`), so the place validator actively emits the non-conforming `&mut` projection instead of catching it. This violates "A fixed-trait dunder body that attempts any receiver mutation is rejected during receiver analysis with `SIFR-PROTO-0006`; this issue does not generate a non-conforming `&mut self` trait signature", the acceptance criterion on fixed-receiver dunders, and Sifr's no-leaked-rustc-error guarantee. The two new fixtures only cover *direct* field mutation, which `body_contains_receiver_mutation` catches — the delegated form is the untested bypass. Fix requires deciding PROTO-0006 **after** the fixed point converges (or seeding from the `mutable` set rather than from call annotations).

**3. (High) `MutableReceiverTarget::OwnedTemporary` is proven by expression shape only, and is unsound for `IfExpr`.**
`method_receiver_places.rs:541-551` classifies any `IfExpr` as an owned temporary without checking that its branches are rvalues. Reproduced:

```python
a: Helper = Helper([]); b: Helper = Helper([]); flag: bool = True
(a if flag else b).bump()
assert a.items == [1]
```
→ `check` clean, emitted `(if flag { a } else { b }).bump();` → rustc `E0382` "value moved here … value borrowed here after move". When the moved local is not used afterwards it instead compiles and silently discards the mutation — the exact defect class this issue exists to close. `method_call_verifier.rs:263` re-uses the same `is_owned_temporary` predicate, so the fail-closed verifier cannot detect a forged/mismatched `OwnedTemporary`; it is circular rather than independent proof. The plan requires "explicit owned-temporary proof variants" and "no mutable call path guesses or falls back based on expression shape."

**4. (Medium) Owned temporaries produced by method calls, binary ops, comprehensions, and awaits are rejected with `SIFR-OWN-0014`.**
The `is_owned_temporary` allow-list (`method_receiver_places.rs:541-551`) covers only `Call | ConstructorCall | IfExpr | ListLiteral | SetLiteral | DictLiteral`. Reproduced rejections of previously-valid code:

```
"a,b,c".split(",").pop()   -> SIFR-OWN-0014: mutable method receiver list[str] is not a supported storage place
values.copy().pop()        -> SIFR-OWN-0014
(a + b).pop()              -> SIFR-OWN-0014
```
while `list(values).pop()` (a `Call`) is accepted. Plan §2.1: "Mutating an owned rvalue temporary, such as `Helper().bump()` or a fresh list temporary, remains valid … It is not diagnosed as an unsupported storage place." Also missing: `MethodCall`, `IntrinsicCall`, `IteratorCall`, `PythonCall`, `SuperCall`, `Await`, `FString`, `TupleLiteral`, `ListComp`/`SetComp`/`DictComp`/`GeneratorExpr`, `WalrusExpr`, `QuestionMark`, `OkWrap`/`ErrWrap`. No fixture or unit test covers any non-`Call`/non-literal temporary.

**5. (Medium) The authoritative merge gate has not been run — and it is the lane that would catch findings 1 and 4.**
Supplied evidence is `--profile create-pr` only. Comparing `verification/profiles/create-pr.json` with `merge.json`, the merge profile additionally selects `regression:{fixedbugs,crashes}`, `cpython_differential:{policy,hand_seeded_merge}`, `ecosystem_compatibility:oss-curated`, `algorithmic_compatibility:representative-subset` (create-pr only runs `profile-manifest`), `generated_code_quality:representative` (create-pr only `smoke`), `diagnostics:baselines`, `project_workspace:baselines`, `core_language:*`, `stdlib_parity`, `fuzz_property`. Plan Item 2 step 5 and the closure criteria require `scripts/run_all_tests.sh`. Treating this as satisfied on create-pr evidence is not supportable for a milestone-closure review.

**6. (Medium) `validate_protocol_receiver_conventions` skips `operator_impls`.**
`method_receiver_analysis.rs:213` iterates only `class.methods`. Fixed-trait dunders are therefore never conformance-checked against a protocol declaration, which is a second missing gate on the same surface as finding 2. No positive or negative coverage exists for a protocol method implemented by an operator/Display dunder.

**7. (Low/Medium) `FlowEffect::Borrow { mutable: true }` is no longer produced anywhere in the compiler.**
`regular_calls.rs:453-463` deleted the inline `ctx.record_flow_effect(FlowEffect::Borrow { mutable: true/false })` for `mut`/shared borrow arguments, and the HIR-derived replacement in `flow_graph/effects.rs:268-273` only ever pushes `mutable: false` (verified: no `FlowEffect::Borrow { mutable: true }` construction remains in `crates/`). Plan §2 sanctioned removing the speculative inline effects for *method-call* borrows and required the effects to be re-derived from annotated HIR; regular-call mutable-borrow fidelity was dropped instead.

**8. (Low) `COMPILER_GENERATED_MUTATING_METHODS` was relocated, not narrowed.**
`ir_optimize/compiler_generated_mutating_methods.rs` is the old `MUTATING_METHODS` list verbatim minus a duplicate `"write"`, still containing `append/pop/insert/remove/sort/extend/clear/push/take/set`, i.e. exactly the user-visible lowered method names. Plan §6 requires the table be "limited to compiler-generated Rust patterns that lack HIR provenance." (The HIR-analysis side *was* correctly consolidated onto `receiver_convention` — no name table remains there.) Effect is over-conservative `mut` retention, not miscompilation, hence low.

**9. (Low) Issue plan status, checklist, and review ledger are not updated.**
`plans/issues/active/ad-hoc-class-field-mutating-receiver-place-semantics.md` is unmodified in the delta; Item 2 remains marked "remains" with no checklist state and no ledger entry, and `plans/reviews/active/…item2-claude-opus-review-pass-1.md` is a 0-byte placeholder. Item 2 step 6 and the AGENTS.md workflow require these.

**10. (Low) Coverage substitution in `crates/sifr/tests/e2e/pass/open_write.sifr`.**
The existing plain-local idiom (`f = open(...); f.write(...); f.close()`) was replaced by the `with` form. I confirmed the plain form still checks and builds on this tree, so the edit was not required; the plan asked to *retain* the existing `open()`/`write()` idiom while adding stable-local coverage, and this drops the non-`with` file-handle receiver from the suite.

## Non-blocking suggestions

- `specialized_indexed_storage_base` (`method_receiver_places.rs:392-398`) accepts `"pop"` on plain `Type::Dict` values, but plain dict indexing types as `T | None`, so `d[k].pop()` fails earlier with `SIFR-STDLIB-0001`; the reachable `defaultdict` `pop` path is handled by a different emitter and there is no `SpecializedIndexedStorage` `pop` branch in codegen — if that arm ever became reachable it would return `Ok(None)` and drop to the legacy textual path. Consider removing the arm or adding the branch. (Architecture.md's `bucket[key].pop()` claim itself checks out for the `defaultdict` shape — verified emitting `buckets.get_mut("a").and_then(|b| b.pop())`.)
- `protected_mutable_place_roots` is a global set of bare Rust local names (`place_emitter.rs:32,66`), so protection is not function-scoped; a same-named local in a different function can retain an unnecessary `mut`.
- `report_legacy_name_conflicts` (`method_receiver_places.rs:248-310`) no longer gates on `ty.ownership() == Move`, widening the legacy bare-name conflict check to value-semantic parameters relative to the deleted block.
- The specified read-overlap rule is a real user-visible break (`stdlib/sifr/heapq.sifr` had to hoist `len(heap)` out of `_sift_down(heap, 0, len(heap))`, and `fill(v, len(v))` is now `SIFR-OWN-0002`). Correct per plan §4, but worth a release note.

NOT SATISFIED

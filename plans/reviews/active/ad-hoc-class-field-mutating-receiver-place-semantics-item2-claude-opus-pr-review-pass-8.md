I have everything needed. Here is the review.

---

## Item 2 PR Review — Class-Field Mutating Receiver Place Semantics (PR #3082), Pass 8

**Range:** `f1c34cf9aaabadda546e670fca190decc580c935` → `e174a9ec86567fb76675cf59c5e3ad56ba38bb3f` (185 files, +5918/−1061). `git rev-parse HEAD` = `e174a9ec8…`; working tree clean except my own untracked pass-8 placeholder. No files modified, no commit/push, no PR-state change. I read the phase contract, all seven prior Item 2 artifacts, the validation ledger, and the full diff. All probes ran against a release build of exact head (`cargo build --release`, exit 0) with `SIFR_SYSROOT=/private/tmp/sifr-class-field-receiver-item1`. For base comparison I used the pre-existing exact-base worktree `/private/tmp/wave5base` (`git rev-parse HEAD` = `f1c34cf9a…`) and its built binary.

The head commit `e174a9ec8` touches only `class_semantics.rs` (+3 lines), one lowering test, one fail fixture, and docs. The delta since the merge-profile-pinned commit `31af48ac8` is confined to three lowering source files:

```
$ git diff --name-only 31af48ac8 e174a9ec8 | grep -vE "^plans/|^docs/|tests\.rs$|/tests/"
crates/sifr_lowering/src/lower/classes/class_body_lowering.rs
crates/sifr_lowering/src/lower/classes/class_semantics.rs
crates/sifr_lowering/src/lower/ownership_diagnostics.rs
```

No codegen, runtime, or stdlib change since that pin.

---

## 1. Pass-7 finding 1 — repeated constructor field before storage init: **CLOSED AT THE ROOT**

### The fix is the required correction, verbatim

`crates/sifr_lowering/src/lower/classes/class_semantics.rs:116,139,145` now keeps two sets:

```rust
116  let mut explicit_initializers = HashSet::new();
...
137      if object == "self"
138          && required.contains(field)
139          && !explicit_initializers.contains(field)
140          && !body_references_receiver(&[HirStmt::Expr { expr: value.clone() }])
141      {
144          initialized.insert(field.clone());
145          explicit_initializers.insert(field.clone());
146          continue;
```

`initialized` (`:111-115`) is still seeded with same-named parameters and is what feeds the missing-field difference at `:151-154`; `explicit_initializers` gates the `continue` alone. This is exactly the separation the pass-7 correction required.

### Parity with codegen is now exact

| | lowering `class_semantics.rs:137-142` | codegen `class_method_emitter.rs:341-346` |
|---|---|---|
| receiver | `object == "self"` | `object == "self"` |
| declared field | `required.contains(field)` (own fields) | `class.fields.iter().any(…)` |
| not already an explicit init | `!explicit_initializers.contains(field)` | `!field_inits.iter().any(…)` |
| value is self-free | `!body_references_receiver(…)` | `!Self::constructor_body_references_self(…)` |

And the *supplied-storage* sets match at the materialization boundary: codegen supplies `field_inits ∪ {class.fields that are same-named params and not in field_inits}` (`:515-521`), lowering's `initialized` = `param_seeds ∪ explicit_initializers` (`:111-115,144`). The predicates themselves remain byte-identical two-visitor functions (`class_semantics.rs:77-91` vs `class_method_emitter.rs:433-447`).

### Each required behavior, independently reproduced at exact head

**The repeat before complete storage is now rejected at the repeated source statement, no rustc leak:**

```
$ sifr check dup3.sifr
error[SIFR-OWN-0014]: constructor uses self before field storage is initialized: self.items
  --> dup3.sifr:7:9
   7 |         self.count = n
     |         ^^^^^^^^^^^^^^
  = help: initialize every declared field and inherited storage before the first statement that reads or mutates self
exit=1
```

(source is pass-7's `dup3`: `self.count = 0; self.count = n; self.items = []`). Exit 1 from `check` — the E0063 pass 7 reproduced is gone.

**The first explicit initializer still wins even when the field has a same-named parameter seed, and the repeat still falls through:**

| Probe | Shape | Result at head |
|---|---|---|
| `seedrepeat.sifr` | `self.a = 0; self.a = 5; self.items = []` (param `a`) | `SIFR-OWN-0014: … self.items` at `seedrepeat.sifr:7:9` (`self.a = 5`) |
| `seedrepeat2.sifr` | `self.a = a; self.a = a + 1; self.items = []` | `SIFR-OWN-0014: … self.items` at `seedrepeat2.sifr:7:9` (`self.a = a + 1`) |
| `aug.sifr` | `self.count = n; self.count += 1; self.items = []` | `SIFR-OWN-0014: … self.items` at `aug.sifr:7:9` |

Both param-seeded repeats are correctly rejected, matching codegen: the field is already in `field_inits`, so the repeat materializes `Self { count: … }` with `items` unsupplied.

**Repeat after complete storage remains accepted, with correct source-order materialization:**

```
$ sifr run dup4.sifr    # self.count = 0; self.items = []; self.count = n
7
0
exit=0
$ sifr emit dup4.sifr
    fn new(n: i64) -> Self {
        let __sifr_field_init_0: i64 = 0_i64;
        let __sifr_field_init_1: Vec<i64> = vec![];
        let mut __sifr_self = Self { count: __sifr_field_init_0, items: __sifr_field_init_1 };
        __sifr_self.count = n;
        __sifr_self
    }
```

**`ParameterSeededOwner` and the pass-6 finding-3 shape still check and run:**

```
$ sifr run crates/sifr/tests/e2e/pass/constructor_mutating_receiver_places.sifr   # exit=0
$ sifr run crates/sifr/tests/e2e/pass/class_field_mutating_receiver_places.sifr   # exit=0
$ sifr run crates/sifr/tests/e2e/pass/class_method_mut_borrowed_field_argument.sifr # exit=0
$ sifr run items.sifr    # self.items=[]; append; append; self.a = a*2+1  →  2 / 15
```

**Full param-seed parity edges hold both directions:**

- `allseed.sifr` — all fields are same-named params, `print(self.a)` first → accepted, runs `1 / 2 / 3`.
- `partseed.sifr` — only `a` is a param, `b` unassigned → `SIFR-OWN-0014: … self.b` at `partseed.sifr:6:9`.

**NonSend, stdlib-typed, and parent constructors remain accepted:** `nonsend_bad.sifr` (`class Handle(NonSend)` with no `super().__init__`) → `no errors found`, confirming the marker parent is still excluded from the parent requirement (`class_body_lowering.rs:613-615`). `stdlibctor.sifr` (`dict[str, int]` field, mid-constructor `self.q["a"] = n` and `self.total = self.total + n`) runs `1 / 4`. `ChildOwner(BaseOwner)` in the pass fixture runs at exit 0.

**Missing-super and mid-initialization reads remain source-facing with `args.place = self`:**

```
$ sifr --diagnostic-format json check missuper.sifr
{"code": "SIFR-OWN-0014", "args": {"message": {"kind":"string","value":"constructor uses self before inherited storage is initialized; call super().__init__(...) first"}, "place": {"kind":"string","value":"self"}}, "message_template": "{message}", "severity": "Error"}
$ sifr --diagnostic-format json check midread.sifr      # self.b = self.a + 1
{"code": "SIFR-OWN-0014", "args": {… "self.b"}, "place": {"kind":"string","value":"self"}, …}
$ sifr --diagnostic-format json check dup3.sifr
{"code": "SIFR-OWN-0014", "args": {… "self.items"}, "place": {"kind":"string","value":"self"}, …}
```

All three carry the registry-declared `place` arg (`registry_entries/calls_flow_and_protocols.rs:214-224`), no `__sifr_parent`/`__sifr_` leakage, text spans on the first offending source statement.

**Span integrity under lowering expansion.** I probed the `func.body.get(gap.statement_index)` index mapping (`class_body_lowering.rs:617-620`) with constructors whose pre-offender statements could expand during lowering — a tuple unpack (`x, y = 1, 2`) and a `for` loop over a list — and both landed on the correct source statement (`spanshift.sifr:8:9` → `self.a = y + n`; `spanshift2.sifr:11:9` → `self.a = n`).

### The tightening rejects nothing previously accepted

The guard only changes behavior for a constructor with a *repeated* assignment to the same field. I scanned every `.sifr` source in the repo:

```
scanned 4073 .sifr files
('crates/sifr/tests/e2e/fail/constructor_repeated_field_before_initialization.sifr', 6, {'count': 2})
('crates/sifr/tests/e2e/pass/constructor_mutating_receiver_places.sifr', 50, {'count': 2})
total constructors with repeated field assignment: 2
```

Only the new fail fixture (intended) and `OrderedOwner` (`constructor_mutating_receiver_places.sifr:50-61`, repeat *after* complete storage — runs at exit 0). Nothing in the E2E corpora, demos, stdlib, or verification corpora is affected. Combined with the lowering-only delta above, this closes the risk of the heavy merge-profile lanes being pinned at `31af48ac8`.

### Test adequacy

`method_receiver_analysis_tests.rs:156-179` pins the code, `self.items` in the message, and `primary_range == range_for(source, "self.count = n")`, and `panic!`s if lowering succeeds — which is precisely what pass 7 observed before the fix, so the test is load-bearing. `crates/sifr/tests/e2e/fail/constructor_repeated_field_before_initialization.sifr:1` uses `# expect-error[col=9]: SIFR-OWN-0014`, matching the sibling `constructor_receiver_before_initialization.sifr:1` convention, and is one of the 562 fixtures in the corpus that `test_e2e_fail` covers.

---

## 2. Prior pass-6 findings — all remain closed

- **Finding 1 (validation counts).** Independently reproduced at exact head: `cargo test -p sifr_lowering --lib` → `922 passed; 0 failed; 1 ignored`; `cargo test -p sifr_codegen --lib` → `941 passed; 0 failed`. Both match the ledger as amended by this commit (`plans/issues/active/…-place-semantics.md:779-781`). `git diff --name-only f1c34cf9a e174a9ec8 -- verification/areas/performance` → empty; no budget, baseline, or waiver edit.
- **Finding 2 (`SIFR-OWN-0014` diagnostic).** Re-reproduced above: source-facing text, canonical `args.place = "self"`, first-offending-statement spans, NonSend exclusion honoured.
- **Finding 3 (same-named parameter seeding).** Re-reproduced above; the remediation's collateral guard deletion is now the only thing that changed, and it is repaired without regressing the original shape.

---

## 3. Independent re-inspection of the full PR — no actionable gaps

- **No silent clone.** `pending_self_field_clone_suppression`, `method_call_needs_field_clone_suppression`, `body_contains_field_assign_codegen`, `needs_field_clone_suppression` → **0 hits** across `crates/`. `sifr emit class_field_mutating_receiver_places.sifr` yields `fn bump(&mut self, …)`, `fn mutate(&mut self)`, `fn mutate_inherited(&mut self)`, `fn replace(&mut self, …)` and **zero** matches for `clone(` or `to_vec(`.
- **No unchecked receiver/place path.** The `None` / `Ok(None)` arms at `place_emitter.rs:110-111,120,138,171-172,178,196` propagate as "not emitted" rather than falling back to value lowering (`stmt_expr_method_and_question_mark.rs:45-52,178-202`, `recursive_method_calls.rs:72-96`, `collection_methods.rs:193-197`). They are unreachable by invariant, and the invariant runs in **production**, not just tests: `mod_impl.rs:830-840` calls `verify_module_method_calls` on every successfully lowered module and converts any violation into a hard `INTERNAL_COMPILER_PANIC` diagnostic before codegen. `method_call_verifier.rs:142-209` rejects a `MutableBorrow` receiver with no target, a target that fails `receiver_target_matches`, a convention mismatch against the signature, and any `mut`-borrow parameter lacking a proof slot. Violations are diagnostics, not panics — correct for a no-panic user path.
- **`ReceiverConvention` propagation.** Carried on the HIR node (`hir_nodes.rs:721`) and consumed as the sole switch in both the stmt and registry emitters (`place_emitter.rs:104-122,165-180`).
- **Overlap and owned-temporary proof.** `verify_mutable_argument_targets` (`method_call_verifier.rs:213-249`) validates `Place` targets via `expression_matches_place` and `OwnedTemporary` via `method_receiver_places::is_owned_temporary` — no self-asserted proof. The overlap fixtures (`mutable_receiver_overlapping_mut_argument`, `mutable_receiver_overlapping_shared_read`, `shared_receiver_mutable_argument_overlap`, `mutable_receiver_equal_argument_place`, `mutable_receiver_prefix_argument_read`, `mutable_receiver_nested_argument_read`, `mutable_argument_prefix_receiver`) and `owned_temporary_mutable_receivers.sifr` all pass.
- **Optimizer protected roots.** Inserted at both eligible sites (`place_emitter.rs:47,81`), consumed in test assembly (`entrypoints.rs:159`) *and* production assembly (`lib_modules_and_codegen.rs:630`), honoured for `except` bindings (`try_handlers.rs:170`). `class_method_mut_borrowed_field_argument.sifr` and `owned_temporary_mutable_receivers.sifr` compile and run at exit 0 — rustc would raise `E0596` if `mut` were stripped from a protected root.
- **Fixed trait/protocol constraints.** `SIFR-PROTO-0005`/`0006` registered (`registry.rs:164-165`) with docs pages and diagnostic-code index entries; `protocol_receiver_mutability_mismatch.sifr` (fail) and `protocol_receiver_conformance_controls.sifr` (pass, exit 0) both hold.
- **HIR effects.** `flow_graph/effects.rs` routes each `Call`/`PythonCall`/`IteratorCall` argument through `argument_effects` with its `mutable_arg_places` slot; `cfg.rs` changes are test-only field additions. Covered by the green 922-test lowering suite including `flow_graph/tests.rs`.
- **Generic functions.** `verify_module_method_calls` walks `module.functions` and `module.classes` but not `module.generic_functions`; I probed that surface (`genfn.sifr`, a generic function mutating a `Bag` parameter) and it is rejected earlier by the ownership rules with a proper source diagnostic (`SIFR-OWN-0005` at `genfn.sifr:12:5`), so no unverified place reaches codegen.
- **Internal panics.** No `unwrap`/`expect`/`panic!`/`unreachable!`/`todo!` in `method_receiver_places.rs`, `place_emitter.rs`, `method_call_verifier.rs`, or `class_semantics.rs`.
- **Gates at exact head.** `cargo fmt --check` pass; `cargo clippy --workspace --all-targets -- -D warnings` exit 0; `python3 scripts/check_hir_maintainability_guardrails.py` → PASS; `python3 scripts/check_file_size_guardrails.py` → `PASS (3054 files, limit 900 lines)`; `python3 scripts/check_docs_error_code_links.py` → passed; `git diff --check f1c34cf9a e174a9ec8` clean. `cargo test -p sifr --test e2e -- test_e2e_fail` → `1 passed` over the complete annotated corpus.
- **Submodules.** `leetcode` at `7772857c6f` (merged corpus PR #40); all others unchanged and clean.
- **Source compatibility.** The `SIFR-OWN-0002` narrowing is mandated by contract §4, documented in `docs/errors/SIFR-OWN-0002.mdx`, and pinned by `mutable_receiver_overlapping_shared_read.sifr`. Deliberate.

---

## 4. Noted and explicitly *not* counted against this PR

All three were verified against the base binary at `/private/tmp/wave5base` (`f1c34cf9a`) and leak at base too — the diff strictly *shrinks* the leak set rather than expanding it.

| Shape | Base `f1c34cf9a` | Head `e174a9ec8` |
|---|---|---|
| `self.count = 0; self.count = n; self.items = []` | `check` ok → `E0062: field count specified more than once` | **`SIFR-OWN-0014`** ✅ improved |
| `self.name = name; self.tags = []; self.tags.append("x")` | `check` ok → `E0424: expected value, found module self` | **runs, exit 0** ✅ improved |
| never-initialized field, no self use (`incomplete.sifr`) | `check` ok → `E0063: missing field b` | `check` ok → `E0063` — unchanged |
| param moved into a field then reused (`movebug.sifr`, `mv_list.sifr`) | `check` ok → `E0424` / `E0062` | `check` ok → `E0382: borrow of moved value` — still leaks, different code |
| conditional `super().__init__()` | `check` ok → `E0063` | `check` ok → `E0063` — unchanged (pass-7 note) |

The move-of-parameter-into-field gap is orthogonal to this diff's check: it is a general ownership-tracking hole in field assignment, not a storage-completeness question, and it is present identically in **ordinary methods** at both revisions (`mv_method.sifr` → `E0308` at base and head). The local-variable move tracker is byte-identical in behavior (`localmove2.sifr` → `SIFR-OWN-0001` at base and head). Worth a follow-up issue alongside the conditional-`super` note, but not this PR's scope and not regressed by it.

**Bookkeeping observation (not actionable):** the ledger's heavy-lane merge-profile evidence is still pinned at `31af48ac8` (`plans/issues/active/…-place-semantics.md:786-793`), and the create-PR-profile evidence cited in the brief (Python lane 19/19, `lsp-smoke` 6/6 in isolation) is not recorded in the ledger. I closed the substantive risk empirically — the delta since that pin is three lowering files, and the 4073-file corpus scan shows the tightening affects no existing source — so this is a documentation-freshness suggestion, not a defect.

---

## Actionable findings

**None.** Pass-7 finding 1 is closed at the root with exact lowering/codegen predicate parity, verified in both the reject and accept directions, and pinned by a load-bearing lowering test plus an annotated fail fixture. Pass-6 findings 1–3 remain closed and independently reproduced. The wider PR — clone paths, checked-place invariant enforcement in production lowering, `ReceiverConvention` propagation, place eligibility and overlap, owned-temporary proof, constructor source-order materialization, optimizer protected roots, fixed trait/protocol constraints, HIR flow effects, diagnostic stability and JSON `place` args, panic safety, source compatibility, submodule state, file size, and fixture/test coverage — I found sound at this exact head, with every gate green.

**SATISFIED**

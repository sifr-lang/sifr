## Item 2 PR Review — Class-Field Mutating Receiver Place Semantics (PR #3082), Pass 7

**Range:** `f1c34cf9aaabadda546e670fca190decc580c935` → `8ec228b3e1529e985c64479e1737e0ffb86f18a2` (183 files, +5765/−1061). `git rev-parse HEAD` = `8ec228b3e…`; working tree clean except my own untracked pass-7 placeholder. No files modified, no commit/push, no PR-state change. I read the phase contract, all six prior Item 2 artifacts, the LRU corpus artifact, and the full diff; `cargo build` at head is current, and all probes below used that binary.

The head commit `8ec228b3e` touches only `class_body_lowering.rs`, `class_semantics.rs`, `ownership_diagnostics.rs`, one callable-diagnostics test, `method_receiver_analysis_tests.rs`, one pass fixture, and the ledger — so the broad receiver/codegen/optimizer surface is unchanged from what pass 6 verified, and I re-verified its load-bearing claims rather than re-deriving all of them.

---

## 1. Pass-6 finding 1 — validation counts and performance evidence: **CLOSED**

| Claim at head | Independent result |
|---|---|
| `cargo test -p sifr_lowering --lib` | `921 passed; 0 failed; 1 ignored` — matches ledger `:783-784` |
| `cargo test -p sifr_codegen --lib` | `941 passed; 0 failed` — matches ledger |
| `perf.check.project.project_graph` budget | `budgets.json` `budgets[18].thresholds.median_ms = 1357.524`, `budget_id perf.check.project.project_graph` — ledger's `1339.235ms < 1357.524ms` is against the tracked, unmodified budget |
| `perf.check.single.arithmetic` | `budgets[20]` = `1334.139` (baseline `1212.854`) — matches ledger `1328.513ms` |
| `perf.diagnostic.json_diagnostic_schema` | `budgets[31]` = `1335.954` (baseline `1214.504`) — matches ledger `1317.663ms` |
| Perf data changed by diff? | `git diff --name-only f1c34cf9a..8ec228b3e -- verification/areas/performance` → **empty**; no budget/baseline/waiver/manifest edit |

All three sample counts (`5`) and medians are now recorded (ledger `:795-802`), and the recorded thresholds are auditable against tracked data. Per your instruction I did not re-measure the performance lane; the numbers are internally consistent, tied to the repo's own `check_budgets.py --allow-subset` gate, and the `MIN_P95_SAMPLE_COUNT = 20` fact from pass 6 still holds (`check_budgets.py:27`), so p95 is not enforced in normal runs either.

## 2. Pass-6 finding 2 — constructor `SIFR-OWN-0014` diagnostic: **CLOSED**

Missing-`super` case (`/tmp/p7/missing_super.sifr`, subclass `__init__` reading `self.y` with no `super().__init__`):

```
error[SIFR-OWN-0014]: constructor uses self before inherited storage is initialized; call super().__init__(...) first
  --> missing_super.sifr:12:9
   12 |         print(self.y)
```

`self.b = self.a + 1` case (`/tmp/p7/selfb.sifr`):

```
error[SIFR-OWN-0014]: constructor uses self before field storage is initialized: self.b
  --> selfb.sifr:7:9
    7 |         self.b = self.a + 1
```

`--diagnostic-format json` for both records `args.place = {"kind":"string","value":"self"}` — a canonical place, and identical in shape to the non-constructor path (`unsupported_narrowed_optional_mutating_receiver.sifr` → `args.place = "maybe"`, both with `message_template: "{message}"`, so the registry's declared `arg!("place")` contract at `registry/registry_entries/calls_flow_and_protocols.rs:214-224` is satisfied). `__sifr_parent` no longer appears anywhere in user-facing output; `ownership_diagnostics.rs:250-289` maps it to "inherited storage / `call super().__init__(...) first`". Spans are the first offending source statement (`class_body_lowering.rs:614-618`, `func.body.get(gap.statement_index)`), and the NonSend marker parent is correctly excluded from the parent requirement (`class_body_lowering.rs:615` matches `class_field_emitter.rs:66` and `class_type_collection.rs:840`). Four new lowering tests pin message text, `place` arg, span, and absence of `__sifr_`.

## 3. Pass-6 finding 3 — same-named parameter seeding: **fixed for the reported shape, but the fix opened a new hole**

The reported shape now works:

```
class Holder:  a: int;  items: list[int]
    def __init__(self, a: int):
        self.items = []; self.items.append(1); self.items.append(2); self.a = a
```
`sifr run` → `2`, `7`. And with a non-trivial later value (`self.a = a * 2 + 1`) → `1`, `15`, so the param seed is genuinely overwritten by the post-materialization assignment rather than shadowing it (`class_method_emitter.rs:511-540` seeds only by param-name match; `:379-395` re-emits the later assignment against `__sifr_self`). The pass fixture `constructor_mutating_receiver_places.sifr:64-71` + `constructor_same_named_parameter_remains_seeded_before_explicit_assignment` pin it. No regression in explicit initializers, stdlib constructors, NonSend, or parent checks: full lowering/codegen suites green, `test_e2e_fail` green (1 passed, full annotated corpus), and `constructor_mutating_receiver_places.sifr` / `class_field_mutating_receiver_places.sifr` run with exit 0.

**However**, the remediation removed *two* guards, not one. Besides dropping the `explicit_initializers` exclusion (the actual fix), it also deleted `&& !initialized.contains(field)` from the `FieldAssign` arm (`class_semantics.rs:136-144`). That guard is what made a *repeated* field assignment fall through to the first-self-use check. See finding 1 below.

## 4. Independent inspection of the wider PR — no further gaps

- **No silent clone / no unchecked receiver path.** `grep` for `pending_self_field_clone_suppression`, `method_call_needs_field_clone_suppression`, `body_contains_field_assign_codegen`, `needs_field_clone_suppression` across `crates/` → **0 hits**. `emit class_field_mutating_receiver_places.sifr` contains `fn bump(&mut self)`, `fn mutate(&mut self)`, `fn mutate_inherited(&mut self)`, `fn replace(&mut self, …)` and **no** `clone(`/`to_vec(`.
- **The `Ok(None)` arms in `place_emitter.rs:100-121,144-200` are genuinely unreachable, not a fallback.** `method_call_verifier.rs:160-200` rejects a `MutableBorrow` call with no target, a target whose shape does not match the HIR object (`receiver_target_matches`), a convention mismatch against the signature, and any `mut`-borrow parameter without a proof slot. This is the load-bearing invariant and it holds.
- **Constructor predicate parity.** `class_semantics.rs:77-91` (`body_references_receiver`) and `class_method_emitter.rs:433-447` (`constructor_body_references_self`) are byte-for-byte the same two-visitor predicate, so the materialization boundary is shared. The *only* divergence between the two sides is the missing `field_inits` dedup — finding 1.
- **Protected roots / optimizer**: `protected_mutable_place_roots` is inserted at both `place_emitter.rs:47,81`, consumed in test assembly (`entrypoints.rs:159`) *and* production assembly (`lib_modules_and_codegen.rs:630`), and honoured for `except` bindings (`try_handlers.rs:170`).
- **HIR flow effects**: `flow_graph/effects.rs` splits `Call`/`PythonCall`/`IteratorCall` and routes each argument through `argument_effects` with its `mutable_arg_places` slot — mutation effects are now recorded for `mut` arguments rather than lost in a catch-all. `cfg.rs` changes are test-only field additions.
- **Internal panics**: no `unwrap`/`expect`/`panic!`/`unreachable!`/`todo!` in `method_receiver_places.rs`, `place_emitter.rs`, `method_call_verifier.rs`, or `class_semantics.rs`.
- **Gates at exact head**: `cargo fmt --check` pass; `cargo clippy --workspace -- -D warnings` exit 0 (also clean with `--all-targets`); file-size guardrail pass (3053 files, limit 900); HIR maintainability pass; docs error-code links pass; `git diff --check` clean.
- **Submodules**: `leetcode` at `7772857c6f` (the merged corpus PR #40), all others unchanged and clean.
- **Source compatibility**: the `SIFR-OWN-0002` narrowing (`values.append(len(values))`, `heapq.sifr`, LRU corpus) is mandated by the approved contract §4, documented in `docs/errors/SIFR-OWN-0002.mdx`, and pinned by `mutable_receiver_overlapping_shared_read.sifr`. Deliberate, not a defect.

## 5. Noted and explicitly *not* counted against this PR

A conditional `super().__init__()` (`/tmp/p7/condsuper.sifr`) passes `sifr check` and then leaks `E0063: missing field 'base'`. `inheritance_parent` at head recognizes only a top-level `SuperCall` statement (`class_method_emitter.rs:275-285`), and base `f1c34cf9a`'s `has_super` (`git show f1c34cf9a:…/class_method_emitter.rs:275-281`) is the identical top-level-only predicate — so this behaves the same before and after the diff. Pre-existing, not expanded; recorded for the follow-up, not blocking.

---

## Actionable findings

**1 — Medium (correctness of a new check; rustc leak through `sifr check`). The head commit's removal of the `initialized`-dedup guard makes an ordinary repeated field assignment pass `sifr check` and then fail with raw `rustc` `E0063`.**

`crates/sifr_lowering/src/lower/classes/class_semantics.rs:136-144` now treats *every* `self.<field> = <non-self value>` as a pre-materialization field initializer, including a second assignment to a field already initialized. Codegen does not: `crates/sifr_codegen/src/class_method_emitter.rs:334-340` requires `!field_inits.iter().any(|(name, _)| name == field)`, so the repeat is *not* a field init and falls into the `constructor_body_references_self` arm at `:378-397`, materializing `Self { … }` while later fields are still missing.

Reproduced at exact head (`/tmp/p7/dup3.sifr`, an entirely ordinary constructor):

```python
class Counter:
    count: int
    items: list[int]

    def __init__(self, n: int) -> None:
        self.count = 0
        self.count = n
        self.items = []
```
```
$ sifr check dup3.sifr   → no errors found   (exit 0)
$ sifr run   dup3.sifr
error[SIFR-BUILD-0005]: cargo build failed:
error[E0063]: missing field `items` in initializer of `Counter`
  --> src/main.rs:11:31
   11 |         let mut __sifr_self = Self { count: __sifr_field_init_0 };
```

The asymmetry is specific to this deletion: the augmented-assign sibling (`self.count = n; self.count += 1; self.items = []`) is cleanly rejected — `SIFR-OWN-0014: constructor uses self before field storage is initialized: self.items` at `aug.sifr:7:9` — and the shape compiles fine when the duplicate comes *after* all fields are initialized (`/tmp/p7/dup4.sifr` → `2`, `0`). The immediately preceding PR commit `31af48ac8` diagnosed `dup3` correctly, because its `!initialized.contains(field)` guard (deleted in `8ec228b3e`) pushed the repeat into the first-self-use branch. This directly contradicts the contract clause this PR records for itself — "rejects receiver use before complete own/inherited storage with check-time `SIFR-OWN-0014`" (`plans/issues/active/…-place-semantics.md:936-938`) — and the "if it compiles, it works" / no-rustc-leak expectation. Base `f1c34cf9a` also mishandled this shape (it partitioned all `FieldAssign`s into one struct literal, `git show f1c34cf9a:…/class_method_emitter.rs:383-392`, yielding a duplicate-field `E0062`), so this is not a regression versus pre-PR `main` — but it is a self-inflicted false negative in code this PR adds specifically to eliminate such leaks, it regressed within the PR, and no test covers it.

*Required correction:* make lowering's skip condition mirror codegen's `is_uninitialized_own_field` exactly — track the explicitly-assigned pre-materialization fields in their own set (as codegen's `field_inits` does) separately from the same-named-parameter seeds, and skip the statement only when the field is **not already in that explicit set**. Concretely: keep `initialized = param_seeds ∪ explicit_field_inits` for the missing-field computation, but gate the `continue` on `!explicit_field_inits.contains(field)`. Verified by analysis against every shape probed here: `dup3` then reports `SIFR-OWN-0014 (self.items)` at `self.count = n`, while `ParameterSeededOwner`, `/tmp/p7/items.sifr`, `/tmp/p7/items2.sifr`, `self.a = a; self.items = []`, and `dup2`/`dup4` all keep compiling. Add a fail fixture plus a lowering test for the repeated-assignment shape.

---

Pass-6 findings 1 and 2 are fully closed at the root and independently reproduced. Finding 3's reported symptom is closed, but its remediation carried a second, unrelated guard deletion that reopens a check-time hole in the same function. Everything else in the PR — clone paths, `ReceiverConvention` propagation, place eligibility and overlap, owned-temporary proof, source-order materialization, optimizer protected roots and fallback, fixed Rust-trait/protocol constraints, flow effects, diagnostic stability, panic safety, source compatibility, module responsibility, submodule state, file size, and fixture/test coverage — I found sound at this head.

**NOT SATISFIED**

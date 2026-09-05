# Ad Hoc Phase: Emitted Rust Excellence

Status: active

Baseline commit: `e9df29f7e4cada7b376b2d455790f9c80a5647a0`

## Objective

Make every Rust program emitted by Sifr correct, panic-safe, idiomatic,
efficient, portable, and clean under the repository's strongest generated-code
quality policy.

This is a full-solution phase. It does not preserve known emitter debt behind
lint allowances, checked-in stale output, corpus exclusions, compatibility
paths, silent fallbacks, or deferred quality tiers.

## Source of Truth

- this phase record
- `verification/areas/generated_code_quality/emitted_rust_audit_inventory.json`
- `verification/areas/generated_code_quality/check_emitted_rust_audit_inventory.py`
- the compiler and runtime sources that produce generated Rust
- every compiler-generated Rust surface reached through `emit`, `build`,
  `run`, `test`, single-file, project, static-program, sysroot, and interop
  entrypoints
- generated demo companions and verification-generated Cargo projects

The inventory reconciles the internal review with the external audit supplied
by the user. A claim marked `rejected` is preserved to prevent it from being
reintroduced as an unsupported requirement. A `confirmed` or
`partially_confirmed` claim has exactly one implementation owner.

## Locked Quality Contract

### Semantic correctness

1. Emitted Rust preserves the canonical Sifr language and stdlib semantics in
   debug and release profiles.
2. Integer arithmetic, floor division, modulo, conversion, and fixed-width
   boundaries use one exact and explicit model. Compiler optimization must not
   change observable overflow behavior.
3. Collection reads, writes, deletes, unpacking, and mutation use one checked
   access architecture. Missing and out-of-range operations return the typed
   Sifr error required by the source contract.
4. Iterators and generators preserve laziness, termination, error timing, and
   infinite-source behavior. No finite cap can stand in for an infinite
   iterator.
5. Stdlib adapters preserve error categories, argument semantics, precision,
   Unicode behavior, and resource limits.

### Runtime safety

1. User data cannot reach Rust panic, abort, process exit, undefined behavior,
   capacity overflow, indexing panic, arithmetic panic, or an impossible-state
   macro.
2. `unwrap`, `expect`, `panic!`, `unreachable!`, `abort`, and `exit` are not
   generated as error handling. A compiler-proven invariant must be represented
   structurally or converted to a checked internal diagnostic before generated
   code materialization.
3. `unsafe` is forbidden in emitted user crates unless a future phase record
   approves one audited, encapsulated runtime implementation. No such approval
   exists in this phase.
4. Silent no-op writes and fallback values are forbidden when the Sifr contract
   requires an error.

### Rust quality

1. Generated Rust passes `rustfmt --check` without first mutating the files.
2. Generated crates pass the strongest agreed Clippy policy with warnings
   denied. Every allowance must identify a language-driven necessity, an owner,
   and removal criteria. Allowances for emitter convenience are forbidden.
3. Public and private APIs use `str`, slices, iterators, references, owned
   values, and standard collection entry APIs according to Rust ownership
   norms.
4. Emission contains no redundant clones, identity maps, needless returns,
   unreachable tails, constant dead branches, one-character `String`
   allocations, or scaffolding that a structured Rust IR can avoid.
5. Generated names remain deterministic and legal without globally suppressing
   ordinary Rust naming and dead-code diagnostics.

### Performance and portability

1. Compiler lowering must not turn an asymptotically efficient source program
   into a worse algorithm through cloning, indexing, front removal, eager
   materialization, or Unicode rescans.
2. Runtime and stdlib support is demand-driven and emitted once. Duplicate
   bridges, duplicate APIs, and dead support modules are forbidden.
3. Generated Cargo projects contain no machine-specific absolute paths in
   distributable output. Build-local paths may exist only in ephemeral build
   state that is never presented as portable emitted source.
4. Process APIs preserve argument boundaries. The compiler does not introduce
   shell parsing or command concatenation that was absent from the source API.
5. Full-corpus qualification records generated source size, relevant operation
   counts, lint allowances, and selected complexity budgets so quality cannot
   regress while tests remain behaviorally green.

## Scope

### In scope

- `crates/sifr_codegen/**`
- generated-project materialization in `crates/sifr_driver/**`
- generated runtime support in `crates/sifr_runtime/**` and sysroot-owned
  support used by emitted programs
- generated-code verification adapters, manifests, negative seeds, profiles,
  and evidence
- checked-in generated demo companions and stale generated snapshots
- generated Cargo manifests and bridge assembly
- focused language, stdlib, e2e, algorithmic, and performance fixtures needed
  to prove each mechanism

### Out of scope

- rewriting hand-authored `idiomatic.rs` reference files
- changing Sifr semantics merely to make Rust emission easier
- hand-editing generated output as the fix instead of correcting its producer
- general compiler architecture work with no emitted-code acceptance effect
- user-authored shell commands whose injection risk is already present in the
  source-level API and is not introduced by lowering

An out-of-scope defect found during implementation is recorded with an owner.
It does not broaden the active item.

## Execution Rules

1. Work one item at a time in the order below.
2. Implement the complete item before running its tests. Then run focused
   validation, repair failures in scope, and collect exact-SHA evidence.
3. Each implementation item receives one exact-SHA agent review and at
   most one remediation review. A second-review mechanism defect becomes a
   later owned item; there is no third review.
4. Compiler-changing items receive exactly one create-PR gate and one merge
   gate on the final candidate SHA. Neither gate is repeated. Items without
   compiler changes omit both gates. For this phase, the user authorizes the
   following narrow ordering override: run the constituent pre-review checks,
   open the draft PR, complete the exact-SHA review/remediation sequence, and
   only then run the named create-PR and merge gates once on the resulting
   final SHA. This preserves the draft-PR review workflow while ensuring both
   named gates qualify exactly the code that can merge.
   Item 8 has one explicitly adjudicated exception: its reviewed SHA
   `a77acce704ccab8bf568ea4156ff05dd706c66c1` exposed a missing
   `sifr_runtime::count_byte` manifest owner in the sole create-PR gate. The
   user authorized documentation-only manifest commit
   `fa661c6eccd4c1fa3eb0092e3106ac4d44dddeda`, the targeted guard passed, and
   neither the review nor create-PR gate was repeated. This is not precedent
   for another item or another gate mismatch.
5. Merge the item, update this record, and start the next unfinished item.
6. The closure-only final item receives the only whole-phase agent review.
7. Before each item starts, rebase its branch point on current `origin/main`
   and re-audit any relevant mechanism that another merged phase changed.
   Unmerged branches are not silently treated as delivered work.
8. Generated companions are regenerated from the candidate compiler. They are
   never manually polished.
9. Generated-code lint debt is exact evidence, not a name-based tolerance.
   Each retained diagnostic is selected by companion, lint name, count, and
   stable signature. Unknown diagnostics, count growth, signature drift, and
   diagnostics outside the recorded companion selection fail closed. Item 12
   must remove the remaining owned debt rather than rebase it.
10. A named one-shot gate that identifies an in-scope candidate defect stops
    the item for explicit adjudication. The defect is not deferred, waived, or
    hidden by changing the gate, and the one-shot gate is not rerun.
11. Existing item commits are preserved. Follow-up work is added as new
    commits; local history is not rewritten or squashed before review.

## Sequential Items

| Item | Status | Name | Required outcome |
|---:|---|---|---|
| 0 | complete | Contract and audit inventory lock | The full quality contract, reconciled finding ledger, baseline, ownership, review limits, and closure rules are machine checked and merged. |
| 1 | complete | Comprehensive corpus and non-vacuous gates | Every generated surface is discoverable; freshness, rustfmt, Clippy, panic/static analysis, determinism, and negative self-tests fail closed without broad quality suppressions. |
| 2 | complete | Exact integer and overflow architecture | Canonical `int` storage and all arithmetic use one exact semantic model; debug/release behavior agrees; fixed-width boundaries remain explicitly checked. |
| 3 | complete | Checked failure and impossible-state model | Generated user paths use typed errors; abort/exit/unreachable discharge and silent value fallbacks are removed; compiler invariants fail before materialization. |
| 4 | merged | Collection access and mutation architecture | Reads, writes, deletes, nested access, augassign, membership, and unpacking share checked place semantics with no panic or silent no-op path. |
| 4A | merged | Residual checked-place lifecycle closure | Loop-carried witnesses, post-mutation missing behavior, and callback argument decoding preserve exact semantics and compile on every generated surface. |
| 4B | merged | Structured-loop witness state closure | Async-for guard state cannot escape a possibly empty loop, and missing loop-carried witnesses take the loop-kind's terminating control-flow path instead of skipping progress. |
| 4C | merged | Mutation-tail witness continuation closure | Refreshed witnesses use region-scoped continuations and current typed failure semantics across nested and straight-line mutation tails. |
| 5 | merged | Lazy iterator and generator architecture | Yield, generator state, `count`, `islice`, chained adapters, and errors are lazy and semantically unbounded where required. |
| 6 | merged | Stdlib emitted-semantics closure | String widths, IO reads/seeks/errors, decimal precision, iteration arguments, and every inventory-owned stdlib defect have exact behavior and resource safety. |
| 6A | merged | Generic-bound substitution and residual string parity | Generic arithmetic bounds use receiving-parameter identity and `str.center` matches CPython's odd-margin behavior. |
| 7 | merged | Ownership, borrowing, and clone quality | Signatures and expressions use idiomatic borrowing; avoidable container, row, tree, and scalar clones are eliminated without weakening ownership safety. |
| 7A | merged | Receiver-effect precision and owned-boundary closure | Receiver effects invalidate only facts they can falsify and every `setdefault` entrypoint shares one owned-value boundary. |
| 7B | merged | End-relative receiver facts and affine boundary closure | Growth invalidates end-relative facts without discarding stable absolute facts, and affine `setdefault` has one checked ownership contract. |
| 8 | merged | Canonical Rust IR and emission cleanup | Structured IR represents all maintained code; dead branches/tails, identity transforms, needless returns, stale snapshots, and generated ceremony are removed at the producer. |
| 8A | merged | Canonical cleanup effect and identity hardening | Every second-review cleanup edge is effect-, type-, scope-, and concurrency-safe, with one shared format-capture mechanism and no target invalidation between concurrent quality runs. |
| 9 | merged | Algorithmic and Unicode performance | Emission preserves source complexity; string traversal avoids repeated scans/materialization; collection algorithms avoid quadratic clone/front-removal behavior; budgets prevent recurrence. |
| 9A | merged | Character comparison state disambiguation | Allocation-free character/string comparison keeps an absent indexed character distinct from a present empty or multi-character string in every operand and optionality form. |
| 10 | merged | Runtime, stdlib bridge, and API deduplication | Each demanded support body and public adapter is assembled once, unused support is absent, and generated crates have one canonical API path per operation. |
| 10A | merged | Module-scoped builtin error shadow identities | Project support demand preserves user-defined and builtin error identities per module, without crate-wide suppression or dangling generated paths. |
| 11 | merged | Portable and secure generated projects | Reviewed candidate `78c28c1e4c42bd85d685d3a3cffdf132fcdfcc40` is preserved and merged through Item 11A after its consumed gate's stale companions were regenerated. |
| 11A | merged | Generated-companion freshness and Item 11 integration | The reviewed Item 11 candidate and all 15 compiler-regenerated companions are merged through a separately bounded review and gate without rerunning Item 11's consumed gate. |
| 12 | pending | Residual semantic completion and full-corpus qualification | Finish remaining semantic/profile work, remove all governed generated-code debt, regenerate every owned surface, and pass the uncompromising final qualification and applicable one-shot gates. |
| 12B | in progress | Bounded algorithmic dependency repair | Continue the recorded native repair batch and qualification under the latest authority. |
| 12C | incorporated into 12B | Builtin-registration Clippy blocker | No independent item, review, or gate remains. |
| 12D | incorporated into 12B | Native corpus emission dependencies | The retained failure inventory bounds the authorized repair; no separate review or gate. |
| 12A | pending | Phase closure and whole-phase review | Review the fully merged phase once, reconcile architecture/roadmap/evidence, and archive only when no actionable row remains. |

## Item Acceptance Contracts

### Item 0: Contract and audit inventory lock

- [x] Every internal and external audit mechanism is confirmed, partially
  confirmed, or rejected with evidence.
- [x] Every actionable finding has exactly one owner in Items 1-11.
- [x] The checker rejects missing ownership, invalid status, duplicate IDs,
  invalid item references, and unsupported rejected claims.
- [x] The Item 0 mutation self-test proves its implemented rejection classes;
  the sole remediation review's newly identified missing branches are owned by
  Item 1 under the no-third-review rule.
- [x] The roadmap names this active phase.
- [x] The exact-SHA review process followed the initial/remediation limit, its
  new mechanism defect is assigned to Item 1, and the item is merged.

### Item 1: Comprehensive corpus and non-vacuous gates

- [x] Corpus discovery covers all generated entrypoint classes and cannot be
  reduced without a failing self-test.
- [x] Checked-in generated files are fresh or are removed as non-authoritative.
- [x] `rustfmt --check` runs before any formatter mutation.
- [x] Clippy warnings are denied without emitter-convenience blanket allows.
- [x] Static safety analysis covers impossible-state macros, termination calls,
  indexing, casts, allocation widths, arithmetic, and generated `allow` use.
- [x] Negative seeds prove each gate can fail for the owned defect class.
- [x] The audit-inventory self-test covers every validation branch, including
  empty item/baseline containers and both required finding text fields.
- [x] Baseline provenance requires its named command, toolchain, and note keys.
- [x] Evidence rows use governed semantic anchors, not path existence alone;
  glob and repository-boundary handling fails closed.

### Item 2: Exact integer and overflow architecture

- [x] `int` has one canonical runtime representation through locals,
  parameters, returns, fields, containers, constants, unions, and interop.
- [x] All arithmetic and conversions preserve the language's exact semantics.
- [x] Floor division/modulo and zero/overflow errors are consistent.
- [x] Debug and release differential/property evidence agrees.

### Item 3: Checked failure and impossible-state model

- [x] No generated user path contains abort, exit, unreachable, panic, unwrap,
  expect, or a silent fallback for an error-producing operation.
- [x] Compiler invariants are validated structurally before source rendering.
- [x] Typed errors preserve category, payload, source span where applicable,
  and error timing.

### Item 3A: Residual checked-flow and proof mechanism closure

- [x] Suppressible Python context errors rejoin every enclosing try carrier with
  a structurally valid continuation, including direct-return contexts.
- [x] Exact-integer facts distinguish module constants from shadowing locals and
  are invalidated by called nested-function `nonlocal` mutation.
- [x] Sync/async context and loop regressions compile and run with the intended
  dynamic values and no mechanism-owned warning debt.
- [x] Static flow summaries and emitted carriers agree for every repaired path.

### Item 4: Collection access and mutation architecture

- [x] Every collection access form uses one typed checked-place plan.
- [x] Negative indices, nested indices, unpacking cardinality, missing keys,
  deletes, and writes preserve Sifr semantics.
- [x] Membership and read-only access do not mutate containers.
- [x] Out-of-range writes never become no-ops.

### Item 4A: Residual checked-place lifecycle closure

- [x] A checked-place witness does not survive a loop back-edge after any
  mutation of its object or index dependencies.
- [x] A fresh read after mutation uses the operation's current typed failure
  semantics; deletion followed by access cannot replay an earlier guard exit.
- [x] Fixed-arity callback argument decoding is type-directed, panic-free, and
  compiles for one or more callback arguments across the Python interop matrix.

### Item 4B: Structured-loop witness state closure

- [x] Async-for saves and restores sequence-guard state like `for` and `while`,
  so a proof established only in its body cannot escape a zero-iteration loop.
- [x] Loop-carried refresh assigns loop-kind-specific missing control flow:
  `break` for `while` and `continue` for `for`/`async for`, including witnesses
  originally established by an enclosing branch without a missing action.
- [x] Focused diagnostics assert the checked-place error identity rather than
  accepting an arbitrary lowering failure.
- [x] Native regressions cover ordinary and closable async-for refresh, empty
  async-for guard restoration, and terminating `while` behavior after an
  indirect mutable call invalidates a witnessed place.

### Item 4C: Mutation-tail witness continuation closure

- [x] A witness refresh nested under another loop or branch derives control
  flow from its current structured region, never from an outer witness's stored
  `break`, `continue`, return, or fallback payload.
- [x] Straight-line mutation tails cannot silently skip following statements or
  replay proof-establishment exits; mutable-call invalidation exposes the
  operation's current optional/typed-failure contract before codegen.
- [x] Simple and structured loop lowering share one canonical break/loop-else
  marker constructor.
- [x] Native and codegen regressions cover outer `while ... else` with nested
  `for`/`if` mutation and read, straight-line positive-branch mutation tails,
  and condition-refresh loop-else marker emission.

### Item 5: Lazy iterator and generator architecture

- [x] Generator bodies are resumable state machines or an equivalently lazy
  representation, not eager vectors.
- [x] Infinite iterators remain infinite and consumers control termination.
- [x] `islice` and related adapters validate arguments and preserve error timing.
- [x] Laziness, partial consumption, side effects, and memory use have native
  runtime regressions.

### Item 6: Stdlib emitted-semantics closure

- [x] All Item 6 inventory rows are covered by focused differential tests.
- [x] Signed sizes and widths are validated before allocation/casting.
- [x] IO operations honor size/offset arguments and preserve error kinds.
- [x] Decimal precision never silently falls back.

### Item 6A: Generic-bound substitution and residual string parity

- [x] Propagated arithmetic bounds refer to the caller's corresponding type
  parameter, never a callee-local spelling.
- [x] Differently named `Addable` forwarding compiles and runs through an
  authoritative emitted companion.
- [x] Bound propagation remains demand-driven and preserves the Item 6
  `PartialOrd`, `Display`, and `Hash + Eq` closure.
- [x] `str.center` matches CPython's odd-margin placement as well as signed and
  oversized-width behavior.

### Item 7: Ownership, borrowing, and clone quality

- [x] APIs prefer `str` and slices where ownership is not required.
- [x] Clone insertion is driven by an explicit ownership plan.
- [x] Clone counts and representative emitted shapes have regression budgets.
- [x] Recursive and dynamic-programming fixtures preserve linear work where the
  source algorithm is linear.

### Item 7A: Receiver-effect precision and owned-boundary closure

- [x] One receiver-effect summary distinguishes growth, removal, reordering,
  and value mutation for every builtin and user-defined mutable receiver.
- [x] Length and membership guards survive operations that preserve their proof,
  while shrinking/removing operations and positional reordering invalidate the
  exact facts they can falsify.
- [x] Every `setdefault` emission entrypoint materializes owned key/default
  values at the operation boundary, including local-binding fallback emission.
- [x] Focused shape and native regressions cover guard preservation/invalidation
  and borrowed plus owned `setdefault` values without redundant clones.

### Item 7B: End-relative receiver facts and affine boundary closure

- [x] Growth invalidation distinguishes stable absolute subscript facts from
  end-relative negative-index facts whose referent changes after append/extend.
- [x] Negative-index append/extend regressions reject stale non-`None` facts,
  while nonnegative-index growth preservation remains covered.
- [x] Affine `setdefault` values have one explicit checked contract for both
  insertion and returned-value ownership, with reaching emitted/native evidence.
- [x] Mutable non-collection receiver summaries preserve facts only when the
  receiver type cannot own a relevant sequence fact.

### Item 8: Canonical Rust IR and emission cleanup

- [x] Maintained emission uses structured IR through final rendering.
- [x] Canonical simplification removes dead and identity constructs without
  textual postprocessing.
- [x] Generated names and support items are demand-driven and warning-clean.
- [x] Stale or mislabeled generated snapshots are removed or regenerated by an
  authoritative producer.
- [x] Every retained companion diagnostic is governed by exact companion,
  lint, count, and signature evidence; unknown or growing debt fails closed.

#### Item 8 closure ledger

All rows below were implemented, qualified, reviewed, and merged in Item 8.
Second-review suggestions are deliberately outside this closed ledger and are
owned by Item 8A under the no-third-review rule.

| Row | Deferred mechanism | Producer/evidence | Candidate state |
|---:|---|---|---|
| I8-01 | Three non-authoritative test-project references | `scripts/check_demo_emitted_freshness.py` | Implemented: the exact three references are classified and fail closed if an authoritative `emitted.rs` appears. |
| I8-02 | ERQ-025 stale snapshot wording | `emitted_rust_audit_inventory.json` | Implemented: baseline and current companion counts/roles are distinct. |
| I8-03 | Companion rustfmt/Clippy governance | `generated_code_quality.py`, `quality_policy.py`, `inventory_gates.py` | Qualified in the candidate: all 262 authoritative companions passed; the retained summary passed exact companion-set, lint-owner, count, and signature validation after removing the eliminated `manual_let_else` debt. |
| I8-04 | Dead `SIFR-TYPE-0901` surface | diagnostics registry/render/catalog/docs diffs | Implemented: producer, IR, registry, indexes, and dedicated page are removed; historical references remain explicitly historical. |
| I8-05 | Dead Arrow `handle` method | `crates/sifr_stdlib/src/python/arrow.rs` | Implemented. |
| I8-06 | Non-snake-case constant helpers | `lower_item/module_constants.rs`, identifier canonicalizer, `module_constant_helper_names_are_injective_and_warning_clean` | Implemented with injective declaration/reference rewriting. |
| I8-07 | Bare return in promoted `Result[None, E]` try | `try_binding_bare_result_return.sifr`, canonical control-flow lowering | Implemented. |
| I8-08 | Loop control escaping a try/finally closure | `try_finally_loop_control.sifr`, structured carrier lowering | Implemented. |
| I8-09 | Raise without a compatible error channel | `result_diagnostics.rs`, `error_raise_requires_a_compatible_result_channel` | Implemented as a source diagnostic before codegen. |
| I8-10 | Phase 34 retired integer diagnostic claim | `plans/phases/34_generated_code_quality_and_production_readiness.md` | Implemented as explicit historical provenance. |
| I8-11 | `FormatMacro` missing from forbidden-failure validation | `ir_validate.rs::rejects_failure_discharge_in_every_structured_macro_variant` | Implemented. |
| I8-12 | Dead exact-literal source bindings | `ir_optimize/dead_bindings.rs`, canonicalizer liveness regressions | Implemented structurally. |
| I8-13 | `true`/`false` identifier-pattern ambiguity | identifier policy/canonicalizer and injectivity/literal-preservation regressions | Implemented. |
| I8-14 | Bare-name module integer facts | `ModuleConstIntegerFacts` in `lower/mod_context.rs` | Implemented with immutable binding identity and export-name separation. |
| I8-15 | Maintained compiler/test Clippy debt | canonicalizer/API/source expectation passes and moved responsibility-based test modules | Qualified in the candidate: workspace Clippy passed for all targets with warnings denied, and no blanket allow was added. |
| I8-16 | Stale 701-path surface inventory | `surface_inventory.json` | Qualified at the current 724-path set by the fail-closed generated inventory gate. |
| I8-17 | Aggregate rustfmt debt drift | structured canonicalizer plus empty rustfmt debt | Qualified by full generated `rustfmt --check` with empty rustfmt debt. |
| I8-18 | Optional read after invalidation diagnostic | `mutable_call_sequence_guard_tests.rs` | Decision implemented: retain canonical `SIFR-TYPE-0002`-family unsupported-operator reporting for the widened `None | T`; no special proof-history diagnostic is warranted. |
| I8-19 | Return-ending while/else E0317 | `while_else_return_tail.sifr`, canonical control-flow pass | Implemented. |
| I8-20 | Implicit straight-line refresh fallback invariant | `checked_place/control_flow.rs`, `refresh_fallback_rejects_presence_removing_mutations` | Implemented as a checked codegen invariant. |
| I8-21 | Duplicated loop/else scaffolds | canonical loop-control constructors and sync/async/block regressions | Implemented through shared structured control paths. |
| I8-22 | Misclassified `numeric_sentinels` fixture | `e2e/pass/numeric_sentinels.sifr` | Implemented by making the source establish the required checked index proof. |
| I8-23 | Nested sync generator dangling yielder | `reject_unsupported_nested_generator`, `nested_sync_generator_is_rejected_before_codegen` | Implemented as explicit checked rejection pending dedicated nested lazy lowering. |
| I8-24 | 718-path inventory and formatting drift | `surface_inventory.json`, canonical source pipeline | Reconciled with I8-16/I8-17 against the current 724-path set. |
| I8-25 | Optional key passed to `HashSet::remove` | `sliding_window_narrowing.sifr`, optional-place/method argument normalization | Implemented with an isolated authoritative fixture. |
| I8-26 | Bare-class compiler `open()` defaults | canonical nominal compiler-default key, `compiler_open_defaults_do_not_attach_to_local_same_basename_methods` | Implemented. |
| I8-27 | Split class/free-function generic bound closure | `function_generic_bounds.rs`, `class_method_inherits_module_generic_function_bounds` | Implemented through one module callable-demand closure. |
| I8-28 | Nested lexical generic-call demand | `called_nested_function_propagates_captured_generic_demands`, shadow/leak regression | Implemented. |
| I8-29 | Composite actual over-constrains sibling parameters | `structural_correspondence_does_not_overconstrain_sibling_parameters` | Implemented with structural correspondence. |
| I8-30 | Same-basename generic callable contamination | canonical binding/callable identity tests in `function_generic_bounds_tests.rs` | Implemented. |
| I8-31 | Stale `protocol_bounds/idiomatic.rs` | deleted reference plus freshness classification | Implemented: the non-authoritative stale file is retired. |
| I8-32 | Dict silent fallback and double-reference query keys | checked-place dict-key normalization and `dict_keys_membership_guards_equivalent_indexed_reads` | Implemented. |
| I8-33 | Nested generic declaration ambiguity | `nested_generic_function_declaration_is_rejected_explicitly` | Implemented as one checked language boundary. |
| I8-34 | Non-collection receiver fact-domain drift | `sifr_type_system::receiver_mutation`, exhaustive summary regressions | Implemented with structural receiver domains. |
| I8-35 | Literal-only list growth stability | `sequence_guard_detection/subscript_guards.rs`, variable-index and dict-key regressions | Implemented from typed receiver/index facts. |
| I8-36 | Unreachable generic affine `setdefault` branch | lowering ownership contract and `methods/dict.rs` | Implemented with one source-facing owner. |
| I8-37 | Silent affine `setdefault` codegen decline | `methods/dict.rs::setdefault_affine_types_are_an_internal_invariant_violation` | Implemented as an explicit compiler invariant. |

Item 8 also integrates three qualification-discovered producer details without
claiming later-item closure: byte counting uses one optimized runtime primitive
to eliminate generated manual-count ceremony; the stdlib manifest carries the
already-delivered Item 6 ordered-JSON feature in isolated companion builds; and
module assembly invokes canonical demand/import placement. Item 9 still owns
algorithmic budgets, Item 10 still owns the unified runtime/bridge demand graph,
and Item 11 still owns portable materialization.

The schema-1 debt file stored only aggregate signatures, so it could not prove
per-lint ownership. Item 8's schema-2 migration removes every Item 8 lint and
all rustfmt debt, then records the residual later-item lint set as per-lint
counts/signatures. The companion-set selection digest includes every companion
identity, and each merged lint signature includes the contributing companion
identity, count, and diagnostic signature. This is a one-way strengthening of
evidence, not permission to carry changed Item 8 debt.

Compiler candidate `49f375e1619185d76e6cfc3b90d7e20ff786cce0`
passed 1,349 codegen tests, the non-E2E CLI suites, the existing 723-fixture
full E2E pass, direct native execution of the new 724th optional-set-remove
fixture, workspace Clippy for all targets with warnings denied, formatting,
diff hygiene, the 3,726-file size guardrail, HIR maintainability, generated
inventory, panic scans, generated rustfmt, determinism, exact-binary demo
freshness, and intrinsic-panic linting. Exact `origin/main`
`74bbb636744adaacb8c3eca09108b6fff9725357` independently retains only the two
stale TypeVar message assertions owned by #3667 and the stale attached-API
fixture lock owned by #3669.

The authoritative 91-project corpus and 262-companion corpus passed every
individual generated crate. Two isolated companion runs produced byte-identical
summaries with SHA-256
`a00628a95f22967fb52ffe3f119ba819ed29ca1c93fb3e6ffbc0c29e4d83fd65`:
11,324 governed diagnostics across 48 later-item lint families, with merged
diagnostic signature
`5231041489b4043fa7f0239abda8e3192702e1dbfd2e3fc366655be2b8fd4393`.
Two isolated selected full-Clippy runs likewise produced byte-identical
summaries with SHA-256
`42d874ebca265e8260e91c8947274d4770676d07020adf1a3308a00eb2dc17aa`.
The canonical cleanup removed `manual_let_else` from both selections and also
removed `semicolon_if_nothing_returned` and `unnecessary_semicolon` from the
selected corpus. The checked-in schema-2 debt matches the residual summaries
exactly and rejects stale owners, unknown diagnostics, count growth, and
signature drift.

The [initial exact-SHA review](https://github.com/sifr-lang/sifr/pull/3668#issuecomment-5517105667)
on `84ebe95b928cfe076d9af21e1bc06c1da3bc08c4` was NOT SATISFIED. Candidate
`49f375e1619185d76e6cfc3b90d7e20ff786cce0` resolves all four blockers: stale
lint owners were removed rather than re-owned; generic callables use canonical
lexical identities rather than bare names; optional `set.remove` normalization
has direct native regression coverage; and every Item 8 lint was eliminated
rather than deferred. The remediation also guards branch-local shadowing and
format captures, rejects globally ambiguous enum-owner rewrites, computes
cross-module constants to a monotonic fixed point, preserves eager/Drop/unknown
effects during cleanup, and fingerprints all producer inputs. The sole
remediation review on exact SHA
`a77acce704ccab8bf568ea4156ff05dd706c66c1` was
[SATISFIED](https://github.com/sifr-lang/sifr/pull/3668#issuecomment-5523601034)
with no blockers. Its six non-blocking mechanism findings are owned by Item 8A
and [#3670](https://github.com/sifr-lang/sifr/issues/3670).

The sole create-PR gate passed every reached check before finding the missing
`sifr_runtime::count_byte` manifest owner. Under the explicit Item 8 exception,
documentation-only final candidate
`fa661c6eccd4c1fa3eb0092e3106ac4d44dddeda` added that owner and the targeted
allowlist guard passed with 14 direct runtime roots; the create-PR gate and
review were not repeated. The sole merge gate then passed every Item 8
guardrail, including demo freshness and the corrected allowlist, before its
only failure in the unchanged SQL coverage/taxonomy matrix. That existing
qualification defect remains Item 12-owned and the merge gate was not
repeated. [PR #3668](https://github.com/sifr-lang/sifr/pull/3668) merged as
`99ec90c15e1dbffd68626fa5f9eaa90528d0624a`.

### Item 8A: Canonical cleanup effect and identity hardening

- [x] Shared branch suffix factoring preserves effects and lexical drop order.
- [x] IR and syntax cleanup share one conservative discardability contract,
  including unknown binary effects.
- [x] Private-field pruning preserves initializer effects and nested-module
  demand.
- [x] Iterator, length, and `None` rewrites require structural/type proof rather
  than method or token names.
- [x] All liveness consumers share one format-capture parser that handles
  width/precision captures.
- [x] Generated Clippy isolation prevents concurrent runs from invalidating a
  shared target while retaining deterministic diagnostics.

### Item 9: Algorithmic and Unicode performance

- [x] Indexed string operations do not repeatedly rescan Unicode text.
- [x] Character comparison does not allocate one-character strings.
- [x] Queue/deque and sorting operations use appropriate Rust structures and
  algorithms.
- [x] Representative corpus cases enforce asymptotic and allocation budgets.

### Item 9A: Character comparison state disambiguation

- [x] Out-of-range indexed characters compare unequal to present empty and
  multi-character strings, and inequality is the exact inverse.
- [x] Both operand orders, literals, variables, optional strings, negative and
  positive out-of-range indices, and valid Unicode scalar matches are covered.
- [x] Two genuinely absent optional values preserve their existing equality
  contract without allocating one-character strings.
- [x] The unrelated `compiler_safety` demo source behavior drift introduced in
  Item 9 is restored or moved under an explicit owner, and its companion is
  regenerated from the corrected source.

### Item 10: Runtime, stdlib bridge, and API deduplication

- [x] Runtime/support demand is computed once and rendered once.
- [x] No generated crate contains duplicate bridge bodies or duplicate public
  operation paths.
- [x] Unused support is absent, and bridge-size budgets catch recurrence.

### Item 10A: Module-scoped builtin error shadow identities

- [x] A user-defined error class keeps its exact module-qualified identity.
- [x] A builtin error referenced by a sibling module remains present even when
  another module shadows its bare name.
- [x] Late file-derived support demand is module-aware and cannot turn a
  per-module shadow into a crate-wide suppression veto.
- [x] Single-file, project, and generated test-project paths share the corrected
  identity and demand contract.
- [x] The flat generated-support trait invariant is explicit and enforced, and
  no production-unused error-reference helper remains.

### Item 11: Portable and secure generated projects

- [x] Portable emitted artifacts contain no host-specific absolute paths.
- [x] Ephemeral local dependency resolution is separated from distributable
  source and manifests.
- [x] Process invocation keeps executable/argument boundaries unless the user
  explicitly selected a shell API.
- [x] Allocation, path, and resource-limit conversions are checked.

### Item 11A: Generated-companion freshness and Item 11 integration

- [x] Start from reviewed Item 11 candidate
  `78c28c1e4c42bd85d685d3a3cffdf132fcdfcc40`; preserve its accepted
  portable-project, argument-boundary, and checked-conversion mechanisms.
- [x] Regenerate, through the candidate compiler rather than manual edits, the
  15 stale companions reported by the consumed Item 11 merge gate:
  `additional_modules`, `bisect`, `config_json_csv`, `container_methods`,
  `ergonomics`, `file_streams`, `glob`, `io`, `ordered_collections`, `stdlib`,
  `stdlib_ownership`, `structured_parsing_serialization`,
  `subscript_assignment`, `tempfiles_and_zip`, and `text_and_patterns`.
- [x] `python3 scripts/check_demo_emitted_freshness.py` passes on the exact
  Item 11A candidate, with no hand-edited generated output.
- [x] Close or supersede draft [#3687](https://github.com/sifr-lang/sifr/pull/3687)
  only after the integrated candidate receives Item 11A's own exact-SHA agent
  review and sole merge-profile gate. Do not rerun Item 11's consumed gate.

### Item 12: Residual semantic completion and full-corpus qualification

- [ ] Every actionable inventory row is closed with merged evidence.
- [ ] All generated demos, verification fixtures, project modes, and benchmark
  representatives are regenerated by the final compiler.
- [ ] Full generated-code quality, e2e, stdlib, algorithmic, formatting,
  Clippy, file-size, HIR, create-PR, and merge gates pass as applicable on the
  exact final source SHA.
- [ ] The remaining `islice` parity form, generated-code debt, qualification
  profile composition, and cold/warm timing evidence have explicit passing
  coverage.
- [ ] Item 12 receives its normal exact-SHA implementation review; it does not
  consume the whole-phase review.

### Item 12B: Bounded algorithmic dependency repair

On 2026-09-05, the user authorized the same worker to repair both repositories.

- Scope: repair external conversion and index-error source contracts, plus the
  compiler ownership mechanisms required to compile and execute those fixtures.
- Compiler scope includes loop sentinel reuse, repeat-count reuse, directly
  necessary same-mechanism corrections, and focused regression coverage.
- External source changes preserve every original case and algorithm behavior.
- The item includes the external PR/merge and the Sifr compiler/gitlink PR/merge.
- Earlier restrictions against these compiler changes are superseded.
  Unrelated Item 12 generated-quality work remains separate.
- Implementation starts from Sifr base
  `2dc4165fd9e7c34432a9b0d098188dc645aaca55` on the isolated Item 12B branch.
  Any prerequisite from retained Item 12 work requires explicit path-level provenance.
- External checkpoint `f6db5bd5d363b19a3040afd2a092f44ce32fd5bb`,
  Sifr handoff `1efb8720fa827f3bf19de17c7f010e3009f0e484`, and retained
  compiler candidate `8ad089a9458f35fcfa228e93fe44f4d69731828b` remain preserved.
- Qualification uses a newly built compiler from the isolated candidate.
  The retained frozen compiler is historical diagnostic evidence only.
- Review: one exact-SHA Opus review identifies both repository candidates.
  At most one remediation review is permitted. No whole-phase review is permitted.
- Gate: one merge-profile gate covers the exact final Sifr candidate.
  Skip create-PR. Do not repeat the merge gate.
- Close Item 12B and update its records, then stop. Do not start Item 12 or 12A.

#### Item 12B required tests

These commands run from the isolated Sifr worktree after the bounded implementation.
The focused regression names are fixed before test execution.

```bash
cargo build -p sifr
cargo test -p sifr_codegen item12b_loop_sentinel_reuse
cargo test -p sifr_codegen item12b_repeat_count_reuse
cargo test -p sifr_codegen
target/debug/sifr check verification/areas/algorithmic_compatibility/corpora/leetcode/src/0004_median_of_two_sorted_arrays.sifr
target/debug/sifr run verification/areas/algorithmic_compatibility/corpora/leetcode/src/0004_median_of_two_sorted_arrays.sifr
target/debug/sifr check verification/areas/algorithmic_compatibility/corpora/leetcode/src/0006_zigzag_conversion.sifr
target/debug/sifr run verification/areas/algorithmic_compatibility/corpora/leetcode/src/0006_zigzag_conversion.sifr
uv run --project verification --locked python -m sifr_verify areas run --area algorithmic_compatibility --suite leetcode-full
cargo fmt --check
cargo clippy -p sifr_codegen -- -D warnings
python3 scripts/check_file_size_guardrails.py
python3 scripts/check_hir_maintainability_guardrails.py
scripts/run_all_tests.sh
```

Apply the same `check` and `run` commands to every repaired corpus fixture.
Record the compiler SHA and binary digest for focused and full-corpus evidence.
Include relevant additional changed crates in the Clippy command.
Loop regressions cover repeated iterations, branch paths, and later sentinel uses.
Repeat-count regressions cover later uses, nested scopes, and effectful counts.

#### Item 12B checkpoint: qualification blocked on an unrelated Clippy defect

State on 2026-09-05: implementation checkpoint preserved; Item 12B is not closed.

- Sifr candidate: `673593f3ee234d58f03694e018abb145a843f787`,
  branch `codex/emitted-rust-excellence-item-12b`.
- External candidate: `330544ecf4f787c1a5fbed847469797ead92d24c`,
  branch `codex/item12b-source-contracts` in `sifr-lang/leetcode`.
- Both candidates are pushed. Neither repository has an Item 12B PR or merge.
- No Opus review or merge-profile gate was consumed.
- The isolated worktree remains `/tmp/sifr-item12b.akguMz/sifr`.
  The retained Item 12 compiler candidate remains separate and unchanged.

The newly built compiler has SHA-256
`56ef1dac97c474d76341f77aebefa37e750002bdf82e6a6f6c5509a91d85847c`.
The binary digest remained unchanged throughout this qualification attempt.

Completed evidence under `/tmp/sifr-item12b.akguMz/`:

- `codegen-singleton-full-2.log`: all 1,412 codegen tests pass.
  This includes all four named Item 12B ownership regressions.
- `compiler-singleton-build.log`: the compiler build passes.
- `native-primary/0004_median_of_two_sorted_arrays.json`: check and native run pass.
- `native-primary/0006_zigzag_conversion.json`: check and native run pass.
  Each JSON record contains both candidate SHAs, input digests, commands, and logs.
  These runs retain the original cases and execute the added ownership assertions.
- `cargo fmt --check`: pass.
- File-size guardrail: pass for 3,755 files, with the 900-line limit unchanged.
- HIR maintainability guardrail: pass.

Incomplete evidence is not a qualification pass:

- `leetcode-full-candidate.log` contains 113 passing cases before interruption.
  It is not a complete 411-case result.
- The two helper checks pass, but their native runs were interrupted.
  The remaining repaired fixtures still require their native runs.
- The worker stopped all owned qualification processes after the scope blocker.
  No background qualification process remains.
- Earlier complete diagnostic matrices cover earlier inputs.
  They do not qualify these candidates.

`clippy.log` records the blocker from
`cargo clippy -p sifr_codegen -- -D warnings`.
The unchanged `project_stdlib_nominals.rs:45` uses `Option::expect` in
`ProjectNominalRegistry::register_builtin`.
This defect exists in base `2dc4165fd9e7c34432a9b0d098188dc645aaca55` and current
main `2af89e75e5f97ec75e1b72c000fb3a6ebbbbb7cc`.
It concerns builtin-error registration, not sentinel or repeat-count ownership.
The worker did not suppress the diagnostic or import unrelated retained Item 12 code.

Next action: authorize or merge the builtin-registration repair recorded as Item 12C.
Then resume Item 12B qualification on the identified candidate inputs.
Complete every required fixture run and the canonical full corpus before review.
The exact-SHA review allowance and the single merge-profile gate remain unused.

#### Incorporated Item 12C: Builtin-registration scope amendment

- State: incorporated into Item 12B by explicit user authority on 2026-09-05.
- The earlier exclusion of this mechanism is superseded.
- This repair has no separate item, review, or gate.
- The implementation preserves both repository checkpoints and unrelated Item 12 work.
- Registration must consume validated builtin identity without a fallback,
  diagnostic suppression, or replacement panic.
- Focused command, recorded before execution:
  `cargo test -p sifr_codegen item12b_builtin_registration`.
- The regressions cover canonical identities, module shadows, and rejected non-builtin names.
- After this repair, resume the remaining Item 12B validation and closure steps.
- Owner: compiler builtin-error registration.
- Defect: `crates/sifr_codegen/src/project_stdlib_nominals.rs:45` fails strict
  Clippy with `clippy::expect_used`.
- Dependency: this unchanged defect blocks Item 12B's required crate Clippy check.
- The repair must preserve builtin-error identity and the registration invariant.
  It must not add a fallback or diagnostic suppression.

#### Item 12B checkpoint: builtin repair passes; native qualification fails

This checkpoint supersedes the earlier builtin-registration blocker.
Item 12C is implemented inside Item 12B. Item 12B is not closed.

- Sifr implementation candidate: `3f422b01633d23c2bc8d8ce8ca59057c6e56adea`.
- External candidate: `330544ecf4f787c1a5fbed847469797ead92d24c`.
- Both candidates are pushed. Neither repository has a PR or merge.
- No Opus review, remediation review, or merge-profile gate was consumed.
- The retained Item 12 compiler work remains separate and unchanged.

Registration now accepts a validated `BuiltinError` token.
The registry no longer performs a partial name lookup or calls `expect`.
Two regressions cover canonical identities, module shadows, and rejected non-builtin names.

The newly built compiler has SHA-256
`dbe640b31bdd181b93f82d967dd9e7c82092146482c554fb14e96fe42f28a3c3`.
The compiler binary and both source trees stayed unchanged throughout qualification.
Evidence is under `/tmp/sifr-item12b.akguMz/`:

- `builtin-focused.log`: both named builtin-registration regressions pass.
- `builtin-codegen-full.log`: all 1,414 codegen tests pass, including all four ownership regressions.
- `builtin-build.log`: the compiler build passes.
- `builtin-clippy.log`: strict codegen crate Clippy passes.
- Formatting, file-size (3,756 files), and HIR guardrails pass.
- `leetcode-full-3f422.log`: the complete canonical 411-case check finishes.
  It reports 410 passes and one failure, fixture 2002.
  This is a complete failing result, not a partial pass or native qualification.
  The canonical result is
  `target/verification/areas/algorithmic-compatibility-results.json`.
  Per-case results and taxonomy remain under
  `target/verification/areas/algorithmic_compatibility/`.
- `native-3f422/matrix.json`: complete coverage of all 90 changed source files.
  Checks pass for 89 files. Native builds and runs pass for 43 files.
  Native builds fail for 46 files. The failed check prevents the remaining native run.
  Median and zigzag both pass their checks and native assertions.
- `native-3f422/diagnostic_inventory.json`: every failing file, command, log, and diagnostic group.
  The following counts overlap where one file has several diagnostics.

| Diagnostic group | Files | Representative fixture |
|---|---:|---|
| Handler binding captured outside its scope | 13 | 0017 |
| Reused value moved | 12 | 0072 |
| Missing structured `TryExcept` lowering | 10 | 0044 |
| Narrowed value compared with `None` | 8 | 0102 |
| Missing `UnionFind.union` method emission | 4 | helpers/dsu |
| Recursive optional field mutability | 3 | 0025 |
| Nested assignment receives `Option<SifrInt>` instead of `SifrInt` | 1 | 0048 |
| Borrowed `str` clone emission | 1 | 1397 |
| Empty collection assertion type inference | 1 | 1203 |
| Unreceived checked shift result in source | 1 | 2002 |

The checked-shift receiving omission and approved ownership corrections remain Item 12B work.
They are not external authority blockers.
Other failures require control-flow, type-representation, or declaration-demand changes.
Those mechanisms are not builtin registration or sentinel/repeat reuse.

#### Item 12B continuation authority and regression commands

The user authorized all necessary next actions after the complete failure inventory.
The recorded execution dependencies now form part of the bounded Item 12B repair.
The previous scope-adjudication stop is superseded.
This includes checked-read control flow, exception capture and lowering, ownership,
method retention, assertion typing, and the checked-shift source omission.
It does not include unrelated Item 12 quality work or Item 12A.
The original review and single-gate limits remain unchanged.

Additional focused commands, recorded before test execution:

```bash
cargo test -p sifr_codegen item12b_
cargo test -p sifr_codegen item12b_exception_capture
cargo test -p sifr_codegen item12b_checked_read_control_flow
cargo test -p sifr_codegen item12b_repeated_value_ownership
cargo test -p sifr_codegen item12b_recursive_optional_mutability
cargo test -p sifr_codegen item12b_structured_exception
cargo test -p sifr_codegen item12b_method_retention
cargo test -p sifr_codegen item12b_empty_collection_assertion
```

The complete native matrix is the repair checklist, not a new discovery run.
Each regression must cover the relevant lexical, control-flow, or ownership negative case.
The new compiler requires new affected-input evidence. Earlier passes retain their recorded provenance.

The source batch also completes two missing checked-value contracts.
Fixture 2002 receives each checked shift into an explicit integer binding.
Fixture 0048 tests each optional matrix read before its corresponding write.
The matrix changes preserve read/write order and raise `IndexError` on absent values.
They do not substitute a default or remove an original case.

#### Item 12B integrated-base provenance

Main advanced to `c83dd7cde8daf54cdc4abd952903e9aa093c4183` through PR #3692.
Merge `b3d836354` integrates that reviewed base, including its dependency-feature normalization.
The test-module conflict keeps both new regression modules and main's renamed modules.
No retained Item 12 implementation was imported.

The merged naming policy forbids numbered planning labels in source names.
The new tests therefore use mechanism-oriented module names and the `corpus_repair_` filter.
This is a mechanical identity change; every case and assertion remains present.
The exact replacement focused command is recorded before execution:

```bash
cargo test -p sifr_codegen corpus_repair_
```

Main records unresolved SQL coverage classifications in
`ad-hoc-schema-first-sql-platform-review-follow-ups.md`.
They remain externally owned and are not absorbed into this dependency repair.
The single-gate limit is unchanged.

#### Item 12B continuation implementation evidence

The complete `7b50a83a91ce65dc17d91d73e54c14dcd1b67901` qualification has 411/411 canonical checks.
Its repaired-fixture matrix has 90/90 checks and 71/90 native passes.
The 19 native failures remain qualification failures, not a partial pass claim.
Raw evidence: `leetcode-full-continuation.log` and `native-continuation/diagnostic_inventory.json` under the owned temporary root.
The follow-up batch repairs those same mechanism paths: expression-local checked reads,
child-scope last-use accounting, owned argument adaptation, imported mutable receivers,
and empty assertions inside exception carriers. No unrelated Item 12 work is included.

Additional exact regression commands, recorded before execution:

```bash
cargo test -p sifr_codegen item12b_checked_read_control_flow_short_circuit_assignment
cargo test -p sifr_codegen item12b_structured_exception_root_error_and_dictionary_reads
cargo test -p sifr_codegen item12b_structured_exception_nested_while_checked_comparison
cargo test -p sifr_codegen item12b_repeated_value_ownership_condition_and_branch
cargo test -p sifr_codegen item12b_repeated_value_ownership_nested_arithmetic_and_defaults
cargo test -p sifr_codegen item12b_empty_collection_assertion_in_exception_carrier
```

Compiler implementation `580e3374c3aac2aa669ad06354fba02c618e0942` completes the recorded dependency batch.
Commit `18ab9bd969e70876a99875d8c719ad8b8d4daeb3` updates the existing union-rendering test expectation.
External candidate `0ef88e8b4f4906e410a3b2e9216248c11149b247` completes the two remaining source contracts.
No retained Item 12 compiler changes were imported.

Evidence under `/tmp/sifr-item12b.akguMz/`:

- `continuation-focused-4.log`: all 17 Item 12B regressions pass.
- `continuation-codegen-full-2.log`: all 1,425 codegen tests pass.
- `continuation-clippy.log`: strict codegen Clippy passes.
- `continuation-build.log`: the new compiler build passes.
- Formatting and file-size/HIR guardrails pass; the size check covers 3,760 files.

The compiler binary SHA-256 is
`04e449044644533db98fad9289d89355078f12b3e3bbd9bdb77d7f42398dfbfa`.
These are focused and crate-level results, not full-corpus qualification.
The earlier complete failing matrices remain historical evidence.
No Opus review or merge-profile gate has run.

#### Incorporated Item 12D: Native corpus emission dependencies

State: incorporated into Item 12B under the continuation authority.
Owner: compiler emission, tracked in this issue and the algorithmic issue.
This item does not reopen Item 12C or request authority for its completed repair.

The confirmed scope blocker is checked-read control-flow and optional representation.
In fixture 0102, the source tests a left read only inside its left-length branch.
Generated Rust inserts a left-read `let Some(...) else { break; }` before that branch.
A second read narrows the value to `Vec<SifrInt>`, but its `None` comparison remains.
The first transformation can terminate a valid right-only iteration.
The second transformation fails Rust compilation with `E0277`.

Evidence: `native-3f422/0102.emitted.rs:116` and
`native-3f422/0102_binary_tree_level_order_traversal.run.log`.
The relevant producer is `crates/sifr_codegen/src/checked_place.rs`.
Its `checked_place_read_witness` path removes the optional representation.
This producer is unchanged from the isolated base.
A repair must preserve branch-local read demand, absence paths, and effect order.
Removing source guards or adding a fallback would not correct that mechanism.

The full diagnostic inventory also records structured exception lowering,
handler capture scope, missing method emission, and assertion type inference.
Their final producer-level decomposition remains unimplemented.
The ownership groups stay in Item 12B rather than moving into this later item.

Next action: adjudicate the newly recorded emission mechanisms as dependency scope.
Then finish the approved source and ownership corrections on the preserved branch.
Complete qualification on the final inputs before either repository merge.
The exact-SHA Opus allowance and single merge-profile gate remain unused.

All owned qualification commands completed. No background qualification run remains.
No compiler, fixture, test, baseline, or safety policy changed after this evidence.
Later commits update records only and do not claim a new implementation SHA pass.

#### Item 12B implementation provenance

The compiler changes start from the merged base, not the retained Item 12 candidate.
The existing ownership materializer now serves integer local bindings and repeat counts.
Its rename does not change the other callers.
The repeat producer preserves operand order for both operand positions.
Singleton repetition uses the existing exact-integer range without a host-sized cast.
This correction also satisfies the two existing singleton-repeat codegen tests.

The median fixture adds native loop, branch, sentinel-reuse, and large-integer assertions.
The zigzag fixture adds native repeat-count, operand-order, and single-evaluation assertions.
Both fixtures retain every original case.
External repairs propagate checked errors through explicit receiving contexts.
Reassignments retain their original binding identity and statement order.

### Item 12A: Phase closure and whole-phase review

- [ ] One exact-SHA whole-phase agent review is satisfied.
- [ ] Architecture and roadmap records reflect the delivered architecture.
- [ ] This issue is archived only after every closure condition is true.
- [ ] Closure contains no compiler implementation work. If the whole-phase
  review finds a new implementation mechanism defect, create a later
  implementation item and a subsequent closure item instead of repairing it
  inside Item 12A or taking a third review round.

## Item Ledger

| Item | State | PR | Merge SHA | Validation | Exact-SHA review | Result |
|---:|---|---|---|---|---|---|
| 0 | merged | [#3574](https://github.com/sifr-lang/sifr/pull/3574) | `8d292f9395fee51ef8b348a413ea496a33c5ce38` | Candidate `b75a3c471f7ec8b4cb798e112e123bfb13d78b83`: inventory, mutation self-test, Python/JSON syntax, file-size, HIR maintainability, docs-link, and diff hygiene checks passed. No compiler files changed, so Sifr gates were omitted. | [Initial and sole remediation review](https://github.com/sifr-lang/sifr/pull/3574#issuecomment-5462303681): both NOT SATISFIED. The original evidence blocker was fixed; the remediation review's new checker mechanism is assigned to Item 1 under the explicit review limit. | Contract and 32-row inventory merged; three missing mutation branches and related checker provenance hardening are owned by Item 1. |
| 1 | merged | [#3578](https://github.com/sifr-lang/sifr/pull/3578) | `b86eec0be7b7be2b5ddf012fea9cbcced286c342` | Candidate `b0fb5c2049b81fe28fc4b076c34ac624f8249e94`: full generated-code-quality profile passed 9 variants with 0 failures across 91 positive projects; exact safety, rustfmt, 38,957-diagnostic/105-lint Clippy, determinism, all 262 authoritative companions, recursive freshness, audit/debt/surface mutations, Python/JSON, file-size, HIR, driver, docs-link, and diff hygiene passed. No compiler files changed, so Sifr gates were omitted. | [Initial review](https://github.com/sifr-lang/sifr/pull/3578#issuecomment-5463056720) NOT SATISFIED; [sole remediation review](https://github.com/sifr-lang/sifr/pull/3578#issuecomment-5463053848) SATISFIED with all four blockers resolved and no new in-scope mechanism defect. | Exact surface digests, fail-closed quality protocols, strict source/lint policies, 18 negative seeds, and a 33-row governed audit inventory merged; all Item 0 deferred checker findings are resolved. |
| 2 | merged | [#3580](https://github.com/sifr-lang/sifr/pull/3580) | `d618a7be107550629c3331ea7fdb3f76e28e0dce` | Compiler candidate `aa97d2ca6d0da1ec5700b02d3f57ef864a450a53`: 1,151 codegen tests and 557 driver tests passed; Clippy, formatting, generated inventory/freshness, diagnostics governance, file-size, HIR, and driver checks passed. The one create-PR gate completed every reached check and all 28 runtime-platform variants with zero failures before its cold rebuild exceeded the 120-second step budget. The one merge gate passed static, core-language, differential, Rust interop, coverage, and all 30 Python-interop variants before finding three stale diagnostic baselines. Follow-up `7b3ba45d25e07adabb820c9f80463534060d42ee` changed only diagnostic fixtures/governance; 178 of 179 full baseline variants passed before the sole new wording mismatch was corrected, and exact checks then passed. Neither gate was repeated. | [Initial review](https://github.com/sifr-lang/sifr/pull/3580#issuecomment-5465345414) on `d4aea519efebdf29bad472a9795afcdd72c4f865` and [sole remediation review](https://github.com/sifr-lang/sifr/pull/3580#issuecomment-5465345486) on `9606b67b84ae5865105415399d647319b455bb99` were NOT SATISFIED. The initial slice-step panic was fixed. The remediation review's new exact-ratio proof/codegen mismatch is assigned to Item 3 under the no-third-review rule. | Canonical inline-small/`BigInt` `SifrInt`, exact arithmetic and conversion paths, fixed-width boundaries, constants, ranges, collections, unions, and Rust/Python interop merged with debug/release and corpus evidence. |
| 3 | merged | [#3587](https://github.com/sifr-lang/sifr/pull/3587) | `fe95d220be2819464d6231080d57e47444b0d429` | Reviewed compiler candidate `229c2687923d97c72531bb4e81deb047833367b1`: 1,156 codegen and 1,053 lowering tests passed; workspace Clippy, formatting, file-size, HIR, demo freshness, generated determinism, panic scan, demo corpus, intrinsic panic lint, diagnostics governance, and smoke/representative/full generated Clippy passed. The one create-PR gate stopped on a stale retained-intrinsic governance row after all preceding checks passed; docs-only `6e7c5b32dc9574a40ff5624834daa768613a0b14` removed it and the exact checker plus self-test passed. The one merge gate passed static, core-language, differential, Rust interop, coverage, and 29 of 30 Python-interop variants; its sole `sqlite-context` compiler failure is assigned to Item 3A. Neither gate was repeated. | [Initial review](https://github.com/sifr-lang/sifr/pull/3587#issuecomment-5466942667) on `2e3867cbe3546e09a94f391672410808315f3b25` was NOT SATISFIED; [sole remediation review](https://github.com/sifr-lang/sifr/pull/3587#issuecomment-5466942761) on `229c2687923d97c72531bb4e81deb047833367b1` was SATISFIED. The loop-constant blocker was fixed; later mechanism findings are assigned to Item 3A under the review limit. | Typed structural failure discharge, exact ratio materialization, checked Decimal/BigDecimal/bytes/random/input operations, structured try/finally and context carriers, pre-render invariant validation, regenerated demos, and retired `SIFR-INT-0006` governance merged. |
| 3A | merged | [#3591](https://github.com/sifr-lang/sifr/pull/3591) | `d88192be94823a6e1c0f30b712d2f7440ac2c6b4` | Compiler candidate `719bd96ad5b4d11c507b356bd6fece2ab6d4ac3f`: 4 IR, 1,167 codegen, and 1,072 lowering tests passed with one intentional ignore; all non-E2E Sifr test groups, focused sync/async/SQLite runtime regressions, formatting, HIR, file-size, and item-owned Clippy checks passed. The sole create-PR run passed every functional check but exceeded the runtime-platform step budget after the required cold-cache cleanup; its later warm merge run passed that area in 24.5 seconds. The sole merge run passed core language, CPython differential, Rust/Python interop, diagnostics, runtime, algorithmic, tooling, and all emitted-Rust corpus, panic-scan, rustfmt, Clippy, determinism, and freshness checks. Its only failure was a pre-existing surface inventory record: both base and candidate contain the same 704 E2E paths and digest while the record expects 701. Neither gate was repeated. | [Initial review](https://github.com/sifr-lang/sifr/pull/3591#issuecomment-5467141026) on `8b7b46cd629e6530d693462e10590ec287b931c3` was NOT SATISFIED; [sole remediation review](https://github.com/sifr-lang/sifr/pull/3591#issuecomment-5467149668) on `719bd96ad5b4d11c507b356bd6fece2ab6d4ac3f` was SATISFIED with no blockers. The imported-constant proof regression was fixed through lexical module-frame resolution. | Suppressible Python contexts now rejoin typed carriers; exact-integer facts respect lexical binding identity and nested-call mutation; loop/context emitted fallthrough agrees with static flow; sync, async-for, and SQLite regressions merged. |
| 4 | merged | [#3601](https://github.com/sifr-lang/sifr/pull/3601) | `ab1bd8371faf090f3f7549524147b0fbabbd3b7a` | Compiler candidate `a91f43d2bace42c5579d02cf0a9bce57e4962300`: 1,172 codegen, 1,073 lowering with one intentional ignore, 84 runtime, and 8 exact-integer architecture tests passed; E2E passed 705/705 with signature `9f98912689339124`; workspace Clippy, formatting, HIR, file-size, generated inventory, demo freshness, governed corpus, and panic scan passed. The full generated-quality run's 91 rustfmt-classified cases passed individually, but its exact aggregate debt signature changed and remains Item 8-owned. The sole create-PR gate passed every reached guardrail plus Rust interop, coverage, diagnostics, and 23 of 24 Python-interop variants. The sole merge gate passed all guardrails, Rust interop, coverage, core language, CPython differential, and 29 of 30 Python-interop variants. Both gates stopped only on the same underconstrained callback-decoder array conversion assigned to Item 4A, and neither was repeated. | [Initial review](https://github.com/sifr-lang/sifr/pull/3601#issuecomment-5470119120) on `054c14f728ed13f6ed548647a5669504a36d729f` was NOT SATISFIED; [sole remediation review](https://github.com/sifr-lang/sifr/pull/3601#issuecomment-5470119110) on `a91f43d2bace42c5579d02cf0a9bce57e4962300` was NOT SATISFIED. The straight-line stale-value and E0502 blocker was fixed. The remediation review's new loop-back-edge and post-deletion failure-semantics defects are assigned to Item 4A under the no-third-review rule. | One typed checked-place architecture now covers negative and nested reads, writes, deletes, augmented assignment, membership, unpacking, optional targets, and generated direct-index removal. Mutation-aware straight-line witness refresh, checked non-empty vectors, typed failure plans, and regenerated companions merged; bounded residual lifecycle defects are owned by Item 4A. |
| 4A | merged | [#3608](https://github.com/sifr-lang/sifr/pull/3608) | `9af05a15e1d2eaae6866b7976f425dc5b3077ca4` | Reviewed compiler candidate `13fc41d0d8e4465305b6bd4402f6f0557be91260`: 1,078 lowering tests passed with one intentional ignore; targeted codegen/lowering Clippy, native checked-place E2E, all seven callback examples, demo freshness, panic scans, formatting, HIR, and file-size checks passed. The one create-PR gate and one merge gate each stopped at the same profile preflight defect because the profile omitted required `postgresql-live-differential`; neither was repeated. After concurrent async-cleanup work reached `main`, integration commit `f8869ebc24647364e3c9d0862d53a18c43030885` preserved both ordinary and closable async-for witness refresh; 1,180 codegen plus lowering/runtime suites, targeted Clippy, two native fixtures, demo freshness, formatting, HIR, and file-size checks passed. | [Initial review](https://github.com/sifr-lang/sifr/pull/3608#issuecomment-5470878459) on `91fe545fcbe75a99bb8b75002fb68d9692a9fdd8` was NOT SATISFIED; [sole remediation review](https://github.com/sifr-lang/sifr/pull/3608#issuecomment-5470912500) on `13fc41d0d8e4465305b6bd4402f6f0557be91260` was SATISFIED. The async-for invalidation blocker was fixed. Its newly identified async-for guard leak and non-terminating missing-witness fallback are assigned to Item 4B under the no-third-review rule. | Loop-carried and while-condition witnesses now refresh at repeat boundaries; mutation dependencies invalidate before sync/async loop lowering; post-delete reads use current typed failure semantics; unused witness scaffolding is demand-driven; callback arrays are explicit and panic-free. The two bounded second-review defects are owned by Item 4B. |
| 4B | merged | [#3612](https://github.com/sifr-lang/sifr/pull/3612) | `67c1804df84d0367e380ebef1ee14845ec1971fb` | Reviewed compiler candidate `68981d07cb6d088803d199e8924ecc9ab06d0a91`: 1,181 codegen and 1,079 lowering tests passed with one intentional ignore; strict targeted Clippy, native checked-place plus ordinary/closable async-for fixtures, demo freshness, formatting, HIR, and file-size checks passed. The sole create-PR and merge gates each stopped before tests because their then-current profiles omitted required `postgresql-live-differential`; neither was repeated. Concurrent PostgreSQL work then repaired the profiles and merged conflict-free as integration commit `5b1739b4853523b7a9b81bf1c8f1a6af28497a4c`; full codegen/lowering suites, targeted Clippy, formatting, diff, and 3,488-file guardrails passed after integration. | [Initial review](https://github.com/sifr-lang/sifr/pull/3612#issuecomment-5471106525) on `0e8bdd33af00c6bab5d43c02b614ee1f8052c70a` was SATISFIED; [sole remediation review](https://github.com/sifr-lang/sifr/pull/3612#issuecomment-5471132474) on `68981d07cb6d088803d199e8924ecc9ab06d0a91` was SATISFIED. Compiler-inserted while witness exits now use the canonical loop-else marker. The remediation review's new deeply nested mutation-tail continuation defect is assigned to Item 4C under the no-third-review rule. | Async-for body guards restore at loop exit; loop-carried witnesses use loop-kind progress/termination; body and condition refreshes preserve loop-else semantics; precise lowering diagnostics and native sync/async regressions merged. Remaining non-back-edge continuation scoping is owned by Item 4C. |
| 4C | merged | [#3615](https://github.com/sifr-lang/sifr/pull/3615) | `2579fcd198acd105da4a93b794a82601524541a8` | Compiler candidate `6a849e8d9d8457b7e463486e52f6e629d5da6b86`: 1,183 codegen and 1,082 lowering tests passed with one intentional ignore; focused mutable-call invalidation, checked-place shape, native nested-loop, workspace Clippy, formatting, HIR, diff, and file-size checks passed. The non-E2E Sifr sweep's `numeric_sentinels.sifr` type diagnostic reproduced identically on exact base `6862b4a21ebd0917a54f5744c6e22960242bf00b` and is Item 8-owned. The sole create-PR and merge gates each stopped before tests because their current profiles omitted required `postgresql-live-differential` and `postgresql-live-runtime`; neither was repeated. | [Initial exact-SHA review](https://github.com/sifr-lang/sifr/pull/3615#issuecomment-5471369789) on `6a849e8d9d8457b7e463486e52f6e629d5da6b86` was SATISFIED with no blockers. Its non-blocking receiver-effect and clone-bound findings are assigned to Item 7; refresh-default evidence and wider loop-else scaffold deduplication are assigned to Item 8. | Stored witness exit payloads are eliminated; straight-line renewal cannot skip tails or replay outer control flow; mutable-call guards invalidate before codegen; simple and structured exits share one constructor; nested while/for/if and condition-marker regressions merged. |
| 5 | merged | [#3622](https://github.com/sifr-lang/sifr/pull/3622) | `79b963aa6a909303b1152546a0f91e699cd8f1cf` | Final compiler candidate `cc63e5d4e86725543ed111b3c194d2e89ab5e629`: 1,183 codegen and 1,085 lowering tests passed with one intentional ignore; workspace Clippy, formatting, HIR, diff, 3,515-file guardrail, audit inventory, and exact demo freshness passed. Native evidence covered suspension-by-suspension side effects, 10,001 pulls from unbounded `count`, `islice` over that source, async lazy start/close/exhaustion, CPython/consolidated itertools behavior, and bounded `cycle` without an extra source effect. The 91-project generated corpus compiled on the initial candidate with panic, intrinsic-panic, determinism, demo, freshness, and every per-project rustfmt/Clippy classification passing; the exact remediation reran all affected lowering, native, Clippy, formatting, and freshness checks. The sole create-PR and merge gates each stopped before tests because both profiles omit required SQL suites `host-tools`, `postgresql-live-differential`, and `postgresql-live-runtime`; neither was repeated. | [Initial review](https://github.com/sifr-lang/sifr/pull/3622#issuecomment-5472340524) on `1029541fd69c9b1d6726f53331cc5319f17f3be3` was NOT SATISFIED; [sole remediation review](https://github.com/sifr-lang/sifr/pull/3622#issuecomment-5472360882) on `cc63e5d4e86725543ed111b3c194d2e89ab5e629` was SATISFIED with no blockers. The discarded `None`-typed return-expression defect and bounded-`cycle` over-pull were corrected. The remediation review's newly noted optional-element `cycle` semantics are assigned to Item 6 under the no-third-review rule. | Sync and async generators now own resumable producer futures; generator returns exhaust without silently discarding expressions; infinite and adapter iterators are consumer-driven; authoritative demos and native/codegen/lowering regressions are merged. |
| 6 | merged | [#3629](https://github.com/sifr-lang/sifr/pull/3629) | `e3980da373afb250bf579ee6636a40bec81de64a` | Final compiler candidate `511ec05e3ff21295fc0ba725f39abbe9900b1cdb`: 1,189 codegen tests passed; strict codegen Clippy, formatting, HIR, diff, 3,554-file guardrail, audit inventory, and demo freshness passed. Ten regenerated generic-bound companions compiled directly. Focused lowering, runtime, exact-integer, stdlib API, E2E, seven native release fixtures, generated corpus, panic, determinism, and freshness evidence covered string padding, signed-zero division, JSON order, sized IO/seek/tell/flush/error kinds, owned iterators, optional-element `cycle`, optional-stop `islice`, contextual option typing, and decimal no-fallback behavior. The sole create-PR and merge gates each stopped before tests because both profiles omit required SQL suites `host-tools`, `migration-engine`, `postgresql-live-differential`, `postgresql-live-migrations`, `postgresql-live-runtime`, `postgresql-live-schema-tools`, `postgresql-migrations`, and `schema-tools`; neither was repeated. | [Initial review](https://github.com/sifr-lang/sifr/pull/3629#issuecomment-5477091120) on `1a11fcf55e578a57463148c0f53d7154f7accf9d` was NOT SATISFIED; [sole remediation review](https://github.com/sifr-lang/sifr/pull/3629#issuecomment-5477091342) on `511ec05e3ff21295fc0ba725f39abbe9900b1cdb` was NOT SATISFIED. The remediation fixed the original transitive `PartialOrd`/`Display`/`Hash + Eq` blocker across the authoritative surface. Its new arithmetic-bound alpha-renaming defect is assigned to Item 6A under the no-third-review rule. | Safe string padding, exact division sign, ordered JSON, complete IO bridges, owned iterator semantics, contextual option emission, demand-driven generic-bound closure, and governed decimal precision merged. Arithmetic bound substitution and odd-center parity are bounded Item 6A follow-ups. |
| 6A | merged | [#3633](https://github.com/sifr-lang/sifr/pull/3633) | `035e71160470d4344851695addaaaecc2fb27f3e` | Compiler candidate `aff53a422ee8c55185d0110c51483be0d600375d`: 1,190 codegen, 89 runtime, and 8 exact-integer architecture tests passed; strict codegen/runtime Clippy, formatting, HIR, diff, 3,562-file guardrail, audit inventory, demo freshness, and standalone emitted metadata compilation passed. Differently named `Addable` forwarding and odd-center fixtures compiled and ran as release-native binaries; the authoritative companion emits `Add<Output = T>` for the callee and `Add<Output = U>` for its relay. The sole create-PR and merge gates each stopped before tests because both profiles omit required SQL suites `host-tools`, `migration-engine`, `postgresql-live-differential`, `postgresql-live-migrations`, `postgresql-live-runtime`, `postgresql-live-schema-tools`, `postgresql-migrations`, `schema-polymorphism`, and `schema-tools`; neither was repeated. | [Exact-SHA review](https://github.com/sifr-lang/sifr/pull/3633#issuecomment-5477506746) on `aff53a422ee8c55185d0110c51483be0d600375d` was SATISFIED with no blockers. It independently reproduced fresh emission, release-native fixtures, all self-output arithmetic substitutions, preserved ordinary closure, and exhaustive small CPython `center` parity. | One canonical structural bound constructor now separates ordinary traits from self-output traits, fixed-point propagation carries no parameter spelling, final rendering uses the receiving parameter, and CPython odd-margin centering is exact. |
| 7 | merged | [#3637](https://github.com/sifr-lang/sifr/pull/3637) | `73465ce982b790094031d174151a8638cfbcf35b` | Compiler candidate `778e13268d0ff619791a44152e4e52c0df369053`: 1,213 codegen and 1,094 lowering tests passed with one intentional ignore; focused ownership, generic-class `Addable`, context capture, receiver mutation, checked witness, IO, recursive/DP, and clone-budget tests passed. The expanded protocol-bound fixture compiled and ran generic `Accumulator[str]`, `list.insert(&str)`, and `set.add(&str)`. Full E2E passed 717 fixtures; only exact-base `numeric_sentinels` failed under its existing Item 8 ownership. Workspace Clippy, formatting, diff, 3,612-file guardrail, HIR maintainability, audit inventory, regenerated demo freshness, and representative direct native builds passed. The sole create-PR and merge gates each stopped at preflight because both profiles omit required SQL suites `host-tools`, `migration-engine`, `mysql-live`, `mysql-provider`, `postgresql-live-differential`, `postgresql-live-migrations`, `postgresql-live-runtime`, `postgresql-live-schema-tools`, `postgresql-migrations`, `schema-polymorphism`, and `schema-tools`; neither was repeated. | [Initial exact-SHA review](https://github.com/sifr-lang/sifr/pull/3637#issuecomment-5481578415) on `27840aa67d438956b87f87f96822a4b868a69e2b` was NOT SATISFIED; [sole remediation review](https://github.com/sifr-lang/sifr/pull/3637#issuecomment-5481578704) on `778e13268d0ff619791a44152e4e52c0df369053` was NOT SATISFIED. The remediation fixed both original blockers: class-owned `__SifrAdd` support demand and raw borrowed-string clones at collection ownership boundaries. Its new receiver-effect precision regression and latent non-registry `setdefault` boundary gap are assigned to Item 7A under the no-third-review rule. | Explicit ownership/materialization planning, unsized views, clone-chain simplification and budgets, recursive borrowed options, callable-effect separation, context-target capture, checked clone diagnostics, IO clone cleanup, and ownership-correct numeric/string `Addable` support merged. The bounded second-review mechanism defects are owned by Item 7A. |
| 7A | merged | [#3639](https://github.com/sifr-lang/sifr/pull/3639) | `917a4e898a881d7966d78e645c01143d9290eb54` | Final compiler candidate `e77bf60695f27cee1fa71a1e3eea2e8facad1b75`: 1,217 codegen and 1,101 lowering tests passed with one intentional ignore; focused receiver-summary, fact-splitting, local-binding fallback, Copy/affine ownership, emitted-shape, and release-native regressions passed. Full E2E passed 718 fixtures; only exact-base `numeric_sentinels` failed under Item 8 ownership. Workspace Clippy, formatting, diff, 3,613-file guardrail, HIR maintainability, audit inventory, regenerated demo freshness, and direct native execution passed. The sole create-PR and merge gates each stopped at preflight because both profiles omit required SQL suites `host-tools`, `migration-engine`, `mysql-live`, `mysql-provider`, `postgresql-live-differential`, `postgresql-live-migrations`, `postgresql-live-runtime`, `postgresql-live-schema-tools`, `postgresql-migrations`, `schema-polymorphism`, and `schema-tools`; neither was repeated. | [Initial exact-SHA review](https://github.com/sifr-lang/sifr/pull/3639#issuecomment-5482649819) on `57a09a3121c34f5e5504ac3c7b7791e665855e8a` was NOT SATISFIED; [sole remediation review](https://github.com/sifr-lang/sifr/pull/3639#issuecomment-5482650047) on `e77bf60695f27cee1fa71a1e3eea2e8facad1b75` was SATISFIED. The remediation split accessibility from non-`None` facts and restored Copy/affine `setdefault` ownership guards. Its new end-relative negative-index finding is assigned to Item 7B under the no-third-review rule. | One typed receiver summary now preserves length/key accessibility while invalidating exact positional/value facts; all `setdefault` entrypoints share an ownership-safe operation boundary; Copy values emit no redundant clones. Bounded end-relative and affine-return follow-ups are owned by Item 7B. |
| 7B | merged | [#3643](https://github.com/sifr-lang/sifr/pull/3643) | `17c6e49d1be6d19834530d6475353539d0efb124` | Exact compiler candidate `5b6c68d0508c5e79b0ddfc8d598480314ef8ef14`: 1,218 codegen and 1,103 lowering tests passed with one intentional ignore; 569 E2E fail fixtures, focused negative-index append/extend, absolute-index preservation, affine `setdefault`, fact-domain, and release-native insertion/existing-return evidence passed. Full E2E passed 718 fixtures; only unchanged `numeric_sentinels` failed under Item 8 ownership. Workspace Clippy, formatting, diff, 3,614-file guardrail, HIR maintainability, audit inventory, and exact demo freshness passed. The sole create-PR and merge gates each stopped at preflight because both profiles omit required SQL suites `host-tools`, `migration-engine`, `mysql-live`, `mysql-provider`, `postgresql-live-differential`, `postgresql-live-migrations`, `postgresql-live-runtime`, `postgresql-live-schema-tools`, `postgresql-migrations`, `schema-polymorphism`, and `schema-tools`; neither was repeated. | [Exact-SHA review](https://github.com/sifr-lang/sifr/pull/3643#issuecomment-5483156923) on `5b6c68d0508c5e79b0ddfc8d598480314ef8ef14` was SATISFIED with no blockers. It independently traced literal-negative classification, growth-sensitive clearing, the affine insertion/return rejection, defensive codegen boundary, and non-collection fact domain. | Append/extend now preserve stable absolute facts while invalidating end-relative facts; affine `setdefault` is rejected before emission with one ownership contract; mutable buffers and join sets carry an explicit no-relevant-sequence-facts domain. |
| 8 | merged | [#3668](https://github.com/sifr-lang/sifr/pull/3668) | `99ec90c15e1dbffd68626fa5f9eaa90528d0624a` | Compiler implementation `49f375e1619185d76e6cfc3b90d7e20ff786cce0`: 1,349 codegen tests, non-E2E CLI suites, workspace Clippy, formatting, diff, file-size/HIR guardrails, 724-path inventory, 91-project corpus, panic/rustfmt/determinism/freshness checks, the direct 724th native fixture, and two byte-identical 262-companion plus selected-Clippy runs passed. The sole create-PR gate found one missing runtime-root manifest entry after all preceding checks passed; explicit documentation-only candidate `fa661c6eccd4c1fa3eb0092e3106ac4d44dddeda` fixed it and the targeted guard passed without repeating the gate. The sole merge gate passed all Item 8 checks and stopped only on unchanged SQL coverage/taxonomy failures owned by Item 12. | [Initial exact-SHA review](https://github.com/sifr-lang/sifr/pull/3668#issuecomment-5517105667) on `84ebe95b928cfe076d9af21e1bc06c1da3bc08c4` was NOT SATISFIED; [sole remediation review](https://github.com/sifr-lang/sifr/pull/3668#issuecomment-5523601034) on `a77acce704ccab8bf568ea4156ff05dd706c66c1` was SATISFIED with no blockers. Its new non-blocking mechanisms are Item 8A/#3670-owned without a third review. | Canonical structured cleanup, exact generated-debt governance, canonical generic identity, optional-place normalization, source-only materialization, regenerated authoritative demos, and focused semantic/shape regressions merged. |
| 8A | merged | [#3672](https://github.com/sifr-lang/sifr/pull/3672) | `484717a156995ccf637b87fcd4ee33f29fd1c4af` | Exact compiler candidate `46c95c86582761c9a1f4003577f97ae8fb723ead`: 1,359 codegen tests and all non-E2E Sifr suites passed; workspace Clippy, formatting, Python syntax, HIR, diff, 3,730-file guardrail, demo freshness, full generated inventory/panic/rustfmt, required-demo corpus/determinism, and two concurrent strict-Clippy runs with distinct run-owned targets and identical diagnostics passed. Full E2E reached 720/724, exposing one in-scope optional-string length callback defect plus one unchanged timeout miss; the callback proof was corrected and all four affected fixtures then passed 4/4. The sole create-PR and merge gates passed every reached Item 8A guardrail and Rust interop check, then stopped only on the same pre-existing SQL coverage/taxonomy readiness debt owned by Item 12; neither gate was repeated. | [Exact-SHA review](https://github.com/sifr-lang/sifr/pull/3672#issuecomment-5525818647) on `46c95c86582761c9a1f4003577f97ae8fb723ead` was SATISFIED with no blockers. Five pre-existing mechanism findings and one infrastructure observation are assigned to Item 12. | Shared conservative discardability, drop-safe branch suffix factoring, effect-safe private-field demand, structurally typed Option/iterator rewrites, one complete format-capture parser, and deterministic run-owned Clippy targets merged. |
| 9 | merged | [#3675](https://github.com/sifr-lang/sifr/pull/3675) | `145fc217606bf3ba85d819d0065b80ec29ea6579` | Exact compiler candidate `6ab6adc08f3ad253bcb4d1d080d5f2c5554cae70`: 1,379 codegen tests and every non-E2E Sifr group passed; full E2E passed 725/725; workspace Clippy, formatting, HIR, diff, 3,739-file guardrail, regenerated demo freshness, and the authoritative 262-companion exact-debt audit passed. Generated panic, intrinsic-panic, rustfmt, and determinism modes passed before the final narrow capture-ABI correction, which was then covered by full E2E and all companions. Fresh generated text/i18n corpus and demo builds stopped only in `tinyvec 1.13.0` under Rust 1.98; changing only the temporary lock to `tinyvec 1.11.0` passed, so Item 11 owns dependency-resolution portability. The sole create-PR and merge gates passed all reached guardrails and Rust interop checks, then stopped on the unchanged SQL coverage/taxonomy readiness debt owned by Item 12; neither gate was repeated. | [Initial exact-SHA review](https://github.com/sifr-lang/sifr/pull/3675#issuecomment-5533536041) on `59b8a6e8b0c586c096510d5f461d968dd409cad2` requested six remediations, all implemented. [Sole remediation review](https://github.com/sifr-lang/sifr/pull/3675#issuecomment-5533471365) on `6ab6adc08f3ad253bcb4d1d080d5f2c5554cae70` was NOT SATISFIED after finding a new character-comparison state-collapse mechanism. Under the no-third-review rule it is immediate Item 9A/[#3676](https://github.com/sifr-lang/sifr/issues/3676). | Unicode scan caching, allocation-free character comparison, constant-time deque front operations, key-once stable sorting, memoized body analysis, deduplicated while witnesses, last-use collection moves, statement `setdefault`, generated complexity budgets, and regenerated companions merged. The bounded second-review semantic defect is owned by Item 9A. |
| 9A | merged | [#3678](https://github.com/sifr-lang/sifr/pull/3678) | `4fde625cf4bd64b712370d8e0515cae97fa58195` | Exact compiler candidate `9f311def58ee809d55f8f12517775c6faedb082d`: 1,380 codegen tests and every non-E2E Sifr group passed; full E2E passed 726/726 with signature `11427061fe6b7498`; the direct native Item 9A and restored `compiler_safety` runs passed; workspace Clippy, formatting, HIR, diff, 3,741-file guardrail, regenerated demo freshness, inventory/self-test, intrinsic panic lint, governed corpus/panic/rustfmt/determinism checks, and the authoritative 262-companion strict audit passed. The fresh text/i18n corpus reproduced only the `tinyvec 1.13.0` Rust 1.98 failure owned by Item 11. The sole create-PR and merge gates passed every reached guardrail and Rust interop check, then stopped on the unchanged SQL coverage/taxonomy readiness debt owned by Item 12; neither gate was repeated. | [Exact-SHA review](https://github.com/sifr-lang/sifr/pull/3678#issuecomment-5534399665) on `9f311def58ee809d55f8f12517775c6faedb082d` was SATISFIED with no blocking findings. Suggestions about literal-typed variable specialization and documenting the demo's intentional discarded callback call are assigned to Item 12. | Nested comparison state now distinguishes absence, present invalid character width, and a present Unicode scalar without one-character allocation; every operand, optionality, index, and comparison-operator form has native and emitted-shape coverage. The `compiler_safety` observable contract is restored and all affected companions are regenerated. |
| 10 | merged | [#3681](https://github.com/sifr-lang/sifr/pull/3681) | `ddc4a55f126845dfde15f27bf00c8356806a8dba` | Exact compiler candidate `0bb73783b2daf2d0f20b63cbe16407493d4d217a`: 1,404 codegen tests and every non-E2E Sifr group passed; full E2E passed 726/726 with signature `11427061fe6b7498`; workspace Clippy, formatting, HIR, diff, 3,750-file guardrail, regenerated demo freshness, inventory, intrinsic-panic, 84-project corpus, panic, rustfmt, 92-check determinism, companion compilation, and support-size budgets passed. The sole create-PR and merge gates passed every reached guardrail and Rust interop check, then stopped on the unchanged SQL coverage/taxonomy readiness debt owned by Item 12; neither gate was repeated. | [Initial exact-SHA review](https://github.com/sifr-lang/sifr/pull/3681#issuecomment-5537359489) was SATISFIED. The [sole remediation review](https://github.com/sifr-lang/sifr/pull/3681#issuecomment-5537359721) was NOT SATISFIED after finding a new cross-module builtin-error suppression mechanism defect; under the no-third-review rule it is immediate Item 10A/[#3682](https://github.com/sifr-lang/sifr/issues/3682). | One typed support plan now owns runtime and stdlib demand across single-file, project, and test-project generation; aggregate support renders once; bridge bodies conflict-check and deduplicate; final-source pruning removes unconsumed support and reconstructs dependency metadata. The bounded second-review identity defect is owned by Item 10A. |
| 10A | merged | [#3684](https://github.com/sifr-lang/sifr/pull/3684) | `948c4d47146cdcaf6dbf49705d30c47e11959cc5` | Exact compiler candidate `c9d0fb34331c32fb90342debf1eea28a0c6ee7e1`: all 5 Item 10A codegen tests and both Item 10A driver tests passed, including native project and generated test-project compilation/execution with distinct local and builtin `ValueError` shapes; formatting and the 3,751-file guardrail passed. Per the session instruction, the create-PR gate was skipped because this exact SHA merged in the same session. The sole merge gate passed generated-demo freshness, HIR/file-size/ownership/dependency/resource/stdlib/driver/verification guardrails, and the complete Rust-interop area, then stopped only on unchanged SQL coverage/taxonomy readiness debt already owned by Item 12; the gate was not repeated. | [Exact-SHA review](https://github.com/sifr-lang/sifr/pull/3684#issuecomment-5538828920) on `c9d0fb34331c32fb90342debf1eea28a0c6ee7e1` was SATISFIED with no blocking findings. No remediation review was required. The pre-existing fixture lock failure is [#3685](https://github.com/sifr-lang/sifr/issues/3685); two non-blocking suggestions are assigned to Item 12. | Builtin errors now use canonical `sifr.builtin.*` identities, module shadows never become project-wide support vetoes, relocation preserves colliding local definitions, single-file suppression remains local, generated support traits fail closed outside the flat owner layout, and the unused reference helper is removed. |
| 11 | merged | [#3689](https://github.com/sifr-lang/sifr/pull/3689) (supersedes closed draft [#3687](https://github.com/sifr-lang/sifr/pull/3687)) | `bbc85bcd3e538e201f7f82fa535c7cef43a5ac6e` | Reviewed Item 11 candidate `78c28c1e4c42bd85d685d3a3cffdf132fcdfcc40` retained its focused passing tests, fixture [#3685](https://github.com/sifr-lang/sifr/issues/3685), formatting, HIR maintainability, and 3,753-file guardrail evidence. Its consumed merge-profile gate found the 15 stale companions later regenerated by Item 11A and was not rerun. | [Initial review](https://github.com/sifr-lang/sifr/pull/3687#issuecomment-5539520805) was NOT SATISFIED; [sole remediation review](https://github.com/sifr-lang/sifr/pull/3687#issuecomment-5539569910) on `78c28c1e4c42bd85d685d3a3cffdf132fcdfcc40` was SATISFIED. Item 11A's [exact-SHA integration review](https://github.com/sifr-lang/sifr/pull/3689#issuecomment-5539747194) confirmed that no accepted mechanism file changed after that candidate. | Portable manifests and dependency resolution, executable/argument boundaries, checked conversions, and the refreshed fixture lock merged through Item 11A. |
| 11A | merged | [#3689](https://github.com/sifr-lang/sifr/pull/3689) | `bbc85bcd3e538e201f7f82fa535c7cef43a5ac6e` | Exact candidate `ec380f0b221d65516516291018008434c1c1e62a`: the canonical updater changed exactly the 15 item-owned companions, and `python3 scripts/check_demo_emitted_freshness.py --sifr target/debug/sifr` passed with all companions fresh. Per the session instruction, the create-PR gate was skipped because this SHA merged in the same session. The [sole merge-profile gate](https://github.com/sifr-lang/sifr/pull/3689#issuecomment-5539791652) passed Cargo setup, HIR, the 3,753-file guardrail, generated-demo freshness, source/ownership/resource/stdlib/driver/verification guardrails, and all 10 Rust-interop variants, then stopped only on unchanged SQL coverage/taxonomy debt already owned by Item 12; it was not rerun. | [Exact-SHA agent review](https://github.com/sifr-lang/sifr/pull/3689#issuecomment-5539747194) on `ec380f0b221d65516516291018008434c1c1e62a` was SATISFIED with no blocking findings. No remediation review was required. Its string-receiver evaluation suggestion is assigned to Item 12. | The reviewed Item 11 candidate was integrated without mechanism changes, all 15 stale companions were compiler-regenerated, draft #3687 was closed as superseded, and the integrated candidate merged. |

## Deferred Findings

| Source | Finding | Owner | Required action |
|---|---|---|---|
| Item 1 remediation review | Three idealized, non-authoritative companions were removed while their sibling sources remain. | Item 8 | Decide whether the sources require authoritative emitted companions; regenerate from the Item 8 compiler when required, otherwise preserve their explicit non-authoritative status through closure. |
| Item 1 remediation review | ERQ-025 still describes the fifteen legacy demo `main.rs` files removed by Item 1. | Item 8 | Close the already-discharged row when Item 8 reconciles stale snapshots and generated ceremony. |
| Item 1 remediation review | The exact discovery inventory is broader than the 91-project executable quality corpus. | Item 12 | State and verify the intended qualification relationship, and ensure final full-corpus closure cannot leave an inventoried entrypoint class unexercised. |
| Item 1 remediation review | Checked-in emitted companions receive freshness and safety scans but are not individually governed by rustfmt and Clippy. | Item 8 | Regenerate or remove the remaining producer debt and make authoritative checked-in output satisfy the canonical formatting/lint contract. |
| Item 2 remediation review | A reduced exact-integer quotient can be float-representable even when either original operand is not; lowering proves the reduced ratio while infallible codegen independently calls `to_f64_proven_exact` on each operand, leaving a source-reachable proof panic. | Item 3 | Make the static proof and emitted operation share one precondition, add the cancelled-large-factor regression, and remove the data-dependent proof assertion from generated user paths. |
| Item 2 remediation review | Exact integer true division loses Python's negative-zero sign for a zero numerator and negative denominator. | Item 6 | Derive a zero quotient's sign from both operands and add signed-zero differential coverage. |
| Item 2 remediation review | Rejected slice-step lowering records an error but recovers with a step-less slice HIR node, which can produce misleading cascading diagnostics. | Item 3 | Propagate failed step lowering consistently with failed start/stop lowering while preserving the primary typed diagnostic and source span. |
| Item 2 merge-gate diagnostics | `SIFR-INT-0006` remains registered and documented although source exact-integer true division now lowers to a typed `Result` and misuse renders contextual `SIFR-TYPE-0002`; only the lower-level type-system path still produces the old code. | Item 3 | Retire the unreachable registry, renderer, test, catalog, and documentation path, or restore a justified source-reachable typed-failure use with a rendered baseline. |
| Item 2 merge-gate diagnostics | `SIFR-TYPE-0901` retains a warning IR variant, renderer, registry entry, catalog, and docs after exact arithmetic removed its final producer. | Item 8 | Remove the dead warning mechanism and regenerate all diagnostic governance artifacts. |
| Item 2 gate output | The Python Arrow resource implementation retains an unused private `handle` method. | Item 8 | Remove the dead method through the canonical support implementation and prove generated support remains warning-clean. |
| Item 2 generated-project inspection | Source constants generate helper names such as `__const_BASE`, retaining avoidable non-snake-case naming debt. | Item 8 | Canonicalize generated constant helper names and references without broad naming allowances, including project imports and re-exports. |
| Item 3 initial review | Bare `return` in a binding-promoting `try`/`except` inside `Result[None, E]` can emit the wrong optional/control-flow payload. | Item 8 | Unify none-like return normalization across direct, optional, and binding-promotion carriers. |
| Item 3 initial review | `break` and `continue` inside `try`/`finally` nested in a loop can escape into a Rust closure and fail with E0267. | Item 8 | Represent loop control structurally in the canonical try/finally carrier. |
| Item 3 initial review | `raise` can type-check in a non-`Result` function and then emit an incompatible Rust `Err`. | Item 8 | Reject the invalid source path before emission through canonical return/error validation. |
| Item 3 initial review | Phase 34 still claims retired `SIFR-INT-0006` behavior. | Item 8 | Reconcile stale historical generated-code records with the current diagnostic surface. |
| Item 3 initial review | Pre-render forbidden-failure validation checks `MacroCall` but not `FormatMacro`. | Item 8 | Cover every macro-bearing Rust IR variant with one structural validation path and mutation evidence. |
| Item 3 initial review | Exact literal materialization can leave source bindings unread in emitted Rust. | Item 8 | Remove dead generated bindings through canonical liveness/simplification rather than warning allowances. |
| Item 3 remediation review | A cleared local exact-integer fact can fall back to a same-named module constant and fold the wrong value. | Item 3A | Make exact-integer proof lookup binding-identity aware and add local-shadow regressions. |
| Item 3 remediation review | A nested function called in a loop can mutate a `nonlocal` integer without invalidating the enclosing loop-carried fact. | Item 3A | Model called nested-function mutation in loop fact invalidation and prove while/for/async-for behavior. |
| Item 3 remediation review | Async-for constant-fact invalidation lacks a dedicated regression. | Item 3A | Add exact async-for evidence alongside the repaired mutation mechanism. |
| Item 3 remediation review | Nested loops re-walk inner bodies once per enclosing level. | Item 9 | Replace repeated body collection with a single pre-pass or memoized summary and enforce a lowering-cost regression. |
| Item 3 generated safety scan | Remaining direct collection indexing is the only generated panic-surface class in the full corpus. | Item 4 | Route every read, write, delete, and nested place through the checked-place architecture. |
| Item 3 merge gate | Suppressed Python-context body errors can leave an enclosing direct-return try carrier expecting `Result` while the emitted suppression arm yields unit. | Item 3A | Make context suppression a typed continuation in static flow analysis and sync/async emission; compile and run the SQLite context example plus reduced regressions. |
| Item 3A initial review | Callable-alias effect closure currently shares mutation summaries with retained-callback contract inference and can overstate `FnMut` requirements for callbacks that do not invoke the alias. | Item 7 | Separate const-fact call effects from retained-callback ownership contracts and add a retained-callback regression before changing inference. |
| Item 3A initial review | Nested-function local-definition collection does not explicitly account for Python context-manager item targets, which can misclassify a context target as an outer capture. | Item 7 | Make captured-binding analysis account for every binding-producing statement, including `with` and `async with` targets, with ownership regressions. |
| Item 3A initial review | Pattern rendering recognizes `true` and `false` as literals even though source identifier validation does not yet reserve those Rust spellings consistently. | Item 8 | Centralize legal generated-name and literal-pattern handling so a source identifier cannot silently change pattern meaning. |
| Item 3A remediation review | Exact imported/module integer facts use a bare-name module map whose immutability and invalidation boundary is implicit. | Item 8 | Encode or assert the module-frame immutability invariant, preferably through binding identity, and document the distinct invalidation boundary before mutable globals can exist. |
| Item 3A local Clippy audit | Strict workspace/all-target Clippy exposes untouched compiler and test lint debt, including annotation-resolution needless borrowing and structural-record ownership/`expect` findings. | Item 8 | Remove the underlying lint debt without broad allowances and make maintained compiler/test surfaces warning-clean under the phase policy. |
| Item 3A merge gate | The generated-code surface inventory expects 701 E2E pass sources, but the exact Item 3A base and candidate both contain the same 704 paths and digest `ef6a17a107fa114027c96eb2947afc71430a781834df64b97df608629dc10b87`. | Item 8 | Refresh the authoritative inventory from the owning producer and add it to stale generated-record reconciliation; Item 3A added or removed no E2E source path. |
| Item 3A create-PR gate | A required first cold-cache run exceeded the runtime-platform 120-second step budget although all 28 variants passed; the warmed merge run completed the same area in 24.5 seconds. | Item 12 | Ensure final qualification budgets distinguish mandated cold-cache setup from warm blocking evidence and retain both timing receipts. |
| Item 4 remediation review | A checked-place witness established outside a loop can survive a mutating loop back-edge, producing stale list values or an E0502 dictionary borrow on later iterations. | Item 4A | Invalidate before entering any repeatable block whose body mutates a witness dependency, and establish a fresh checked read inside each iteration before use. |
| Item 4 remediation review | Refresh after deletion reuses the original membership guard's exit action, so a later missing read can return the guard fallback instead of raising the operation's typed missing-key error. | Item 4A | Derive the refreshed read's failure continuation from the post-mutation operation rather than replaying proof-establishment control flow. |
| Item 4 create-PR and merge gates | Callback argument decoding replaced direct vector indexing with an underconstrained fixed-array `try_into`; one-argument callback examples fail Rust inference with E0282. | Item 4A | Emit an explicit fixed-array type or an equivalent type-directed checked decoder for every callback arity, with Python interop regressions. |
| Item 4 full generated-quality run | All 91 governed rustfmt cases retained their expected individual classification, but changed emitted source produced aggregate signature `c62a991cbb6e89aa92fa2cd0514ed03433d88b135fb662f52ab19527ca955687` instead of the locked Item 1 signature. | Item 8 | Remove the underlying formatting debt through canonical Rust IR/emission cleanup; do not rebase the exact debt signature to changed debt. |
| Item 4A remediation review | Async-for does not restore sequence guards after its body, so a proof established only inside a possibly zero-iteration loop can escape and authorize a later checked read. | Item 4B | Give async-for the same save/restore guard-state bracket as sync loops and prove zero-iteration behavior. |
| Item 4A remediation review | A loop-carried witness without an original missing action wraps the entire body in `if let`; in a `while`, indirect mutation can then skip the progress update forever. | Item 4B | Make loop-kind control flow override the branch fallback at refresh sites and prove the missing path terminates or advances. |
| Item 4A remediation review | Negative loop-invalidation regressions assert only that lowering failed, and async-for refresh lacks native runtime coverage. | Item 4B | Assert the checked-place diagnostic identity and add ordinary plus closable async-for runtime regressions. |
| Item 4A remediation review | A key read in both a while condition and body can be refreshed twice per iteration. | Item 9 | Deduplicate condition and body refresh plans and include the operation count in emitted-complexity evidence. |
| Item 4A direct full E2E | `parsers_and_encoders` and `structured_data_formats` deterministically disagree on JSON object order because the isolated generated group enables `serde_json` without `preserve_order`. | Item 6 | Reconcile generated JSON map-order semantics with the language contract and add deterministic isolated-group coverage. |
| Item 4A create-PR and merge gates | Both generated verification profiles omit the required `postgresql-live-differential` suite and therefore fail before running tests. | Item 12 | Repair or reconcile final qualification profile composition so required platform suites are selected and preflight passes. |
| Item 4B initial review | Straight-line mutation-tail refresh still replays an earlier witness missing action or wraps the remaining tail in body-skipping `if let` when no action exists. | Item 4C | Invalidate mutable-call dependencies before lowering and derive the fresh read from its current operation contract; never skip or replay the proof-establishment path. |
| Item 4B remediation review | An outer loop witness's stored missing action can be emitted inside a deeply nested inner loop/branch mutation tail, targeting the wrong loop and, after Item 4B, assigning the outer `_broke` marker. | Item 4C | Scope refresh continuations to the current structured region and prove outer `while ... else` plus inner `for`/`if` mutation/read behavior. |
| Item 4B remediation review | Simple loop lowering independently constructs `_broke = true; break` instead of sharing the structured emitter's canonical helper. | Item 4C | Route simple and structured loop breaks through one canonical constructor and cross-path regressions. |
| Item 4B remediation review | Condition-refresh plus `while ... else` has native evidence but no direct codegen shape assertion. | Item 4C | Add a unit assertion for `_broke = true` before the condition-refresh break and preserve the natural condition-false bare break. |
| Item 4B remediation review | Loop-invalidated optional reads report the downstream unsupported operator rather than a dedicated proof-invalidation diagnostic. | Item 8 | Decide the canonical user-facing diagnostic during structured emission cleanup and add governed rendering evidence if a dedicated code is warranted. |
| Item 4B native remediation fixture | A `while ... else` whose else body returns and is followed by another return can emit a non-exhaustive Rust `if` in value position (E0317). | Item 8 | Normalize loop-else tail/control-flow representation in structured Rust IR and add the return-ending else regression. |
| Item 4C exact-SHA review | Receiver-mutating calls are not represented by `mutable_arg_places`; checked-place invalidation therefore depends on lowering's currently incomplete fixed builtin receiver-mutation list. | Item 7 | Unify mutable receiver and argument effect summaries with the explicit ownership plan, then add user-defined `mut self`, class-method, and builtin shrinking-method checked-place regressions. |
| Item 4C exact-SHA review | Borrowed witness preparation inserts element clones before mutation without proving or diagnosing a `Clone` requirement for non-copy class elements. | Item 7 | Make witness preservation participate in the explicit ownership/clone plan and add a `list[NonCopyClass]` regression that either borrows safely or reports a Sifr diagnostic before Rust compilation. |
| Item 4C exact-SHA review | Straight-line renewal uses the previous binding as the absent fallback; current lowering makes absence unreachable for surviving guard-preserving mutations, but the reachability invariant is implicit. | Item 8 | Encode the refresh precondition structurally or validate it before rendering, and add negative mutation evidence so a future new mutation form cannot silently retain stale data. |
| Item 4C exact-SHA review | Loop-else setup and dispatch scaffolds remain duplicated across structured loop emitters even though the break-marker constructor is now canonical. | Item 8 | Give canonical Rust IR one loop-else scaffold constructor and prove sync, async, and statement-block paths render the same structure. |
| Item 4C broad validation | `numeric_sentinels.sifr` is still classified as an E2E pass fixture although `nums[l]` lacks a statically established index proof and fails with `None | int`; exact Item 4C base and candidate agree. | Item 8 | Reconcile the fixture with the checked-place contract or implement a sound proof mechanism, then restore the non-E2E Sifr sweep without weakening optional-read diagnostics. |
| Item 4C create-PR and merge gates | Both current verification profiles omit required `postgresql-live-differential` and `postgresql-live-runtime` suites and stop at preflight before running tests. | Item 12 | Repair final profile composition and retain a mutation test proving every required SQL platform suite is selected before the one final qualification run. |
| Item 5 initial review | Nested generator lowering can reach the statement-block yield path with `in_generator_closure` false and emit an undefined `__sifr_yielder`; the new architecture did not introduce the former dangling-support behavior. | Item 8 | Represent nested generator bodies through the canonical structured generator-emission path or reject the unsupported form before Rust emission, with direct nested-function coverage. |
| Item 5 initial review | The owned-iterator adaptations deliberately require explicit `iter(...)`, and `islice(it, start, None)` does not yet model CPython's unbounded-tail form. | Item 6 | Decide and document the language-level iterator ownership/parity contract, then add differential coverage for explicit ownership and the optional-stop form. |
| Item 5 remediation review | `cycle` advances its output count without yielding when an instantiated optional element is represented by `None`, so optional-element sources can be dropped or miscounted. | Item 6 | Give generic optional values an unambiguous element representation in `cycle` and add focused optional-element semantic coverage. |
| Item 5 full generated-quality and direct E2E runs | The generated surface inventory expects 705 E2E paths while the current tree has 718; aggregate rustfmt debt also changed although every one of the 91 individual project classifications passed. | Item 8 | Reconcile the authoritative inventory and remove producer formatting debt through canonical emission; do not bless a changed aggregate debt signature. |
| Item 5 direct full E2E | `sliding_window_narrowing.sifr` emits an `Option<String>` key into `HashSet<String>::remove`, causing one generated compile defect to fan out across 279 fixtures after a cold rebuild. | Item 8 | Normalize the checked optional index/place before method-argument emission and add an isolated native regression before restoring the broad E2E sweep. |
| Item 5 create-PR and merge gates | Both profiles omit required SQL platform suites `host-tools`, `postgresql-live-differential`, and `postgresql-live-runtime`, so both one-shot gates stopped at preflight before tests. | Item 12 | Repair final profile composition and preserve mutation coverage proving every required SQL suite is selected before the phase's final qualification run. |
| Item 6 initial review | Compiler-special `open()` default metadata is keyed by a bare class name, so a same-basename user class can receive synthetic defaults despite distinct nominal identity. | Item 8 | Key compiler-owned method defaults by canonical class identity and add a same-basename negative regression. |
| Item 6 initial review | The single-argument unbounded form `islice(it, None)` remains unsupported although `islice(it, start, None)` is implemented. | Item 12 | Complete and document the remaining iterator parity form during full-corpus semantic closure. |
| Item 6 initial review | New IO carriers retain redundant nested clones such as `(size.clone()).clone()`. | Item 7 | Eliminate the redundant ownership operations through the explicit clone plan and include the new IO shapes in clone budgets. |
| Item 6 remediation review | Propagated arithmetic bounds copy the callee's embedded type-parameter spelling into the caller, so differently named `Addable` forwarding can emit an out-of-scope Rust type and fail with E0412. | Item 6A | Represent parameterized bounds structurally or substitute the formal parameter with each corresponding caller parameter; add lowering, emitted-shape, and native forwarding evidence. |
| Item 6 remediation review | Generic class-method bounds and module-level function bounds use disjoint closures, leaving class-method forwarding outside the repaired mechanism. | Item 8 | Unify generic-bound demand across free functions and class methods through canonical callable identity, with a class-method forwarding regression. |
| Item 6 remediation review | Local-scope-only call collection excludes generic callees invoked from nested functions or closures inside a generic body. | Item 8 | Model lexical generic-call effects explicitly and prove nested forwarding without leaking nested-only demands into unrelated scopes. |
| Item 6 remediation review | Structural-mismatch fallback can propagate a callee's full bound set to every caller parameter mentioned inside one composite actual type. | Item 8 | Replace fallback over-constraint with structural parameter correspondence and add multi-parameter composite regressions. |
| Item 6 remediation review | Generic-bound requirements and callee lookup use bare function names, leaving same-named generic functions vulnerable to cross-contamination. | Item 8 | Key the closure by canonical function identity and prove same-basename functions remain distinct. |
| Item 6 create-PR and merge gates | Both profiles omit required SQL suites `host-tools`, `migration-engine`, `postgresql-live-differential`, `postgresql-live-migrations`, `postgresql-live-runtime`, `postgresql-live-schema-tools`, `postgresql-migrations`, and `schema-tools`, so both one-shot gates stopped at preflight. | Item 12 | Reconcile final profile composition and retain mutation coverage proving every required SQL suite is selected before final qualification. |
| Item 6A exact-SHA review | `Addable` admits `str`, but generic `+` emits owned right-hand operands and therefore requires unavailable `String: Add<String>` instead of Rust's `String: Add<&str>`. | Item 7 | Make generic binary ownership and bounds agree with every admitted `Addable` member, and add a string instantiation beside the integer forwarding fixture. |
| Item 6A exact-SHA review | The hand-authored, non-authoritative `demos/protocol_bounds/idiomatic.rs` no longer mirrors the source demo's added `relay_add` behavior. | Item 8 | Reconcile or retire non-authoritative idiomatic companions under the canonical generated-snapshot policy. |
| Item 6A create-PR and merge gates | Both profiles omit required SQL suites `host-tools`, `migration-engine`, `postgresql-live-differential`, `postgresql-live-migrations`, `postgresql-live-runtime`, `postgresql-live-schema-tools`, `postgresql-migrations`, `schema-polymorphism`, and `schema-tools`, so both one-shot gates stopped at preflight. | Item 12 | Reconcile final profile composition and prove every currently required SQL suite is selected before final qualification. |
| Item 7 initial review | Unguarded dictionary value reads and dictionary/set lookup-key borrowing retain a silent fallback and `Borrow<&str>`-family mismatch that reproduce on the exact base. | Item 8 | Normalize checked optional-place reads and borrowed lookup keys through canonical Rust IR without silent values or double-reference query types. |
| Item 7 remediation review | Convention-driven receiver invalidation currently clears length and membership facts for growth-only and proof-preserving operations after the legacy shrinking-only summary was removed. | Item 7A | Introduce one typed receiver-effect summary that invalidates only facts an operation can falsify; prove growth, removal, and positional-reordering behavior. |
| Item 7 remediation review | `methods/dict.rs::lower_setdefault` relies on registry callers to materialize key/default ownership, while the local-binding fallback emitter can reach the operation without that contract. | Item 7A | Put owned key/default materialization at the shared `setdefault` boundary or route every entrypoint through one prepared-argument plan, with reaching shape/native evidence. |
| Item 7 remediation review | Registry literal and entry boundaries clone every non-copy named local even when the value is dead after the operation. | Item 9 | Add ownership-plan last-use move promotion and allocation budgets without weakening reuse semantics. |
| Item 7 remediation review | Nested generic functions with source bounds are rejected before codegen, so support-demand closure has no nested bound source today. | Item 8 | Reconcile nested generic declaration support or preserve an explicit checked rejection while canonical generic callable identity is implemented. |
| Item 7 create-PR and merge gates | Both profiles omit required SQL suites `host-tools`, `migration-engine`, `mysql-live`, `mysql-provider`, `postgresql-live-differential`, `postgresql-live-migrations`, `postgresql-live-runtime`, `postgresql-live-schema-tools`, `postgresql-migrations`, `schema-polymorphism`, and `schema-tools`, so both one-shot gates stopped at preflight. | Item 12 | Reconcile final profile composition and prove every currently required SQL suite is selected before final qualification. |
| Item 7A remediation review | Growth preserves a non-`None` fact for negative indices even though append/extend changes which element an end-relative index names. | Item 7B | Classify end-relative subscript facts separately and invalidate their exact value fact on growth, with append/extend negative-index regressions and preserved nonnegative evidence. |
| Item 7A remediation review | Affine `setdefault` storage arguments bypass materialization, but the returned-value path and evidence do not yet establish one valid affine ownership contract. | Item 7B | Prove and implement the operation's affine return contract or reject the unsupported surface before emission; add reaching emitted/native evidence. |
| Item 7A remediation review | Statement-position `setdefault` still computes a discarded cloned return value. | Item 9 | Make emission context and last-use planning avoid return-value materialization when the result is discarded, with clone/allocation budgets. |
| Item 7A remediation review | `PythonBuffer.write` is classified as value mutation and preserves receiver facts without an explicit proof that no relevant sequence fact can target the buffer. | Item 7B | Prove the fact-domain exclusion or conservatively invalidate relevant facts, with a receiver-summary regression. |
| Item 7A create-PR and merge gates | Both profiles omit required SQL suites `host-tools`, `migration-engine`, `mysql-live`, `mysql-provider`, `postgresql-live-differential`, `postgresql-live-migrations`, `postgresql-live-runtime`, `postgresql-live-schema-tools`, `postgresql-migrations`, `schema-polymorphism`, and `schema-tools`, so both one-shot gates stopped at preflight. | Item 12 | Reconcile final profile composition and prove every currently required SQL suite is selected before final qualification. |
| Item 7B exact-SHA review | `JoinSet` shares the no-relevant-sequence-facts domain with `PythonBuffer`; this is sound for today's growth-only mutable methods but a future removal method could inherit preservation silently. | Item 8 | Make fact-domain eligibility structural and exhaustive per receiver operation, and cover every mutable non-collection method in the summary regression. |
| Item 7B exact-SHA review | Growth stability is conservatively derived from literal index sign, so variable list indices over-invalidate and dict keys carry irrelevant growth metadata. | Item 8 | Derive reference stability from canonical typed index facts and receiver kind without weakening negative-index soundness. |
| Item 7B exact-SHA review | The generic reusable-value method set retains an unreachable `setdefault` affine branch after the dedicated ownership rejection. | Item 8 | Give the affine `setdefault` contract one canonical diagnostic owner and remove the unreachable generic branch. |
| Item 7B exact-SHA review | Defensive affine `setdefault` codegen declines with `None`, which would become a silent lowering miss if the frontend contract regressed. | Item 8 | Replace defensive silent decline with a structured internal codegen invariant diagnostic while preserving the source-facing rejection. |
| Item 7B create-PR and merge gates | Both profiles omit required SQL suites `host-tools`, `migration-engine`, `mysql-live`, `mysql-provider`, `postgresql-live-differential`, `postgresql-live-migrations`, `postgresql-live-runtime`, `postgresql-live-schema-tools`, `postgresql-migrations`, `schema-polymorphism`, and `schema-tools`, so both one-shot gates stopped at preflight. | Item 12 | Reconcile final profile composition and prove every currently required SQL suite is selected before final qualification. |
| Item 8 package validation | Exact `origin/main` `74bbb636744adaacb8c3eca09108b6fff9725357` fails two TypeVar diagnostic-message assertions after the producer wording changed from “simple type name(s)” to “type name(s)” without updating the tests. | [#3667](https://github.com/sifr-lang/sifr/issues/3667) | Repair the stale exact-message expectations in their own owner; Item 8 does not change TypeVar semantics or absorb this exact-base failure. |
| Item 8 package validation | Exact `origin/main` `74bbb636744adaacb8c3eca09108b6fff9725357` fails `tests::attached_api_codegen::non_string_leaf_negative_is_package_compilable` because its checked-in fixture lock is stale. | [#3669](https://github.com/sifr-lang/sifr/issues/3669) | Refresh and govern the attached-API fixture lock in its own owner; Item 8 does not absorb an exact-base artifact failure. |
| Item 8 remediation review | Shared `if` suffix factoring can move a token-identical effect after branch-local values are dropped because its guard proves name disjointness but not effect/drop-order equivalence. | Item 8A / [#3670](https://github.com/sifr-lang/sifr/issues/3670) | Make suffix factoring effect- and drop-order-aware, with direct non-Copy/Drop and side-effect regressions. |
| Item 8 remediation review | IR dead-binding cleanup treats every binary expression with pure operands as discardable while syntax cleanup deliberately treats unknown binary effects conservatively. | Item 8A / [#3670](https://github.com/sifr-lang/sifr/issues/3670) | Give both layers one conservative discardability contract and prove unknown operator effects survive. |
| Item 8 remediation review | Private-field pruning can delete side-effecting struct-literal initializers and omits nested-module demand. | Item 8A / [#3670](https://github.com/sifr-lang/sifr/issues/3670) | Preserve initializer effects and traverse qualified nested-module references before pruning. |
| Item 8 remediation review | Iterator, length, and `None` rewrites rely on names or token shapes that can match incompatible `Option`, `Result`, slice, or non-option operations. | Item 8A / [#3670](https://github.com/sifr-lang/sifr/issues/3670) | Require structural/type proof for each rewrite and add negative lookalike regressions. |
| Item 8 remediation review | Three format-capture collectors are duplicated and omit dynamic width/precision captures such as `{:width$}`. | Item 8A / [#3670](https://github.com/sifr-lang/sifr/issues/3670) | Consolidate capture parsing and prove all liveness consumers preserve width/precision bindings. |
| Item 8 remediation review | Per-package generated Clippy cleanup can invalidate a shared target when two quality runs overlap. | Item 8A / [#3670](https://github.com/sifr-lang/sifr/issues/3670) | Isolate or synchronize cleanup while retaining deterministic diagnostics and explicit concurrency evidence. |
| Item 8 merge gate | The sole merge gate passed every Item 8 guardrail and stopped in coverage/taxonomy readiness on unclassified SQL packages/targets and stale SQL milestone wording; no reported failure path changed in Item 8. | Item 12 | Reconcile the final coverage/profile taxonomy and prove every current SQL package and target is classified before final qualification. |
| Item 8A exact-SHA review | `assignment_cleanup.rs` still deletes some unused initializers through a separate name-based purity rule instead of the shared conservative discardability contract. | Item 12 | Route every deletion-adjacent cleanup through one proved discardability contract and add effectful lookalike regressions. |
| Item 8A exact-SHA review | Implicit captures in `panic!`, `unreachable!`, and `todo!` are not recognized by the shared format parser's macro-family routing. | Item 12 | Make macro-family coverage explicit and exhaustive, or prove those macros cannot occur on governed generated surfaces. |
| Item 8A exact-SHA review | Struct literals nested inside macro token streams are invisible to private-field demand, effect retention, and pruning. | Item 12 | Traverse or conservatively retain macro-contained struct construction so definitions and literals cannot diverge. |
| Item 8A exact-SHA review | The `Option[str]` length callback uses `String::len` byte length while ordinary Sifr string length counts Unicode scalar values. | Item 12 | Emit the canonical character-count operation for optional strings and add non-ASCII semantic coverage. |
| Item 8A exact-SHA review | IR iterator classification in `stmt_support_emitter/iterator_lowering.rs` still uses method names instead of structural type proof. | Item 12 | Replace the remaining name-based iterator classification with the canonical typed proof and negative lookalike coverage. |
| Item 8A exact-SHA review | Failed generated-Clippy runs preserve invocation-owned Cargo targets, which can accumulate substantial disk usage. | Item 12 | Add bounded evidence retention or explicit safe cleanup while preserving failed-run diagnostics and concurrent isolation. |
| Item 8A create-PR and merge gates | Both one-shot gates passed every reached Item 8A guardrail and Rust interop check, then stopped on the unchanged unclassified SQL packages/targets and stale SQL milestone taxonomy already reproduced by Item 8. | Item 12 | Reconcile coverage/profile taxonomy and prove every current SQL package and target is classified before final qualification. |
| Item 9 remediation review | Allocation-free `Char`/`Str` comparison collapses an absent indexed character and a present empty or multi-character string to the same `None`, making out-of-range equality true and inequality false. | Item 9A / [#3676](https://github.com/sifr-lang/sifr/issues/3676) | Keep absence distinct from failed single-character extraction across operand order, literals, variables, optionals, Unicode, and both comparison operators without restoring one-character allocation. |
| Item 9 remediation review | The unified body prepass does not record the `anext(x)` mutation source retained by the previous query implementation. | Item 12 | Reconcile iterator-advance mutation fidelity in the canonical prepass and add a reachable negative or proof that the omitted fact cannot affect witnesses or narrowing. |
| Item 9 remediation review | Isinstance-arm mutation now includes signature and nested-capture effects and can conservatively add `mut` bindings beyond the former query. | Item 12 | Remove any resulting generated `unused_mut` debt while preserving the wider sound mutation analysis. |
| Item 9 remediation review | `demos/compiler_safety/main.sifr` changed its callable-field behavior and asserted output even though Item 9 required producer and generated-companion work, not demo behavior changes. | Item 9A / [#3676](https://github.com/sifr-lang/sifr/issues/3676) | Restore the prior demo contract or move the coverage under an explicit semantic owner, then regenerate the authoritative companion. |
| Item 9 generated-project validation | Fresh generated text/i18n projects resolve `tinyvec 1.13.0`, which fails to compile under Rust 1.98; changing only the temporary generated lock to the workspace-compatible `tinyvec 1.11.0` passes. | Item 11 | Give generated projects a reproducible, toolchain-compatible dependency-resolution contract and prove fresh materialization without hand-edited temporary locks. |
| Item 9 initial review | The checked-read collector's narrow lowering path can fail closed on nested boolean conditions and lose optimization facts even though semantic lowering remains correct. | Item 12 | Reconcile checked-read collection with canonical short-circuit condition lowering and add negative lookalike and nested-boolean coverage. |
| Item 9 create-PR and merge gates | Both one-shot gates passed every reached Item 9 guardrail and Rust interop check, then stopped on the unchanged unclassified SQL packages/targets and stale SQL milestone taxonomy already owned by Item 12. | Item 12 | Reconcile coverage/profile taxonomy and prove every current SQL package and target is classified before final qualification. |
| Item 9A exact-SHA review | A string variable retaining `Type::LiteralStr(_)` takes the allocation-free runtime `chars()` comparison-state path instead of the compile-time literal specialization. | Item 12 | Preserve correctness while extending compile-time single-scalar/invalid-width specialization to literal-typed bindings when canonical constant evidence is available. |
| Item 9A exact-SHA review | The restored `compiler_safety` source intentionally discards `c.callback(c.value)` to keep the callable field live, but the reason is not documented at the source site. | Item 12 | Add a concise source comment or equivalent self-documenting coverage without changing the restored observable output contract. |
| Item 9A generated Clippy validation | Existing `emitted_rust_item9_complexity` output triggers `clippy::missing_const_for_fn` for `signed_zero_key`. | Item 12 | Remove the producer-level residual and prove strict generated Clippy over the governed complexity fixture without rebasing debt. |
| Item 9A exact-SHA review | An ignored local file named `crates/sifr/tests/e2e/pass/Untitled` is present in this worktree's governed fixture directory, although it does not match the `*.sifr` inventory. | Item 12 | During final corpus qualification, verify the fixture root contains no unexplained local artifacts and remove this file only after confirming ownership. |
| Item 9A create-PR and merge gates | Both one-shot gates passed every reached Item 9A guardrail and Rust interop check, then stopped on the unchanged unclassified SQL packages/targets and stale SQL milestone taxonomy already owned by Item 12. | Item 12 | Reconcile coverage/profile taxonomy and prove every current SQL package and target is classified before final qualification. |
| Item 10 remediation review | Project-wide merging unions module-local builtin-error suppressions into a crate-wide veto, so one module's user-defined `ValueError` can remove builtin support required by a sibling and leave dangling Rust paths. | Item 10A / [#3682](https://github.com/sifr-lang/sifr/issues/3682) | Preserve exact module identities and make late builtin-error demand module-aware across project and test-project generation. |
| Item 10 remediation review | The project nominal registry keys builtin and user-defined error classes by bare name, allowing a same-basename user class to be silently replaced by the builtin identity. | Item 10A / [#3682](https://github.com/sifr-lang/sifr/issues/3682) | Separate builtin identity from module-qualified user error identity and compile/run the cross-module collision fixture. |
| Item 10 remediation review | `referenced_error_classes_with_source` is production-unused, and the flat support-trait ownership assumption is implicit. | Item 10A / [#3682](https://github.com/sifr-lang/sifr/issues/3682) | Remove or integrate the dead helper and encode or enforce the flat generated-support trait invariant. |
| Item 10 create-PR and merge gates | Both one-shot gates passed every reached Item 10 guardrail and Rust interop check, then stopped on the unchanged unclassified SQL packages/targets and stale SQL milestone taxonomy already owned by Item 12. | Item 12 | Reconcile coverage/profile taxonomy and prove every current SQL package and target is classified before final qualification. |
| Item 10A exact-SHA review | The static class-adapter negative fixture lock lacks the existing `memchr` dependency and fails `cargo metadata --locked`. | Item 11 / [#3685](https://github.com/sifr-lang/sifr/issues/3685) | Refresh the fixture lock through its owning workflow and prove locked package-compilability. |
| Item 10A exact-SHA review | Generated support trait-layout errors propagate structurally in project support pruning but become compiler `panic!` calls at four project/test-project assembly sites. | Item 12 | Use one checked compiler-diagnostic propagation contract for support-layout invariant failures. |
| Item 10A exact-SHA review | An identity-less class whose name matches a builtin error still resolves through the canonical builtin path; the identity-presence invariant is not asserted at lookup. | Item 12 | Enforce or diagnose the project-union nominal identity invariant without changing valid builtin lookup. |
| Item 10A merge gate | The sole gate passed every reached Item 10A guardrail and the Rust-interop area, then stopped on the unchanged unclassified SQL packages/targets and stale SQL milestone taxonomy already owned by Item 12. | Item 12 | Reconcile coverage/profile taxonomy and prove every current SQL package and target is classified before final qualification. |
| Item 11 merge gate | The sole gate on reviewed candidate `78c28c1e4c42bd85d685d3a3cffdf132fcdfcc40` found 15 stale generated demo companions after Cargo cache setup, HIR, and file-size checks passed. | Item 11A | Integrate the reviewed Item 11 candidate, regenerate the 15 named companions through the compiler, prove `scripts/check_demo_emitted_freshness.py`, and use Item 11A's separately bounded review and gate without rerunning Item 11's gate. |
| Item 11A exact-SHA review | `replacement_or_split_limit` duplicates the string receiver expression while computing its length, which is harmless for current literal companions but can re-evaluate an expensive or side-effecting receiver in the already-accepted Item 11 mechanism. | Item 12 | Bind the receiver once before count/limit conversion and add semantic coverage for a nontrivial receiver without reopening Item 11A's generated-companion-only scope. |
| Item 11A merge gate | The sole gate passed every reached Item 11A guardrail and all 10 Rust-interop variants, then stopped on the unchanged unclassified SQL packages/targets and stale SQL milestone taxonomy already owned by Item 12. | Item 12 | Reconcile coverage/profile taxonomy and prove every current SQL package and target is classified before final qualification. |

New out-of-scope findings must name a concrete active owner before the current
item can close.

## Current Handoff

- Items 11 and 11A are merged through [PR #3689](https://github.com/sifr-lang/sifr/pull/3689)
  as `bbc85bcd3e538e201f7f82fa535c7cef43a5ac6e`; exact candidate
  `ec380f0b221d65516516291018008434c1c1e62a` preserved reviewed Item 11
  candidate `78c28c1e4c42bd85d685d3a3cffdf132fcdfcc40`, regenerated exactly the
  15 stale companions through that compiler, passed exact freshness and agent
  review, and passed every reached item-owned check in its sole merge-profile
  gate. Closed draft [#3687](https://github.com/sifr-lang/sifr/pull/3687) is
  superseded, and Item 11's consumed gate was not rerun. Item 12 owns `anext`
  mutation fidelity, conservative `mut` cleanup, checked-read condition
  fidelity, literal-typed comparison specialization, generated
  `missing_const_for_fn` debt, final fixture-root hygiene, checked
  support-layout propagation, identity-presence enforcement, single-evaluation
  string receiver lowering, and the unchanged SQL coverage/taxonomy gate
  failures.
- Item 12 residual semantic completion and full-corpus qualification is next.
  It is implementation/qualification only. Item 12A is closure-only and
  receives the sole whole-phase review.
- No whole-phase review has been consumed.
- Next action: start Item 12 in a new session and stop after its own merge or
  blocker.

## Naming cleanup validation findings (2026-09-05)

The repository naming cleanup changes test names, demo paths, comments, and
verification metadata. It does not change list-repetition lowering. The full
codegen unit suite reports 1,406 passing tests and these two failures in
unchanged tests and implementation:

- `lib_codegen_tests::collections_and_stdlib_codegen_tests::test_list_repeat_lowers_without_vec_mul_shape`
- `lib_codegen_tests::performance_codegen_tests::single_element_list_repeat_uses_std_repeat_not_extend_loop`

Both expect `std::iter::repeat(SifrInt::from_i64(0))`; current emission uses an
explicit loop that extends the output from the repeated source list. These
failures remain owned by this emitted-Rust quality issue. Local evidence:
`target/naming-cleanup/codegen-tests.log`.

The full emitted-Rust audit validator also rejects the existing `ERQ-032`
current-source anchor in `crates/sifr_codegen/src/methods/list.rs`: its recorded
`exact_int_to_usize_expr` argument expression is absent. The naming cleanup
preserves this anchor and its enforcement. The new ownership schema passes the
validator mutation suite when that unrelated anchor is replaced by a valid
metric in an in-memory test copy. Local evidence:
`target/naming-cleanup/audit-tests.log`.

The full 92-program Clippy corpus also blocks quality-signature migration.
Restoring every pre-rename corpus identity in the captured diagnostics still
fails the original exact baseline (`selection-54c4863d30438d64`). The mismatch
therefore exceeds an identity-only rename. The run reports unowned
`clippy::missing_const_for_fn`, `clippy::redundant_pub_crate`,
`clippy::wildcard_imports`, `dead_code`, and `unused_imports`; its existing lint
counts and signatures also drift. Evidence:
`target/naming-cleanup/corpus-clippy.log`,
`target/naming-cleanup/clippy-diagnostics.json`, and
`target/naming-cleanup/quality-blocker.json`.

No lint allowance, owner exception, or diagnostic signature was refreshed to
accept that drift. Selection IDs and source-path inventory fingerprints were
migrated to the descriptive names; the exact diagnostic-signature migration
remains blocked. The independent full companion Clippy run was stopped when
this blocker was established. All 261 checked-in emitted companions had
already passed the complete freshness check. Resume the signature migration
only after this issue restores the authoritative quality baseline, then run
the required final gates for the completed candidate.

The cleanup invoked `scripts/run_all_tests.sh` once. It exited with a failure
in coverage-matrix readiness because SQL Cargo packages and test targets lack
classification (plus one stale PostgreSQL library-target classification).
Cargo cache setup, HIR, file-size, full demo freshness, Rust interop checks,
and taxonomy passed. The SQL blocker is recorded in
`plans/issues/active/ad-hoc-schema-first-sql-platform-review-follow-ups.md`.
This is not passing merge evidence. Log: `target/naming-cleanup/merge-gate.log`.

Cleanup-specific checks passed: taxonomy and mutation tests, surface inventory
and mutation tests, quality ownership/completion mutation tests, Rust interop
matrix/support checks, SQL qualification mutations, compatibility checks,
regression metadata, all 261 emitted companion freshness checks, all three
changed compact diagnostic outputs and their metadata coverage, the two
renamed E2E fixtures, the portable generated-project E2E test, four driver
portability tests, two driver error-identity tests, the process-argument stdlib
test, formatting, shell syntax, file-size and HIR guardrails, and diff checks.
All 534 edited Sifr source files retain their non-comment content, and every
fixture expectation remains in its original order.

## Demo directory follow-up (2026-09-05)

The three remaining standalone Sifr demos now have `main.sifr`, `emitted.rs`,
and `idiomatic.rs` companions. Their Sifr sources are byte-identical to the
previous commit. The companion inventory now contains 264 programs. Its
Clippy selection identifier changed to `selection-ee7a2285bedf4da8`; existing
debt counts and signatures were preserved. The quality-baseline reconciliation
described above must cover this expanded selection.

All three idiomatic references compiled and ran. The dependency-plan and typed
compiler-boundary Sifr demos also ran. Native execution of the runtime
observability demo fails with `SIFR-BUILD-0005` / Rust `E0433`: the generated
Cargo project does not enable `sifr_stdlib`'s `runtime-observability` feature.
The same failure reproduces with the original source from commit `79e04636d`.
This issue owns the generated-project dependency-feature correction; no
compiler or feature-selection workaround was added during the directory move.
Evidence: `target/demo-layout/runtime_observability_boundary.log` and
`target/demo-layout/original-runtime.log`.

## Abbreviated-label cleanup validation (2026-09-05)

The naming follow-up replaced opaque sysroot fixture module names, the mapped
token label in the structural bridge fixture and its expected output, and an
environment-test key. Six sysroot interop unit tests and the environment E2E
fixture passed. The taxonomy check now rejects abbreviated delivery labels in
paths, identifiers, metadata, and comments while preserving technical uses
such as percentile metrics, point variables, math functions, and migration IDs.

The ignored `test_build_structural_bridge_runtime` integration test fails before
compilation because `cargo metadata --locked --offline` rejects the copied
fixture's lockfile. Replacing the copied Sifr source with its original bytes
from `d1fb93d46` reproduces the same metadata failure. This issue owns restoring
the generated-project integration evidence; no fixture lockfile or dependency
was changed during naming cleanup. Evidence:
`target/abbreviation-cleanup/structural-runtime.log` and
`target/abbreviation-cleanup/original-structural-metadata.log`.

## Naming cleanup review remediation (2026-09-05)

The newly enrolled runtime-observability companion failed to build because
dependency pruning compared the Cargo feature `runtime-observability` with
the Rust namespace `runtime_observability`. The compiler now normalizes
hyphenated feature names before matching generated paths. A regression test
checks retention of runtime demand, rejection of unrelated JSON demand, and
pruning when the generated paths disappear. All four dependency-metadata
tests passed, and `demos/runtime_observability_boundary/main.sifr` built and
ran successfully. Astra high reviewed this fix without actionable findings.

The identity-dependent Clippy signature migration remains blocked by this
issue's pre-existing baseline provenance gap. The tracked baseline stores
aggregate hashes rather than their contributing per-entry diagnostics. Its
`baseline_commit` predates later aggregate updates. Replaying the historical
compiler and fixtures at `6ab6adc08` and the earlier complexity fixture at
`59b8a6e8` did not reproduce all old aggregates. Matching historical sysroots
also did not recover the baseline. Existing run evidence in other local
worktrees was inspected read-only; none matched the required aggregates.

An identity-only migration must first reproduce the old aggregates exactly
from original per-entry records. It must then apply renames and duplicate
consolidation, account separately for the three added companions, and
recompute selection IDs and diagnostic signatures together. Replacing the
baseline with current or reconstructed diagnostics would also accept
unrelated compiler drift. No such baseline refresh or lint allowance was
made. An experimental consistency validator was removed because requiring
unavailable baseline evidence would leave the repository's loader broken.

The three added companions emitted unchanged Rust and compiled through
Clippy without Rust compilation errors after binding their temporary Cargo
manifests to the local runtime and standard-library crates. They exposed
76 lint diagnostics; this is not a passing strict-Clippy gate or accepted
debt. Their unmodified exported manifests reference this unpublished
branch revision through Git, which prevented dependency resolution.
Final qualification owns that separate materialization/gate integration
problem. Evidence is under `target/review-remediation/`, including
`dependency-tests.log`, `observability-run.log`,
`new-companion-diagnostics.json`, `historical-complexity.log`, and
`rebound-companions.log`.

The remediation's sole merge gate passed all 264 emitted-companion freshness
checks, HIR and file-size guardrails, formatting, Rust interop, and naming
checks. It then stopped on the unchanged SQL coverage classifications owned
by `ad-hoc-schema-first-sql-platform-review-follow-ups.md`. This is not
passing merge evidence. Log: `target/review-remediation/merge-gate.log`.

## Naming cleanup PR qualification (2026-09-05)

PR [#3692](https://github.com/sifr-lang/sifr/pull/3692) contains the cleanup
and runtime feature fix. Final CLI validation with
`cargo test -p sifr -- --skip test_e2e_pass` passed: 172 tests, no failures,
seven ignored tests, and the explicitly excluded positive E2E suite. This
includes the negative/runtime-failure E2E suites, emission panic-shape scan,
portable dependency-plan checks, and Python, host-tool, runtime-observability,
and sysroot integration tests. Log: `target/pr-cleanup/cli-tests.log`.

The create-PR gate passed all 264 companion freshness checks and reached
guardrails, then reproduced the existing SQL coverage classification
failures. Its log is `target/pr-cleanup/create-pr.log`; the previously recorded
merge gate applies to the same implementation. GitHub Actions also rejects
the unchanged workflow before starting any jobs: the same failure occurs
on base `2af89e75e` in
[run 33963698543](https://github.com/sifr-lang/sifr/actions/runs/33963698543).
Final qualification owns the workflow repair. The diagnostic-baseline
identity migration and pre-existing quality failures described above remain
unresolved; these passing CLI results do not qualify the Clippy baseline.

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
3. Each implementation item receives one exact-SHA Claude Opus review and at
   most one remediation review. A second-review mechanism defect becomes a
   later owned item; there is no third review.
4. Compiler-changing items receive exactly one create-PR gate and one merge
   gate on the final candidate SHA. Neither gate is repeated. Items without
   compiler changes omit both gates.
5. Merge the item, update this record, and start the next unfinished item.
6. The final item receives the only whole-phase Opus review.
7. Before each item starts, rebase its branch point on current `origin/main`
   and re-audit any relevant mechanism that another merged phase changed.
   Unmerged branches are not silently treated as delivered work.
8. Generated companions are regenerated from the candidate compiler. They are
   never manually polished.

## Sequential Items

| Item | Status | Name | Required outcome |
|---:|---|---|---|
| 0 | complete | Contract and audit inventory lock | The full quality contract, reconciled finding ledger, baseline, ownership, review limits, and closure rules are machine checked and merged. |
| 1 | in progress | Comprehensive corpus and non-vacuous gates | Every generated surface is discoverable; freshness, rustfmt, Clippy, panic/static analysis, determinism, and negative self-tests fail closed without broad quality suppressions. |
| 2 | pending | Exact integer and overflow architecture | Canonical `int` storage and all arithmetic use one exact semantic model; debug/release behavior agrees; fixed-width boundaries remain explicitly checked. |
| 3 | pending | Checked failure and impossible-state model | Generated user paths use typed errors; abort/exit/unreachable discharge and silent value fallbacks are removed; compiler invariants fail before materialization. |
| 4 | pending | Collection access and mutation architecture | Reads, writes, deletes, nested access, augassign, membership, and unpacking share checked place semantics with no panic or silent no-op path. |
| 5 | pending | Lazy iterator and generator architecture | Yield, generator state, `count`, `islice`, chained adapters, and errors are lazy and semantically unbounded where required. |
| 6 | pending | Stdlib emitted-semantics closure | String widths, IO reads/seeks/errors, decimal precision, iteration arguments, and every inventory-owned stdlib defect have exact behavior and resource safety. |
| 7 | pending | Ownership, borrowing, and clone quality | Signatures and expressions use idiomatic borrowing; avoidable container, row, tree, and scalar clones are eliminated without weakening ownership safety. |
| 8 | pending | Canonical Rust IR and emission cleanup | Structured IR represents all maintained code; dead branches/tails, identity transforms, needless returns, stale snapshots, and generated ceremony are removed at the producer. |
| 9 | pending | Algorithmic and Unicode performance | Emission preserves source complexity; string traversal avoids repeated scans/materialization; collection algorithms avoid quadratic clone/front-removal behavior; budgets prevent recurrence. |
| 10 | pending | Runtime, stdlib bridge, and API deduplication | Each demanded support body and public adapter is assembled once, unused support is absent, and generated crates have one canonical API path per operation. |
| 11 | pending | Portable and secure generated projects | Generated manifests/artifacts are relocatable and reproducible; process arguments preserve boundaries; path, capacity, and resource-limit handling is checked. |
| 12 | pending | Full-corpus qualification and phase closure | Regenerate all owned output, run uncompromising full-corpus and repository gates once on the final SHA, satisfy one whole-phase Opus review, archive the record, and leave zero actionable inventory rows. |

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

- [ ] `int` has one canonical runtime representation through locals,
  parameters, returns, fields, containers, constants, unions, and interop.
- [ ] All arithmetic and conversions preserve the language's exact semantics.
- [ ] Floor division/modulo and zero/overflow errors are consistent.
- [ ] Debug and release differential/property evidence agrees.

### Item 3: Checked failure and impossible-state model

- [ ] No generated user path contains abort, exit, unreachable, panic, unwrap,
  expect, or a silent fallback for an error-producing operation.
- [ ] Compiler invariants are validated structurally before source rendering.
- [ ] Typed errors preserve category, payload, source span where applicable,
  and error timing.

### Item 4: Collection access and mutation architecture

- [ ] Every collection access form uses one typed checked-place plan.
- [ ] Negative indices, nested indices, unpacking cardinality, missing keys,
  deletes, and writes preserve Sifr semantics.
- [ ] Membership and read-only access do not mutate containers.
- [ ] Out-of-range writes never become no-ops.

### Item 5: Lazy iterator and generator architecture

- [ ] Generator bodies are resumable state machines or an equivalently lazy
  representation, not eager vectors.
- [ ] Infinite iterators remain infinite and consumers control termination.
- [ ] `islice` and related adapters validate arguments and preserve error timing.
- [ ] Laziness, partial consumption, side effects, and memory use have native
  runtime regressions.

### Item 6: Stdlib emitted-semantics closure

- [ ] All Item 6 inventory rows are covered by focused differential tests.
- [ ] Signed sizes and widths are validated before allocation/casting.
- [ ] IO operations honor size/offset arguments and preserve error kinds.
- [ ] Decimal precision never silently falls back.

### Item 7: Ownership, borrowing, and clone quality

- [ ] APIs prefer `str` and slices where ownership is not required.
- [ ] Clone insertion is driven by an explicit ownership plan.
- [ ] Clone counts and representative emitted shapes have regression budgets.
- [ ] Recursive and dynamic-programming fixtures preserve linear work where the
  source algorithm is linear.

### Item 8: Canonical Rust IR and emission cleanup

- [ ] Maintained emission uses structured IR through final rendering.
- [ ] Canonical simplification removes dead and identity constructs without
  textual postprocessing.
- [ ] Generated names and support items are demand-driven and warning-clean.
- [ ] Stale or mislabeled generated snapshots are removed or regenerated by an
  authoritative producer.

### Item 9: Algorithmic and Unicode performance

- [ ] Indexed string operations do not repeatedly rescan Unicode text.
- [ ] Character comparison does not allocate one-character strings.
- [ ] Queue/deque and sorting operations use appropriate Rust structures and
  algorithms.
- [ ] Representative corpus cases enforce asymptotic and allocation budgets.

### Item 10: Runtime, stdlib bridge, and API deduplication

- [ ] Runtime/support demand is computed once and rendered once.
- [ ] No generated crate contains duplicate bridge bodies or duplicate public
  operation paths.
- [ ] Unused support is absent, and bridge-size budgets catch recurrence.

### Item 11: Portable and secure generated projects

- [ ] Portable emitted artifacts contain no host-specific absolute paths.
- [ ] Ephemeral local dependency resolution is separated from distributable
  source and manifests.
- [ ] Process invocation keeps executable/argument boundaries unless the user
  explicitly selected a shell API.
- [ ] Allocation, path, and resource-limit conversions are checked.

### Item 12: Full-corpus qualification and phase closure

- [ ] Every actionable inventory row is closed with merged evidence.
- [ ] All generated demos, verification fixtures, project modes, and benchmark
  representatives are regenerated by the final compiler.
- [ ] Full generated-code quality, e2e, stdlib, algorithmic, formatting,
  Clippy, file-size, HIR, create-PR, and merge gates pass as applicable on the
  exact final source SHA.
- [ ] One exact-SHA whole-phase Opus review is satisfied.
- [ ] Architecture and roadmap records reflect the delivered architecture.
- [ ] This issue is archived only after every closure condition is true.

## Item Ledger

| Item | State | PR | Merge SHA | Validation | Exact-SHA review | Result |
|---:|---|---|---|---|---|---|
| 0 | merged | [#3574](https://github.com/sifr-lang/sifr/pull/3574) | `8d292f9395fee51ef8b348a413ea496a33c5ce38` | Candidate `b75a3c471f7ec8b4cb798e112e123bfb13d78b83`: inventory, mutation self-test, Python/JSON syntax, file-size, HIR maintainability, docs-link, and diff hygiene checks passed. No compiler files changed, so Sifr gates were omitted. | [Initial and sole remediation review](https://github.com/sifr-lang/sifr/pull/3574#issuecomment-5462303681): both NOT SATISFIED. The original evidence blocker was fixed; the remediation review's new checker mechanism is assigned to Item 1 under the explicit review limit. | Contract and 32-row inventory merged; three missing mutation branches and related checker provenance hardening are owned by Item 1. |

## Deferred Findings

| Source | Finding | Owner | Required action |
|---|---|---|---|
| Item 0 remediation review | Empty `implementation_items`, empty `baseline`, and empty finding `mechanism` validation branches lack mutation cases. | Item 1 | Add one mutation per branch as part of the comprehensive gate self-test; do not run a third Item 0 review. |
| Item 0 remediation review | `baseline_context` accepts arbitrary keys. | Item 1 | Require the command, toolchain version, and explanatory note keys. |
| Item 0 remediation review | Evidence validation proves path existence but not that the cited location supports the mechanism. | Item 1 | Add governed semantic anchors or an equivalent fail-closed evidence contract. |
| Item 0 remediation review | Glob metacharacters and symlink boundaries are not fully constrained. | Item 1 | Restrict accepted inventory path syntax and prove repository containment. |

New out-of-scope findings must name a concrete active owner before the current
item can close.

## Current Handoff

- Active item: Item 1, comprehensive corpus and non-vacuous gates, based on
  `d7a08f12a2ba638b98542c7745913f7e83be5ab9`.
- Implementation state: all entrypoint and maintained-source families have an
  exact path-set inventory; broad Clippy suppression and format-before-check
  behavior are removed; safety, formatting, lint, freshness, and deterministic
  output use fail-closed negative checks and exact owner-bound debt signatures.
- Audit correction: ERQ-020 is confirmed, not rejected. Current decimal
  lowering contains the reported `with_prec(28)` fallback, owned by Item 6.
- Next action: validate Item 1, populate only exact existing producer debt,
  regenerate or remove non-authoritative checked-in output, and obtain the
  required exact-SHA Opus review.

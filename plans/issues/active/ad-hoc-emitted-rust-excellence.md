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
| 1 | complete | Comprehensive corpus and non-vacuous gates | Every generated surface is discoverable; freshness, rustfmt, Clippy, panic/static analysis, determinism, and negative self-tests fail closed without broad quality suppressions. |
| 2 | complete | Exact integer and overflow architecture | Canonical `int` storage and all arithmetic use one exact semantic model; debug/release behavior agrees; fixed-width boundaries remain explicitly checked. |
| 3 | complete | Checked failure and impossible-state model | Generated user paths use typed errors; abort/exit/unreachable discharge and silent value fallbacks are removed; compiler invariants fail before materialization. |
| 4 | merged | Collection access and mutation architecture | Reads, writes, deletes, nested access, augassign, membership, and unpacking share checked place semantics with no panic or silent no-op path. |
| 4A | merged | Residual checked-place lifecycle closure | Loop-carried witnesses, post-mutation missing behavior, and callback argument decoding preserve exact semantics and compile on every generated surface. |
| 4B | merged | Structured-loop witness state closure | Async-for guard state cannot escape a possibly empty loop, and missing loop-carried witnesses take the loop-kind's terminating control-flow path instead of skipping progress. |
| 4C | merged | Mutation-tail witness continuation closure | Refreshed witnesses use region-scoped continuations and current typed failure semantics across nested and straight-line mutation tails. |
| 5 | merged | Lazy iterator and generator architecture | Yield, generator state, `count`, `islice`, chained adapters, and errors are lazy and semantically unbounded where required. |
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
| 1 | merged | [#3578](https://github.com/sifr-lang/sifr/pull/3578) | `b86eec0be7b7be2b5ddf012fea9cbcced286c342` | Candidate `b0fb5c2049b81fe28fc4b076c34ac624f8249e94`: full generated-code-quality profile passed 9 variants with 0 failures across 91 positive projects; exact safety, rustfmt, 38,957-diagnostic/105-lint Clippy, determinism, all 262 authoritative companions, recursive freshness, audit/debt/surface mutations, Python/JSON, file-size, HIR, driver, docs-link, and diff hygiene passed. No compiler files changed, so Sifr gates were omitted. | [Initial review](https://github.com/sifr-lang/sifr/pull/3578#issuecomment-5463056720) NOT SATISFIED; [sole remediation review](https://github.com/sifr-lang/sifr/pull/3578#issuecomment-5463053848) SATISFIED with all four blockers resolved and no new in-scope mechanism defect. | Exact surface digests, fail-closed quality protocols, strict source/lint policies, 18 negative seeds, and a 33-row governed audit inventory merged; all Item 0 deferred checker findings are resolved. |
| 2 | merged | [#3580](https://github.com/sifr-lang/sifr/pull/3580) | `d618a7be107550629c3331ea7fdb3f76e28e0dce` | Compiler candidate `aa97d2ca6d0da1ec5700b02d3f57ef864a450a53`: 1,151 codegen tests and 557 driver tests passed; Clippy, formatting, generated inventory/freshness, diagnostics governance, file-size, HIR, and driver checks passed. The one create-PR gate completed every reached check and all 28 runtime-platform variants with zero failures before its cold rebuild exceeded the 120-second step budget. The one merge gate passed static, core-language, differential, Rust interop, coverage, and all 30 Python-interop variants before finding three stale diagnostic baselines. Follow-up `7b3ba45d25e07adabb820c9f80463534060d42ee` changed only diagnostic fixtures/governance; 178 of 179 full baseline variants passed before the sole new wording mismatch was corrected, and exact checks then passed. Neither gate was repeated. | [Initial review](https://github.com/sifr-lang/sifr/pull/3580#issuecomment-5465345414) on `d4aea519efebdf29bad472a9795afcdd72c4f865` and [sole remediation review](https://github.com/sifr-lang/sifr/pull/3580#issuecomment-5465345486) on `9606b67b84ae5865105415399d647319b455bb99` were NOT SATISFIED. The initial slice-step panic was fixed. The remediation review's new exact-ratio proof/codegen mismatch is assigned to Item 3 under the no-third-review rule. | Canonical inline-small/`BigInt` `SifrInt`, exact arithmetic and conversion paths, fixed-width boundaries, constants, ranges, collections, unions, and Rust/Python interop merged with debug/release and corpus evidence. |
| 3 | merged | [#3587](https://github.com/sifr-lang/sifr/pull/3587) | `fe95d220be2819464d6231080d57e47444b0d429` | Reviewed compiler candidate `229c2687923d97c72531bb4e81deb047833367b1`: 1,156 codegen and 1,053 lowering tests passed; workspace Clippy, formatting, file-size, HIR, demo freshness, generated determinism, panic scan, demo corpus, intrinsic panic lint, diagnostics governance, and smoke/representative/full generated Clippy passed. The one create-PR gate stopped on a stale retained-intrinsic governance row after all preceding checks passed; docs-only `6e7c5b32dc9574a40ff5624834daa768613a0b14` removed it and the exact checker plus self-test passed. The one merge gate passed static, core-language, differential, Rust interop, coverage, and 29 of 30 Python-interop variants; its sole `sqlite-context` compiler failure is assigned to Item 3A. Neither gate was repeated. | [Initial review](https://github.com/sifr-lang/sifr/pull/3587#issuecomment-5466942667) on `2e3867cbe3546e09a94f391672410808315f3b25` was NOT SATISFIED; [sole remediation review](https://github.com/sifr-lang/sifr/pull/3587#issuecomment-5466942761) on `229c2687923d97c72531bb4e81deb047833367b1` was SATISFIED. The loop-constant blocker was fixed; later mechanism findings are assigned to Item 3A under the review limit. | Typed structural failure discharge, exact ratio materialization, checked Decimal/BigDecimal/bytes/random/input operations, structured try/finally and context carriers, pre-render invariant validation, regenerated demos, and retired `SIFR-INT-0006` governance merged. |
| 3A | merged | [#3591](https://github.com/sifr-lang/sifr/pull/3591) | `d88192be94823a6e1c0f30b712d2f7440ac2c6b4` | Compiler candidate `719bd96ad5b4d11c507b356bd6fece2ab6d4ac3f`: 4 IR, 1,167 codegen, and 1,072 lowering tests passed with one intentional ignore; all non-E2E Sifr test groups, focused sync/async/SQLite runtime regressions, formatting, HIR, file-size, and item-owned Clippy checks passed. The sole create-PR run passed every functional check but exceeded the runtime-platform step budget after the required cold-cache cleanup; its later warm merge run passed that area in 24.5 seconds. The sole merge run passed core language, CPython differential, Rust/Python interop, diagnostics, runtime, algorithmic, tooling, and all emitted-Rust corpus, panic-scan, rustfmt, Clippy, determinism, and freshness checks. Its only failure was a pre-existing surface inventory record: both base and candidate contain the same 704 E2E paths and digest while the record expects 701. Neither gate was repeated. | [Initial review](https://github.com/sifr-lang/sifr/pull/3591#issuecomment-5467141026) on `8b7b46cd629e6530d693462e10590ec287b931c3` was NOT SATISFIED; [sole remediation review](https://github.com/sifr-lang/sifr/pull/3591#issuecomment-5467149668) on `719bd96ad5b4d11c507b356bd6fece2ab6d4ac3f` was SATISFIED with no blockers. The imported-constant proof regression was fixed through lexical module-frame resolution. | Suppressible Python contexts now rejoin typed carriers; exact-integer facts respect lexical binding identity and nested-call mutation; loop/context emitted fallthrough agrees with static flow; sync, async-for, and SQLite regressions merged. |
| 4 | merged | [#3601](https://github.com/sifr-lang/sifr/pull/3601) | `ab1bd8371faf090f3f7549524147b0fbabbd3b7a` | Compiler candidate `a91f43d2bace42c5579d02cf0a9bce57e4962300`: 1,172 codegen, 1,073 lowering with one intentional ignore, 84 runtime, and 8 exact-integer architecture tests passed; E2E passed 705/705 with signature `9f98912689339124`; workspace Clippy, formatting, HIR, file-size, generated inventory, demo freshness, governed corpus, and panic scan passed. The full generated-quality run's 91 rustfmt-classified cases passed individually, but its exact aggregate debt signature changed and remains Item 8-owned. The sole create-PR gate passed every reached guardrail plus Rust interop, coverage, diagnostics, and 23 of 24 Python-interop variants. The sole merge gate passed all guardrails, Rust interop, coverage, core language, CPython differential, and 29 of 30 Python-interop variants. Both gates stopped only on the same underconstrained callback-decoder array conversion assigned to Item 4A, and neither was repeated. | [Initial review](https://github.com/sifr-lang/sifr/pull/3601#issuecomment-5470119120) on `054c14f728ed13f6ed548647a5669504a36d729f` was NOT SATISFIED; [sole remediation review](https://github.com/sifr-lang/sifr/pull/3601#issuecomment-5470119110) on `a91f43d2bace42c5579d02cf0a9bce57e4962300` was NOT SATISFIED. The straight-line stale-value and E0502 blocker was fixed. The remediation review's new loop-back-edge and post-deletion failure-semantics defects are assigned to Item 4A under the no-third-review rule. | One typed checked-place architecture now covers negative and nested reads, writes, deletes, augmented assignment, membership, unpacking, optional targets, and generated direct-index removal. Mutation-aware straight-line witness refresh, checked non-empty vectors, typed failure plans, and regenerated companions merged; bounded residual lifecycle defects are owned by Item 4A. |
| 4A | merged | [#3608](https://github.com/sifr-lang/sifr/pull/3608) | `9af05a15e1d2eaae6866b7976f425dc5b3077ca4` | Reviewed compiler candidate `13fc41d0d8e4465305b6bd4402f6f0557be91260`: 1,078 lowering tests passed with one intentional ignore; targeted codegen/lowering Clippy, native checked-place E2E, all seven callback examples, demo freshness, panic scans, formatting, HIR, and file-size checks passed. The one create-PR gate and one merge gate each stopped at the same profile preflight defect because the profile omitted required `postgresql-live-differential`; neither was repeated. After concurrent async-cleanup work reached `main`, integration commit `f8869ebc24647364e3c9d0862d53a18c43030885` preserved both ordinary and closable async-for witness refresh; 1,180 codegen plus lowering/runtime suites, targeted Clippy, two native fixtures, demo freshness, formatting, HIR, and file-size checks passed. | [Initial review](https://github.com/sifr-lang/sifr/pull/3608#issuecomment-5470878459) on `91fe545fcbe75a99bb8b75002fb68d9692a9fdd8` was NOT SATISFIED; [sole remediation review](https://github.com/sifr-lang/sifr/pull/3608#issuecomment-5470912500) on `13fc41d0d8e4465305b6bd4402f6f0557be91260` was SATISFIED. The async-for invalidation blocker was fixed. Its newly identified async-for guard leak and non-terminating missing-witness fallback are assigned to Item 4B under the no-third-review rule. | Loop-carried and while-condition witnesses now refresh at repeat boundaries; mutation dependencies invalidate before sync/async loop lowering; post-delete reads use current typed failure semantics; unused witness scaffolding is demand-driven; callback arrays are explicit and panic-free. The two bounded second-review defects are owned by Item 4B. |
| 4B | merged | [#3612](https://github.com/sifr-lang/sifr/pull/3612) | `67c1804df84d0367e380ebef1ee14845ec1971fb` | Reviewed compiler candidate `68981d07cb6d088803d199e8924ecc9ab06d0a91`: 1,181 codegen and 1,079 lowering tests passed with one intentional ignore; strict targeted Clippy, native checked-place plus ordinary/closable async-for fixtures, demo freshness, formatting, HIR, and file-size checks passed. The sole create-PR and merge gates each stopped before tests because their then-current profiles omitted required `postgresql-live-differential`; neither was repeated. Concurrent PostgreSQL work then repaired the profiles and merged conflict-free as integration commit `5b1739b4853523b7a9b81bf1c8f1a6af28497a4c`; full codegen/lowering suites, targeted Clippy, formatting, diff, and 3,488-file guardrails passed after integration. | [Initial review](https://github.com/sifr-lang/sifr/pull/3612#issuecomment-5471106525) on `0e8bdd33af00c6bab5d43c02b614ee1f8052c70a` was SATISFIED; [sole remediation review](https://github.com/sifr-lang/sifr/pull/3612#issuecomment-5471132474) on `68981d07cb6d088803d199e8924ecc9ab06d0a91` was SATISFIED. Compiler-inserted while witness exits now use the canonical loop-else marker. The remediation review's new deeply nested mutation-tail continuation defect is assigned to Item 4C under the no-third-review rule. | Async-for body guards restore at loop exit; loop-carried witnesses use loop-kind progress/termination; body and condition refreshes preserve loop-else semantics; precise lowering diagnostics and native sync/async regressions merged. Remaining non-back-edge continuation scoping is owned by Item 4C. |
| 4C | merged | [#3615](https://github.com/sifr-lang/sifr/pull/3615) | `2579fcd198acd105da4a93b794a82601524541a8` | Compiler candidate `6a849e8d9d8457b7e463486e52f6e629d5da6b86`: 1,183 codegen and 1,082 lowering tests passed with one intentional ignore; focused mutable-call invalidation, checked-place shape, native nested-loop, workspace Clippy, formatting, HIR, diff, and file-size checks passed. The non-E2E Sifr sweep's `numeric_sentinels.sifr` type diagnostic reproduced identically on exact base `6862b4a21ebd0917a54f5744c6e22960242bf00b` and is Item 8-owned. The sole create-PR and merge gates each stopped before tests because their current profiles omitted required `postgresql-live-differential` and `postgresql-live-runtime`; neither was repeated. | [Initial exact-SHA review](https://github.com/sifr-lang/sifr/pull/3615#issuecomment-5471369789) on `6a849e8d9d8457b7e463486e52f6e629d5da6b86` was SATISFIED with no blockers. Its non-blocking receiver-effect and clone-bound findings are assigned to Item 7; refresh-default evidence and wider loop-else scaffold deduplication are assigned to Item 8. | Stored witness exit payloads are eliminated; straight-line renewal cannot skip tails or replay outer control flow; mutable-call guards invalidate before codegen; simple and structured exits share one constructor; nested while/for/if and condition-marker regressions merged. |
| 5 | merged | [#3622](https://github.com/sifr-lang/sifr/pull/3622) | `79b963aa6a909303b1152546a0f91e699cd8f1cf` | Final compiler candidate `cc63e5d4e86725543ed111b3c194d2e89ab5e629`: 1,183 codegen and 1,085 lowering tests passed with one intentional ignore; workspace Clippy, formatting, HIR, diff, 3,515-file guardrail, audit inventory, and exact demo freshness passed. Native evidence covered suspension-by-suspension side effects, 10,001 pulls from unbounded `count`, `islice` over that source, async lazy start/close/exhaustion, CPython/consolidated itertools behavior, and bounded `cycle` without an extra source effect. The 91-project generated corpus compiled on the initial candidate with panic, intrinsic-panic, determinism, demo, freshness, and every per-project rustfmt/Clippy classification passing; the exact remediation reran all affected lowering, native, Clippy, formatting, and freshness checks. The sole create-PR and merge gates each stopped before tests because both profiles omit required SQL suites `host-tools`, `postgresql-live-differential`, and `postgresql-live-runtime`; neither was repeated. | [Initial review](https://github.com/sifr-lang/sifr/pull/3622#issuecomment-5472340524) on `1029541fd69c9b1d6726f53331cc5319f17f3be3` was NOT SATISFIED; [sole remediation review](https://github.com/sifr-lang/sifr/pull/3622#issuecomment-5472360882) on `cc63e5d4e86725543ed111b3c194d2e89ab5e629` was SATISFIED with no blockers. The discarded `None`-typed return-expression defect and bounded-`cycle` over-pull were corrected. The remediation review's newly noted optional-element `cycle` semantics are assigned to Item 6 under the no-third-review rule. | Sync and async generators now own resumable producer futures; generator returns exhaust without silently discarding expressions; infinite and adapter iterators are consumer-driven; authoritative demos and native/codegen/lowering regressions are merged. |

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

New out-of-scope findings must name a concrete active owner before the current
item can close.

## Current Handoff

- Active item: Item 6, stdlib emitted-semantics closure, based on Item 5 merge
  `79b963aa6a909303b1152546a0f91e699cd8f1cf`.
- Item 5 state: merged with resumable sync/async producer futures, demand-driven
  support, unbounded `count`, eager adapter validation plus lazy consumption,
  literal-only generator exhaustion, and native side-effect/partial-consumption
  evidence. Its sole remediation review was SATISFIED with no blockers.
- Item 6 scope includes every inventory-owned stdlib semantic defect plus the
  deferred JSON map ordering, exact-integer signed-zero, iterator ownership and
  optional-stop parity, and optional-element `cycle` semantics. Canonical HIR,
  stale inventory/fixtures, nested-generator emission, and rustfmt debt remain
  Item 8-owned; SQL profile composition remains Item 12-owned.
- Next action: rebase Item 6 on current `origin/main`, re-audit all Item 6
  inventory rows and deferred stdlib findings, implement the complete semantic
  closure without testing, then run focused and required validation, bounded
  exact-SHA Opus review, and the single exact-candidate gate sequence.

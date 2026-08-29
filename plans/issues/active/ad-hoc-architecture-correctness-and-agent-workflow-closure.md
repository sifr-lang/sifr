# Ad Hoc Architecture Correctness And Agent Workflow Closure

status: active

## Objective

Close the still-valid correctness, architecture, verification, documentation, and
maintainability findings from the 2026-08-27 latest-main audit. The execution
order favors defects that can make later validation or agent work unreliable.

This document is the source of truth for the phase. Each milestone is completed,
validated, reviewed at an exact candidate SHA, merged, and recorded before the
next milestone starts.

## Scope Decisions

The audit was repeated against `e9df29f7e4cada7b376b2d455790f9c80a5647a0`.
Only findings reproduced on that tree are in scope.

The following review claims are explicitly excluded:

- The generated-artifact cache is not shared across worktrees. Its effective
  identity includes the canonical cache scope. Cross-worktree cache races are
  therefore not a reproduced defect.
- Source dependency direction is already enforced by the current guardrail.
- The prior workspace Clippy display failure is fixed.
- `crates/sifr_rust_interop_inventory/` is not tracked repository content.
- Existing progress plumbing is not a demonstrated no-op subsystem.
- Existing inline `insta` snapshots do not justify a blanket snapshot rewrite.

## Execution Rules

- One unfinished milestone at a time.
- Complete implementation before running tests for that milestone.
- Use one exact-SHA Opus review, with at most one remediation review.
- Run compiler create-PR and merge gates only when compiler files change, once
  each on the final candidate SHA.
- Use a whole-phase Opus review only for M13.
- Record mechanism defects found by a second review as later work instead of
  starting a third review round.

## Milestones

| ID | Milestone | Status | PR | Candidate |
| --- | --- | --- | --- | --- |
| M1 | Warm-cache lock correctness and serialization failures | implementation complete; merge deferred | [#3553](https://github.com/sifr-lang/sifr/pull/3553) | `1c43fe34847925a269288b4073f5ca7ca7d6063e` |
| M2 | Canonical test/build materialization | implementation complete; merge deferred | [#3554](https://github.com/sifr-lang/sifr/pull/3554) | `16024325813dbee56e84a838e42679340f0f829a` |
| M3 | Verification gate integrity | implementation staged; second-review defect deferred | [#3555](https://github.com/sifr-lang/sifr/pull/3555) | `07e3d7d0f5123a89a30a4fcf149e51ebff7d6c7e` |
| M4 | Architecture documentation accuracy and generated crate map | implementation staged; second-review defect deferred | [#3556](https://github.com/sifr-lang/sifr/pull/3556) | `0cb9720cb80e66bc2be3c73e78206106cd998bd1` |
| M5 | Structural generated-code safety | implementation staged; final-review residual remediated | [#3557](https://github.com/sifr-lang/sifr/pull/3557) | `5644505a6badeb51b39e38df82b3f8972c545265` |
| M6 | Structured codegen error propagation | implementation staged; integration deferred | [#3558](https://github.com/sifr-lang/sifr/pull/3558) | `ef46a3eac5f7e54b374f6c648609e49a3dc5f302` |
| M7 | Canonical frontend project compilation product | implementation staged; integration deferred | [#3559](https://github.com/sifr-lang/sifr/pull/3559) | `30c7ab5e1b5bffc4a9e16f65c061e07498951fae` |
| M8 | LSP hot paths and compiler-service dependency direction | implementation staged; integration deferred | [#3560](https://github.com/sifr-lang/sifr/pull/3560) | `da39eb709ddffef80d3dc6297cde959f88d85bc5` |
| M9 | Method-lowering authority and unsafe-code documentation | implementation staged; integration deferred | [#3561](https://github.com/sifr-lang/sifr/pull/3561) | `f6d1f4edaab6a2bfa0952ab89fd1980bec284703` |
| M10 | Collision-resistant cache identity and cache lifecycle | implementation staged; second-review defect deferred | [#3562](https://github.com/sifr-lang/sifr/pull/3562) | `2e2ad86fb80ee916542738967935039c157fa18e` |
| M11 | Real fuzz and semantic property targets | implementation staged; create-PR gate defect deferred | [#3563](https://github.com/sifr-lang/sifr/pull/3563) | `7664b902bf4697ca20bc39c683cdb617d26032e2` |
| M12 | Maintainability ratchets and evidence-based flow decisions | implementation staged; second-review defect closed by M12A | [#3564](https://github.com/sifr-lang/sifr/pull/3564) | `17d7ac63eae4a9417a2d60415c06d7de7016ce6c` |
| M12A | Process-group deadlines and terminal signal propagation | merged into M12 branch | [#3565](https://github.com/sifr-lang/sifr/pull/3565) | `2b0820dabf890dc19850289273ade09d8b048cd5` |
| M12B | Restore canonical list-method lowering on structured fallback paths | merged into M12 branch | [#3566](https://github.com/sifr-lang/sifr/pull/3566) | `c4a23973f80d8eb796e4b9c984c89a248b58ac88` |
| M12C | Terminal-signal escalation and remaining hardening command lifecycle | merged into M12 branch | [#3567](https://github.com/sifr-lang/sifr/pull/3567) | `9442b51faa8a15c1466919ad797d0cfcd9d0e8ef` |
| M12D | Documentation mutation-registry consistency | merged into M12 branch | [#3568](https://github.com/sifr-lang/sifr/pull/3568) | `e97e7332e6ef537738ebd6b6f9fad60384ba1f2f` |
| M12E | Atomic repeated-terminal-signal escalation entry | merged into M12 branch | [#3569](https://github.com/sifr-lang/sifr/pull/3569) | `2a8adc32dfed16933dcb22b4d77f989d97c80734` |
| M12F | Restore generated demo freshness after compiler corrections | pending | | |
| M13 | Phase closure and whole-phase review | pending | | |

## M1 Warm-Cache Lock Correctness And Serialization Failures

### Scope

- Make the generated-binary cache identity include the complete Cargo
  resolution policy that can affect a constrained build.
- Validate constrained authoritative lock inputs before accepting a warm hit.
- Ensure normal, locked, offline, and frozen requests cannot reuse artifacts
  produced under a different lock contract.
- Remove silent empty-byte fallback when package graph/cache identities cannot
  be serialized. Propagate the failure through public callers.

### Acceptance Criteria

- A warm constrained cache hit rejects missing or unreadable authoritative lock
  inputs with the existing structured Cargo-resolution diagnostic.
- Cache identities differ when lock mode, vendor mode, authoritative lock
  contents, or trusted vendor roots differ.
- Serialization failure cannot collapse to the digest of an empty byte string.
- Targeted cache, resolution, package-digest, and affected caller tests pass.
- The file-size guardrail and the required compiler validation gates pass on the
  exact reviewed candidate SHA.

## M2 Canonical Test/Build Materialization

Route `sifr test` through the normal generated-project materialization and Cargo
resolution authority. Test execution must not mutate a reusable cache entry,
and build/test dependency, sysroot, interop, native-link, tracing, and lock
behavior must share one implementation contract.

### Scope

- Extract one generated Cargo-project materialization authority shared by
  binary builds and `sifr test`.
- Route test Cargo resolution through the same lock policy, sysroot Cargo
  configuration, tracing, hermetic environment, and native-link validation used
  by normal builds.
- Keep reusable cache entries immutable. Execute `cargo test` from an
  invocation-owned workspace or another explicitly non-cached execution root.
- Remove the test runner's independent manifest/source/cache orchestration.

### Acceptance Criteria

- `sifr test` and normal builds use one implementation for generated manifest,
  bridge sources, support modules, namespace modules, and Cargo resolution.
- Locked, offline, and frozen test requests enforce the same authority and
  unchanged-lock checks as builds.
- Test execution cannot write target output or a lockfile into a reusable
  generated-artifact cache entry.
- Cargo invocation traces and native-link evidence include test builds.
- Focused driver and CLI test-runner tests cover cache immutability, constrained
  resolution, and parity with normal project materialization.

## M3 Verification Gate Integrity

Enforce real subprocess deadlines, expand direct-filesystem inventory to all
production compiler/tooling roots, and add self-tests proving each guardrail can
fail. Classify legitimate CLI parsing separately from compiler semantic-source
authority instead of banning syntax use mechanically.

Acceptance criteria:

- Every profile step resolves to a positive blocking budget. The runner applies
  that budget as one absolute deadline across the step's subprocesses and
  terminates the complete subprocess group on expiry.
- Hardening variants use a finite default deadline when a suite does not define
  a narrower deadline, and timeout cleanup includes descendant processes.
- Direct-read inventory scans every production compiler and tooling crate,
  including `sifr_analysis` and `sifr_lsp`, and recognizes byte reads as well as
  text and directory reads.
- The global split-brain guard rejects direct `sifr_syntax::parse_module` use
  outside compiler-owned crates, with documented syntax-only classifications
  for CLI Python-requirement discovery, lint token/AST rules, and formatter
  round-trip validation. Inline test modules are excluded by parsed module
  range, not by whole-file allowlists.
- Deadline, direct-read inventory, and split-brain self-tests each seed a defect
  and prove that the owning guard rejects it.

## M4 Architecture Documentation Accuracy And Generated Crate Map

Correct immediately false API and topology claims, remove phantom crates and
machine-local paths, separate current architecture from history/future design,
generate the workspace crate map from Cargo metadata, and block documented
crate/path/profile drift.

Acceptance criteria:

- `internal_docs/architecture.md` describes current implemented boundaries;
  execution history points to `plans/`, and future design is labeled explicitly.
- The dict-order and `random.shuffle` rows match the current stdlib/compiler API.
  Obsolete parser-fixture, test-utility, cargo-fuzz, corpus, snapshot, and
  benchmark commands are replaced by the authoritative verification commands.
- No machine-local absolute path or nonexistent first-party crate remains in the
  architecture document.
- A deterministic generator renders the workspace crate/dependency map from
  locked Cargo metadata and the validation-profile inventory from committed
  profile JSON.
- Documentation verification rejects generated-map drift, unknown first-party
  crate references, broken relative Markdown links, machine-local paths, and
  profile inventory drift; its self-test proves each rejection path.

## M5 Structural Generated-Code Safety

Make forbidden generated Rust constructs fail on every codegen invocation.
Replace unrestricted verbatim escape hatches with typed, origin-bearing
fragments or remove them; ensure validation and import/capability analysis cover
every remaining fragment. Keep corpus scanning as defense in depth.

Acceptance criteria:

- Raw compiler-owned item, statement, and expression fragments are opaque values whose
  constructors record the producing source callsite. External callers cannot
  construct an untracked fragment or invoke the unchecked renderer directly.
- Fragment syntax and the generated-code forbidden set (`unwrap`, `expect`,
  panic/todo/unimplemented macros, unsafe Rust, and allow attributes) are
  rejected structurally with origin-bearing evidence.
- Import/runtime-capability collection parses every remaining compiler fragment
  instead of treating it as an opaque leaf.
- Complete generated Rust is parsed and structurally checked after renderer and
  project postprocessing, and every `.rs` materialization path rejects invalid
  or forbidden source before writing it.
- Focused negative tests prove expression, statement, final-source, import, and
  materialization rejection. Existing generated-code corpus scanning remains a
  separate defense-in-depth gate.

## M6 Structured Codegen Error Propagation

Replace normal codegen panic/error-discard paths with structured diagnostics and
Result-returning public entrypoints. Preserve unwind containment only as the
last defensive boundary and add focused reproductions for each converted path.

Acceptance criteria:

- Public single-module, multi-module, test-project, and Cargo-project codegen
  entrypoints return `Result` and preserve the first structured `CodegenError`.
- Unsupported statement/expression lowering and invalid codegen input types do
  not emit `compile_error!` recovery artifacts or panic; they stop codegen with
  a contextual error.
- Assembled-IR validation, generated-source validation, Rust reparse
  postprocessing, and sysroot dependency planning return structured errors.
- The driver renders structured codegen failures with a dedicated diagnostic
  identity. Its unwind boundary remains only for unexpected invariant panics.
- Focused tests cover each converted failure mechanism and distinguish a
  structured codegen error from the final panic boundary.
- Remaining `assert!`/`unreachable!` sites in production codegen are documented
  or mechanically classifiable as programmer invariants; generated source still
  forbids compiler-owned `unwrap`/`expect`, panic/todo/unimplemented macros,
  unsafe Rust, and lint suppression.

Deferred M5 review follow-ups:

- Replace generated-source and assembled-IR assertions with structured codegen
  errors so a future provenance defect cannot turn user input into an internal
  compiler panic. Keep the driver pre-write validator as defense in depth.
- State the generated-source forbidden-set boundary explicitly: compiler-owned
  `unwrap`/`expect`, panic/todo/unimplemented macros, unsafe Rust, and lint
  suppression are forbidden; programmer-invariant `assert!` and
  `unreachable!` remain permitted only under the project invariant rule.

## M7 Canonical Frontend Project Compilation Product

Make the shared frontend session/context own project compile order and return a
stable product containing semantic outputs and diagnostics. Remove driver-side
`LoweringResult` reconstruction and prove CLI/analysis equivalence for one
snapshot.

Acceptance criteria:

- `FrontendContext` owns dependency-safe project order and source-backed cycle
  diagnostics. The driver has no separate compile-order implementation.
- One deterministic frontend product contains full lowering results, HIR, flow
  graphs, exports, diagnostics, and compile order.
- Single-file, project, package-project, and test-project frontend flows use the
  same product authority.
- The driver does not reconstruct `LoweringResult` or collect semantic exports.
- A focused test proves that product HIR and diagnostics equal analysis queries
  from the same frontend context snapshot.
- Project-order and source-backed cycle tests prove stable behavior.

## M8 LSP Hot Paths And Compiler-Service Dependency Direction

Move cache-hit checks before expensive Python declaration recomputation, reuse
existing HIR for lint rules, strengthen warm-workspace benchmarks, and extract
lower-level environment services so analysis/LSP do not depend upward on build
orchestration.

Acceptance criteria:

- Python declaration cache hits are decided from document and graph revisions
  before package fingerprinting, environment probing, interop planning, or
  workspace diagnostics run. Tests prove that repeated completion and hover
  requests reuse the snapshot and do not repeat those operations.
- Analysis passes its canonical HIR to HIR lint rules. The editor diagnostic
  path does not build a second frontend context for the same source revision.
- A lower-level `sifr_compiler_services` crate owns stdlib bootstrap, tooling
  sysroot views, generated Rust preview, Python runtime selection, and Python
  declaration validation. It does not own Cargo execution, CLI behavior, build
  workspaces, or LSP protocol handling.
- `sifr_analysis` and `sifr_lsp` do not depend on or reference `sifr_driver`.
  The dependency-direction guard and its negative self-tests enforce this
  boundary and prevent compiler services from depending upward.
- LSP benchmarks report server cache counters instead of fixed values.
  Completion and hover budgets enforce warm snapshot reuse. Workspace-shaped
  cases require at least 25 source modules.
- Focused service, analysis, lint, LSP, dependency, performance, documentation,
  and file-size checks pass. The compiler validation gates pass once each on
  the final reviewed candidate SHA.

## M9 Method-Lowering Authority And Unsafe-Code Documentation

Classify method-name dispatch sites, centralize language method semantics under
the scoped lowering authority, and ratchet against new ad hoc semantic dispatch.
Add local `SAFETY` contracts for each retained unsafe ABI operation and remove
blanket unsafe allowances where possible.

## M10 Collision-Resistant Cache Identity And Cache Lifecycle

Replace correctness-critical 64-bit FNV identities with a collision-resistant
digest and full-key verification for persisted hits. Derive compiler identity
from build/source revision and add bounded cache status/clean lifecycle commands.

Deferred M1 review follow-ups:

- Remove silent empty serialization fallback from Python certification cache
  identities in `build/python_runtime.rs` and the corresponding omission in
  `build/project_codegen.rs`.
- Normalize the safe-but-unstable cache misses caused when a policy path changes
  from non-canonical to canonical after the path is created.

Deferred M2 remediation-review follow-up:

- Replace the fresh per-invocation test execution path with a stable
  per-cache-key execution root, and reclaim its external Cargo target when the
  owning source-cache key is invalidated. This preserves full warm reuse and
  prevents fingerprint/target siblings from growing without bound.

## M11 Real Fuzz And Semantic Property Targets

Add coverage-guided targets for parser, lowering, ownership, codegen validation,
diagnostics, and project graphs, plus semantic properties for normalization,
narrowing, incremental/full equivalence, and deterministic codegen. Wire a real
sustained lane into nightly/release policy and name deterministic mutation smoke
accurately.

## M12 Maintainability Ratchets And Evidence-Based Flow Decisions

Add ratchets for module/function complexity, public API/fan-out growth, and
near-limit source concentration. Narrow broad dead-code/glob-export surfaces.
Instrument nested-inference divergence, remove the proven narrowing no-op, and
record an evidence-backed keep/refactor/remove decision for the flow graph.

Deferred M11 review and gate follow-ups:

- Replace the forbidden `SIFR-TYPE-0001` catch-all in the diagnostic fuzz
  target with a specific active diagnostic identity. The one permitted
  create-PR gate found this M11-owned hygiene defect after both allowed review
  rounds had finished. The gate was not repeated, and the merge gate did not
  run.
- Add `SIFR-INTERNAL-0003` to the diagnostic catalog and baseline-coverage
  registry, with executable coverage. This active code predates M11 and the
  create-PR gate reported the existing gap.
- Distinguish a missing fuzz tool, an offline dependency failure, an
  instrumented-build failure, a target timeout, and a real fuzz finding in the
  sustained result. Keep a bounded output tail for failed build preflights.
- Reconcile the fuzz area `timeout_seconds` and resource-class declaration with
  the cold-build preflight and the release budget.
- Remove the unused fuzz-project `serde_json` dependency.
- State or enforce the one-run contract for Rust semantic-property entries.
  The current `cargo-test` adapter does not use `repeat_runs`.
- Add a cross-process deterministic-codegen property before a release claim
  depends on process-randomized ordering.
- Hoist project-graph fixture creation out of the fuzz hot loop. Expand the
  ownership and valid-codegen grammars beyond fixed literal and statement
  templates.
- Give sustained target variants globally unique release-evidence labels before
  the suite enters release-evidence custody.
- Update the remaining archived phase-29 suite names and the harness comment
  that still use the old `fuzz-smoke` wording.

Deferred M3 review follow-ups:

- Extend the compiler/tooling direct-read roots to `sifr_stdlib_manifest` and
  `sifr_sysroot`, inventory their production manifest/source/digest probes, and
  pin both roots in the negative self-test. This is the new mechanism defect
  found by M3's final allowed review and must close as a later item.
- Detect wildcard and aliased `sifr_syntax` imports that expose a bare or
  alias-qualified `parse_module` call. No current production consumer uses
  these forms; add a negative seed when closing the deferred M3 mechanism.
- Route determinism-scale external commands and reproduction-command targets
  through the shared process-group deadline primitive, and propagate terminal
  interruption signals to detached gate subprocess groups.
- Decide whether production Rust binaries under compiler/tooling crates belong
  in the direct-filesystem inventory instead of preserving the current `bin`
  carve-out.

Deferred M4 remediation-review follow-ups:

- Reconcile the five legacy **Implementation responsibilities** blocks in
  `internal_docs/architecture.md`: remove shipped assignments, move execution
  history to `plans/`, and label genuinely unimplemented design as **Future**.
  Add a structural check that prevents current/future authority from mixing
  again. This is the new mechanism defect found by M4's final allowed review.
- Extend architecture mutations to seed `/root`, Windows user paths, missing
  generated-section markers, and qualified phantom crate references.
- Move the architecture mutation-registry binding into create-PR/merge
  execution (or run the structure suite there), and schedule the cheap
  documentation guard before expensive selected areas.
- Generalize the Cargo/disk workspace cross-check if first-party workspace
  members are ever allowed outside the current `crates/<name>` topology.

Deferred M5 review and gate follow-ups:

- Replace or structurally parse remaining raw `RustMatchArm::pattern` strings,
  including import/capability analysis of pattern paths.
- Measure the warm-codegen cost of per-render `syn` parsing and consolidate
  validation if evidence shows material overhead; do not use cold-cache timing.
- Reconcile the architecture document's three delivery-taxonomy lines at
  current lines 945, 946, and 1430. The M5 merge gate proved that the M4
  current/future authority defect reaches the coverage-matrix taxonomy check.
- Explicitly classify Rust interop probe crates as compiler diagnostic
  infrastructure outside the generated user-artifact `.rs` materialization
  gate, while retaining their owning probe validation.

Deferred M6 review follow-ups:

- Add direct focused reproductions for unsupported statement lowering,
  assembled-IR validation, stdlib-preamble reparse, and sysroot planning. The
  first-error accumulator is covered by the invalid-type reproduction, but
  these individual boundaries currently rely on broader tests.
- Remove or reconnect the dead ref-expression display lowering and structured
  try/except helper; rename any retained helper whose name still claims it
  panics. Close this with the M12 dead-code ratchet.
- Mechanically classify or convert the remaining production-codegen panics in
  union nominal-path lookup, class-method lowering, rendering, and output
  helpers. Close this with M9's method-lowering and unsafe/invariant audit.
- Record final-boundary generated-source validation as the canonical contract;
  fragment renderers no longer repeat the same validation after M6 made the
  public boundary fallible.
- Stop discarding `syn::parse_file` failures in stdlib filtering and relocation.
  Return a structured error from the owning canonical boundary.
- Keep source-language `assert` lowering explicitly outside the
  compiler-owned forbidden Rust construct set. Its user-triggered assertion is
  language behavior, not a compiler recovery panic.

Deferred M7 review follow-ups:

- Validate diagnostic-registry `owner_module` and fixture metadata against real
  first-party modules and emission sites. Add a negative self-test that seeds a
  relocated owner. M7's final allowed review found this mechanism defect after
  the stale `SIFR-IMPORT-0007` owner was remediated.
- Restore validated related spans for project-cycle diagnostics and add a JSON
  shape baseline. Preserve the concrete test-file prefix for span-less
  test-project diagnostics.
- Remove or reconnect the now-unused `compile_module_hir` and specialization
  metadata extraction APIs. Include them in the M12 dead-code and API ratchet.
- Normalize empty-input behavior between project loading and compilation.
- Measure the compilation product's repeated HIR/flow-graph retention and suite
  rematerialization before changing its required deterministic output shape.
- Reconcile the separate active compatibility-removal plan's reference to the
  deleted driver compile-order module during M13 record closure.

Deferred M8 review follow-ups:

- Pin and document the generated-preview specialization prefix and explain the
  intentional differences from the driver `emit` path, including display
  paths and Rust/stdlib interop resolution.
- Replace copied compiler-service Python-interop wrapper tests with direct
  service tests and add direct coverage for the real driver wrappers.
- Remove unused driver stdlib/tooling re-exports and unused compiler-preview
  fields under the M12 dead-code and public-API ratchets.
- Register the LSP file watcher explicitly. Preserve the current cache-hit
  order while making external package invalidation independent of optional
  client watcher behavior.
- Correct the stale `internal_docs/tooling_analysis.md` sentence that limits
  LSP dependencies to analysis and protocol conversion helpers.
- Preserve the analysis-to-lowering constraint in dependency-direction test
  sources when the guard is refined.
- Reuse the canonical analysis HIR in lint code-action paths, not only editor
  diagnostics, so `safe_fix_all_action` and `safe_fix_actions` do not construct
  another frontend context.
- Replace the remaining cold-start hard-coded LSP cache counters with measured
  deltas before a budget depends on them.
- Split `python_declarations.rs` by responsibility before its next material
  edit; it is 895 lines and has five lines of guardrail headroom.

Deferred M9 review follow-ups:

- Decide whether frontend constant evaluation is a second source-semantics
  authority. Extend the method-dispatch ratchet to it or document why its
  compile-time behavior is outside codegen method authority.
- Replace per-file method-dispatch counts with normalized site fingerprints.
  Cover equivalent binding names instead of only `method` and `method_name`.
- Reject item-level unsafe allowances that widen beyond one function. Review
  the three large file-wide callback and DLPack allowances for tighter
  function-level ownership.
- Use the system temporary directory for guard self-tests, or create the
  repository target directory explicitly, so a cold checkout cannot fail
  before it reaches a policy verdict.
- Mask literal and comment text before the local `INVARIANT:` lookback. Add
  direct character-literal fixtures to each shared scanner consumer, or add a
  dedicated shared scanner test. Document the conservative character-literal
  and lifetime distinction beside the scanner helper.
- Add a focused reproduction for the strict-registry decline path after the
  removed statement-level list `append` and `cloned` fallback. The M9 gates
  stopped at M11 taxonomy before the E2E and algorithmic suites.
- Convert the union nominal-path and context-manager renderer invariants to
  structured compiler errors if focused multi-module reproductions can reach
  them from source input.

Deferred M10 review and gate follow-ups:

- Group ownerless test-runner `.execution` and `.target` siblings by cache-key
  stem. Count and remove each path once. Add a lifecycle test for both
  companions without the source entry. The final allowed M10 review found this
  new mechanism defect, so it is deferred without a third review.
- Normalize the Rust interop backend manifest path before it enters the probe
  cache key.
- Add a same-key concurrent `sifr test` reproduction for the stable execution
  sibling. Keep the existing cross-key isolation test.
- Coordinate `sifr cache clean` with active cache users before it removes a
  workspace or shared probe target.
- Consolidate the repeated SHA-256 lowercase encoding helper only if one
  ownership boundary can serve compiler, package, driver, LSP, CLI, build
  script, and test code without adding an upward dependency.

## M12A Process-Group Deadlines And Terminal Signal Propagation

Close the mechanism omission found by M12's second and final review without
running a third M12 review.

Scope and acceptance criteria:

- Route determinism-scale external commands and reproduction-command targets
  through the shared process-group deadline primitive.
- Forward terminal `SIGINT` and `SIGTERM` from the gate entrypoint to every
  live detached child process group before exit.
- Extend the same primitive to the two coverage-fuzz call sites identified by
  the review so the known duplicate mechanism does not remain open.
- Add focused negative tests proving timeout cleanup and terminal-signal
  forwarding reach descendants rather than only the immediate child.
- Preserve existing output-tail, timeout, and failure-classification behavior.
- Run one exact-SHA Opus review for M12A. Do not revisit or run a third review
  for M12.

## M12B Restore Canonical List-Method Lowering On Structured Fallback Paths

Close the compiler regression exposed by M12A's real determinism-scale run and
already anticipated by M9's deferred strict-registry-decline reproduction.

Scope and acceptance criteria:

- Route locally recovered list receiver types through the canonical method
  authority after strict registry lowering declines.
- Ensure Sifr `list.append(value)` emits Rust `Vec::push(value)` on normal,
  exception-handler, and other structured statement paths. Do not restore an
  independent name-only fallback.
- Add a focused compiler reproduction for the strict-registry-decline path and
  compile the affected process fixtures that previously emitted
  `Vec::append(bool)`.
- Preserve ownership-aware argument adaptation for non-copy list elements.
- Run one exact-SHA Opus review, with at most one remediation review.

## M12C Terminal-Signal Escalation And Remaining Hardening Command Lifecycle

Close the new mechanism gap reported by M12A's final allowed review and the
remaining hardening command lifecycle omissions without running a third M12A
review.

Scope and acceptance criteria:

- After forwarding terminal `SIGINT` or `SIGTERM`, ensure a descendant that
  ignores the signal cannot outlive the verification runner. Use bounded
  group-liveness escalation without re-entering an interrupted `Popen.wait`.
- Extend the terminal-signal self-test with a descendant that ignores the
  forwarded signal.
- Give the remaining Cargo property command a finite, process-group-aware
  lifecycle instead of a bare unbounded `subprocess.run`.
- Preserve an explicit timeout bit through variant and determinism wrappers if
  either path gains timeout-specific classification; do not infer a deadline
  from native exit code 124.
- Run one exact-SHA Opus review, with at most one remediation review.

## M12D Documentation Mutation-Registry Consistency

Close the M4/M12 registry drift exposed by the record-only documentation check.

Scope and acceptance criteria:

- Make `docs_inventory.json` exactly match every executable architecture
  mutation case, including generated-marker, qualified-crate, `/root`, and
  Windows-path cases.
- Keep the structure checker as the binding between executable mutation
  registries and the committed inventory.
- Prove the positive documentation structure check and its mutation self-tests
  pass from a clean checkout state.
- Run one exact-SHA Opus review, with at most one remediation review.

## M12E Atomic Repeated-Terminal-Signal Escalation Entry

Close the new mechanism gap reported by M12C's final allowed review without
running a third M12C review.

Scope and acceptance criteria:

- Remove the bytecode-scale window between marking terminal-signal handling
  active and disabling subsequent `SIGINT`/`SIGTERM` delivery.
- Ensure a repeated terminal signal cannot take the re-entrant `SystemExit`
  branch before the process-group escalation worker starts.
- Preserve the first signal's `128 + signum` exit contract and bounded
  process-group escalation.
- Add a deterministic self-test for repeated-signal entry rather than relying
  on timing-sensitive external delivery.
- Run one exact-SHA Opus review, with at most one remediation review.

## M12F Restore Generated Demo Freshness After Compiler Corrections

Close the checked-in generated-companion drift exposed by the phase's single
create-PR gate.

Scope and acceptance criteria:

- Regenerate `demos/decimal_verification/emitted.rs` with the exact compiler
  built from the completed implementation stack.
- Confirm the only changes are the two BigDecimal rounding expressions emitted
  by the corrected canonical method-lowering path.
- Prove the generated-demo freshness guard and the decimal verification demo
  pass without rerunning the consumed create-PR gate.
- Run one exact-SHA Opus review, with at most one remediation review.

## M13 Phase Closure And Whole-Phase Review

Reconcile every milestone record, deferred finding, architecture/roadmap status,
and reused validation artifact. Run documentation checks and one exact-SHA
whole-phase Opus review without repeating unchanged implementation validation.

## Evidence Ledger

Evidence is added after each merge. Review files remain outside the reviewed Git
tree and are keyed by candidate SHA.

### M1 Deferred Integration Handoff

- Branch: `codex/architecture-audit-closure-m1`
- Draft PR: [#3553](https://github.com/sifr-lang/sifr/pull/3553)
- Implementation candidate: `1c43fe34847925a269288b4073f5ca7ca7d6063e`
- Opus review: `SATISFIED`, no blocking findings; published in the PR and
  preserved outside the Git tree under the candidate SHA.
- Targeted validation: `cargo test -p sifr_package` (143 passed), driver
  cache-identity tests (7 passed), binary-key tests (4 passed), Cargo-resolution
  tests (9 passed), affected production checks, workspace Clippy, formatting,
  and the 3,263-file size guardrail passed.
- Create-PR gate: the one permitted run exited 124. Runtime-platform correctness
  passed 28 variants with zero failures and one declared skip, but its first
  cold-cache run took 217,638 ms against a 120,000 ms blocking budget. The
  profile stopped before later required areas and toolchain suites.
- Merge gate: the one permitted run executed on the exact implementation SHA.
  Every completed compiler and verification area passed until the externally
  owned distribution-release suite rejected the expired single-maintainer
  approval waiver. Distribution qualification passed 67 of 68 variants; the
  sole failure was `schema-v2-preview-epoch-bootstrap`. The merge report
  SHA-256 is `bda4726743a52eaf6e552467432b93231f38bb239e48d9a94fe26f1145097763`;
  the distribution report SHA-256 is
  `475a211ece2ef7d55b30d600ba09c15151549a1a930fd9fc3ab51ae6ad4d9096`.
- External owner: `ad-hoc-distinct-release-reviewer-restoration.md`. Do not
  extend the expired waiver or weaken its validation.
- User direction on 2026-08-27: defer the distinct-human-reviewer dependency and
  maximize implementation through the final phase step. M2 and later items may
  be staged as sequential stacked draft PRs without claiming M1 is merged.
- Final integration action: restore a genuinely distinct release reviewer,
  update stacked bases if repository governance changes, and reuse unchanged M1
  evidence. Validate only the affected distribution boundary unless the user
  explicitly authorizes a second full gate.

### M2 Deferred Integration Handoff

- Branch: `codex/architecture-audit-closure-m2`
- Stacked draft PR: [#3554](https://github.com/sifr-lang/sifr/pull/3554), based
  on the M1 branch.
- Initial candidate: `ec77ce12fa05e4497d160a7b9dc8e39ade3a43dc`.
- Initial exact-SHA Opus review: `BLOCKERS`. It found JSON-valued test output
  suppression, discarded warm Cargo artifacts, and an unintended Python-only
  native-link policy expansion. All three were remediated.
- Final candidate: `16024325813dbee56e84a838e42679340f0f829a`.
- One permitted remediation review: `SATISFIED`. The review also reported the
  non-blocking stable-execution-root/cache-reclamation mechanism recorded for
  M10 above; no third M2 review was run.
- Targeted validation: 17 test-runner tests; frozen resolution, cache, and Cargo
  trace coverage; JSON stdout preservation; warm external-target reuse; CLI
  frozen-mode parsing; shared materializer and support-main integration tests;
  workspace Clippy; formatting; and the HIR/file-size guardrail passed.
- An optional cold all-codegen/all-driver run completed the 1,142 codegen tests
  but was stopped during unrelated nested driver Cargo-probe contention after
  more than 15 minutes. It is not claimed as passing evidence.
- Compiler create-PR/merge gates and integration remain deferred with M1 under
  the user's instruction to maximize later phase implementation while the known
  distinct-human-reviewer dependency is unavailable. Do not rerun unchanged M2
  validation during final integration unless its stacked diff changes.

### M3 Deferred Integration Handoff

- Branch: `codex/architecture-audit-closure-m3`.
- Stacked draft PR: [#3555](https://github.com/sifr-lang/sifr/pull/3555), based
  on the M2 branch.
- Initial candidate: `4e0ebb8ebb68a47b16451f4f0ea9a16a1e449407`.
- Initial exact-SHA Opus review: `NOT SATISFIED`. It found that imported/bare
  syntax parsing evaded the split-brain guard and that the deadline descendant
  marker lacked a guaranteed parent. Both findings were remediated in one
  batch.
- Final M3 implementation candidate:
  `07e3d7d0f5123a89a30a4fcf149e51ebff7d6c7e`.
- The one permitted remediation review verified both original corrections, then
  reported a new direct-read root mechanism: `sifr_stdlib_manifest` and
  `sifr_sysroot` remain outside the inventory. Per the phase execution rule, the
  defect and the related wildcard/alias parse-import seed are recorded under
  M12 above; no third M3 review was run.
- Review evidence is published in PR #3555 and preserved outside the Git tree
  under both candidate SHAs.
- Targeted validation: verification runner and hardening self-tests; positive
  and seeded-negative split-brain and direct-read guards; performance
  `frontend-syntax-guardrails` (4/4, including four focused frontend cache unit
  tests); developer-tooling `typescript-go-transfer` (2/2); touched-file Ruff;
  profile/area schema checks; HIR maintainability; the 3,266-file size
  guardrail; and whitespace checks passed.
- No `crates/**` compiler source changed, so create-PR and merge gates were not
  applicable. Integration stays deferred with the stacked chain.

### M4 Deferred Integration Handoff

- Branch: `codex/architecture-audit-closure-m4`.
- Stacked draft PR: [#3556](https://github.com/sifr-lang/sifr/pull/3556), based
  on the M3 branch.
- Initial candidate: `058780a5b505500fd04a80eca71aa467dfe037d8`.
- Initial exact-SHA Opus review: `NOT SATISFIED`. It found dev-only dependency
  edges in the generated map, incomplete phantom crate/path rejection, and a
  stale PR-gate paragraph. All three were remediated in one batch.
- Final M4 implementation candidate:
  `0cb9720cb80e66bc2be3c73e78206106cd998bd1`.
- The final allowed review verified the original fixes and reported a new
  current-vs-future authority defect in five legacy implementation-responsibility
  blocks. The defect and related hardening are recorded under M12 above; no
  third M4 review was run.
- Review evidence is preserved outside the Git tree under both candidate SHAs.
- Targeted validation: architecture positive and mutation checks; all three
  documentation suites; verification profile/runner self-tests; touched-file
  Ruff and new-file formatting; JSON parsing; HIR maintainability; the
  3,267-file size guardrail; and whitespace checks passed.
- The architecture guard now runs in create-PR and merge profiles. No
  `crates/**` compiler source changed, so the compiler create-PR and merge gates
  were not applicable. Integration stays deferred with the stacked chain.

### M5 Deferred Integration Handoff

- Branch: `codex/architecture-audit-closure-m5`.
- Stacked draft PR: [#3557](https://github.com/sifr-lang/sifr/pull/3557), based
  on the M4 branch.
- Initial candidate: `b5060ba1984f4ba9b9bf8964c95f86f6826854c5`.
- Initial exact-SHA Opus review: `NOT SATISFIED`. It found that the final
  forbidden-call scan could treat legal user functions or methods named
  `unwrap` or `expect` as compiler-owned and trigger an internal assertion.
  The consolidated remediation added raw-identifier provenance for source
  callables, structured call/macro rejection, nested lint-suppression checks,
  structural item-fragment import parsing, and the exact-integer limit
  diagnostic.
- Reviewed remediation candidate:
  `74fb63bcf00a3d46b7ec03878065ca9bb10a0426`. The final allowed review confirmed
  the mechanism but found the same original defect at one recursive
  argument-position lowering site. Per the no-third-review rule, that residual
  site and the audited super/operator/plain/nested name flows were corrected
  without another Opus round. The focused reproduction now emits and compiles
  `values.push(wrapper.r#unwrap())` and `r#expect(...)`.
- Final M5 implementation candidate:
  `5644505a6badeb51b39e38df82b3f8972c545265`.
- Targeted validation: all 1,149 `sifr_codegen` tests; driver generated-project
  materialization tests; the raw-code guard; workspace Clippy; formatting; HIR
  maintainability; the 3,269-file size guardrail; a direct `sifr emit`
  reproduction; and the generated-code smoke corpus (corpus, panic scan,
  intrinsic-panic lint, rustfmt, and determinism) passed.
- The single create-PR gate ran on `6cf7fd5dc25b799c47d61ea49ff5a62816731204`
  and stopped on the stale retained-runtime-root entry exposed when M5 removed
  generated `DEFAULT_MAX_INTEGER_DIGITS` use. The obsolete entry was removed;
  its owning allowlist guard then passed. The create-PR gate was not rerun.
- The single merge gate ran on the exact final implementation SHA. It passed
  the M5-related guardrails, runner foundations, and all ten Rust interop
  variants, then stopped in coverage taxonomy on three M4 architecture lines
  that describe future M9/M11 delivery work. That already-deferred
  current/future authority mechanism is recorded under M12; the merge gate was
  not rerun.
- Review evidence is published in PR #3557 and preserved outside Git under both
  reviewed SHAs. Integration remains deferred with the stacked chain.

### M6 Deferred Integration Handoff

- Branch: `codex/architecture-audit-closure-m6`.
- Stacked draft PR: [#3558](https://github.com/sifr-lang/sifr/pull/3558), based
  on the M5 branch.
- Exact implementation candidate:
  `ef46a3eac5f7e54b374f6c648609e49a3dc5f302`.
- The one exact-SHA Opus review returned `SATISFIED` with no blocking findings.
  It verified that all live sentinel/error-accumulator paths are checked before
  rendering or materialization and that structured failures use
  `SIFR-INTERNAL-0003` while the final unwind boundary retains
  `SIFR-INTERNAL-0001`. Evidence is published in PR #3558 and preserved outside
  Git under the candidate SHA. No remediation review was needed.
- Targeted validation: all 1,151 `sifr_codegen` tests; 565 passing and 76
  ignored `sifr_driver` tests; workspace Clippy with warnings denied;
  formatting; HIR maintainability; and the 3,271-file size guardrail passed.
- The single create-PR and merge gates both ran on the exact reviewed SHA. Each
  passed generated-demo freshness, dependency/ownership/sysroot/stdlib/driver
  guardrails, verification-runner foundations, and all ten Rust interop
  variants, then stopped at the same three M4 architecture delivery-taxonomy
  lines (current lines 945, 946, and 1430). The owning current/future authority
  defect is already assigned to M12; neither gate was rerun.
- Integration remains deferred with the stacked chain under the user's
  instruction to continue through the phase before restoring the distinct
  human reviewer.

### M7 Deferred Integration Handoff

- Branch: `codex/architecture-audit-closure-m7`.
- Stacked draft PR: [#3559](https://github.com/sifr-lang/sifr/pull/3559), based
  on the M6 branch.
- Initial candidate:
  `7bfbf667fb7baed927e30435ad5f1604507f3ae0`.
- Initial exact-SHA Opus review: `NOT SATISFIED`. The implementation met the
  six M7 compiler criteria, but the `SIFR-IMPORT-0007` registry owner and its
  generated documentation still named the deleted driver compile-order module.
- Final M7 implementation candidate:
  `30c7ab5e1b5bffc4a9e16f65c061e07498951fae`.
- The one permitted remediation review returned `SATISFIED`. It verified the
  relocated frontend owner, generated catalog consistency, all M7 acceptance
  criteria, and a clean regeneration check. It also found that owner and
  fixture registry metadata are unvalidated free-form strings. That mechanism
  is recorded under the deferred M7 follow-ups; no third review was run.
- Review evidence is published in PR #3559 and preserved outside Git under
  both reviewed SHAs.
- Targeted validation: 126 `sifr_frontend` tests; 565 passing and 76 ignored
  `sifr_driver` tests; 32 `sifr_diagnostics` tests; workspace Clippy with
  warnings denied; formatting; frontend, HIR, driver, documentation, and
  file-size guardrails; generated diagnostic documentation; and whitespace
  checks passed.
- The single create-PR and merge gates both ran on the exact final candidate.
  Each passed generated-demo freshness, dependency, ownership, sysroot,
  stdlib, driver, and verification-runner guardrails plus all ten Rust interop
  variants. Each then stopped at the already-recorded M4 architecture delivery
  taxonomy on lines 945, 946, and 1430. Neither gate was rerun.
- Integration remains deferred with the stacked chain under the user's
  instruction to continue through the phase before restoring the distinct
  human reviewer.

### M8 Deferred Integration Handoff

- Branch: `codex/architecture-audit-closure-m8`.
- Stacked draft PR: [#3560](https://github.com/sifr-lang/sifr/pull/3560), based
  on the M7 branch.
- Initial candidate:
  `ddd6721b387d22c0279d222d4c1ba1ff9471141b`.
- The initial exact-SHA Opus review returned `SATISFIED` with no blocking
  findings. It verified the cache-hit ordering, HIR reuse, lower compiler
  service boundary, dependency guards, real cache counters, warm budgets, and
  workspace scale.
- The single create-PR gate ran on the initial candidate and exposed one
  M8-owned omission: `sifr_compiler_services` lacked Cargo coverage
  classification. It also reached the already-recorded M4 architecture
  delivery taxonomy on current lines 961, 962, and 1446. The gate was not
  rerun.
- Final M8 implementation candidate:
  `da39eb709ddffef80d3dc6297cde959f88d85bc5`. It classifies the package,
  library target, and test-support feature, and adds executed crate-test
  membership to the create-PR, merge, nightly, and release profiles.
- The one permitted remediation review returned `SATISFIED` with no blocking
  findings. Review evidence is published in PR #3560 and preserved outside Git
  under both reviewed SHAs.
- Targeted validation: 82 compiler-service tests; 48 analysis tests; 22 lint
  tests; 78 LSP tests; 32 diagnostic tests; workspace Clippy with warnings
  denied; formatting; real completion and hover benchmarks with 40 hits and
  zero misses each; architecture, dependency-direction and negative,
  performance-manifest and self-test, HIR, driver-maintainability,
  documentation, coverage classification, profile, and the 3,277-file size
  guardrails passed.
- The single merge gate ran on the exact final candidate. It confirmed strict
  coverage classification, profile assignment, negative self-tests, generated
  demo freshness, dependency/ownership/sysroot/stdlib/driver guardrails,
  verification-runner foundations, and all ten Rust interop variants. It then
  stopped only on the same three M4 architecture delivery-taxonomy lines at
  current lines 961, 962, and 1446. The merge gate was not rerun.
- Integration remains deferred with the stacked chain under the user's
  instruction to continue through the phase before restoring the distinct
  human reviewer.

### M9 Deferred Integration Handoff

- Branch: `codex/architecture-audit-closure-m9`.
- Stacked draft PR: [#3561](https://github.com/sifr-lang/sifr/pull/3561), based
  on the M8 branch.
- Initial candidate:
  `a41ade982d463865a933f9b5200ff4f334869086`.
- The initial exact-SHA Opus review returned `NOT SATISFIED`. It found that the
  shared Rust source masker did not consume character or byte-character
  literals, so a quote literal could hide a later production policy site.
- Final M9 implementation candidate:
  `f6d1f4edaab6a2bfa0952ab89fd1980bec284703`. The remediation consumes normal,
  byte, escaped, hexadecimal, and Unicode character literals without consuming
  lifetimes or loop labels. Its negative fixture proves that a panic after a
  byte quote remains visible.
- The one permitted remediation review returned `SATISFIED` with no blocking
  findings. Review evidence is published in PR #3561 and preserved outside
  Git under the final candidate SHA.
- Targeted validation: all 1,151 `sifr_codegen` tests; all 82 `sifr_runtime`
  library tests; workspace Clippy with warnings denied; formatting; verification
  runner profile self-tests; method-dispatch, unsafe-ABI, codegen-invariant,
  HIR-maintainability, and the 3,282-file size guardrails passed. All three new
  guardrails also passed their negative self-tests.
- The single create-PR and merge gates both ran on the exact final candidate.
  Each passed every M9 guard and self-test, generated-demo freshness,
  dependency, ownership, sysroot, stdlib, driver, and runner-foundation checks,
  plus all ten Rust interop variants. Each then stopped only on the two M11
  delivery-taxonomy lines at current `internal_docs/architecture.md` lines 965
  and 1449. Neither gate was rerun.
- Integration remains deferred with the stacked chain under the user's
  instruction to continue through the phase before restoring the distinct
  human reviewer.

### M10 Deferred Integration Handoff

- Branch: `codex/architecture-audit-closure-m10`.
- Stacked draft PR: [#3562](https://github.com/sifr-lang/sifr/pull/3562), based
  on the M9 branch.
- Initial candidate:
  `ed6c7c2e6181e5607d7ff952dc5985c410d42e20`.
- The initial exact-SHA Opus review returned `NOT SATISFIED`. It found the
  retained empty certification-serialization fallback, the fresh test
  execution root, and orphaned external Cargo targets.
- Final M10 implementation candidate:
  `2e2ad86fb80ee916542738967935039c157fa18e`. It makes certification identity
  serialization fallible, always includes both certification fragments, uses
  a stable per-key execution sibling, reclaims execution and target siblings,
  normalizes policy paths, fingerprints formatter sources, and publishes the
  cache CLI page in documentation navigation.
- The one permitted remediation review verified all three original blockers.
  It then found a new ownerless-companion grouping defect. Under the phase
  review limit, that mechanism is recorded under M12 above and no third review
  ran. Both reviews are published in PR #3562 and preserved outside Git under
  their candidate SHAs.
- Validation: the initial broad non-E2E-pass command passed for `sifr`, driver,
  frontend, package, and LSP. After remediation, all 17 test-runner tests, all
  82 compiler-service tests, all 78 LSP tests, and focused certification,
  lifecycle, invalidation, and execution tests passed. Workspace Clippy with
  warnings denied, formatting, documentation structure, frontend cache,
  package-manager, TypeScript-Go transfer and negative self-test, HIR, driver,
  and the 3,289-file size guards passed.
- The single create-PR and merge gates both ran on the exact final candidate.
  Each passed generated-demo freshness, dependency, ownership, method,
  unsafe-ABI, codegen-invariant, sysroot, stdlib, driver, runner-foundation,
  and all ten Rust interop variants. Each then stopped on the unclassified new
  frontend build-script target and the two already-recorded M11 delivery-
  taxonomy lines at `internal_docs/architecture.md` lines 975 and 1459.
  Neither gate was repeated.
- Integration remains deferred with the stacked chain under the user's
  instruction to continue through the phase before restoring the distinct
  human reviewer.

### M11 Deferred Integration Handoff

- Branch: `codex/architecture-audit-closure-m11`.
- Stacked draft PR: [#3563](https://github.com/sifr-lang/sifr/pull/3563), based
  on the M10 branch.
- Initial candidate:
  `b62fe3afaf82315eccad495d3287a1fec5685b7d`. The initial exact-SHA Opus
  review found that a cold sanitizer build could consume the first fuzz
  target's time budget while the non-blocking CI job stayed green.
- Final implementation candidate:
  `7664b902bf4697ca20bc39c683cdb617d26032e2`. It adds a separate 1,200-second
  runner build preflight and a blocking CI build step before timed target
  execution. A cold offline build with a fresh target directory passed in 2
  minutes 8 seconds. The one permitted remediation review returned
  `SATISFIED` with no blocking findings.
- Implementation: six libFuzzer targets cover parsing, lowering, ownership,
  generated-Rust validation, diagnostic presentation, and project graphs.
  Rust semantic properties cover union normalization, narrowing,
  incremental/full diagnostics equivalence, and deterministic codegen. The
  deterministic suite is now named `mutation-smoke`. Nightly and release
  profiles select the non-blocking `sustained-fuzz` suite.
- Targeted validation: all four semantic-property tests passed; the blocking
  fuzz/property area passed 43 variants; all six fuzz targets built offline
  and passed the 45-second nightly sustained lane; workspace and fuzz-project
  Clippy passed with warnings denied; formatting, coverage readiness,
  architecture, runner self-tests, profile plans, HIR maintainability, and the
  3,292-file size guard passed.
- Review evidence is published in PR #3563 and preserved outside Git under
  both reviewed SHAs.
- The one permitted create-PR gate ran on the exact final candidate. It passed
  generated-demo freshness, all compiler guardrails and self-tests, runner
  foundations, Rust interop, and coverage readiness. It then stopped in the
  diagnostic rules suite. The new diagnostic fuzz target used the forbidden
  `SIFR-TYPE-0001` catch-all. The same suite also reported the pre-existing
  missing catalog and baseline-coverage rows for `SIFR-INTERNAL-0003`.
- The create-PR gate was not repeated. The merge gate did not run after the
  create-PR failure. Both review rounds are exhausted, so changing the
  implementation would require new review authority. The M11-owned defect and
  related follow-ups are recorded under M12 above.
- Integration remains deferred with the stacked chain and the human reviewer
  remains skipped under the user's current instruction.

### M12 Deferred Integration Handoff

- Branch: `codex/architecture-audit-closure-m12`.
- Stacked draft PR: [#3564](https://github.com/sifr-lang/sifr/pull/3564), based
  on the M11 branch.
- Initial candidate:
  `13e7e9159f8a63810c6ff43cd9786381cd55d90a`. The initial exact-SHA Opus
  review returned `NOT SATISFIED` with seven blockers covering tooling docs,
  structured stdlib parse failures, canonical-HIR lint fixes, LSP watcher and
  external invalidation behavior, active cache coordination, structured match
  patterns, and stale phase-29 names.
- Final M12 candidate:
  `17d7ac63eae4a9417a2d60415c06d7de7016ce6c`. It remediates all seven initial
  blockers and the M3-M11 follow-ups implemented in its reviewed diff.
- The second and final M12 review returned `NOT SATISFIED` because the already
  recorded M3 process-group deadline and terminal-signal follow-up remained
  open. Under the review limit, no third M12 review will run. The mechanism is
  now the bounded M12A item above. Review evidence is preserved outside Git at
  `.codex/review-evidence/architecture-closure/m12-17d7ac63eae4a9417a2d60415c06d7de7016ce6c-remediation.md`.
- Validation on the final candidate: workspace and fuzz-project Clippy passed
  with warnings denied; formatting and whitespace checks passed; codegen,
  package, diagnostics, runtime, lint, analysis, and compiler-service library
  suites passed; focused active-cache, same-key cross-process, watcher,
  external-invalidation, and codegen-boundary tests passed. Method-dispatch,
  unsafe-ABI, codegen-invariant, maintainability, HIR, architecture,
  TypeScript-Go transfer, diagnostic-doc, fuzz-lock, and 900-line guards all
  passed, including applicable self-tests.
- The single create-PR and merge gates remain reserved for the final compiler
  candidate after the later implementation items. Integration and the human reviewer remain deferred
  under the user's current instruction.

### M12A Merged Handoff

- Branch: `codex/architecture-audit-closure-m12a`.
- Stacked PR: [#3565](https://github.com/sifr-lang/sifr/pull/3565), merged into
  the M12 branch as `15b8c063957154d8e6e691f28ce1c63bfa372cd7`.
- Initial candidate: `d858b12aac1763e850ccb10f6ba37d4f6e7b4995`.
  Its exact-SHA Opus review returned `SATISFIED` with no blockers. Three
  candidate-level suggestions were accepted for the single remediation batch:
  the non-reentrant signal-registry lock, the prematurely reset signal guard,
  and native exit-code 124 timeout ambiguity.
- Final candidate: `2b0820dabf890dc19850289273ade09d8b048cd5`.
  The one permitted remediation review returned `SATISFIED` with no blockers.
  It reported a new terminal-forwarding escalation gap, recorded as M12C; no
  third M12A review will run.
- Validation: touched Python Ruff and compilation, whitespace, both runner
  self-test layers, maintainability, file-size, HIR, driver, and method-dispatch
  guardrails passed. Self-tests cover SIGTERM-ignoring deadline descendants,
  terminal forwarding, signal delivery while the registry lock is held, and a
  native exit code 124 that is not a timeout.
- The real merge-profile determinism-scale suite reached nested E2E compilation
  and failed both commands with exit code 1. Generated Rust contained
  `actual.append(bool)` where the canonical Sifr list method must emit
  `actual.push(bool)`. The source trace points to M9's removed direct fallback
  after strict registry decline; this compiler defect is M12B. The unchanged
  failing suite was not repeated.
- Review evidence is outside Git under both exact candidate SHAs in
  `.codex/review-evidence/architecture-closure/`.
- M12A changed no compiler files, so it did not run or consume the reserved
  Sifr create-PR or merge gate.
- The record-only documentation check then reported architecture mutation-case
  registration drift: the executable registry has 12 cases while the inventory
  retains 8. This pre-existing M4/M12 closure defect is M12D; it is not absorbed
  into the M12A handoff commit.

### M12B Merged Handoff

- Branch: `codex/architecture-audit-closure-m12b`.
- Stacked PR: [#3566](https://github.com/sifr-lang/sifr/pull/3566), merged into
  the M12 branch as `054bbe1c5277c50fb0d609fc953be4349f942e16`.
- Exact implementation candidate:
  `c4a23973f80d8eb796e4b9c984c89a248b58ac88`.
- The one exact-SHA Opus review returned `SATISFIED` with no blockers. It
  verified extraction equivalence for collection argument conversion,
  TypeVar/non-copy ownership preservation, absence of double conversion, and
  canonical authority dispatch without new user-method interception. No
  remediation review was needed.
- Validation: the focused strict-registry-decline reproduction passed; six
  no-cache process fixtures from the failing determinism batch compiled and ran
  6/6 across two generated groups; all 1,163 codegen tests and workspace Clippy
  with warnings denied passed. Formatting, codegen invariant, maintainability,
  HIR, method-dispatch, file-size, and whitespace guards passed.
- Review evidence is outside Git at
  `.codex/review-evidence/architecture-closure/m12b-c4a23973f80d8eb796e4b9c984c89a248b58ac88.md`.
- M12B changes compiler files. Under the phase rule, it did not run a per-item
  Sifr gate; the single create-PR and merge gates remain reserved for the final
  implementation SHA after M12C, M12D, and M12E.

### M12C Merged Handoff

- Branch: `codex/architecture-audit-closure-m12c`.
- Stacked PR: [#3567](https://github.com/sifr-lang/sifr/pull/3567), merged into
  the M12 branch as `c77485c27f489e63a5d6f8c178a02a57234ad23d`.
- Initial candidate: `a99e1128c1dfccbefbc45a95bcf593db5cf84252`.
  Its exact-SHA Opus review returned `SATISFIED` with no blockers. The accepted
  remediation prevents a later terminal signal from interrupting the
  non-daemon escalation worker's interpreter-shutdown join and adds SIGINT
  coverage.
- Final candidate: `9442b51faa8a15c1466919ad797d0cfcd9d0e8ef`.
  The one permitted remediation review returned `SATISFIED` with no blockers.
  It reported a new bytecode-scale repeated-signal entry race, recorded as
  M12E; no third M12C review will run.
- Validation: touched Python Ruff and compilation, both runner self-test
  layers, the real 19-variant property suite, maintainability, file-size, HIR,
  driver, whitespace, and manifest JSON checks passed. The self-tests cover
  SIGTERM- and SIGINT-ignoring descendants and explicit native-124 versus
  timeout classification.
- Review evidence is outside Git under both exact candidate SHAs in
  `.codex/review-evidence/architecture-closure/`.
- M12C changed no compiler files, so it did not run or consume the reserved
  Sifr create-PR or merge gate.

### M12D Merged Handoff

- Branch: `codex/architecture-audit-closure-m12d`.
- Stacked PR: [#3568](https://github.com/sifr-lang/sifr/pull/3568), merged into
  the M12 branch as `23551f57b74f1ac4fe0fcade553773d8761dd48d`.
- Exact implementation candidate:
  `e97e7332e6ef537738ebd6b6f9fad60384ba1f2f`.
- The one exact-SHA Opus review returned `SATISFIED` with no blockers or
  remediation. It verified exact ordered equality between the committed
  inventory and all 12 executable architecture mutation cases, plus the
  reciprocal structure-checker binding.
- The parent inventory reproducibly failed with `architecture mutation case
  registration drifted`. On the clean candidate, JSON parsing, the structure
  and GA mutation harnesses, all 12 architecture mutation cases, the positive
  architecture check, file-size, and whitespace checks passed.
- Review evidence is outside Git at
  `.codex/review-evidence/architecture-closure/m12d-e97e7332e6ef537738ebd6b6f9fad60384ba1f2f.md`.
- M12D changed no compiler files, so it did not run or consume the reserved
  Sifr create-PR or merge gate.

### M12E Merged Handoff

- Branch: `codex/architecture-audit-closure-m12e`.
- Stacked PR: [#3569](https://github.com/sifr-lang/sifr/pull/3569), merged into
  the M12 branch as `da98972128607699b9598f6c1fda89b19cce0d38`.
- Exact implementation candidate:
  `2a8adc32dfed16933dcb22b4d77f989d97c80734`.
- The one exact-SHA Opus review returned `SATISFIED` with no blockers or
  remediation. It verified that repeated entry returns to the in-progress
  first handler, preserving first-signal forwarding, worker startup, bounded
  escalation, and the first signal's exit status.
- Validation: touched Python Ruff and compilation, the full runner self-test,
  maintainability, file-size, and whitespace checks passed. Opus also ran the
  deterministic helper against both revisions: the parent exited 130 from the
  nested SIGINT, while the candidate resumed the outer SIGTERM and exited 143.
- Review evidence is outside Git at
  `.codex/review-evidence/architecture-closure/m12e-2a8adc32dfed16933dcb22b4d77f989d97c80734.md`.
- M12E changed no compiler files, so it did not run or consume the reserved
  Sifr create-PR or merge gate.

### Final Create-PR Gate Attempt

- Exact stack SHA: `7cb37df1571e0a785b41116cb208713d3bdcd782`.
- Before the long gate, the worktree's private target was 25 GiB and unused;
  the required local `cargo clean` removed 30.7 GiB.
- The one permitted create-PR gate ran once and exited 1. Cargo setup, HIR
  maintainability, file-size, and maintainability-ratchet steps passed. The
  generated-demo freshness guard then reported only
  `demos/decimal_verification/emitted.rs` as stale and stopped the lane.
- The emitted diff is limited to two BigDecimal rounding expressions. This
  in-scope generated companion defect is M12F. The create-PR gate will not be
  rerun.
- Machine evidence is retained under
  `target/validation_lane_reports/create-pr.latest.json` and its referenced
  log/time artifacts for this worktree.

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
| M5 | Structural generated-code safety | in progress | | |
| M6 | Structured codegen error propagation | pending | | |
| M7 | Canonical frontend project compilation product | pending | | |
| M8 | LSP hot paths and compiler-service dependency direction | pending | | |
| M9 | Method-lowering authority and unsafe-code documentation | pending | | |
| M10 | Collision-resistant cache identity and cache lifecycle | pending | | |
| M11 | Real fuzz and semantic property targets | pending | | |
| M12 | Maintainability ratchets and evidence-based flow decisions | pending | | |
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

## M6 Structured Codegen Error Propagation

Replace normal codegen panic/error-discard paths with structured diagnostics and
Result-returning public entrypoints. Preserve unwind containment only as the
last defensive boundary and add focused reproductions for each converted path.

## M7 Canonical Frontend Project Compilation Product

Make the shared frontend session/context own project compile order and return a
stable product containing semantic outputs and diagnostics. Remove driver-side
`LoweringResult` reconstruction and prove CLI/analysis equivalence for one
snapshot.

## M8 LSP Hot Paths And Compiler-Service Dependency Direction

Move cache-hit checks before expensive Python declaration recomputation, reuse
existing HIR for lint rules, strengthen warm-workspace benchmarks, and extract
lower-level environment services so analysis/LSP do not depend upward on build
orchestration.

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

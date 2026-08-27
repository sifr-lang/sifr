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
| M1 | Warm-cache lock correctness and serialization failures | blocked: cold create-PR budget | [#3553](https://github.com/sifr-lang/sifr/pull/3553) | `1c43fe34847925a269288b4073f5ca7ca7d6063e` |
| M2 | Canonical test/build materialization | pending | | |
| M3 | Verification gate integrity | pending | | |
| M4 | Architecture documentation accuracy and generated crate map | pending | | |
| M5 | Structural generated-code safety | pending | | |
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

## M3 Verification Gate Integrity

Enforce real subprocess deadlines, expand direct-filesystem inventory to all
production compiler/tooling roots, and add self-tests proving each guardrail can
fail. Classify legitimate CLI parsing separately from compiler semantic-source
authority instead of banning syntax use mechanically.

## M4 Architecture Documentation Accuracy And Generated Crate Map

Correct immediately false API and topology claims, remove phantom crates and
machine-local paths, separate current architecture from history/future design,
generate the workspace crate map from Cargo metadata, and block documented
crate/path/profile drift.

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

## M13 Phase Closure And Whole-Phase Review

Reconcile every milestone record, deferred finding, architecture/roadmap status,
and reused validation artifact. Run documentation checks and one exact-SHA
whole-phase Opus review without repeating unchanged implementation validation.

## Evidence Ledger

Evidence is added after each merge. Review files remain outside the reviewed Git
tree and are keyed by candidate SHA.

### M1 Blocked Handoff

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
- Exact next action: restore a genuinely distinct release reviewer, update the
  PR base if repository governance changes, and reuse unchanged M1 evidence.
  Validate only the affected distribution boundary unless the user explicitly
  authorizes a second full gate. Do not start M2 before M1 is merged and
  recorded.

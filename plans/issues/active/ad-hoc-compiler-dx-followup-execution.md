# Compiler DX follow-up execution plan

Status: planned; implementation not started. This is the canonical scope for the
user-authorized follow-up work, separate from the completed Phase DX.
Planning baseline: `f9c0d303104fca8181e4624f49d0433779b0e964` on
`origin/main`, verified 2026-09-20. Remote checkout was clean on
`codex/dx16-record`, with HEAD equal to that baseline.

## Authority and boundaries

Use [phase-closure-loop](../../../.cursor/skills/phase-closure-loop/SKILL.md)
and [talk-to-claude-opus](../../../.cursor/skills/talk-to-claude-opus/SKILL.md).
The user authorizes sequential subagent implementation, targeted validation,
scoped Opus review and merge. The orchestrator coordinates; it does not
implement, test or review. Only one agent owns a live worktree/index/branch at
a time. Stop each item after its merged record or an external blocker; hand the
next item to a new bounded session. No implementation starts in this planning PR.

Implementation home is `/home/yaser5/projects/sifr/compiler-dx-orchestration`
via `tailscale ssh yaser5@yaser.tailaa73b4.ts.net`. The inherited Mac checkout
is stale and has unrelated untracked IntoIterator work; leave it untouched
until the final handoff item inspects actual state. Do not infer installation
state from the archived phase's package receipt.

The user's prospective follow-up validation policy is cheap checks first,
fail fast, named focused checks and input-bound evidence reuse. No per-item
create-PR/full merge gate, repeated monolithic gate or whole-phase Opus review
is required. The completed DX.15 gate remains historical evidence, not a
passing result for future changed code. The final item reconciles affected
contracts and uncovered risk explicitly; do not silently waive missing
coverage or rerun the entire old phase. No release, publication, Marketplace,
channel, account or site mutation is authorized.

## Source findings and dispositions

Link history rather than copying it:

- [DX.13](ad-hoc-dx13-project-review-followups.md): F2/F3/F4 housekeeping/history
  belong to DXF.2–3; F6 prune semantics to DXF.3/6; F7 is a guardrail whenever
  that owner is touched. F1's old qualification request is superseded by the
  completed phase gate; it is not a known current failure. F5 remains separate
  unless DXF.4 establishes a production ownership problem.
- [DX.14](ad-hoc-dx14-interface-review-followups.md): F5 package limitation is
  DXF.1; F1/F2 bounded observations and lookup are DXF.2. Preserve F10's
  transitive proof obligation in DXF.1. F4 was a suspected lint shape, not a
  measured remaining failure. F3/F6–9 are not independent authorized expansion;
  change them only if necessary to satisfy a selected item's demonstrated defect.
- [Trace](ad-hoc-dx-trace-artifact-followups.md): size-boundary finalization and
  private directory creation are DXF.5. Optional command-label/event expansion
  stays deferred. The original missing trace surface is already fixed in #3871.
- [DX.16](ad-hoc-dx16-review-followups.md): embedding constructors, cache CLI,
  moved-workspace coverage and actual broken links are DXF.4/6. Previously
  corrected command/profile/status prose is superseded, not reopened.
- [DX.11](ad-hoc-dx11-editor-review-followups.md): push-diagnostic scheduling,
  malformed-buffer UX, refresh override APIs and rich editor/memory work remain
  deferred. No demonstrated need makes them dependencies here.
- [Archived phase](../archive/ad-hoc-compiler-dx-and-toolchain-reuse.md#current-handoff--dx16-2026-09-20)
  and [architecture](../../../internal_docs/compiler_dx_architecture.md) retain
  original performance limits, failures, skips, supported-host split and receipts.
  No new HIR/editor restoration, broad interface-proof language expansion,
  fallback path or backward compatibility layer is in scope.

## Order and dependencies

| Item | Scope | Depends on | Initial status |
| --- | --- | --- | --- |
| DXF.1 | Real package-project importer reuse and measurement | planning PR merged | next ready |
| DXF.2 | Bounded record retention, observations and lookup | DXF.1 | queued |
| DXF.3 | Owner-safe abandoned/orphan storage reclamation | DXF.2 | queued |
| DXF.4 | Production embedding identity audit and necessary fixes | DXF.3 (execution order) | queued |
| DXF.5 | Trace boundary truncation and private directory | DXF.4 (execution order) | queued |
| DXF.6 | Cache CLI/moved-workspace gaps and five documentation links | DXF.3, DXF.5; after DXF.4 | queued |
| DXF.7 | Final affected-contract evidence reconciliation | DXF.1–6 | queued |
| DXF.8 | Authoritative user binary/local checkout handoff | DXF.7 | queued |

DXF.4 and DXF.5 do not technically depend on package reuse; the edges express
the approved priority and single-owner execution order. If an item needs a
new semantic mechanism beyond its scope, record that need and stop rather
than moving speculative work into this plan.

## Validation setup and exact selection convention

These are execution instructions, not results. No compiler tests or gates ran
to prepare this plan. Existing names below were checked in source; names marked
**proposed** must be implemented and registered before they can count as evidence.

Use Rust 1.98.1 and the session-owned existing `target`, ordinary default
features and the normal test profile for these focused Rust checks. Do not
introduce `--all-features`, a new target directory or a whole-workspace test
build. Inspect existing receipts/configuration before choosing a different
feature/profile family. Preparation must match execution.

```bash
cd /home/yaser5/projects/sifr/compiler-dx-orchestration
export RUSTUP_TOOLCHAIN=1.98.1
export CARGO_TARGET_DIR="$PWD/target"
export CARGO_BUILD_JOBS=2
export INSTA_UPDATE=no
git status --short --branch
git diff --check
df -h .
du -sh target
```

Before expensive work, reserve space for the actual selected operation and
inspect existing artifacts; clean only inactive obsolete session-owned artifacts
when required. Do not clean a shared or other worktree target. If insufficient
space remains, record the blocker and stop.

For each changed code item: `cargo fmt --all --check`,
`python3 scripts/check_file_size_guardrails.py` and
`python3 scripts/check_hir_maintainability_guardrails.py` run before expensive
acceptance. Run relevant Python syntax/JSON checks before invoking a harness.
The source guard must pass before closure. Split touched oversized owners by
responsibility, never by arbitrary line ranges.

Each exact Rust selection uses:
`cargo test --locked -p PACKAGE TARGET FILTER -- --exact --nocapture`.
First use the same command with `-- --exact --list` and require exactly one
matching test (zero matches is failure, never a pass). Module filters explicitly
listed below run that whole small named module and must report its test count.
Stop on the first failed prerequisite; diagnose before running its dependents.

For acceptance, bind `DXF_BINARY` to the actual compiler artifact from its
receipt, `DXF_RECEIPT` to that receipt, and `DXF_EVIDENCE` to a new external
candidate-keyed directory under `/home/yaser5/projects/sifr/dxf-evidence`.
Record literal expanded paths and hashes in the evidence index. Reuse existing
preparation via `verification/areas/performance/compiler_lanes.py` (its existing
API/CLI), not an invented package builder. Inspect its current interface before
invocation. Contributor binaries establish functionality; only a prepared
optimized installed artifact establishes product performance.

## DXF.1 — package importer reuse

Owner: frontend semantic dependencies and driver project/package context.
Expand existing proven interface-stable body reuse to ordinary resolved pure
package projects from their actual package cwd. Keep the current narrow proof
class initially; do not promise generic/default/effectful body reuse. Retain
package graph, lock, trust, target, compiler, metadata, source/resolver and live
external-context authority. Recheck edited code before restoring importers and
prove chained restored records remain valid. Native output must reflect edits.

Acceptance: a representative package with multiple modules and a local package
dependency restores unchanged eligible importer checks after a supported helper
body edit. Baseline/fresh checks must have identical diagnostics and exit codes.
Defaults/constants/signature/errors/dependency/manifest/lock/trust/resolver
changes invalidate appropriately; Python/SQL/native live contexts remain
conservative. Package visibility and source ownership must stay authoritative.

Exact existing regressions:
```bash
cargo test --locked -p sifr_driver --lib project_cache::dx14_tests:: -- --nocapture
cargo test --locked -p sifr_driver --lib project_cache::tests::dx13_resolved_package_context_and_live_external_inventory -- --exact --nocapture
python3 verification/areas/developer_tooling/dx14_interface_acceptance.py --binary "$DXF_BINARY" --output "$DXF_EVIDENCE/legacy-interface"
python3 verification/areas/developer_tooling/dx13_project_acceptance.py --binary "$DXF_BINARY" --output "$DXF_EVIDENCE/project" --compiler-profile release --source-root "$PWD"
```

**Proposed** driver test module `project_cache::package_reuse_tests`:
`package_cwd_body_edit_restores_importer`,
`package_authority_changes_invalidate`,
`package_chained_restore_matches_fresh`. Run each with the exact convention.
**Proposed** harness:
`python3 verification/areas/developer_tooling/dxf_package_reuse_acceptance.py --binary "$DXF_BINARY" --output "$DXF_EVIDENCE/package-reuse" --samples 21`.

The harness must cover actual package cwd, a multi-module application and local
dependency, deterministic independent fresh references, module restored/computed
counts, chained edits and native output 1 then 2. Collect 21 paired prepared
samples per representative edit scenario (one predeclared warmup excluded),
including wall time, cache validation/proof work and RSS when available.
Separate cold preparation from warm measurements and baseline versus candidate.
Report median/p95 and variability without claiming a speedup unless measured;
reuse effectiveness (avoided importer work) and correctness are required even
when end-to-end improvement is below noise. If normal package context forces an
additional proof mechanism, rescope before implementing it.

## DXF.2 — bounded history and lookup

Owner: driver project-cache storage/interface lookup with frontend observation
capture only where required. Replace write-unavailable behavior at 4096 records
with bounded retention. Keep latest generation integrity, useful recent A/B/A
retention, reader leases, deterministic eviction and optional-cache semantics.
Use cheap candidate filtering before proof parsing. Bound redundant observations
without losing ordered absence/resolver facts. Do not promise unlimited history.

Acceptance: more than 4096 distinct records/edits continue publishing within a
documented cap; newest and selected recent A/B/A results remain useful. Evicted
history safely misses. Live readers survive publication/prune contention and
writer death. Measure record count, bytes, candidate/proof counts, observation
count and lookup latency at 1, 128, 1024 and 4097 history inputs. Expensive proof
work must not grow linearly with irrelevant history; report retained-cap lookup
cost honestly rather than promising constant total storage I/O.

Exact existing regressions:
```bash
cargo test --locked -p sifr_driver --lib project_cache::tests:: -- --nocapture
cargo test --locked -p sifr_driver --lib project_cache::dx14_tests:: -- --nocapture
```
**Proposed** module `project_cache::history_tests`, each exact:
`history_over_4096_keeps_publishing_and_recent_aba`,
`eviction_preserves_active_reader_and_latest`,
`candidate_filter_bounds_proofs_without_false_hits`,
`observation_compaction_preserves_ordered_absence`.
**Proposed** harness:
`python3 verification/areas/developer_tooling/dxf_history_acceptance.py --binary "$DXF_BINARY" --output "$DXF_EVIDENCE/history" --history-sizes 1,128,1024,4097`.
The storage stress may synthesize canonical records to avoid thousands of CLI
startups; retain a smaller real-edit differential sequence to connect it to
actual semantics. No raised cap alone qualifies as the fix.

## DXF.3 — abandoned and orphan storage

Owner: project storage/prune API. Reclaim abandoned staging/pointer scratch and
deleted-workspace namespaces only with explicit ownership, inactivity and
pressure proof. Do not unlink live lock inodes or cross namespaces belonging to
another worktree. Preserve active readers/writers and newest live generation.
Define accessible explicit orphan cleanup policy without requiring a deleted
workspace to exist; do not add unconditional startup cleanup.

Acceptance includes live contention, killed writer, deleted workspace,
ownership ambiguity (must preserve), replacement/symlink paths and idempotent
cleanup. Clarify reserve default/no-op and examined/eligible/deleted counters
if this API changes; DXF.6 locks the CLI contract.

Existing: run exact `project_cache::tests::dx13_c09_concurrent_process_reader_and_gc`,
`project_cache::tests::process_death_and_new_process_restore` and
`project_cache::tests::dx13_c08_c09_inherited_payloads_reader_gc`
with `-p sifr_driver --lib`.
**Proposed** module `project_cache::housekeeping_tests`, each exact:
`abandoned_stages_reclaimed_live_locks_preserved`,
`orphan_prune_requires_owner_and_inactivity`,
`pressure_prune_counts_are_truthful`.

## DXF.4 — embedding identity audit

Owner: public analysis/driver/LSP constructors. Inventory every non-test
`CompilerContext::for_test*` call and trace actual production callers.
Initial search found `LspAnalysisWorkspace::default`, analysis lint/Python
helpers and stdlib bootstrap helper names; cfg guards and production reachability
must be established before calling these bugs. Compiled test identities are
dependency-bound, not inherently constant/unsafe merely because of their name.
Shipped CLI already supplies the embedded product identity.

Fix only demonstrated production identity substitution by carrying the caller's
explicit context through the existing owner. Preserve legitimate bare-test
identity composition. Do not invent a synthetic product identity, silent
fallback, new compatibility API or unrelated editor feature.

Existing exact checks, `-p sifr_driver --lib`:
`compiler_context::tests::dx_identity_contexts_do_not_share_incompatible_stdlib_owners`,
`compiler_context::tests::dx_identity_bare_test_uses_compiled_dependency_configuration`.
Their module qualification was verified during planning; still use the exact
list guard against later renames before execution.
**Proposed**, only for production boundaries actually changed:
`cargo test --locked -p sifr_analysis --lib dxf_embedding_tests::caller_context_reaches_production_queries -- --exact --nocapture`;
`cargo test --locked -p sifr_lsp --lib dxf_embedding_tests::production_workspace_preserves_context -- --exact --nocapture`.
Tests must distinguish two real supplied identities and prove no test-identity
substitution, while explicit test constructors retain their old test ownership.
If all candidates are test-only/unreachable, close with the audited call graph
and source/cfg evidence; no gratuitous code or compiler gate is required.

## DXF.5 — trace boundary behavior

Owner: CLI trace sink. Account for final timing/counter serialization before
enforcing the 32768-byte limit. Drop bounded optional events/details with truthful
truncation accounting, preserving valid versioned JSON, outcome and redaction.
Real I/O failures must remain truthful errors. Create owned trace directories
privately on Unix independent of umask; preserve destination-collision behavior
and avoid chmod of unrelated existing directories. File mode remains 0600.

Exact existing:
```bash
cargo test --locked -p sifr --bin sifr trace_artifacts::tests::trace_dir_redaction_and_size_bound -- --exact --nocapture
cargo test --locked -p sifr --bin sifr eager_cli_contract_tests::trace_dir_cli_contract -- --exact --nocapture
python3 verification/areas/developer_tooling/trace_dir_acceptance.py --receipt "$DXF_RECEIPT" --evidence "$DXF_EVIDENCE/trace"
```
**Proposed** exact `trace_artifacts::tests` tests under the same Cargo target:
`final_timing_width_at_byte_boundary_truncates_truthfully`,
`trace_directory_is_private_under_permissive_umask`.
Cover just below/at/above the cap, widest serialized counters/timing,
all-events-dropped and real finalization failure. Preserve the existing trace
acceptance's artifact/exit/redaction/overhead contracts. Unix mode claims require
Unix execution; do not report them as tested on Windows.

## DXF.6 — focused CLI coverage and documentation

Owner: CLI contract tests, project-cache tests and documentation.
Add help/argument/global placement/JSON contract checks for actual cache
subcommands, including reserve/no-op/deletion counts and malformed arguments.
Use isolated caches; do not run tests against user cache storage.
Add moved-workspace miss coverage if not already supplied by DXF.1–3. A
byte-identical project at a new canonical root must miss and compute correctly;
do not create relocation reuse as a new feature.

Exact existing:
`cargo test --locked -p sifr --bin sifr eager_cli_contract_tests::all_command_schemas_keep_eager_help_errors_groups_defaults_and_global_order -- --exact --nocapture`.
**Proposed**:
`cargo test --locked -p sifr --bin sifr eager_cli_contract_tests::cache_cli_contract -- --exact --nocapture`;
`cargo test --locked -p sifr_driver --lib project_cache::tests::moved_workspace_misses_without_changing_diagnostics -- --exact --nocapture`.
**Proposed** isolated process harness:
`python3 verification/areas/developer_tooling/dxf_cache_cli_acceptance.py --binary "$DXF_BINARY" --output "$DXF_EVIDENCE/cache-cli"`.

Resolve these five audited links to actual current owners: one moved
`python-interop-verification-production.md` link in `plans/phases/index.md`;
the same moved issue plus sysroot declaration cleanup and stdlib compiler
boundary issues in `plans/roadmap.md`; and its missing
`milestone_psp_7_parity_governance_inventory.md` report link. Find the canonical
replacement; if none exists, replace the claim with an honest historical
reference rather than fabricate a report. The architecture `item: T` match is a
regex false positive, not a sixth broken link. These are link repairs, not
roadmap status changes. Update architecture only for actual changed architecture.
Run `python3 verification/areas/documentation/check_structure.py`,
`git diff --check`, source guard and a local-link existence/anchor check of
changed Markdown, excluding code/inline-code false positives.

## DXF.7 — final evidence reconciliation

Build an external candidate-keyed index of each item's base/candidate, merged PR,
expanded command, selected test count, result/log hash, compiler receipt,
features/profile/toolchain/target/lock/fixture/environment identities, review and
limits. Reuse logs only when their relevant source/validation inputs are
unchanged. Documentation-only or unrelated base changes do not invalidate
evidence. Any changed test/harness requires its affected rerun; compiler-byte
changes invalidate binary-specific receipts/timings even when some library
evidence remains reusable. Never relabel a historical failed/incomplete run.

Run only uncovered affected checks from DXF.1–6 on the final candidate; avoid
repeating checks already proven input-equivalent. Reuse existing default and
supported feature targets when their receipts match. No unmeasured speed or
active-memory improvement claim, no claim that the Linux host is a graphical
12 GB Mac desktop.

Assess four-platform exact-package qualification against actual changed
supported contracts (identity/metadata selection, package installation,
platform-specific filesystem behavior). It is required only where those
changes or the planned installation require it, once for the final affected
package set; it is not a gate per tiny fix. Pure documentation does not trigger
it. Record why each supported platform/contract is covered, requires execution,
or remains an honest limitation. Unchanged DX.15 package bytes keep their
historical evidence; they do not qualify newly built binaries by assertion.
If a new uncovered cross-platform dependency blocks installation, record the
owning issue and stop rather than start a release.

## DXF.8 — local checkout and installed binary handoff

Separate final item, not part of remote implementation. Inspect the user's
actual local branch/HEAD/status/untracked files, executable resolution, binary
hash/version/identity, installed sysroot manifest and package receipts. Resolve
the authoritative merged source and exact locally usable candidate package.
Do not assume either old beta or trace-remediation receipt is currently active.

Preserve all user tracked/untracked changes, particularly IntoIterator work.
Only safe fast-forward updates with no overlapping changes are allowed; do not
stash/reset/clean or overwrite user files. If the checkout cannot update safely,
leave it intact and use an isolated checkout for the handoff. Update the local
binary only using an exact validated platform package and the existing atomic
installer path; preserve recoverable prior generation. No publication/release.

Validate the actual selected binary's identity, metadata readiness, one package
check/edit/recheck smoke, cache help/JSON and bounded trace smoke on the local
host using the applicable DXF.1/5/6 assertions. Reuse exact package qualification
where input-equivalent; a local smoke does not imply four-target qualification.
Report paths, hashes, source SHA, checkout state, installation state and any
unperformed platform checks explicitly.

## Per-item review, merge and records

After named acceptance succeeds, open one draft implementation PR. Scoped Opus
gets exact base/candidate, changed paths, this item's criteria and immutable
evidence; read-only, no new requirements and no repeated broad validation.
Only regressions/in-scope omissions block. Apply valid blockers once and rerun
affected checks/review; a second mechanism-level defect requires rescoping.
Use skill retry/timeout rules; an empty/failed review is not approval.

Before merge verify final candidate coverage and relevant base stability.
Publish review evidence externally keyed by SHA, never into the commit it
approves. Merge after passing named checks and scoped approval, then record
candidate/merge/PR/evidence/deferred findings here and cross-link the original
owning issue. Record-only updates require documentation checks, no new Opus.
Planning alone requires documentation checks and no whole-phase review.

## Current handoff

Planning only; all DXF items remain unstarted. Next-ready batch is **DXF.1 only**:
inspect real package context and the existing narrow proof, add the proposed
package differential tests, establish baseline work/timing on representative
package edits, implement the bounded package reuse, then qualify and request
scoped Opus review. No DXF.2 work runs in that session. Preserve the original
phase's qualification and limitations; this plan does not claim any new test,
speedup, package installation or release has occurred.

Planning validation (2026-09-20): documentation structure, all nine local
links/anchors, eight-item structure, file-size guard (4098 files) and diff
whitespace checks pass. Test selectors were inspected in source only. No Cargo
tests/builds, compiler gates, implementation or Opus review ran for this plan.

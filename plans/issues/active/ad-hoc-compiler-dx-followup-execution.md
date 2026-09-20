# Compiler DX follow-up execution plan

Status: in progress; DXF.1–3 merged and recorded, DXF.4 next ready. This is the canonical scope for the
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
| DXF.1 | Real package-project importer reuse and measurement | planning PR merged | merged #3875 |
| DXF.2 | Bounded record retention, observations and lookup | DXF.1 | merged #3877 |
| DXF.3 | Owner-safe abandoned/orphan storage reclamation | DXF.2 | merged #3879 |
| DXF.4 | Production embedding identity audit and necessary fixes | DXF.3 (execution order) | next ready |
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

## DXF.1 merged record — 2026-09-20

DXF.1 is closed by [PR #3875](https://github.com/sifr-lang/sifr/pull/3875).
Execution base: `f0682eb2a9200cf8912a1eaa14311805bfbfbc36`.
Approved implementation candidate: `c0aaf5ebb68ec371d2dc71b169c54c9867910580`.
Implementation merge: `041f9a2aba05c83668105e2db72d74603a0470f0`.

The ordinary package resolver now supplies canonical module names and rewritten
imports to the existing frontend checking queries. The same narrow body proof
checks edited code before restoring eligible importer diagnostics. Package,
lock-mode, trust, source ownership/resolution, compiler, metadata, target and
live-context authorities remain pinned. Repeated restored-success hops retain
current source bytes and observations. Multiple logical aliases of one physical
source conservatively miss under the existing per-path persistence inventory;
no broader body-erasure class or alternate checker was added.

### Validation and artifact identities

All work ran through Tailscale SSH in
`/home/yaser5/projects/sifr/compiler-dx-orchestration`, using Rust 1.98.1,
default features, the existing session-owned `target`, and two Cargo jobs.
Formatting, file-size and HIR maintainability guards passed before expensive
acceptance; the source guard reported 4100 files under its 900-line policy.
Python syntax and `git diff --check` passed. Pressure inspection found 24 GiB
free with a 185 GiB target before candidate preparation; no cleanup was needed.

The exact three proposed package tests passed, the existing
`project_cache::dx14_tests::` module passed all eight tests, and the exact
`dx13_resolved_package_context_and_live_external_inventory` test passed.
Each selection first listed the exact nonzero expected count: **12 tests total**.
The three named CLI harnesses all passed: new package reuse, legacy DX.14
interface reuse, and existing DX.13 project acceptance (including its specified
source-root override). No per-item full create-PR/merge gate or release
qualification was run under the approved policy.

The candidate evidence directory is
`/home/yaser5/projects/sifr/dxf-evidence/c0aaf5ebb68ec371d2dc71b169c54c9867910580/`.
Its `evidence-index.json` binds literal paths, source bytes, compiler/receipt
identities, all raw reports, logs and the named selections.
SHA-256: `9405a3312e8d97ca74eb5f74d513f24ffd09cff3f0fcce9f00196ad9bf26302c`.

The prepared optimized installed compiler is
`/home/yaser5/projects/sifr/dxf-evidence/c0aaf5ebb68ec371d2dc71b169c54c9867910580/product/installed/bin/sifr`;
SHA-256: `4cc627765fc14c800883e5711d0c7825f40401d42aaf4cd00e8f67c5deaf819e`.
Its receipt is the same directory's `product/receipt.json`;
SHA-256: `93391ac7275ca92bf61911de45e8221b84d4b4a19706aa39f82780877c0b26ca`.
The pre-edit baseline, its installed receipt and original probe remain under
`/home/yaser5/projects/sifr/dxf-evidence/f0682eb2a9200cf8912a1eaa14311805bfbfbc36/`.

### Measured package behavior

The representative actual-package-cwd application has 11 modules, eight importer
layers (160 wrapper functions) and a local package dependency. Each scenario used
21 paired prepared incremental/fresh samples and one predeclared excluded
warmup; pair order alternated. Preparation and native-build work were outside
these warm samples.

| Edit scenario | Baseline incremental median / p95 | Candidate incremental median / p95 |
| --- | --- | --- |
| Application helper body | 1167 / 1208 ms | 282 / 298 ms |
| Local dependency body | 1569 / 2153 ms | 326 / 344 ms |

Candidate independent fresh medians were 1082 and 1083 ms; baseline fresh medians
were about 1077 ms. Baseline computed all 11 module checks. Candidate reports
showed **9–10 restored and 1–2 computed**, depending on the historical record
selected. Chained edits, independent diagnostics/exit codes and the conservative
negative cases passed; actual native stdout was **1 then 2**.

The index contains median/p95/range/coefficient-of-variation summaries for wall
time, RSS, validation/proof and serialization/publication work. Baseline
incremental wall CV was 2.49% / 18.94%; candidate CV was 5.55% / 4.39%.
Median incremental RSS changed from 75324 / 78048 KiB to 49236 / 49216 KiB.
These are descriptive observations on this Linux host, not new general budget
or release claims. History growth remains visible and belongs to DXF.2.
Disabled fresh reports omit module details; no zero-module computation claim
is inferred from those unavailable counters.

Historical failed preflight
`/home/yaser5/projects/sifr/dxf-evidence/focused-preflight.log` remains failed:
a test-fixture `Vec::insert` call was corrected before successful compilation
and focused execution. It is not relabeled as passing. All earlier phase
failures and evidence identities remain unchanged.

### Review, follow-ups and handoff

[Scoped Opus review](https://github.com/sifr-lang/sifr/pull/3875#issuecomment-5749374222)
returned **SATISFIED**, with no blocking findings, for the exact candidate.
External `opus-review.md` SHA-256:
`2a6b2f5679608e40a1d7072bf02e6cf492426e116785a8b889a857053a15f28b`.
Its nonblocking findings and their separate dispositions are recorded in
[DXF.1 review follow-ups](ad-hoc-dxf1-package-reuse-review-followups.md).
This record-only update changes no reviewed implementation or validation inputs
and requires documentation checks only, without another external review.

Blocker: **none**. Stop after this merged record. The next action is a new bounded
session for **DXF.2**; no DXF.2 implementation, release/publication, or user-local
checkout/binary change was performed here. The latter remains reserved for DXF.8.


## DXF.2 merged record — 2026-09-20

Implementation [PR #3877](https://github.com/sifr-lang/sifr/pull/3877) merged as
`34ab684118cd2c2a026165fc9c29118571e361fa`.
Reviewed candidate: `94d35655a065859acdecf1aa3ee33c31a3165305`;
base: `614e7ea23422419aef4a9257a82c7cbc6b11207e`.

Schema-2 manifests retain the newest 128 distinct publications within 64 MiB
of record payloads, evicting oldest records deterministically. Republishing
refreshes publication order. The existing 16 MiB individual-record limit,
complete integrity validation, immutable generations, reader leases and atomic
publication remain in force. This caps latest-generation history, not all
physical predecessors; explicit pressure pruning retains that ownership.
Recent A/B/A restores remain useful; evicted history safely misses.

Lookup checks recent exact candidates first, filters interface candidates
cheaply, then attempts at most one existing frontend proof. Capture keeps each
distinct observation once in first-occurrence order, including absence and
conflicting outcomes; 16384 distinct facts mark capture incomplete and decline
persistence without interrupting normal checking. No semantic proof class,
external-context authority or cleanup mechanism was expanded.

### Validation and evidence

All four proposed history tests passed with exact one-test list guards.
Existing `project_cache::tests` (12), `project_cache::dx14_tests` (8),
`project_cache::package_reuse_tests` (3), and frontend `persistence::tests`
(9) also passed: **36 selected tests total**. Coverage includes live-reader
eviction/prune, subprocess contention/writer death, corrupt storage, conservative
semantic invalidation and ordered absence/overflow. The cheap format, file-size
(4102 files), maintainability, Python syntax, diff and documentation checks passed.

The 4097-publication canonical-record stress uses the actual storage owner
and existing explicit prune, with no thousands of CLI starts. It also verifies
recent A/B/A and an evicted-history miss. The candidate-filter test adds 100
unrelated canonical entrypoints and proves one successful proof attempt, then
a type-error result equal to a fresh reference with at most one proof.
The named history CLI harness passes five real helper edits against independent
fresh checks: published, interface-restored, exact A restoration, equal error,
then interface-restored recovery.

| Historical inputs | Retained records | Retained payload bytes | Exact lookup, µs | Candidates / proofs / observations |
| --- | --- | --- | --- | --- |
| 1 | 1 | 1434 | 422 | 1 / 0 / 1 |
| 128 | 128 | 183844 | 21669 | 1 / 0 / 1 |
| 1024 | 128 | 184112 | 21003 | 1 / 0 / 1 |
| 4097 | 128 | 184320 | 22137 | 1 / 0 / 1 |

These are single descriptive normal-test-profile observations following
publication, including full retained-manifest integrity reads. They are not
controlled-host product performance qualification or a speedup claim.
Retained-cap lookup still pays storage/decoding cost; no constant-total-I/O claim
is made. The stress took 95.27 seconds; its ordinary-suite cost is a recorded
nonblocking follow-up.

External evidence:
`/home/yaser5/projects/sifr/dxf-evidence/94d35655a065859acdecf1aa3ee33c31a3165305/`.
Its `evidence-index.json` SHA-256 is
`478e14a73f1a7496bf07b8ef9393dce67b65d7bb1916014fc832758ead3ca1c4`.
The index binds expanded commands, environment, source/fixture/lock hashes,
selected counts, raw logs, receipt, artifact identity, review and reuse proof.

The canonical contributor preparation selected
`/home/yaser5/projects/sifr/compiler-dx-orchestration/target/debug/sifr`,
SHA-256 `8a5db4634d4eb566fbd75e60094c8252e4b3594133b6a3f2abecba60c08276c1`.
Its external `contributor/receipt.json` SHA-256 is
`4e44b47aef85a181c5405c13529401473e5a0194c29551f54a121df382b0b2ea`.
Rust 1.98.1, ordinary defaults, two Cargo jobs and the existing private target
were used. Free space was 24 GiB before named checks and 21–23 GiB before
contributor preparations; the 8 GiB reserve required no cleanup.

Library evidence remains under
`/home/yaser5/projects/sifr/dxf-evidence/e2a296c9afcf4b3976eebc3ed255470b4687e0a9/`.
The sole later change corrected the CLI harness's missing explicit legacy
source-root fixture (`sifr.toml` plus manifestless invocation cwd); that harness
was rerun successfully. Rust/fixture/Cargo/toolchain inputs and actual compiler
bytes are identical, as verified in `reused-evidence.json`. The original
failed harness log remains failed, not relabeled. All earlier historical
failures remain unchanged. No repeated library stress, per-item monolithic
gate, release qualification or user-local installation ran.

### Review and handoff

[Scoped Opus review](https://github.com/sifr-lang/sifr/pull/3877#issuecomment-5749545936)
returned **SATISFIED**, with no blockers, for the final candidate.
External `opus-review.md` SHA-256:
`00216157dcb1dc384a2a62b196113f63186a078789cf838784f1dc2846bea8e2`.
Five nonblocking findings are preserved in
[DXF.2 review follow-ups](ad-hoc-dxf2-history-review-followups.md).
This record changes no implementation/validation inputs; documentation checks
only apply, without another external review.

Blocker: **none**. Stop after the merged record. The next action is a new bounded
DXF.3 session for owner-safe abandoned/orphan storage reclamation. No DXF.3
implementation or user-local checkout/binary changes were performed here;
the latter remains DXF.8.


## DXF.3 merged record — 2026-09-20

Implementation [#3879](https://github.com/sifr-lang/sifr/pull/3879) merged as
`a6625ff0e2f72d49f31af0e3ad1df84cd331a6d0`. Reviewed and validated candidate:
`0bc95340dfe6a5e4e32e0824a7b62455dae24df7`, based on
`9af5e3b8d5b0d17c37c855f144d3747a937f9466`.

### Implemented boundary

Explicit pressure cleanup now reclaims owned abandoned staging directories,
writer-serialized stage lock files, cache pointer scratch and completely identified
workspace hint scratch. Current bounded history, active readers/writers and
permanent live generation/writer lock inodes remain protected. Immutable namespace
owners bind canonical workspace path, device, inode and directory creation time;
nonempty ownerless namespaces cannot be retroactively claimed.

`cache prune-project` accepts the **original absolute canonical path** of a deleted
workspace. Exclusive namespace and context-lock leases prove inactivity before
orphan context reclamation. Stores and detached readers share the namespace lease.
The namespace owner tombstone and external namespace lock remain stable; replaced
roots, ambiguous ownership, symlinks, unsafe entries and other namespaces remain
protected. There is no startup or global cleanup. Directory creation-time support
is required for persistence; an unsupported filesystem still computes normally,
without a claimed cache result. Recreated-root residue and filesystem qualification
remain documented limitations, not silently waived coverage.

The API/CLI distinguish examined, eligible and deleted entries, with separate
live-generation counters. Zero reserve remains an explicit no-op. Dry-run and
repeated-cleanup counters are tested; orphan candidates count context trees.
[Architecture policy](../../../internal_docs/compiler_dx_architecture.md#73-pressure-based-cleanup)
defines the exact scope and retained tombstones. Detailed CLI contract coverage
remains DXF.6.

### Validation and evidence

All work, compilation and review ran in the sole remote worktree
`/home/yaser5/projects/sifr/compiler-dx-orchestration` through Tailscale SSH, on
Rust **1.98.1**, ordinary default features, the normal test profile, the existing
private `target`, `CARGO_BUILD_JOBS=2` and `INSTA_UPDATE=no`. The host retained
**21 GiB free / 187 GiB target**; the selected incremental build reserve was
6 GiB and no size-based or other cleanup occurred.

Cheap prerequisites passed: `cargo fmt --all --check`, file-size guardrails
(4104 files; 900-line cap), HIR maintainability guardrails and `git diff --check`.
Each exact Rust test listed exactly one selection before execution; all eight
passed on the final implementation:

- `project_cache::housekeeping_tests::abandoned_stages_reclaimed_live_locks_preserved`
- `project_cache::housekeeping_tests::orphan_prune_requires_owner_and_inactivity`
- `project_cache::housekeeping_tests::pressure_prune_counts_are_truthful`
- `project_cache::tests::dx13_c09_concurrent_process_reader_and_gc`
- `project_cache::tests::process_death_and_new_process_restore`
- `project_cache::tests::dx13_c08_c09_inherited_payloads_reader_gc`
- `project_cache::history_tests::eviction_preserves_active_reader_and_latest`
- `eager_cli_contract_tests::all_command_schemas_keep_eager_help_errors_groups_defaults_and_global_order`

The first seven use `-p sifr_driver --lib`; the last uses `-p sifr --bin sifr`.
Every command uses `cargo test --locked`, then `-- --exact --list` and
`-- --exact --nocapture`. The housekeeping tests exercise real child-process
contention/killed writers, a detached reader and a live child reader across
workspace deletion, unrecorded/corrupt owners, replacement directories, dangling
and nested symlinks, unsafe permissions, dry-run parity and idempotence.

External evidence:
`/home/yaser5/projects/sifr/dxf-evidence/0bc95340dfe6a5e4e32e0824a7b62455dae24df7/`.
`evidence-index.json` binds base/candidate, source hashes, test artifacts, exact
selection counts, raw logs, guards and review. Earlier seven passing tests at
`6d5c77b5c089f8e546321e50ec054f31d1986672` remain preserved; the ownerless-namespace
refinement justified rerunning the affected selections. No full per-item gate,
performance benchmark, release or installed-artifact qualification was run or
claimed under the approved fast-feedback policy. No local user checkout or
installation was changed.

### Review, follow-ups and handoff

[Scoped Opus review](https://github.com/sifr-lang/sifr/pull/3879#issuecomment-5749706095)
returned **SATISFIED**, with **no blocking findings** for the final candidate.
External `opus-review.md` SHA-256:
`c0e5c3613cb8de2667ec886640c37d406d684ee72de83e00df93c242716ab2db`.
The five nonblocking observations are preserved in
[DXF.3 review follow-ups](ad-hoc-dxf3-housekeeping-review-followups.md).
The review used the remote
[talk-to-claude-opus skill](../../../.cursor/skills/talk-to-claude-opus/SKILL.md),
read-only, without repeated broad validation. Its completed response was atomically
published; an owned orphan watchdog sleep was terminated after completion to
release the SSH output pipe. No failed or incomplete review was counted as passing.

This record-only update requires documentation checks only; it changes no reviewed
implementation or validation inputs and adds no new review cycle. DX13-F2/F3 and
the DXF2-F1 stale storage comment are resolved; CLI coverage remains its existing
DXF.6 item. Stop this session after the record merges. The exact next action is
a new bounded session for **DXF.4**. No DXF.4 implementation, release/publication,
installation or user-local handoff occurred here.

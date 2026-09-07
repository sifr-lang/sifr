# Ad hoc issue: Schema-first SQL platform review follow-ups

Status: active, non-blocking

Owner: SQL compiler, schema tools, and verification

## Later 12K-B8: SQL spelling compatibility guard (2026-09-06)

[Owner #3723](https://github.com/sifr-lang/sifr/issues/3723) records a new
external qualification blocker, not a repeated coverage-classification failure.
Original12K candidate `56907f59cc7d9f9fedb89434970c074c0247dee9` passed its full
integration review and readiness4, then its sole gate reported PostgreSQL
`types.rs:479`, PostgreSQL `postgresql_regressions.rs:380`, and MySQL
`types.rs:73` as removed public bigint support. These are SQL database type
spellings. All three files, the compatibility checker and retained-contract
registry are byte-identical to assessed main f11e1cd. No SQL semantics or guard
policy was changed or repaired by 12K. The tooling/SQL owners must preserve
real SQL type names and rejection of removed Sifr bigint support when resolving
the boundary; this record does not authorize a broad scan suppression.

Exact unchanged blobs and originating commits are in owned
`target/verification/areas/item12k-delivery-other-blockers.json`, SHA256
`ba655e2869464524d2d831cb6c3311fb933b4683dea33fa3ba24ee4ec504bf80` under
`/private/tmp/sifr-item12k-delivery.4J6JeK/sifr`.
Gate log SHA256 `336f5b1345f2495c7a197d81658f36ddfbcebf9c57b110675c5dc4b5c8f219b3`.
Two other developer-tooling owners are #3722 inventory and #3724 formatter
reference drift. Gate failed after3736.19s; later SQL/full E2E/stdlib/ignored
driver lanes were unreached. No merge, repair, second gate or next-item code.

## Approved Item 12B qualification dependency (2026-09-05)

The user explicitly authorized the Item 12B owner to repair the 23 recorded
SQL package/target classification diagnostics in authoritative coverage metadata.
This is a bounded dependency of PR #3694, not permission to change SQL runtime
or compiler semantics. Preserve actual Cargo target kinds and package roles;
do not weaken readiness, exclude tests, or add accepted debt. The earlier failed
gate remains failed. One replacement exact-SHA merge gate is authorized after
the sole remaining Opus remediation review of both repository candidates.
Required checks: coverage_matrix_readiness.py, its existing negative self-tests,
profile_assignment_matrix.py, verification_taxonomy.py, and the complete canonical
coverage_matrix readiness suite. Item 12B records the full qualification commands.

## Item12B SQL coverage repair qualified (2026-09-05)

The user authorized this bounded classification dependency and one replacement
merge gate. Candidate `a3198ab9f936986b5ca1f9ce3fa73d36ac9ab74d` classifies
the nine missing SQL packages and their real targets, fills omitted integration
tests, corrects PostgreSQL's primary target kind, and gives MySQL/SQLite compiler
crates real ordinary-profile test membership. No SQL/Cargo behavior or checker
requirement changed. The three added negative cases bring self-test coverage to 27.

Standalone and replacement-gate coverage readiness both pass all four variants,
including taxonomy. Both Item12B reviews are SATISFIED. The replacement gate
still failed later on five Python-interop variants; see
`ad-hoc-python-interop-qualification-dependencies.md`. No overall gate pass or
merge is claimed, and later crate-test stages were not reached.
Evidence: `/tmp/sifr-item12b.akguMz/coverage-remediation-results.json` and
`/tmp/sifr-item12b.akguMz/replacement-a319-coverage-results.json`.

Non-blocking reviewer suggestion: confirm whether `sqlite-runtime-probe` is
intended to remain SQLite-only. Its test-fixture classification is accurate;
no other provider defect or implementation requirement is asserted.

## Item 12B merge-gate reproduction (2026-09-05)

Sifr PR [#3694](https://github.com/sifr-lang/sifr/pull/3694) ran its one
merge-profile gate on `6ce83824e0315e5f89383fc666344b99431e1e76`.
The gate exited 1 after 173.63 seconds and reproduced 23 pre-existing SQL
package/target classification diagnostics: nine missing package classifications,
missing SQL/host-tool targets, and stale PostgreSQL `lib` versus `rlib`.
The candidate changes no SQL Cargo packages, target declarations, or coverage
registry inputs. This issue retains ownership of that blocker; no SQL code,
classification, or safety requirement was changed by Item 12B.

The same run also found 428 corpus naming diagnostics in a separate taxonomy
variant. Those newly introduced `contract_result_*` names belong to Item 12B,
not this SQL issue. Neither failing variant was waived. Both implementation
PRs remain unmerged, and the gate was not repeated.

Evidence:
`/tmp/sifr-item12b.akguMz/merge-6ce83824e0315e5f89383fc666344b99431e1e76.log`,
`/tmp/sifr-item12b.akguMz/merge-6ce83824e-coverage-results.json`, and
`/tmp/sifr-item12b.akguMz/merge-6ce83824e-lane-report.json`.

## Objective

Resolve the new mechanism findings from the final schema-first SQL platform
remediation review. These findings do not reopen the completed platform phase.

## Scope

- Make every generated profile annotation resolve through an explicit import or
  alias. Cover datetime, UUID, JSON, and network types.
- Diagnose a missing schema-profile import when a decorator names a configured
  profile. Do not capture unrelated decorators that also end in `.query`.
- Keep explicit user-owned PostgreSQL sequences in live schema evidence. Exclude
  only sequences that PostgreSQL creates as an identity implementation detail.
- Prove reproducible linked native SQL artifacts, not only reproducible Cargo
  check plans.

## Acceptance criteria

- [ ] A generated profile with date, time, timestamp, UUID, JSON, IP, network,
  and MAC fields compiles without an undeclared annotation path.
- [ ] Generated imports and type annotations come from one closed mapping. A
  mutation test rejects an annotation whose import is missing.
- [ ] A configured but unimported profile decorator produces one targeted
  diagnostic with an import correction.
- [ ] An unrelated decorator such as `@cache.query` remains outside SQL
  discovery and does not produce the profile-import diagnostic.
- [ ] PostgreSQL live catalog tests retain explicit `CREATE SEQUENCE` and
  `ALTER SEQUENCE ... OWNED BY` objects.
- [ ] PostgreSQL live catalog tests exclude only implementation-owned identity
  sequences and preserve DDL-versus-introspection parity.
- [ ] Native build qualification links supported SQL artifacts twice from clean,
  locked, offline inputs and compares stable content hashes.
- [ ] Cross-target limitations are explicit. The qualification does not claim
  byte reproducibility for a target that it cannot link locally.
- [ ] Focused compiler, schema-tool, build-qualification, mutation, formatting,
  lint, HIR, and file-size checks pass.
- [ ] One exact-candidate external review and the applicable repository gates
  pass when this issue is selected for implementation.

## Source evidence

- Final implementation: [PR #3645](https://github.com/sifr-lang/sifr/pull/3645).
- The Milestone 18 remediation review returned `SATISFIED` for the four original
  blockers on `c0c6ae255fc605fc58a24d93a15d5a08b8126121` and reported these four
  findings as new follow-up work.
- The archived phase record contains the complete validation, review, gate, and
  merge evidence.


## Coverage registry blocker observed during naming cleanup (2026-09-05)

### Capture-demand delivery recurrence (2026-09-07)

Tracked by [#3749](https://github.com/sifr-lang/sifr/issues/3749), recorded only,
not started. 12K-B15 / #3748, draft PR #3746, exact candidate
`4a0a03f430f1ad87b080a6172bc46209082e955a` (main base
`06ea86334b72f49f5aab250a64498ee955ec9331`) ran its single merge-profile gate.
It naturally FAILEDexit1 after1170.64s at coverage readiness:9missing SQL
package classifications,13missing targets,1stale PostgreSQLlib versus rlib.
The other3readiness variants passed, as did production92graph setup, all13
guards including264companion freshness, and10RustInterop variants.
Remaining18areas/2toolchain steps were UNREACHED. SQL packages/manifests and
coverage inputs are unchanged from main; no B15 baseline replay was executed.

This existing owner must reconcile
`verification/areas/coverage_matrix/data/cargo_metadata_classification.json`
with the actual SQL Cargo graph. No classification or assertion was changed
by B15. No new mechanism implementation, second gate, or merge was attempted.
The approved compiler/companion delta is preserved on
`codex/implicit-format-capture-demand`, independent clone
`/private/tmp/sifr-companion.KIogHV/sifr`.
Full diagnostics and bounded later-owner scope are in #3749; raw gate log
`/private/tmp/sifr-companion.KIogHV/evidence/merge.4a0a03f430f1ad87b080a6172bc46209082e955a.log`
SHA256 `d6d70bfacf3d1a8078348a393ea27e222bbee7c93880ea9894f7c8be4a85f204`;
supervisor receipt SHA256
`28b86d3734b75cc2fe7b8615095fe4dd07fbc9f6b67fd96761404448454e646c`.
Coverage result SHA256
`963eb6d42bad2867db29a47c1fed9d0428244b1b1e8cc3c5accbd2e1b69d167b`.
These are failed-gate evidence, not full qualification. No cleanup or resource
termination occurred; minimum free capacity39,878,000,640bytes.

### SQL registry delivery registration (2026-09-07)

**12K-B17 / #3750 and B16 / #3749 are delivered** by
[PR #3751](https://github.com/sifr-lang/sifr/pull/3751), both issues CLOSED.
Actual-main base `06ea86334b72f49f5aab250a64498ee955ec9331`, candidate/review
`e3862e5895fb2ace34571295e267f4951917752f`, verified normal-main merge
`0b97b3a3f1dd3f93bc724f75e1e40240f15ff942`.
The complete registry covers37 packages/117 targets; both compiler packages
now have full-mode blocking executed `crate_test_membership.suites` entries.
All seven named checks passed once: profiles/plan, MySQL11+1ignored,
SQLite12+0ignored, readiness4/4 (profile19/negative26/taxonomy), file-size3759,
diff. No ignored live test is counted as executed. Initial combined Opus
SATISFIED/no blockers, providers1/retries0/remediation0; zero broad gates.
The phase document's top terminal indexes all raw paths/hashes and the
separate normally pushed post-merge record. No further review/gate is required.

Later nonblocking SQL verification audit [#3752](https://github.com/sifr-lang/sifr/issues/3752)
owns establishing actual runtime/tooling merge execution before deciding
whether the existing compiler-only membership policy should change.
Budget headroom belongs to the existing B15/#3748 qualification owner;
the cold crate timings are not controlled-host merge performance evidence.
Neither follow-up is started here. Other SQL acceptance criteria above remain
open and all historical blocked handoffs below retain their original evidence.

Terminal: B16 #3749 is **blocked, not merged**, preserved in draft
[#3751](https://github.com/sifr-lang/sifr/pull/3751), candidate
`a7ea5b8106068ee9394d82dd46e8e95bc1263a36`, main base
`06ea86334b72f49f5aab250a64498ee955ec9331`. All original23 registry diagnostics
are resolved against all37 packages/117 targets, but one exact-SHA readiness
run is3/4PASS: strict registry now identifies missing merge crate-test membership
for `sifr_sql_mysql` and `sifr_sql_sqlite`. Profile19/negative26/taxonomy PASS;
file-size3759 and diff PASS. No review, gate or merge was attempted.

Later owner **12K-B17 / [#3750](https://github.com/sifr-lang/sifr/issues/3750)**,
SQL verification / compiler-verification, owns the two entries in
`verification/profiles/merge.json:44`. Preserve compiler classifications and
require blocking full-mode executed memberships with exact package commands.
The later issue records concrete acceptance commands and distinguishes profile
metadata/verification helpers from workflow files under the zero-gate rule.
No later implementation started. Full terminal/evidence hashes are at the top
of `ad-hoc-emitted-rust-excellence.md`; raw evidence remains under
`/private/tmp/sifr-sql-registry.xaAOLM/evidence/`. B14/B15/original12K counters
and draft #3746 remain unchanged. This worker stops at the preserved handoff.

Item **12K-B16**, issue [#3749](https://github.com/sifr-lang/sifr/issues/3749),
owns the bounded nine-package/thirteen-target registry reconciliation and stale
PostgreSQL `lib` to `rlib` replacement. Its canonical scope, isolated main-based
clone, exact base and three named checks are registered at the top of
`ad-hoc-emitted-rust-excellence.md`. Complete metadata inventory is prepared
from actual main; previous stack-only qualification is not reused as a pass.
Two SQL negative regressions join the existing readiness self-tests. All prior
coverage policy and checks remain intact; no compiler, lockfile or gate changes.
Live-server targets are explicit `test-fixture` inputs to the existing live SQL
adapters, rather than claims of execution in the offline nightly profile.

The B15 recurrence on `4a0a03f430f1ad87b080a6172bc46209082e955a` failed its one
merge gate with the same 23 registry diagnostics (readiness 3/4). Its raw
terminal remains at `/private/tmp/sifr-companion.KIogHV/evidence/terminal.json`,
SHA256 `657424e6ab98d7adaf150062f34f91abbba350d806cad23e3eacb73c9f6ffae7`.
B14/B15 PR #3746 remains unmerged. Their historical review/gate counts are
unchanged by this separate registry delivery. No successor work starts here.
### Historical observations

The repository naming cleanup ran `scripts/run_all_tests.sh` once. The gate
failed in coverage-matrix readiness with nine unclassified SQL packages,
unclassified SQL/host-tool test targets, an unclassified PostgreSQL `rlib`,
and a stale PostgreSQL `lib` classification. The naming cleanup changes no
SQL Cargo packages, targets, or coverage classifications.

Examples include `sifr_sql_mysql`, `sifr_sql_mysql_runtime`,
`sifr_sql_postgresql_runtime`, `sifr_sql_sqlite`, `sifr_sql_tool`,
`test:host_tool_cli`, `test:sql_migrations`, and `test:runtime_policies`.
The complete failure list is in `target/naming-cleanup/merge-gate.log` and
`target/verification/areas/coverage-matrix-merge-results.json`.

This issue owns reconciling the coverage registry with the existing SQL
package and target graph. No classification or coverage requirement was
weakened during naming cleanup. The merge gate was not repeated.

The subsequent demo directory follow-up ran its own final merge gate once on
2026-09-05 and reproduced the same SQL coverage classifications failure.
All 264 demo emitted companions passed freshness, along with the file-size,
HIR, Rust interop, and naming checks. No SQL classifications changed.
Evidence: `target/demo-layout/merge-gate.log`.

Item12H's one exact-SHA merge-profile gate reproduced this existing blocker on
2026-09-06, after its bounded field-identity remediation was approved by Opus.
Candidate: `9b52ac20094608c8a31f252db99e49ef7c963384`,
[draft PR #3697](https://github.com/sifr-lang/sifr/pull/3697). The gate failed at
`coverage_matrix:readiness/coverage_matrix_readiness` with nine unclassified SQL
packages, 13 unclassified targets, and one stale PostgreSQL `lib` classification.
The other three coverage variants passed. All 264 demo companions, reached
guardrails and Rust interop checks passed before the failure; later gate stages
were not reached. No SQL source, Cargo target, coverage classification, or skip
policy was changed by12H, and no second gate was run.

[Exact-SHA evidence and disposition](https://github.com/sifr-lang/sifr/pull/3697#issuecomment-5555393502).
Logs and copied reports: `/tmp/sifr-item12h.afJDbk/merge-9b52ac20094608c8a31f252db99e49ef7c963384.log`
and `.json`, plus `coverage-matrix-9b52ac20094608c8a31f252db99e49ef7c963384.json`
in that directory. The candidate remains unmerged. Reconciling and qualifying
these existing SQL coverage classifications is a concrete dependency for
Item12K integration; this receipt does not authorize12H to implement the repair
or merge an unqualified candidate.

The abbreviated-label cleanup also ran its final merge gate once on 2026-09-05.
It reproduced the SQL package/target classification failures above. Demo
freshness, Rust interop matrix checks, naming checks, HIR, and file-size checks
passed. No SQL classification or dependency changed. Evidence:
`target/abbreviation-cleanup/merge-gate.log`.

The naming-review remediation ran its final merge gate once on 2026-09-05.
It reproduced the same SQL package/target classification failures after all
264 demo freshness checks and reached guardrails passed. No SQL code or
classification changed. Evidence: `target/review-remediation/merge-gate.log`.

PR [#3692](https://github.com/sifr-lang/sifr/pull/3692) repeated the create-PR
gate before opening the PR. The same coverage classifications blocked it
after all 264 demo freshness checks and reached guardrails passed. No SQL
changes were made. Log: `target/pr-cleanup/create-pr.log`.

The descriptive-demo-variable follow-up ran its final merge gate on
2026-09-05. It reproduced the same SQL coverage-classification failures
after all 264 demo companions passed freshness. No SQL source or coverage
classification changed. Evidence: `target/demo-name-followup/merge-gate.log`.

Item12I's sole merge-profile gate reproduced this blocker on 2026-09-06 at
Opus-approved exact candidate `f6e8afd964bb214a44c50271dcb2014ee8e828b4`,
[draft PR #3698](https://github.com/sifr-lang/sifr/pull/3698). It failed after
184.65s with nine unclassified SQL packages, 13 unclassified targets, and one
stale PostgreSQL library classification. Generated-companion freshness and
all preceding guards passed; Rust interop passed 10/10 variants, and the other
three coverage variants passed. No SQL implementation/classification changed,
no gate was repeated, and the PR remains unmerged. This owner repair must be
reconciled with the preserved12B changes in12K integrated qualification.
Exact evidence is outside the worker tree under `/private/tmp/sifr-item12i.0l85Lu/`:
`merge-f6e8afd964bb214a44c50271dcb2014ee8e828b4.log` and `.json`, and
`coverage-matrix-f6e8afd964bb214a44c50271dcb2014ee8e828b4.json`.

## B4 independent main readiness receipt (2026-09-06)

Item 12K-B4 / [#3712](https://github.com/sifr-lang/sifr/issues/3712), preserved
in [draft #3714](https://github.com/sifr-lang/sifr/pull/3714), independently
repairs taxonomy fixture path isolation on main
`f11e1cd7eef16a02063555bccc9fd8e19287833b`. Candidate
`eaa4a063b69ee2132bef55514361062e85db3548` passes direct taxonomy and eight
focused path regressions. Its named four-check coverage readiness suite passes
taxonomy, profile assignment (19 rows), and all 24 negative self-tests present
on main, but fails coverage readiness with the same 23 SQL classifications
already owned here. These are nine missing packages, 13 missing targets, and
the stale PostgreSQL `lib` classification replacing the current `rlib` target.

The candidate changes no Cargo/compiler input, coverage checker, classification
registry, or readiness self-test. The base/candidate classification blob is
`c835f5e32761a99db1b0d5aaeafb1053c997ad6e`; the readiness self-test blob is
`71240aa421cb9cfe4d754e1c139ad05f5616e2f7` on both. The previously approved
23-classification repair and 27-case self-test belong to retained Item 12B
[#3694](https://github.com/sifr-lang/sifr/pull/3694) and pending integration
[#3713](https://github.com/sifr-lang/sifr/pull/3713), not this independent main
baseline. B4 does not import that stack or modify this owner's implementation.

Evidence root: `/private/tmp/sifr-item12k-b4.3wWQdN/`.
Canonical `sifr/target/verification/areas/b4-readiness.json` SHA256:
`b515bd1058d1464d374dee98152a6daa3188e09d8f2a703f3a4354e26117780a`.
Full 23-diagnostic log `readiness-eaa4a063b.log` SHA256:
`976734d699050a4c54e45b35dc4f6d9f1e9201c0a25eadfcb948bc4c7066fda4`.
One completed suite invocation followed approved resolution of an initial
network-only runner setup failure. No Sifr gate or Opus review ran.

B4 stops under its explicit external-blocker rule with #3712 open. The parent
must adjudicate the independent B4 dependency boundary or arrange separately
owned SQL prerequisite delivery before B4 proceeds to review/merge. This is
an owner receipt only, not authorization to implement SQL or continue 12K.

### B4 integrated qualification authority (2026-09-06)

The parent has now adjudicated the prerequisite context: the existing reviewed
12B SQL corrections and 27-case negative suite are already ancestral to 12K
record `5de50ecafc84ed1fa724e7384ad85689a6925dfb`. B4 may qualify its unchanged
checker/regressions on that preserved stack. This supersedes only the earlier
dependency stop, not the recorded failed-main result. No new SQL correction is
implemented. The owned B4 phase registration records exact inputs and paths.

B4 qualified and received one SATISFIED exact-SHA Opus review on integrated
candidate `5c711f2d6cb90265b32e04e8b9f6b6e3570855c1`,
[stacked draft #3715](https://github.com/sifr-lang/sifr/pull/3715).
The full readiness suite passed4/4, including all27 actual negative cases and
strict SQL package/target classifications. Registry and negative-self-test
blobs are unchanged from retained12K base. No SQL mechanism was implemented.
[Evidence](https://github.com/sifr-lang/sifr/pull/3715#issuecomment-5559859380)
and [review](https://github.com/sifr-lang/sifr/pull/3715#issuecomment-5559872361)
remain outside the reviewed tree. The prior standalone-main failure is not
reclassified. Independent merge cannot retain exact qualification inputs;
integration delivery stays with the fresh12K owner, with no new SQL item,
no B4 gate, and no whole-stack merge in this worker.

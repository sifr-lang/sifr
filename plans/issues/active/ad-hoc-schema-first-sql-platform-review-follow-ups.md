# Ad hoc issue: Schema-first SQL platform review follow-ups

Status: active, non-blocking

Owner: SQL compiler, schema tools, and verification

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

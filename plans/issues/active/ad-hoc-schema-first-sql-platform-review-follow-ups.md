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

### SQL registry delivery registration (2026-09-07)

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

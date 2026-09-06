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

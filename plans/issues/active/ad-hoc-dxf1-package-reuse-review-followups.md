# DXF.1 package reuse review follow-ups

Status: nonblocking follow-ups recorded separately from merged DXF.1.

Source: [scoped Opus review of PR #3875](https://github.com/sifr-lang/sifr/pull/3875#issuecomment-5749374222).
Candidate `c0aaf5ebb68ec371d2dc71b169c54c9867910580`; implementation merge
`041f9a2aba05c83668105e2db72d74603a0470f0`.
Verdict: **SATISFIED**, no blocking findings.
The [canonical record](ad-hoc-compiler-dx-followup-execution.md#dxf1-merged-record--2026-09-20)
binds source, tests, prepared artifacts and exact external review evidence.

| ID | Owner / disposition | Observation and bounded next action |
| --- | --- | --- |
| DXF1-F1 | Frontend graph performance; separate deferred work | `rebuild_edges` clones cached parsed ASTs before deriving signatures and edges. Review found no stale-AST correctness path. A later bounded change can borrow the suite or compute signature/edges before mutating state to avoid those clones. This is not silently added to DXF.2. |
| DXF1-F2 | DXF.7 affected-contract evidence reconciliation | Source ownership and resolver ambiguity have explicit negatives; private-import edits currently rely on the existing package resolver rejecting `PrivateAccess` before reuse. Record that coverage gap and consider a focused private-import negative when reconciling uncovered risk. No visibility behavior was waived or changed. |
| DXF1-F3 | DXF.7 package checking boundary reconciliation | A restored result is diagnostics-only, whereas ordinary package computation also executes stdlib-for-codegen, codegen and interop metadata work. Reuse is bounded by the original producer's default-interop/no-Python attestation, pure-manifest inventory and unchanged zero-argument, undecorated, nongeneric primitive-return literal/name/binop/unaryop erasure class. That class cannot introduce new interop/probe demand. Any future expansion must qualify those later owners explicitly; no new checker/proof class is authorized here. |
| DXF1-F4 | Driver test infrastructure; existing requirement documented | The new fixture invokes offline Cargo metadata and needs Cargo on PATH plus writable temporary storage. The owning driver suite already uses Cargo metadata in `tests/attached_api_codegen.rs` and other package tests; this is not a newly discovered suite-wide prerequisite or current host blocker. Keep that requirement visible for any future isolated test runner. |
| DXF1-F5 | GitHub orchestration; resolved by approved relay | Remote `gh` is absent by design. The authenticated local GitHub relay independently verified draft PR/base/head and clean mergeability, then merged the exact approved head. The review itself was correctly bound to remote Git/source/evidence bytes. No tool installation or compiler work follows from this observation. |
| DXF1-F6 | Existing DXF.2 ownership | Validation cost rises along the measured history (helper approximately 7–21 ms; dependency 23–37 ms). Raw costs and variability are retained. Bounded observations, lookup and retention remain DXF.2, together with existing DX14-F1/F2 and DX13-F4 records; no history change was absorbed into DXF.1. |

The full review remains outside Git:
`/home/yaser5/projects/sifr/dxf-evidence/c0aaf5ebb68ec371d2dc71b169c54c9867910580/opus-review.md`.
The suggestions above do not invalidate the approved narrow behavior or create
additional DXF.1 merge prerequisites.

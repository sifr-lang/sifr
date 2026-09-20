# DXF.1 package reuse review follow-ups

Status: nonblocking follow-ups recorded separately from merged DXF.1.

Source: [scoped Opus review of PR #3875](https://github.com/sifr-lang/sifr/pull/3875#issuecomment-5749374222).
Candidate `c0aaf5ebb68ec371d2dc71b169c54c9867910580`; implementation merge
`041f9a2aba05c83668105e2db72d74603a0470f0`.
Verdict: **SATISFIED**, no blocking findings.
The [canonical record](../archive/ad-hoc-compiler-dx-followup-execution.md#dxf1-merged-record--2026-09-20)
binds source, tests, prepared artifacts and exact external review evidence.

| ID | Owner / disposition | Observation and bounded next action |
| --- | --- | --- |
| DXF1-F1 | Frontend graph performance; separate deferred work | `rebuild_edges` clones cached parsed ASTs before deriving signatures and edges. Review found no stale-AST correctness path. A later bounded change can borrow the suite or compute signature/edges before mutating state to avoid those clones. This is not silently added to DXF.2. |
| DXF1-F2 | Resolved by focused regression #3893 | Actual package-cwd/local-dependency checking now explicitly rejects private module/symbol access after restored-success history; fresh and empty-cache diagnostics match, and public access/reuse recover. Tests/evidence only; no production defect demonstrated. See merged record below. |
| DXF1-F3 | DXF.7 package checking boundary reconciliation | A restored result is diagnostics-only, whereas ordinary package computation also executes stdlib-for-codegen, codegen and interop metadata work. Reuse is bounded by the original producer's default-interop/no-Python attestation, pure-manifest inventory and unchanged zero-argument, undecorated, nongeneric primitive-return literal/name/binop/unaryop erasure class. That class cannot introduce new interop/probe demand. Any future expansion must qualify those later owners explicitly; no new checker/proof class is authorized here. |
| DXF1-F4 | Driver test infrastructure; existing requirement documented | The new fixture invokes offline Cargo metadata and needs Cargo on PATH plus writable temporary storage. The owning driver suite already uses Cargo metadata in `tests/attached_api_codegen.rs` and other package tests; this is not a newly discovered suite-wide prerequisite or current host blocker. Keep that requirement visible for any future isolated test runner. |
| DXF1-F5 | GitHub orchestration; resolved by approved relay | Remote `gh` is absent by design. The authenticated local GitHub relay independently verified draft PR/base/head and clean mergeability, then merged the exact approved head. The review itself was correctly bound to remote Git/source/evidence bytes. No tool installation or compiler work follows from this observation. |
| DXF1-F6 | Existing DXF.2 ownership | Validation cost rises along the measured history (helper approximately 7–21 ms; dependency 23–37 ms). Raw costs and variability are retained. Bounded observations, lookup and retention remain DXF.2, together with existing DX14-F1/F2 and DX13-F4 records; no history change was absorbed into DXF.1. |

The full review remains outside Git:
`/home/yaser5/projects/sifr/dxf-evidence/c0aaf5ebb68ec371d2dc71b169c54c9867910580/opus-review.md`.
The suggestions above do not invalidate the approved narrow behavior or create
additional DXF.1 merge prerequisites.

## DXF1-F2 bounded private-import regression — 2026-09-21

User-authorized scope: close only the missing independent private-import
negative evidence. Use the real Cargo metadata/package graph/source map from
the application package cwd and a local dependency; do not mock visibility.
Production changes are allowed only for a demonstrated visibility/reuse defect.

Acceptance fixed before implementation:

1. Start with a legitimate public dependency import and publish successful
   checking; make an eligible helper body edit and prove importer restoration.
2. Independently edit the importer to access a symbol directly through a private
   dependency module using supported `from dep.hidden import value as other`
   syntax. Require package visibility diagnostic SIFR-PACKAGE-0203, naming
   the private module.
3. Compare the warm/restored-history result to independent fresh checking and
   a cold empty-cache check of the same sources. Require identical diagnostics,
   zero restored checks and no restored-success status for the edited import.
4. Restore the public import, require success equal to fresh checking, then
   make another eligible body edit and prove valid importer reuse recovers.
   Historical success may be reused only after the public source is restored.

Run exact nonzero selector for the new negative and the existing small
package-reuse module, after formatting/diff/file-size/HIR guardrails. Reuse the
same Rust 1.98.1 default-feature private target, two jobs, pressure-based cleanup.
Scoped remote Opus review covers the exact candidate; no per-item monolithic
gate, new reuse proof class, performance qualification or next Clippy batch.
CI admission, deferred Windows, and interrupted cold-host performance/determinism
remain with their existing owners and are not reclassified by this item.

Test-construction correction before review: bare `import dep.hidden` is an
unsupported import form and correctly produces SIFR-IMPORT-0003. The initial
probe at `d8a8b4b6c31c63954f0dc5849f95529be3b2870a` therefore did not exercise
visibility and is preserved as failed evidence under
`/home/yaser5/projects/sifr/private-import-evidence/d8a8b4b6c31c63954f0dc5849f95529be3b2870a/`.
The supported from-import negative covers actual private module/symbol access;
no new import grammar or product behavior is authorized.

## DXF1-F2 merged record — 2026-09-21

- Implementation [PR #3893](https://github.com/sifr-lang/sifr/pull/3893)
  merged as `5b2cd036d36f43f5a71f69961adce20a32f9d3bc`.
  Exact reviewed/tested candidate: `2f3dfadda5ab0c0498ea7f32b0d616eb6dddec31`;
  base: `694dbc050fcd72fe2107ececd0be2080a7633798`.
- Added `project_cache::package_reuse_tests::package_private_symbol_edit_rejects_restored_success`.
  Real Cargo metadata and the actual source map reject an existing private
  dependency module with `SIFR-PACKAGE-0203`, identically after legitimate
  importer restoration, in fresh checking, and with an independent empty cache.
  Both cached checks restore zero checks. Restoring the public import succeeds;
  the next supported helper body edit again restores the importer.
- Exact selector listed **1 test** and passed **1/1**. The smallest affected
  existing module, `project_cache::package_reuse_tests::`, passed **4/4**.
  Rust 1.98.1, normal default features/test profile, existing private target,
  two jobs, `INSTA_UPDATE=no`. Formatting/diff checks, file-size guardrails
  (4110 files, 900-line limit) and HIR maintainability guardrails passed first.
  Storage admission found 17 GiB free and a 188 GiB owned target; the focused
  relink needed no cleanup.
- [Scoped remote Opus review](https://github.com/sifr-lang/sifr/pull/3893#issuecomment-5753285448):
  **SATISFIED**, no blocking findings, exact candidate above. Review and
  validation remain outside the reviewed Git tree:
  `/home/yaser5/projects/sifr/private-import-evidence/2f3dfadda5ab0c0498ea7f32b0d616eb6dddec31/`
  (`index.json`, `tests.json`, `test-0.log` through `test-2.log`,
  `opus-review.md`). The superseded unsupported-syntax probe remains failed
  historical evidence at the earlier directory recorded above.
- Separate optional test-maintenance follow-ups from review: (F2-R1) inline
  the single-caller negative helper if no additional import form is needed;
  (F2-R2) explicitly assert retained history/candidate lookup to guard against
  a future source-sensitive identity changing the negative's non-vacuity.
  Current identity is source-byte independent, and review verified the warmed
  history is actually consulted. Neither suggestion blocks this completed
  contract or authorizes further work in this session.
- No production behavior changed. No full gate, host performance result,
  Windows qualification, or deterministic cold-host completion is claimed.
  Existing CI/Windows and interrupted performance/determinism owners retain
  their dispositions. No release/install/local-checkout change or next Clippy
  batch ran.

Handoff: DXF1-F2 is complete. Record-only closure uses documentation checks;
validation and review remain bound to the unchanged implementation candidate.
Stop here; any next item requires a new bounded session.

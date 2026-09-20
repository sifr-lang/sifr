# Local-first workflow admission failure

Status: closed — admission/provisioning repaired; unrelated automatic validation failures remain open
Owner: CI / verification workflow admission

DXF.7 observed `.github/workflows/local-first-validation.yml` failing before any
jobs were created. This was already present at its unchanged base
`4f9fc5c0d8ac82d5a9e01c4dc4b20450ebf23e09`:
[base run 35512834988](https://github.com/sifr-lang/sifr/actions/runs/35512834988).
The same failure occurred on the original DXF.7 package candidate:
[run 35513756708](https://github.com/sifr-lang/sifr/actions/runs/35513756708).
Check suite 96158878536 contained no check runs. Workflow bytes are unchanged
between the base and final validation candidate
`db38e61773b351399a0217e502d8011b1212def7`.

At the DXF.7 handoff the underlying admission failure had not been established. Do not describe this
as a failed compiler test or guess a YAML/matrix cause without the admission
error. Inspect the workflow/check-suite admission diagnostics, identify the
owning configuration failure, and qualify its correction in a separate item.
No workflow repair or broad compiler gate restart was absorbed into DXF.7.

Read-only evidence:
`/home/yaser5/projects/sifr/dxf-evidence/f6865cdfb8aa8d0f7d62759ec4fe313e5edc3708/unrelated-workflow-failure.json`.
The follow-up validation policy and completed affected checks are in the
[DXF.7 record](../archive/ad-hoc-compiler-dx-followup-execution.md#dxf7-merged-record--2026-09-20).


## Established cause and bounded correction — 2026-09-20

GitHub's original run annotation states `(Line: 44, Col: 9): Unrecognized
named-value: 'matrix'` in the job-level condition
`github.event_name != 'pull_request' || matrix.profile == 'create-pr'`.
Job conditions are evaluated before matrix expansion. Repository Actions are
enabled with all actions allowed; the original suite has zero check runs.
This is an admission error, not a failed compiler test or a permissions denial.

[PR #3891](https://github.com/sifr-lang/sifr/pull/3891) selects the profile matrix
using only the event context: PRs retain `create-pr`, while main pushes/manual
runs retain `create-pr`, `merge`, and `release`. All other jobs, assertions,
targets, offline execution, fail-fast behavior and advisory lint policy remain.
The automatic contract detects the original invalid condition, narrowed event
coverage, skipped profile execution and omitted preparation.

Real admitted execution exposed empty hosted Cargo caches before the standalone
offline smoke and SQL checks. The authorized bounded provisioning correction
acquires the complete locked workspace graph before those assertions, reuses
existing Cargo/area preparation owners for smoke compiler graphs, and registers
its preparation contract in the automatic runner selftest aggregator. An initial
target-filtered fetch still missed a host dependency (`core_detect`); the final
correction uses canonical `cargo fetch --locked` without restricting acquisition
to the cross target. Compilation and assertion target selections are unchanged.

Implementation candidate: `05c72e32a2b66fc1414929e40b59f6c6e619e454`.
Unchanged base: `89f6d7633da42650e1d9e27e1a2e10d18446f34b`.
Focused remote evidence: all 45 setup-policy tests (including the newly registered
smoke acquisition/offline contract), workflow regressions, uv invariant and its
46 self-tests, action pins, HIR guardrail, file-size guardrail (4,110 files), and
`git diff --check` pass. No monolithic local compiler gate was repeated.
Scoped remote Claude Opus 5 verdict is **SATISFIED** on that exact implementation
candidate after resolving its one test-registration omission.
Evidence is outside Git under
`/home/yaser5/projects/sifr/ci-admission-evidence/<candidate>/`, including
`policy-checks.log`, `review.md`, `run.json`, `jobs.json` and completed-job logs.

## Actual automatic execution

[Final automatic PR run 35533124294](https://github.com/sifr-lang/sifr/actions/runs/35533124294)
admitted all ten intended PR jobs and executed their commands. Seven jobs pass:
uv/workflow invariants, the existing advisory lint job, smoke fuzz/property
(43 variants, zero failures), all three Unix native component/SQL jobs, and WASI
SQL clean/incremental/locked/offline/reproducible qualification. The advisory
Clippy step itself fails under the pre-existing policy; its green job is not a
claim that Clippy is clean.

Windows executes all 16 component tests successfully, then SQL qualification
fails with 55 pre-existing Unix-only driver compilation errors. The user chose
to record that work separately: [Windows portability owner](../active/ad-hoc-windows-driver-portability.md),
linked from existing DX9-F6. No Windows checks were disabled.
Deterministic-report execution is interrupted by a runner shutdown signal and
exit 143 after 34m32s; no completed signature comparison is claimed. Exact
unchanged-command evidence is in the [runner-interruption issue](../active/ad-hoc-ci-determinism-runner-interruption.md).
The create-pr profile passes all 45 preparation-policy tests and 13 Rust interop
variants, then exits 124 on its unchanged 20,000 ms Rust interop performance
budget (57,170 ms observed). Cold setup passes at 10,287,999 ms; total lane time
is 10,603.50 s. The first cold hosted run is not warm-host qualification and
later profile checks are not claimed executed after this fail-fast stop.
The [cold-host performance owner](../active/ad-hoc-ci-create-pr-cold-host-performance.md)
records the exact receipt and a named preparation/execution comparison as the
next action. Overall final run conclusion is failure: seven job successes and
three failures. This issue closes only admission/provisioning, not those failures.

Historical evidence is retained: original zero-job runs 35512834988/35513756708;
admitted initial run 35532393023 with missing offline dependencies; intermediate
run 35532657611 with missing host dependency (cancelled after preserving failure
to avoid redundant work); run 35532810784 with successful corrected preparation,
Windows driver failures and deterministic-runner interruption. Earlier failures
are not reclassified as passing by later candidates.


## Merged closure — 2026-09-21 (Europe/Stockholm)

Merged [PR #3891](https://github.com/sifr-lang/sifr/pull/3891) at
2026-09-20T22:42:13Z as main commit
`57a587df0a84612a5aac43c5f22f777127abf250`.
Validation and scoped Opus approval cover implementation candidate
`05c72e32a2b66fc1414929e40b59f6c6e619e454`; this closure changes records only.
The original final-run API receipt and all three failure logs are preserved.
The remaining duplicate create-pr job in run 35532810784 was cancelled only
after the final candidate completed; its earlier Windows and deterministic
failures remain intact. No final-candidate job was cancelled or skipped.

Next actions are scoped to the linked owners: establish the deterministic runner
termination cause with retained progress/resource evidence; compare the named
maintained-demo preparation/execution inputs and warm reuse before any performance
policy change; leave Windows portability separately recorded as the user chose.
Metadata selftest-registration review observations are recorded in the existing
metadata follow-up issue. Private-import and Clippy implementation work was not
included. Only documentation checks apply to these record-only closure edits; further
implementation belongs to a separately scoped item.

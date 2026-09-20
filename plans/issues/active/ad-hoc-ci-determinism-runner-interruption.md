# Deterministic-report CI runner termination

Status: open; separate runner/execution investigation
Owner: CI hosted-runner resources and e2e report qualification

Admission is restored and the intended command executes, but the automatic
`deterministic-report-signature` job does not produce a completed assertion.
On final admission candidate `05c72e32a2b66fc1414929e40b59f6c6e619e454`,
[job 106137284284 in run 35533124294](https://github.com/sifr-lang/sifr/actions/runs/35533124294/job/106137284284)
ran for 34m32s and ended at 2026-09-20T20:15:58Z with GitHub's
"The runner has received a shutdown signal" and exit 143.
The job ran `bash verification/runner/e2e/check_report_determinism.sh --profile merge`.
No completed report-signature comparison is visible in the log.

The same command on earlier admission candidates was interrupted in
[job 106135185916 / run 35532393023](https://github.com/sifr-lang/sifr/actions/runs/35532393023/job/106135185916)
and [job 106136353558 / run 35532810784](https://github.com/sifr-lang/sifr/actions/runs/35532810784/job/106136353558).
None of these jobs was cancelled by this session. The session explicitly
cancelled superseded intermediate run 35532657611 after preserving its
missing-host-dependency failure, and later cancelled only the still-running
create-pr job in superseded run 35532810784 after the final candidate completed.
That cancellation occurred after its deterministic-report job had already failed. No completed/failed run was reclassified green.

Exact logs are preserved outside Git as `determinism.log` beneath
`/home/yaser5/projects/sifr/ci-admission-evidence/` in each matching candidate
SHA directory (`dde99fdc32153afdcf93c448db9e709c8d32828d`,
`f8164f788185e40cef32f5a8c2b7c4e5dc4a748b`, and the final SHA above).
The runner termination cause is not established; do not label this a compiler
assertion, deterministic-signature mismatch, or resource exhaustion without
further evidence. The script currently redirects e2e output to runner-local
`/tmp/sifr-e2e-determinism-*.XXXXXX` logs, limiting post-termination diagnosis.

A separate investigation should preserve progress/resource/log evidence outside
that ephemeral runner, establish the terminating owner/cause, and qualify the
unchanged two-run signature comparison on a real automatic execution. Do not
skip the check, narrow its corpus, remove assertions, or blindly repeat expensive
runs on unchanged inputs. This issue does not expand the CI admission repair.

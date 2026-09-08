# Ad Hoc Issue: Permanent Solo-Maintainer Release Approval

## Status

Implementation owned by latest-stable convergence Item 65. On 2026-09-08 the
user permanently superseded the distinct-person requirement: `stable-release`
requires explicit GitHub-recorded approval by `yaseralnajjar`, permits self-review,
and disables admin bypass. This decision approves policy only, not any release
run or publication. No second-person invitation is a prerequisite.

Historical waiver, report, and signoff bytes and digests remain immutable.
The expiry failure below is retained evidence, not a reason to renew the waiver.

## Blocking Evidence

On 2026-08-27, latest-stable Item 31 candidate
`dbdbd42915dd45fe0255681c224266dd08f453ea` ran its one authorized merge gate.
Kafka and every completed functional area passed. Distribution qualification
passed 67 of 68 variants. The sole failure was
`schema-v2-preview-epoch-bootstrap`, which rejected the expired waiver. The
gate stopped before later crate and E2E stages.

The merge-gate log SHA-256 is
`b9e004ebc8b2ff494cdab8f8642a0e28759e88d4b0fbc8e652c1520a85fe1069`.
The report SHA-256 is
`cbdb39d9047da8206ebcba10e2c6856beaffaec314e4487413580b67a67181cd`.
The evidence is in the
[#3551 gate comment](https://github.com/sifr-lang/sifr/pull/3551#issuecomment-5432642050).

Do not extend the waiver or alter historical identities. Latest-stable Item 31
remains draft and cannot consume a second merge gate under its current rules.

## Scope and acceptance

- Require the designated maintainer for every exact protected run/attempt and
  release evidence, including normal, rollback, incident roll-forward, and recovery.
- Read back `stable-release`: reviewer `yaseralnajjar`,
  `prevent_self_review: false`, `can_admins_bypass: false`.
- Remove live waiver selection from workflows and direct approval entrypoints.
- Preserve historical validity at the original event time separately from new-use
  authorization; do not reconstruct missing original receipts.
- Reject absent/non-designated approvals, mismatched run/evidence identities,
  bypass-enabled configuration, and new use of the retired waiver.
- Qualify Item 65 with its named focused suites, one exact-SHA Opus review
  (at most one remediation), and one final-candidate merge gate.

Completion and exact evidence are recorded in
[latest-stable convergence](ad-hoc-latest-stable-release-convergence.md).

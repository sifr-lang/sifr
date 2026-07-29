# Phase 40 final exact-source qualification evidence

Status: complete. Workflow transport, source-derived evidence, and the
authoritative local release profile all pass.

## Exact source

- Source commit:
  `c9d611fb7c7c5d05421d784d53a2b78c1a7dcae9`
- Source checkout:
  `/tmp/sifr-phase40-release-source-c9d611fb7c`
- The checkout is detached at the exact source commit, recursively initialized,
  and clean.
- Recursive submodule identities match the qualification artifact index.

## Qualification workflow

- Workflow run:
  [#30416219284](https://github.com/sifr-lang/sifr/actions/runs/30416219284)
- Event: `workflow_dispatch`
- Workflow: `.github/workflows/release-qualification.yml`
- Version: `0.1.0`
- Rollback version: `none`
- Exact workflow head:
  `c9d611fb7c7c5d05421d784d53a2b78c1a7dcae9`
- Result: success
- Completed jobs:
  - immutable candidate identity validation;
  - all four governed target build/package/smoke jobs;
  - aggregate installer and checksums;
  - VS Code package qualification;
  - canonical qualification artifact index collection.

The GitHub workflow-dispatch API rejected a raw commit SHA as `ref`; the run
was dispatched through `main` only after verifying that remote `main` resolved
to the exact source SHA. The workflow's own immutable identity job then
required `github.sha` and `source_commit` to equal that SHA.

## Artifact custody

- Canonical index:
  `/tmp/sifr-phase40-qualification-30416219284/index/qualification-artifact-index.json`
- Index SHA-256:
  `503f4fcc0dcf4843e0476fbbd1aaa02994c431fac3e7aebb89fb5565bba04703`
- Refetched candidate uploads:
  `/tmp/sifr-phase40-qualification-30416219284/payloads`
- Candidate upload count: 6
- Indexed payload count: 20
- Indexed payload bytes: 533,998,429
- Earliest expiry: `2026-08-28T02:17:30Z`
- Latest expiry: `2026-08-28T02:32:17Z`

`fetch_qualification_artifacts.py` refetched every candidate upload by its
immutable GitHub artifact id and required the exact run/source attribution.
An independent replay then verified all 20 files by exact name, size, and
SHA-256 against the canonical index.

The separately uploaded index artifact is GitHub artifact id `8710544640`.
The six candidate uploads are ids `8710312634`, `8710327423`, `8710332418`,
`8710519427`, `8710534546`, and `8710536142`.

## Source-derived evidence

- Documentation qualification:
  `/tmp/sifr-phase40-release-work-c9d611fb7c/qualification-documentation.json`
  - report id: `docs-c9d611fb7c7c-45aaf53188cb`
  - SHA-256:
    `9ec2ec35c1ffcdffaaae7128b9654fbb12588104526ec15dadcd3599631bfa45`
  - canonical JSON: yes
- Stable support claims:
  `/tmp/sifr-phase40-release-work-c9d611fb7c/stable-support-claims.json`
  - SHA-256:
    `b62f5b936be097b31201afa9591d52a5463ae7180fbbda6af3bdffaedcecc3c9`
  - canonical JSON: yes
  - staged deterministically from the exact source checkout by
    `stage-stable-support-claims`.

Both commands left the exact source checkout clean.

## Local release profile

The first cold attempt completed all preceding lanes through CPython
differential, then three Python-interop cases ran before the shared verification
environment had been created by another parallel case. Once the environment
appeared, all later 22 Python-interop variants passed. The unchanged warm
attempt closed those three cases and passed Python interop 25/25, consumed Rust
interop 10/10, developer tooling 48/48, and documentation 2/2.

That warm attempt stopped only at the already indexed `PERF-HOST` condition:
three measured medians were above their unchanged thresholds while every
benchmark command passed. An immediate isolated full-performance replay
reproduced the host variance while another checkout was actively compiling.
No source, threshold, baseline, waiver, or profile selection changed.

After the competing checkout finished, the unchanged isolated performance
suite passed 8/8. A final fresh-parent release-profile invocation then passed
all 24 lane steps, including performance 8/8, Python interop 25/25, consumed
Rust interop 10/10, generated-code release-full, all crate suites, 674 E2E
cases, 290 hardening variants, and sequential/parallel report equivalence.
Every blocking budget and functional gate passed. The 7,610.91-second cold
wall time produced only the declared warm-target advisory.

- Canonical release-profile report:
  `/tmp/sifr-phase40-release-work-c9d611fb7c/release-profile-report.json`
  - report id: `release-c9d611fb7c7c-fa3d95c04f8a`
  - overall status: `pass`
  - SHA-256:
    `faa6844410de98cb6ebe40d740ab6b1edc9aeb176ee0301e4ec181937eeb6e03`
  - source clean: yes
  - canonical JSON: yes
- Exact canonical Rust result copied from that release invocation:
  `/tmp/sifr-phase40-release-work-c9d611fb7c/rust-validation-report.json`
  - variants: 10
  - failures: 0
  - SHA-256:
    `be24b69a7afc0f2f7061657258d9c367946496bf745b3cc17b1cd15e00bba87a`
  - canonical JSON: yes

The release report's `rust_interop_checks` suite rows and result-artifact row
all bind that exact Rust-result digest. Governance validation with
`--require-canonical` passed, and the exact detached source checkout remained
clean.

## Live public boundary

The current public `channels.json` remains schema v1 at SHA-256
`71b3243925670f56dc510b8f45b6614a622f58097a0fea9492f61d20dc4bf9ef`.
The protected `stable-release` environment currently has no protection rules
or required reviewers, and the repository exposes only the initiating account
as a collaborator. Therefore the truthful schema-v2 preview bootstrap remains
blocked on a genuinely distinct human reviewer; no production mutation was
attempted.

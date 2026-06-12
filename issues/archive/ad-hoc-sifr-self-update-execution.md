# Ad Hoc Phase Execution Checklist: Sifr Self Update

Phase contract: [ad-hoc-sifr-self-update.md](./ad-hoc-sifr-self-update.md)

Status: complete

## Checklist

- [x] `milestone_self_update_1`: Metadata And Receipt Contract
- [x] `milestone_self_update_2`: CLI Eligibility And Dry Run
- [x] `milestone_self_update_3`: Installer Delegation
- [x] `milestone_self_update_4`: Distribution Drift Guardrails
- [x] `milestone_self_update_5`: Docs And Release Readiness

## Review Artifacts

- Initial planning review: `reviews/ad-hoc-sifr-self-update-review-pass-1.md` -> `CHANGES_REQUESTED`; addressed metadata URL injection, receipt schema ownership, same-file eligibility, stable metadata rejection, immutable installer drift checks, and concurrent update locking in the phase contract. A later cleanup removed legacy receipt compatibility because Sifr preview distribution is still unstable.
- Follow-up planning review: `reviews/ad-hoc-sifr-self-update-review-pass-2.md` -> `READY`; reviewer confirmed the architecture is elegant enough because the CLI verifies the receipt before network access, consumes version-only metadata, derives immutable installer URLs from constants, and delegates installation to the existing verified installer.
- PR implementation-readiness review: `reviews/ad-hoc-sifr-self-update-review-pass-3.md` -> `READY`; reviewer confirmed the backward-compat cleanup made the contract cleaner and implementation-ready, with polish folded back into the phase contract before milestone work begins.
- PR polish review: `reviews/ad-hoc-sifr-self-update-review-pass-4.md` -> `READY`; reviewer confirmed the pass-3 polish closed remaining interpretive gaps without widening the surface.
- Regression review: `reviews/ad-hoc-sifr-self-update-review-pass-5.md` -> `READY`; reviewer confirmed the contract is implementation-ready after final trust-boundary, diagnostic-range, lock-path, and dry-run clarifications.
- Decision audit review: `reviews/ad-hoc-sifr-self-update-review-pass-6.md` -> `CHANGES_REQUESTED`; reviewer found remaining unmade decisions around dry-run JSON, installer download minimum size, immutable installer `--force` sequencing, rc rejection placement, diagnostic family ownership, and Phase 39 schema-bump behavior.
- Final implementation-readiness review: `reviews/ad-hoc-sifr-self-update-review-pass-7.md` -> `READY`; reviewer confirmed all pass-6 findings are fixed and the phase is implementation-ready without backward compatibility concerns.
- M1 implementation reviews: `reviews/self-update-m1-review-pass-1.md`, `reviews/self-update-m1-review-pass-2.md`, `reviews/self-update-m1-review-pass-3.md` -> `READY`.
- M2 implementation reviews: `reviews/self-update-m2-review-pass-1.md`, `reviews/self-update-m2-review-pass-2.md` -> `READY`.
- M3 implementation reviews: `reviews/self-update-m3-review-pass-1.md`, `reviews/self-update-m3-review-pass-2.md` -> `READY`.
- M4 implementation reviews: `reviews/self-update-m4-review-pass-1.md`, `reviews/self-update-m4-review-pass-2.md`, `reviews/self-update-m4-review-pass-3.md` -> `READY`.
- M5 implementation reviews: `reviews/self-update-m5-review-pass-1.md`, `reviews/self-update-m5-review-pass-2.md` -> `READY`.
- Final implementation review: `reviews/self-update-final-implementation-review-pass-1.md` -> `CHANGES_REQUESTED`; blocking feedback was limited to phase tracking closeout after M5 merged.
- Final closure review: `reviews/self-update-final-implementation-review-pass-2.md` -> `READY`.

## Validation Ledger

Record local validation for each milestone before opening the corresponding PR.

- M1: distribution validation, focused Rust receipt/metadata tests, and create-pr/merge local validation passed before merge.
- M2: `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, `cargo test -p sifr -- self_update`, `scripts/run_distribution_validation.sh`, CLI dry-run smoke, file-size guardrail, `scripts/run_all_tests.sh --profile create-pr`, and `scripts/run_all_tests.sh --profile merge` passed before merge.
- M3: `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, `cargo test -p sifr -- self_update`, `scripts/run_distribution_validation.sh`, file-size guardrail, non-dry-run CLI fake-installer smoke, and `scripts/run_all_tests.sh --profile create-pr` passed before merge.
- M4: `bash -n` for new/updated shell scripts, M4 drift fixtures, explicit `schema_version=true` rejection probe, `scripts/run_distribution_validation.sh`, `cargo fmt --check`, file-size guardrail, and `scripts/run_all_tests.sh --profile create-pr` passed before merge.
- M5: `cargo fmt --check`, file-size guardrail, self-update docs sanity grep, `cargo clippy --workspace -- -D warnings`, `cargo test -p sifr -- self_update`, `scripts/run_distribution_validation.sh`, and `scripts/run_all_tests.sh --profile create-pr` passed before PR. Targeted generated-code rustfmt passed after an interrupted broad run, then `scripts/run_all_tests.sh --profile merge` passed before merge with only wall-time/skew advisories.

## Merged PRs

Record merged PR links here as each milestone lands.

- M1: [PR #2274](https://github.com/sifr-lang/sifr/pull/2274)
- M2: [PR #2275](https://github.com/sifr-lang/sifr/pull/2275)
- M3: [PR #2276](https://github.com/sifr-lang/sifr/pull/2276)
- M4: [PR #2277](https://github.com/sifr-lang/sifr/pull/2277)
- M5: [PR #2278](https://github.com/sifr-lang/sifr/pull/2278)

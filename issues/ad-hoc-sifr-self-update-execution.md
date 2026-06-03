# Ad Hoc Phase Execution Checklist: Sifr Self Update

Phase contract: [ad-hoc-sifr-self-update.md](./ad-hoc-sifr-self-update.md)

Status: draft

## Checklist

- [ ] `milestone_self_update_1`: Metadata And Receipt Contract
- [ ] `milestone_self_update_2`: CLI Eligibility And Dry Run
- [ ] `milestone_self_update_3`: Installer Delegation
- [ ] `milestone_self_update_4`: Distribution Drift Guardrails
- [ ] `milestone_self_update_5`: Docs And Release Readiness

## Review Artifacts

- Initial planning review: `reviews/ad-hoc-sifr-self-update-review-pass-1.md` -> `CHANGES_REQUESTED`; addressed metadata URL injection, receipt schema ownership, same-file eligibility, stable metadata rejection, immutable installer drift checks, and concurrent update locking in the phase contract. A later cleanup removed legacy receipt compatibility because Sifr preview distribution is still unstable.
- Follow-up planning review: `reviews/ad-hoc-sifr-self-update-review-pass-2.md` -> `READY`; reviewer confirmed the architecture is elegant enough because the CLI verifies the receipt before network access, consumes version-only metadata, derives immutable installer URLs from constants, and delegates installation to the existing verified installer.
- PR implementation-readiness review: `reviews/ad-hoc-sifr-self-update-review-pass-3.md` -> `READY`; reviewer confirmed the backward-compat cleanup made the contract cleaner and implementation-ready, with polish folded back into the phase contract before milestone work begins.
- PR polish review: `reviews/ad-hoc-sifr-self-update-review-pass-4.md` -> `READY`; reviewer confirmed the pass-3 polish closed remaining interpretive gaps without widening the surface.
- Regression review: `reviews/ad-hoc-sifr-self-update-review-pass-5.md` -> `READY`; reviewer confirmed the contract is implementation-ready after final trust-boundary, diagnostic-range, lock-path, and dry-run clarifications.
- Decision audit review: `reviews/ad-hoc-sifr-self-update-review-pass-6.md` -> `CHANGES_REQUESTED`; reviewer found remaining unmade decisions around dry-run JSON, installer download minimum size, immutable installer `--force` sequencing, rc rejection placement, diagnostic family ownership, and Phase 39 schema-bump behavior.
- Final implementation-readiness review: `reviews/ad-hoc-sifr-self-update-review-pass-7.md` -> `READY`; reviewer confirmed all pass-6 findings are fixed and the phase is implementation-ready without backward compatibility concerns.

## Validation Ledger

Record local validation for each milestone before opening the corresponding PR.

## Merged PRs

Record merged PR links here as each milestone lands.

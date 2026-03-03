# Canonical Backlog (Phase 15 Reconciliation Baseline)

Last updated: 2026-03-03
Owner: Phase 15 (`milestone_15_1`)
Status: active baseline for Phases 16-36

## Source Set Consolidated
- `reviews/phase14-review.md`
- `reviews/phase14-production-grade-review.md`
- `.cursor/plans/main/roadmap.md` (deferred planning risks for Phases 36 and 37)

## Deduplication Rules Applied
- A finding appears once under a single canonical ID (`BL-15-XXX`).
- Duplicate statements across reviewer docs are merged into one backlog item.
- Severity is normalized to `P0` (critical) through `P3` (low).
- Each finding is tagged to one owning future phase for execution accountability.

## Canonical Findings

| ID | Title | Severity | Owning Phase | Source(s) | Backlog Issue | Status |
|---|---|---|---|---|---|---|
| `BL-15-001` | Demo sweep evidence must be reproducible via explicit command/script reference | `P2` | Phase 16 | `reviews/phase14-review.md` section 4.2 | [/issues/phase15-canonical-backlog-issues.md#phase15-bl-15-001](/issues/phase15-canonical-backlog-issues.md#phase15-bl-15-001) | open |
| `BL-15-002` | Validation evidence should annotate test-count drift and timing variance to avoid ambiguity in phase closeout artifacts | `P3` | Phase 24 | `reviews/phase14-review.md` section 2.1, section 4.3 | [/issues/phase15-canonical-backlog-issues.md#phase15-bl-15-002](/issues/phase15-canonical-backlog-issues.md#phase15-bl-15-002) | open |
| `BL-15-003` | Test-only carve-out (`RawCode`/`SynItem`) must remain isolated from production paths with explicit guardrails | `P1` | Phase 20 | `reviews/phase14-review.md` section 4.1, `reviews/phase14-production-grade-review.md` section 2.3 | [/issues/phase15-canonical-backlog-issues.md#phase15-bl-15-003](/issues/phase15-canonical-backlog-issues.md#phase15-bl-15-003) | open |
| `BL-15-004` | Production banlist and structured-lowering gates must stay enforced in local-first validation loops | `P1` | Phase 25 | `reviews/phase14-production-grade-review.md` sections 3.1, 3.3 | [/issues/phase15-canonical-backlog-issues.md#phase15-bl-15-004](/issues/phase15-canonical-backlog-issues.md#phase15-bl-15-004) | open |
| `BL-15-005` | E2E timing budgets need explicit baseline and regression thresholds | `P2` | Phase 29 | `reviews/phase14-production-grade-review.md` section 5.3 | [/issues/phase15-canonical-backlog-issues.md#phase15-bl-15-005](/issues/phase15-canonical-backlog-issues.md#phase15-bl-15-005) | open |
| `BL-15-006` | Deferred planning gaps for Phases 36 and 37 require explicit closure criteria before track completion | `P2` | Phase 35 | `.cursor/plans/main/roadmap.md` deferred planning drafts | [/issues/phase15-canonical-backlog-issues.md#phase15-bl-15-006](/issues/phase15-canonical-backlog-issues.md#phase15-bl-15-006) | open |

## Deduplication Ledger

| Duplicate Group | Merged Into | Notes |
|---|---|---|
| `DG-15-001` | `BL-15-002` | Test-count variance and timing variance appeared as separate review notes; normalized as one evidence-quality contract item. |
| `DG-15-002` | `BL-15-003` | Test-only carve-out risk appeared in both review files; retained once with shared source attribution. |

No duplicate canonical IDs remain in this backlog.

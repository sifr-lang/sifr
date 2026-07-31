# Class-field mutating-receiver overlap remediation — Claude Opus PR review pass 1

## Review target

- Base: `a7a5df414b985cc95a9ad23c5b006caa84101f0d`
- Head: `a98a8deb747023fd34de7a9a9c9ad1b41e376d46`
- Pull request: #3090
- Scope: structured ownership diagnostics for unsupported field-value footprints, all five OWN0002 emission paths, and phase-fixture create-PR coverage.

The reviewer was instructed to remain read-only and to assess the exact commit range against the active phase plan and repository workflow.

## Validation independently inspected or run by the reviewer

- Inspected the implementation diff and relevant phase requirements.
- Ran the complete E2E failure suite successfully.
- Reproduced the targeted callable-field and recursive-field overlap diagnostics as structured `SIFR-OWN-0002` failures.
- Confirmed all five OWN0002 emission paths attach the structured `binding` argument.
- Confirmed the six phase pass fixtures are present in the create-PR manifest.

## Findings

### Blocking

1. `collect_footprint` used `Footprint::Dynamic(root)` whenever a callable or recursive field could not be extracted as an ordinary value place. This closed the missing-diagnostic gap, but it also collapsed all fields under the root and rejected legal disjoint siblings. For example, `take(owner.inner, owner.callback)` and `take(node.inner, node.next)` must be accepted under the phase's field-identity-aware place-prefix rule. The reviewer requested recovery of a precise field `Place` for footprint comparison even when ordinary field-value place extraction is intentionally unsupported.

### Non-blocking

1. The canonical OWN0002 message template still said “in the same call,” although the pending async-generator advance path is not a same-call diagnostic.
2. The dynamic-root fallback could redundantly collect the object footprint.
3. The create-PR evidence did not embed the reviewed SHA and does not itself run the failure suite; the reviewer independently ran that suite to close the review gap.

## Disposition

- Verdict: **NOT SATISFIED**
- Required action: retain conservative fallback only for genuinely unresolvable bases, preserve field projections for unsupported callable and recursive field values, and add acceptance tests for disjoint siblings alongside the existing overlap-rejection tests.

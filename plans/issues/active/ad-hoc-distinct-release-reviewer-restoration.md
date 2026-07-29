# Ad Hoc Issue: Distinct Release Reviewer Restoration

## Status

Active non-blocking follow-up created from the user-directed Phase 40
single-maintainer approval exception on 2026-07-29.

The repository currently has one maintainer. Phase 40 bootstrap and first-GA
publication may therefore use the canonical, expiring
`plans/releases/single-maintainer-approval-waiver.json`. The exception does
not remove protected approval: `stable-release` must require a GitHub-recorded
approval, admin bypass must be disabled, the approver must equal the named
owner, and retained evidence must bind the waiver digest.

The waiver expires on 2026-08-27 and authorizes only `bootstrap-alpha`,
`bootstrap-index`, and `ga-activation`. It cannot authorize `normal`,
`rollback`, or `incident-roll-forward`.

## Scope

- Add a genuinely distinct human release/distribution reviewer with repository
  access.
- Configure `stable-release` with that user or team as required reviewer,
  `prevent_self_review: true`, and admin bypass disabled.
- Keep historical bootstrap and first-GA evidence readable with its exact
  single-maintainer waiver identity.
- Remove the live waiver input from publication workflows after the environment
  has a distinct reviewer.
- Prove self-approval fails for every future stable and incident operation.

## Acceptance Criteria

- [ ] A distinct reviewer or release/distribution team has accepted repository
  access.
- [ ] `stable-release` requires that reviewer/team, prevents self-review, and
  disallows admin bypass.
- [ ] Bootstrap and first-GA evidence produced under the expired waiver remains
  canonical and validates as historical evidence.
- [ ] No workflow can select the expired waiver for a new publication.
- [ ] Normal, rollback, and incident roll-forward operations reject initiating
  owner self-approval.
- [ ] Focused distribution suites, workflow contracts, the authoritative local
  gates, and repeated Opus review pass.

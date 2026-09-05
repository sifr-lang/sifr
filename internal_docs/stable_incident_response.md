# Stable Release Incident Response

This runbook defines the governed response to a defect in the active stable
Sifr release. It is authoritative for local planning and fixture drills now.
Production workflow inputs and credentials remain intentionally absent until
the protected publication workflow wires this tested core into the single
release-publication workflow.

## Ownership and service target

- Release owner: `release/distribution`.
- Incident owner: `release/distribution`.
- Approval authority: a protected `stable-release` environment approval from
  `release/distribution` for a run not initiated by the same person. Initial
  and resume attempts require distinct recorded approvals.
- Acknowledgement target: 30 minutes from a qualifying trigger. This is
  deliberately longer than the bounded 20-minute site-deployment wait, so a
  terminal site timeout can release the metadata lease and be included in the
  acknowledgement rather than leaving the response pending.

The incident owner acknowledges by assigning the incident identifier, opening
the evidence-only incident PR, and naming either `rollback` or
`incident-roll-forward`. The release owner owns artifact/index verification and
the eventual protected run. The same person may hold both roles, but may not
approve their own protected run.

## Triggers and operation choice

Open a stable incident for a reproducible compiler correctness or safety
regression, an installer or sysroot integrity failure, a supported-target
failure, a governed dependency or security event, or a public smoke failure
that makes the active stable release unfit for selection.

Use `rollback` only when all of these conditions hold:

1. the active release was produced by a `normal` stable plan;
2. that exact plan names the requested active predecessor and approved plan
   digest as its rollback target;
3. the retained target is still active, its immutable installer matches the
   governed index, and the VS Code compatibility range covers it.

Use `incident-roll-forward` when no eligible rollback target exists, including
the first GA release. The approved successor plan binds the incident-request
digest, the affected active version and plan digest, and
`rollback_target: none`. Activation adds the qualified successor and withdrawal
of the affected version in one new index generation.

## Evidence and communication locations

The durable incident identifier is used in all locations:

1. `plans/releases/incidents/<incident-id>/stable-incident-request.json` and
   `withdrawal-evidence.txt` in an evidence-only PR. Repository validation
   permits exactly those two added files, checks canonical request bytes, and
   verifies the evidence digest. Source changes, renames, deletions, or
   unrelated incident files are rejected.
2. Uniquely named, write-once request, release-index generation, realized site
   facts, validation/communication/closure evidence, and incident sign-off
   assets in the governance release.
3. The protected workflow prepare summary and attempt history, which bind the
   exact evidence commit, paths, digests, approval status, and mutations.
4. The affected GitHub release notice and the stable release documentation
   incident notice. Public prose is landed with the GA documentation surface;
   the governed site facts already carry the active stable version and all
   withdrawals for deterministic reconciliation.

Closure requires the public stable installer, exact pin behavior, affected
self-update recovery, out-of-band recovery, site facts, extension range,
release index, immutable assets, communications evidence, and sign-off to
agree.

## Mutation, retry, and retention rules

All preview, stable, rollback, and incident-roll-forward operations share the
metadata concurrency lease. A pending incident blocks a new preview or stable
submission. A cancelled pending operation is recorded and must be explicitly
resubmitted.

The operation reserves the next unused generation by publishing
`channels-generation-<N>.json` without overwrite before replacing
`channels.json`. Allocation considers every retained snapshot, not only the
live index.

- Failure before reservation mutates no release state.
- Failure after reservation retains and burns generation `N`. Resume verifies
  the same request and plans, then reserves `N+1` or later.
- Exact matching version/Marketplace state is verified and reused. Missing
  planned fixture state may be supplied only before channel activation;
  mismatched existing state fails.
- Failure after index replacement never performs a second index mutation.
  Resume proves the current generation/digest equals the intended snapshot and
  starts a new correlated site attempt.
- A site timeout is terminal and releases the lease. Rollback may then
  supersede an outstanding site attempt; a later resume is accepted only while
  the realized generation/digest still matches.

No version tag, version asset, plan, report, evidence commit, request,
generation snapshot, realized site facts, attempt, or sign-off is deleted or
overwritten. Withdrawal changes selection state only.

## Recovery behavior

Fresh installs select the new active stable immediately. An affected
self-update client refuses a downgrade unless the user supplies:

```bash
sifr self update --channel stable --force
```

If the affected binary cannot run self-update, the out-of-band path is:

```bash
curl -fsSL https://sifr.sh/install/stable | sh -s -- --force
```

Both paths resolve the governed active stable record, verify the immutable
installer digest, and delegate to those exact installer bytes. A withdrawn
version is never selectable by a channel or exact pin.

## Local drill boundary

`scripts/distribution/run_incident_fixture.py` accepts only a dedicated system
temporary directory containing a filesystem index, retained governance assets,
immutable installer fixtures, a Marketplace range stub, extension metadata,
and a non-deploying site repository. It refuses production credentials, has no
network or production adapter, and cannot invoke `gh release`, `vsce publish`,
or repository dispatch. The production workflow still exposes neither incident
operation and receives no incident write permissions at this boundary.

Run the capability demo with:

```bash
demos/stable_incident_recovery_demo.sh
```

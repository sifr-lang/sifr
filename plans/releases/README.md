# Stable release evidence

Phase 40 keeps reviewed, immutable release evidence under two controlled
directories:

- `candidates/<version>/` contains the stable plan, canonical release-profile
  report, qualification artifact index, stable support claims, Rust validation
  report, documentation report, and release notes reviewed for that exact
  candidate. A later protected publication may add the matching sign-off.
- `incidents/<incident-id>/` contains one incident request and its digest-bound
  `withdrawal-evidence.txt`. A later protected incident workflow may add the
  matching sign-off.

Changes below either directory are evidence-only: they may not be combined
with compiler, workflow, script, documentation, or other source changes. The
`distribution_release:evidence-custody` suite validates path identity,
canonical JSON, schema epoch, cross-artifact source identity, and every
recorded digest.

Stable prepare never trusts a workstation artifact directory or an
operator-selected release-index generation. It refetches the six qualification
uploads by immutable GitHub artifact ID, reproduces candidate evidence from
exact clean checkouts, and allocates after every retained generation snapshot.
A reusable protected revalidation command also requires caller-supplied clean
checkouts, live index/history, and refetched artifact bytes to reproduce the
exact 30-day review artifact. The canonical protected publish job now performs
that re-fetch and byte-for-byte revalidation before any stable publication and
again immediately before release-index reservation.

For `ga-activation` and `normal`, the same job stages only the exact qualified
bytes and approved plan, creates or verifies the write-once version release,
verifies the raw Marketplace VSIX identity, activates or resumes the governed
generation, waits for the pinned site deployment, runs public install/update
smoke, and retains generation-specific site facts plus the versioned sign-off.
Existing remote bytes are reusable only in `resume` mode for the GitHub release
and only when byte-identical; an existing exact Marketplace version may be
reused in either mode after raw Gallery verification. The Marketplace
publisher is installed without lifecycle scripts from the exact candidate
submodule lockfile before publication secrets are exposed. Sign-off assets are
write-once per protected run/attempt, so repeated resume runs add evidence
without rewriting prior completion records.

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
exact 30-day review artifact. Wiring that command into the protected publish
job is the next publication wave.

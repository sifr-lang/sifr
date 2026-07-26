# Stable release evidence

Phase 40 keeps reviewed, immutable release evidence under two controlled
directories:

- `candidates/<version>/` contains one stable candidate plan, its canonical
  release-profile report, and qualification artifact index. A later protected
  publication may add the matching sign-off.
- `incidents/<incident-id>/` contains one incident request. A later protected
  incident workflow may add its matching sign-off.

Changes below either directory are evidence-only: they may not be combined
with compiler, workflow, script, documentation, or other source changes. The
`distribution_release:evidence-custody` suite validates path identity,
canonical JSON, schema epoch, cross-artifact source identity, and every
recorded digest.

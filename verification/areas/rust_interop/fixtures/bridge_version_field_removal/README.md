# bridge_version_field_removal

Future-owned Native Pydantic-Sifr `milestone_ps_2` evidence for removing the
versioned Rust bridge manifest schema.

The implementation wave replaces the current `bridge_version_mismatch` row:

- an unversioned `[rust]` manifest is accepted by the normal compiler manifest
  diagnostic path; and
- any `bridge-version` field is rejected as removed, with no rewrite,
  compatibility mode, versioned glue, or fallback.

Both planned directions must pass atomically before the removal is claimed.

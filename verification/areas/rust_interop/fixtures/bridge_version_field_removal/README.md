# bridge_version_field_removal

Reserved evidence for removing the versioned Rust bridge manifest schema.

The completed evidence must establish that:

- an unversioned `[rust]` manifest is accepted by the normal compiler manifest
  diagnostic path; and
- any `bridge-version` field is rejected as removed, with no rewrite,
  compatibility mode, versioned glue, or fallback.

Both planned directions must pass atomically before the removal is claimed.

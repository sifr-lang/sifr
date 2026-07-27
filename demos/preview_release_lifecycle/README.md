# Preview Release Lifecycle Demo

This demo exercises the read-only preview release planner against isolated
schema-v2 release-index and website fixtures. Public mutation belongs only to
the governed GitHub workflow.

## Preview plans

```bash
verification/areas/distribution_release/cases/create_new_version_alpha_dry_run.sh
verification/areas/distribution_release/cases/create_new_version_beta_dry_run.sh
```

Each case creates a clean temporary website repository and governed release
index, then renders the proposed source/site commits, index identity, next
channel version, and write-once/replace-only policies. Both fixture inputs
remain byte-for-byte unchanged.

## Read-only and mutation boundaries

```bash
verification/areas/distribution_release/cases/create_new_version_plan_is_read_only.sh
verification/areas/distribution_release/cases/create_new_version_local_mutation_rejected.sh
verification/areas/distribution_release/cases/create_new_version_artifact_mode_rejected.sh
```

The first case proves planning changes neither fixture. The negative cases prove
that removed workstation mutation and artifact modes fail closed.

## Stable boundary

```bash
verification/areas/distribution_release/cases/create_new_version_stable_rejected.sh
```

Stable publication is unavailable from the preview planner and fails before
artifact, release-index, or website mutation.

The automated lifecycle checks are in
`verification/areas/distribution_release/cases/create_new_version_*.sh`.

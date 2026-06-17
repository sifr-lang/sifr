# Preview Release Lifecycle Demo

This demo captures the local .3 release lifecycle using the same planner as production and `--mutation-mode local` to avoid publishing GitHub assets.

## Dry Run

```bash
scripts/distribution/create_new_version.sh \
  --channel beta \
  --version 0.1.0-beta.2 \
  --dry-run
```

The dry-run prints:

- resolved base commit,
- all four artifact names and checksum names,
- GitHub Release target,
- immutable installer path,
- site dispatcher mutation,
- stable entrypoint status,
- `plan_sha256`,
- `dry_run_side_effects=none`.

## Mocked Real Run

```bash
tmp_dir="$(mktemp -d)"
cat >"${tmp_dir}/sifr" <<'EOF'
#!/usr/bin/env sh
set -eu
echo "sifr release lifecycle demo"
EOF
chmod 755 "${tmp_dir}/sifr"

scripts/distribution/create_new_version.sh \
  --channel beta \
  --version 0.1.0-beta.2 \
  --real-run \
  --binary "${tmp_dir}/sifr" \
  --mutation-mode local
```

The real-run writes:

- `target/preview-release/0.1.0-beta.2/plan.txt`,
- `target/preview-release/0.1.0-beta.2/release-checklist.md`,
- `target/preview-release/0.1.0-beta.2/recovery-note.md`,
- `apps/sifr-site/public/install/versions/0.1.0-beta.2` in the site repo,
- regenerated channel dispatchers pointing beta at `0.1.0-beta.2`.

## Stable Gate

```bash
scripts/distribution/create_new_version.sh \
  --channel beta \
  --version 1.0.0 \
  --dry-run
```

The command exits non-zero before artifact, installer, release, or site mutations.

## Attribution Evidence

The generated release checklist records:

- `uv-derived code used: no`,
- `Copied/adapted uv files: none`,
- `MIT license retention required: not applicable`,
- `Pinned uv source URL/reference required: not applicable`.

The automated lifecycle checks are in
`verification/areas/distribution_release/cases/create_new_version_*.sh`.

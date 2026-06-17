# Preview Distribution Demo

This demo records the local installer artifact checksum-verified installer walkthrough.

## Build Mock Preview Artifacts

```bash
tmp_dir="$(mktemp -d)"
cat >"${tmp_dir}/sifr" <<'EOF'
#!/usr/bin/env sh
set -eu
echo "sifr preview demo"
EOF
chmod 755 "${tmp_dir}/sifr"

scripts/distribution/build_preview_artifacts.sh \
  --version 0.1.0-beta.1 \
  --output-dir "${tmp_dir}/artifacts" \
  --binary "${tmp_dir}/sifr"
```

## Generate Immutable Installer

```bash
scripts/distribution/generate_version_installer.sh \
  --version 0.1.0-beta.1 \
  --artifact-dir "${tmp_dir}/artifacts" \
  --out "${tmp_dir}/install/versions/0.1.0-beta.1" \
  --artifact-base-url "file://${tmp_dir}/artifacts"
```

## Install With Checksum Verification

```bash
SIFR_TARGET=x86_64-unknown-linux-gnu \
SIFR_ARTIFACT_BASE_URL="file://${tmp_dir}/artifacts" \
SIFR_INSTALL_DIR="${tmp_dir}/bin" \
sh "${tmp_dir}/install/versions/0.1.0-beta.1"

"${tmp_dir}/bin/sifr"
```

Expected output:

```text
sifr preview demo
```

## Checksum Failure Keeps Existing Binary

```bash
cat >"${tmp_dir}/bin/sifr" <<'EOF'
#!/usr/bin/env sh
echo "old binary"
EOF
chmod 755 "${tmp_dir}/bin/sifr"

printf 'corruption' >>"${tmp_dir}/artifacts/sifr-0.1.0-beta.1-x86_64-unknown-linux-gnu.tar.gz"

SIFR_TARGET=x86_64-unknown-linux-gnu \
SIFR_ARTIFACT_BASE_URL="file://${tmp_dir}/artifacts" \
SIFR_INSTALL_DIR="${tmp_dir}/bin" \
sh "${tmp_dir}/install/versions/0.1.0-beta.1"
```

The installer exits non-zero with a checksum mismatch before replacing `${tmp_dir}/bin/sifr`.

The automated version of this walkthrough is
`verification/areas/distribution_release/cases/artifact_sha256_validated.sh`.

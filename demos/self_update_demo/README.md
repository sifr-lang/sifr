# Self-Update Demo Transcript

This transcript records the local fixture flow used for the self-update
capability. It uses a copied development `sifr` binary, a synthetic standalone
receipt, and a fake `curl` that serves local immutable installer scripts. The
real CLI path still validates the receipt, resolves preview versions, downloads
and validates an installer script, acquires the install lock, and delegates to
the installer.

Expected output uses `<tmp>` as a placeholder for the absolute directory created
by `mktemp -d`.

## Install Fixture

```bash
cargo build -q -p sifr
tmp_dir="$(mktemp -d)"
install_dir="${tmp_dir}/install/bin"
manifest_dir="${tmp_dir}/manifest"
fake_bin="${tmp_dir}/fake-bin"
mkdir -p "${install_dir}" "${manifest_dir}" "${fake_bin}"
cp target/debug/sifr "${install_dir}/sifr"
chmod 755 "${install_dir}/sifr"
```

Create a receipt for the copied standalone binary:

```bash
binary_path="$(cd "${install_dir}" && pwd -P)/sifr"
cat >"${manifest_dir}/install.json" <<EOF
{
  "schema_version": 1,
  "name": "sifr",
  "version": "0.1.0-beta.1",
  "channel": "beta",
  "target": "aarch64-apple-darwin",
  "install_dir": "${install_dir}",
  "binary_path": "${binary_path}",
  "artifact": "sifr-0.1.0-beta.1-aarch64-apple-darwin.tar.gz",
  "modify_path": false
}
EOF
```

## Dry Run

```bash
SIFR_INSTALL_MANIFEST_DIR="${manifest_dir}" \
  "${install_dir}/sifr" self update --version 0.1.0-beta.2 --dry-run
```

Expected output includes:

```text
current_version: 0.1.0-beta.1
target_version: 0.1.0-beta.2
receipt_channel: beta
resolved_channel: beta
action: update
force: false
would_run_installer: true
```

## Update

Serve a local immutable installer through a fake `curl`:

```bash
installer="${tmp_dir}/installer.sh"
{
  printf '%s\n' '#!/bin/sh'
  printf '%s\n' 'set -eu'
  printf '%s\n' 'printf "updated=%s args=%s\n" "$SIFR_INSTALL_DIR" "$*" > "$SIFR_INSTALL_DIR/update.txt"'
  printf '%s\n' 'test -d "$SIFR_INSTALL_DIR/.sifr-update.lock"'
  printf '%s\n' 'exit 0'
  for _ in $(seq 1 160); do printf '%s\n' '# padding'; done
} >"${installer}"
chmod 755 "${installer}"

cat >"${fake_bin}/curl" <<EOF
#!/bin/sh
set -eu
out=""
while [ "\$#" -gt 0 ]; do
  if [ "\$1" = "-o" ]; then shift; out="\$1"; fi
  shift || true
done
cp "${installer}" "\${out}"
EOF
chmod 755 "${fake_bin}/curl"

PATH="${fake_bin}:$PATH" \
  SIFR_INSTALL_MANIFEST_DIR="${manifest_dir}" \
  "${install_dir}/sifr" self update --version 0.1.0-beta.2
cat "${install_dir}/update.txt"
```

Expected output:

```text
updated=<tmp>/install/bin args=
```

## No-Op

After changing the receipt version to `0.1.0-beta.2`, the same target is a no-op:

```bash
python3 - "${manifest_dir}/install.json" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
receipt = json.loads(path.read_text())
receipt["version"] = "0.1.0-beta.2"
receipt["artifact"] = "sifr-0.1.0-beta.2-aarch64-apple-darwin.tar.gz"
path.write_text(json.dumps(receipt, indent=2) + "\n")
PY

SIFR_INSTALL_MANIFEST_DIR="${manifest_dir}" \
  "${install_dir}/sifr" self update --version 0.1.0-beta.2
```

Expected output:

```text
Sifr 0.1.0-beta.2 is already installed at <tmp>/install/bin/sifr
```

## Forced Downgrade

Downgrades are rejected without `--force`:

```bash
SIFR_INSTALL_MANIFEST_DIR="${manifest_dir}" \
  "${install_dir}/sifr" self update --version 0.1.0-beta.1
```

Expected diagnostic:

```text
downgrading self-update from 0.1.0-beta.2 to 0.1.0-beta.1 requires --force
```

With `--force`, the CLI delegates to the immutable installer:

```bash
PATH="${fake_bin}:$PATH" \
  SIFR_INSTALL_MANIFEST_DIR="${manifest_dir}" \
  "${install_dir}/sifr" self update --version 0.1.0-beta.1 --force
cat "${install_dir}/update.txt"
```

Expected output:

```text
updated=<tmp>/install/bin args=--force
```

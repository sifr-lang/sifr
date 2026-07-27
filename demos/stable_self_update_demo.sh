#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-stable-self-update.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

target="$(rustc -vV | sed -n 's/^host: //p')"
case "${target}" in
  aarch64-apple-darwin|x86_64-apple-darwin|aarch64-unknown-linux-gnu|x86_64-unknown-linux-gnu) ;;
  *) echo "unsupported demo host: ${target}" >&2; exit 2 ;;
esac

build_root="${REPO_ROOT}/target/stable-self-update-demo"
SIFR_RELEASE_VERSION="0.1.0-beta.2" \
  CARGO_TARGET_DIR="${build_root}" \
  cargo build -q --locked -p sifr --bin sifr

install_root="${tmp_dir}/managed"
install_dir="${install_root}/bin"
binary_path="${install_dir}/sifr"
mkdir -p "${install_dir}" "${tmp_dir}/installers" "${tmp_dir}/fake-bin"
cp "${build_root}/debug/sifr" "${binary_path}"
chmod 755 "${binary_path}"

write_installer() {
  local version="$1"
  local channel="$2"
  local out="${tmp_dir}/installers/sifr-installer-${version}"
  cat >"${out}" <<EOF
#!/usr/bin/env sh
set -eu
python3 - "\${SIFR_INSTALL_MANIFEST_DIR}/install.json" "\${SIFR_SYSROOT_INSTALL_DIR}/sysroot.toml" "${version}" "${channel}" <<'PY'
import json
import pathlib
import sys

receipt_path, sysroot_path, version, channel = sys.argv[1:]
receipt_file = pathlib.Path(receipt_path)
receipt = json.loads(receipt_file.read_text())
receipt["version"] = version
receipt["channel"] = channel
receipt["sysroot_sifr_version"] = version
receipt["sysroot_content_sha256"] = "f" * 64
receipt_file.write_text(
    json.dumps(receipt, sort_keys=True, separators=(",", ":")) + "\\n"
)
pathlib.Path(sysroot_path).write_text(
    "\\n".join(
        [
            "schema-version = 1",
            f'sifr-version = "{version}"',
            f'target-triple = "{receipt["target"]}"',
            'sysroot-content-sha256 = "' + "f" * 64 + '"',
            "",
        ]
    )
)
PY
# Padding keeps this fixture above the production runner's tiny-download guard.
# The immutable SHA-256 in the governed index covers every byte of this file.
# ---------------------------------------------------------------------------
# stable-self-update-demo-padding-000000000000000000000000000000000000000
# stable-self-update-demo-padding-111111111111111111111111111111111111111
# stable-self-update-demo-padding-222222222222222222222222222222222222222
# stable-self-update-demo-padding-333333333333333333333333333333333333333
# stable-self-update-demo-padding-444444444444444444444444444444444444444
# stable-self-update-demo-padding-555555555555555555555555555555555555555
# stable-self-update-demo-padding-666666666666666666666666666666666666666
# stable-self-update-demo-padding-777777777777777777777777777777777777777
EOF
  chmod 755 "${out}"
}

write_index() {
  local stable_version="$1"
  local out="$2"
  python3 - \
    "${out}" \
    "${stable_version}" \
    "${tmp_dir}/installers/sifr-installer-${stable_version}" \
    "${target}" <<'PY'
import hashlib
import json
import pathlib
import sys

out, stable_version, installer, target = sys.argv[1:]
targets = [
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "aarch64-unknown-linux-gnu",
    "x86_64-unknown-linux-gnu",
]

def record(channel, digest):
    return {
        "channel": channel,
        "status": "active",
        "source_commit": "e" * 40,
        "installer_sha256": digest,
        "targets": {
            item: {
                "artifact_sha256": "a" * 64,
                "sysroot_content_sha256": "b" * 64,
            }
            for item in targets
        },
    }

stable_digest = hashlib.sha256(pathlib.Path(installer).read_bytes()).hexdigest()
value = {
    "schema_version": 2,
    "generation": 1 if stable_version == "0.1.0" else 2,
    "ga_status": "active",
    "channels": {
        "alpha": "0.1.0-alpha.1",
        "beta": "0.1.0-beta.2",
        "stable": stable_version,
    },
    "releases": {
        "0.1.0-alpha.1": record("alpha", "c" * 64),
        "0.1.0-beta.2": record("beta", "d" * 64),
        stable_version: record("stable", stable_digest),
    },
}
pathlib.Path(out).write_text(
    json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n"
)
PY
}

write_installer "0.1.0" stable
write_installer "0.1.1" stable
write_index "0.1.0" "${tmp_dir}/channels-1.json"
write_index "0.1.1" "${tmp_dir}/channels-2.json"

cat >"${tmp_dir}/fake-bin/curl" <<'EOF'
#!/usr/bin/env sh
set -eu
output=""
url=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    -o) output="$2"; shift 2 ;;
    http*) url="$1"; shift ;;
    *) shift ;;
  esac
done
case "${url}" in
  */channels/channels.json)
    if [ -n "${output}" ]; then
      cp "${SIFR_DEMO_INDEX}" "${output}"
    else
      cat "${SIFR_DEMO_INDEX}"
    fi
    ;;
  */sifr-installer-*)
    name="${url##*/}"
    cp "${SIFR_DEMO_INSTALLERS}/${name}" "${output}"
    ;;
  *) echo "unexpected demo URL: ${url}" >&2; exit 2 ;;
esac
EOF
chmod 755 "${tmp_dir}/fake-bin/curl"

cat >"${install_root}/sysroot.toml" <<EOF
schema-version = 1
sifr-version = "0.1.0-beta.2"
target-triple = "${target}"
sysroot-content-sha256 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
EOF
python3 - \
  "${install_root}/install.json" \
  "${install_dir}" \
  "${binary_path}" \
  "${install_root}" \
  "${target}" <<'PY'
import json
import pathlib
import sys

out, install_dir, binary_path, install_root, target = sys.argv[1:]
value = {
    "schema_version": 2,
    "name": "sifr",
    "version": "0.1.0-beta.2",
    "channel": "beta",
    "target": target,
    "install_dir": install_dir,
    "binary_path": binary_path,
    "sysroot_path": install_root,
    "sysroot_schema_version": 1,
    "sysroot_sifr_version": "0.1.0-beta.2",
    "sysroot_target_triple": target,
    "sysroot_content_sha256": "a" * 64,
    "artifact": f"sifr-0.1.0-beta.2-{target}.tar.gz",
    "modify_path": False,
}
pathlib.Path(out).write_text(
    json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n"
)
PY

show_managed_state() {
  python3 - "${install_root}/install.json" "${install_root}/sysroot.toml" <<'PY'
import json
import pathlib
import re
import sys

receipt = json.loads(pathlib.Path(sys.argv[1]).read_text())
sysroot = pathlib.Path(sys.argv[2]).read_text()
match = re.search(r'^sifr-version = "([^"]+)"$', sysroot, re.MULTILINE)
if match is None or match.group(1) != receipt["version"]:
    raise SystemExit("receipt and sysroot versions diverged")
print(
    f'receipt={receipt["channel"]}:{receipt["version"]} '
    f'sysroot={match.group(1)}'
)
PY
}

export PATH="${tmp_dir}/fake-bin:${PATH}"
export SIFR_DEMO_INSTALLERS="${tmp_dir}/installers"
export SIFR_INSTALL_MANIFEST_DIR="${install_root}"

echo "1. Forced beta-to-stable switch through the immutable installer"
SIFR_DEMO_INDEX="${tmp_dir}/channels-1.json" \
  "${binary_path}" self update --channel stable --force
show_managed_state

echo "2. Ordinary stable-to-stable update without --force"
SIFR_DEMO_INDEX="${tmp_dir}/channels-2.json" \
  "${binary_path}" self update
show_managed_state

echo "3. Current stable is a no-op"
SIFR_DEMO_INDEX="${tmp_dir}/channels-2.json" \
  "${binary_path}" self update --dry-run

echo "4. The public preview workflow still has no stable input"
python3 - "${REPO_ROOT}/.github/workflows/preview-release.yml" <<'PY'
import pathlib
import sys

workflow = pathlib.Path(sys.argv[1]).read_text()
if "options:\n          - alpha\n          - beta" not in workflow:
    raise SystemExit("preview workflow channel choices drifted")
if "uses: ./.github/workflows/release-publication.yml" not in workflow:
    raise SystemExit("preview workflow bypasses governed publication")
print("preview-publication=alpha|beta only")
PY

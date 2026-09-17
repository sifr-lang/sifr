#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/common.sh"
tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT
sha256_file() {
  python3 -c 'import hashlib,sys; print(hashlib.sha256(open(sys.argv[1],"rb").read()).hexdigest())' "$1"
}
install_root="$tmp_dir/install"
install_version() {
  local version="$1"
  local artifacts="$tmp_dir/$version"
  make_target_specific_artifacts "$version" "$artifacts"
  "$REPO_ROOT/scripts/distribution/generate_version_installer.sh" --version "$version" \
    --artifact-dir "$artifacts" --out "$artifacts/install.sh" \
    --artifact-base-url "file://$artifacts" >/dev/null
  SIFR_TARGET=x86_64-unknown-linux-gnu SIFR_INSTALL_DIR="$install_root/bin" \
    SIFR_NO_MODIFY_PATH=1 sh "$artifacts/install.sh" --force --no-modify-path
}
install_version 0.1.0-beta.3
first="$(readlink "$install_root/.sifr-current")"
first_binary="$(sha256_file "$install_root/$first/bin/sifr")"
install_version 0.1.0-beta.4
second="$(readlink "$install_root/.sifr-current")"
test "$first" != "$second"
test "$first_binary" = "$(sha256_file "$install_root/$first/bin/sifr")"
test -d "$install_root/$first/lib/sifr/stdlib/sifr"
test -f "$install_root/$first/install.json"
install_version 0.1.0-beta.3
third="$(readlink "$install_root/.sifr-current")"
test "$second" != "$third"
test -d "$install_root/$first"
test -d "$install_root/$second"
python3 - "$install_root" <<'PY'
import json,sys
from pathlib import Path
root=Path(sys.argv[1])
receipt=json.loads((root/"install.json").read_text())
assert Path(receipt["binary_path"]) == (root/"bin/sifr").resolve()
assert Path(receipt["sysroot_path"]) == (root/"sysroot.toml").resolve().parent
assert receipt["version"]=="0.1.0-beta.3"
PY

#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/common.sh"
fixture_root="$(mktemp -d "${TMPDIR:-/tmp}/sifr-native-installer.XXXXXX")"
fixture_root="$(cd "$fixture_root" && pwd -P)"
trap 'rm -rf "$fixture_root"' EXIT
version=0.1.0-beta.17
target=x86_64-unknown-linux-gnu
artifacts="$fixture_root/artifacts"
make_target_specific_artifacts "$version" "$artifacts"
for other in aarch64-apple-darwin x86_64-apple-darwin aarch64-unknown-linux-gnu; do
  rm "$artifacts/sifr-$version-$other.tar.gz" "$artifacts/sifr-$version-$other.tar.gz.sha256"
done
generator="$REPO_ROOT/scripts/distribution/generate_version_installer.sh"
require_failure_contains 'artifact missing for target' "$generator" --version "$version" \
  --artifact-dir "$artifacts" --out "$fixture_root/full-installer"
require_failure_contains 'unsupported qualification target' "$generator" --version "$version" \
  --artifact-dir "$artifacts" --out "$fixture_root/bad-installer" --qualification-target unknown
"$generator" --version "$version" --artifact-dir "$artifacts" --out "$fixture_root/native-installer" \
  --qualification-target "$target" --artifact-base-url "file://$artifacts" >/dev/null
rg -F 'Artifact scope: single-target-qualification' "$fixture_root/native-installer" >/dev/null
SIFR_TARGET="$target" SIFR_INSTALL_DIR="$fixture_root/managed/bin" SIFR_NO_MODIFY_PATH=1 \
  sh "$fixture_root/native-installer" --no-modify-path >/dev/null
rg -F "target=$target" "$fixture_root/managed/bin/sifr" >/dev/null
require_failure_contains 'unsupported target:' env SIFR_TARGET=aarch64-unknown-linux-gnu \
  SIFR_INSTALL_DIR="$fixture_root/other/bin" SIFR_NO_MODIFY_PATH=1 sh "$fixture_root/native-installer" --no-modify-path

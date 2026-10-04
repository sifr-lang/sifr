#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/common.sh"

temporary="$(mktemp -d "${TMPDIR:-/tmp}/sifr-legacy-migration.XXXXXX")"
temporary="$(cd "$temporary" && pwd -P)"
trap 'rm -rf "$temporary"' EXIT
target="x86_64-unknown-linux-gnu"
previous="0.1.0-beta.2"
candidate="0.1.0-beta.3"
make_target_specific_artifacts "$previous" "$temporary/previous"
make_target_specific_artifacts "$candidate" "$temporary/candidate"
"$REPO_ROOT/scripts/distribution/generate_version_installer.sh" --version "$candidate" \
  --artifact-dir "$temporary/candidate" --out "$temporary/install.sh" \
  --artifact-base-url "file://$temporary/candidate" >/dev/null

seed_flat() {
  local root="$1"
  mkdir -p "$root"
  tar -xzf "$temporary/previous/sifr-$previous-$target.tar.gz" -C "$root"
  printf '#!/bin/sh\nprintf "sifr %s\\n"\n' "$previous" >"$root/bin/sifr"
  chmod 755 "$root/bin/sifr"
  printf '{"schema_version":2,"name":"sifr","version":"%s","target":"%s","binary_path":"%s/bin/sifr","sysroot_path":"%s"}\n' \
    "$previous" "$target" "$root" "$root" >"$root/install.json"
}
run_candidate() {
  local root="$1"
  SIFR_TARGET="$target" SIFR_INSTALL_DIR="$root/bin" SIFR_NO_MODIFY_PATH=1 \
    sh "$temporary/install.sh" --no-modify-path
}
snapshot() {
  python3 - "$1" <<'PY'
import hashlib,json,pathlib,sys
root=pathlib.Path(sys.argv[1])
files={str(path.relative_to(root)):hashlib.sha256(path.read_bytes()).hexdigest()
       for path in root.rglob('*') if path.is_file() and '.sifr-generations' not in path.relative_to(root).parts}
print(json.dumps(files,sort_keys=True))
PY
}
assert_flat_restored() {
  local root="$1" expected="$2"
  [[ "$(snapshot "$root")" == "$expected" ]]
  [[ ! -e "$root/.sifr-current" && ! -L "$root/.sifr-current" ]]
  [[ -z "$(find "$root" -path "$root/.sifr-generations" -prune -o -type l -print)" ]]
}

success="$temporary/success"
seed_flat "$success"
before="$(snapshot "$success")"
require_failure_contains "SIFR_MIGRATE_LEGACY=1" run_candidate "$success"
assert_flat_restored "$success" "$before"
SIFR_MIGRATE_LEGACY=1 run_candidate "$success" >/dev/null
[[ -L "$success/.sifr-current" && -L "$success/bin/sifr" ]]
grep -F "\"version\": \"$candidate\"" "$success/install.json" >/dev/null
legacy="$(find "$success/.sifr-generations" -maxdepth 1 -type d -name 'legacy.*')"
grep -F "\"version\":\"$previous\"" "$legacy/install.json" >/dev/null
[[ "$(snapshot "$legacy")" == "$before" ]]

incomplete="$temporary/incomplete"
seed_flat "$incomplete"
rm "$incomplete/Cargo.lock"
before="$(snapshot "$incomplete")"
require_failure_contains "complete flat toolchain" env SIFR_MIGRATE_LEGACY=1 \
  SIFR_TARGET="$target" SIFR_INSTALL_DIR="$incomplete/bin" SIFR_NO_MODIFY_PATH=1 sh "$temporary/install.sh"
assert_flat_restored "$incomplete" "$before"

# Wrong receipt ownership and nested links must refuse before moving payloads.
for invalid in receipt_target receipt_root nested_link special_file wrong_type; do
  root="$temporary/$invalid"
  seed_flat "$root"
  if [[ "$invalid" == nested_link ]]; then
    ln -s "$root/Cargo.lock" "$root/vendor/foreign-link"
  elif [[ "$invalid" == special_file ]]; then
    mkfifo "$root/vendor/foreign-pipe"
  elif [[ "$invalid" == wrong_type ]]; then
    rm "$root/Cargo.toml"
    mkdir "$root/Cargo.toml"
  else
    python3 - "$root" "$invalid" <<'PYCONTROL'
import json,pathlib,sys
receipt=pathlib.Path(sys.argv[1])/'install.json'
payload=json.loads(receipt.read_text())
payload['target' if sys.argv[2]=='receipt_target' else 'sysroot_path']=('aarch64-apple-darwin' if sys.argv[2]=='receipt_target' else '/other/root')
receipt.write_text(json.dumps(payload,indent=2)+'\n')
PYCONTROL
  fi
  before="$(snapshot "$root")"
  case "$invalid" in
    receipt_target) expected="receipt target differs" ;;
    receipt_root) expected="receipt belongs to another sysroot" ;;
    nested_link|special_file) expected="refuses toolchain symlinks or special files" ;;
    wrong_type) expected="complete flat toolchain file" ;;
  esac
  require_failure_contains "$expected" env SIFR_MIGRATE_LEGACY=1 \
    SIFR_TARGET="$target" SIFR_INSTALL_DIR="$root/bin" SIFR_NO_MODIFY_PATH=1 sh "$temporary/install.sh"
  [[ "$(snapshot "$root")" == "$before" ]]
  [[ ! -e "$root/.sifr-current" && ! -L "$root/.sifr-current" ]]
done

faults="$temporary/fault-tools"
mkdir "$faults"
real_cp="$(command -v cp)"
real_mv="$(command -v mv)"
cat >"$faults/cp" <<EOF
#!/bin/sh
case "\$1:\$2" in
  */install.json:*/.sifr-generations/$candidate-*/install.json)
    if [ "\${SIFR_TEST_MIGRATION_POINT:-}" = "receipt" ]; then exit 71; fi ;;
esac
"$real_cp" "\$@" || exit \$?
if [ "\${SIFR_TEST_MIGRATION_POINT:-}" = "stage" ]; then
  case "\$1:\$3" in -R:*/.sifr-generations/.stage.*) kill -TERM "\$PPID" ;; esac
fi
exit 0
EOF
cat >"$faults/mv" <<EOF
#!/bin/sh
"$real_mv" "\$@" || exit \$?
if [ "\${SIFR_TEST_MIGRATION_POINT:-}" = "rename" ]; then
  case "\$2" in */.sifr-generations/legacy.*/Cargo.lock) kill -TERM "\$PPID" ;; esac
fi
if [ "\${SIFR_TEST_MIGRATION_POINT:-}" = "selector" ]; then
  for argument in "\$@"; do
    case "\$argument" in */.sifr-current) kill -TERM "\$PPID" ;; esac
  done
fi
exit 0
EOF
chmod 700 "$faults/cp" "$faults/mv"
for fault in receipt rename stage selector; do
  root="$temporary/$fault"
  seed_flat "$root"
  before="$(snapshot "$root")"
  set +e
  PATH="$faults:$PATH" SIFR_MIGRATE_LEGACY=1 \
    SIFR_TEST_MIGRATION_POINT="$fault" \
    run_candidate "$root" >"$temporary/$fault.stdout" 2>"$temporary/$fault.stderr"
  code=$?
  set -e
  if [[ "$fault" == receipt ]]; then [[ "$code" != 0 ]]; else [[ "$code" == 143 ]]; fi
  assert_flat_restored "$root" "$before"
done
echo 'legacy migration: explicit ownership, retained payload, refusal and transaction/signal rollback passed'

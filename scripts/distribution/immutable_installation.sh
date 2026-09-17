# Embedded in the version installer. Generations are never rewritten or removed:
# a running compiler or language server may still use any prior generation.
atomic_generation_switch() {
  selection="$1"
  pending="${sysroot_dir}/.sifr-current.$$.tmp"
  ln -s "${selection}" "${pending}"
  case "$(uname -s)" in
    Darwin) mv -fh "${pending}" "${sysroot_dir}/.sifr-current" ;;
    Linux) mv -fT "${pending}" "${sysroot_dir}/.sifr-current" ;;
    *) rm -f "${pending}"; fail "unsupported atomic generation switch host" ;;
  esac
}
rollback_install_transaction() {
  if [ "${rollback_active}" != "1" ]; then return 0; fi
  if [ -n "${previous_generation}" ]; then
    atomic_generation_switch "${previous_generation}"
  else
    rm -f "${sysroot_dir}/.sifr-current"
  fi
  rollback_active=0
}
validate_extracted_toolchain
acquire_install_lock
mkdir -p "${install_dir}" "${sysroot_dir}/.sifr-generations"
# A legacy mutable installation must be moved aside explicitly by its owner.
# Silently mutating it could break a still-running pre-generation process.
for relative in .cargo vendor crates lib Cargo.toml Cargo.lock sysroot.toml bin/sifr; do
  destination="${sysroot_dir}/${relative}"
  if [ -e "${destination}" ] && [ ! -L "${destination}" ]; then
    fail "mutable installation at ${destination}; select an empty install root for immutable toolchain generations"
  fi
done
previous_generation=""
if [ -L "${sysroot_dir}/.sifr-current" ]; then
  previous_generation="$(readlink "${sysroot_dir}/.sifr-current")"
fi
generation_base="${APP_VERSION}-${target}-${archive_sha256}"
stage="$(mktemp -d "${sysroot_dir}/.sifr-generations/.stage.XXXXXX")"
cp -R "${extract_dir}/." "${stage}/"
chmod 755 "${stage}/bin/sifr"
generation_name="${generation_base}-$(basename "${stage}")"
generation_root="${sysroot_dir}/.sifr-generations/${generation_name}"
mv "${stage}" "${generation_root}"
for relative in .cargo vendor crates lib Cargo.toml Cargo.lock sysroot.toml bin/sifr; do
  destination="${sysroot_dir}/${relative}"
  case "${relative}" in
    bin/sifr) link_target="../.sifr-current/bin/sifr" ;;
    *) link_target=".sifr-current/${relative}" ;;
  esac
  if [ -L "${destination}" ]; then
    [ "$(readlink "${destination}")" = "${link_target}" ] || fail "unexpected managed installation link: ${destination}"
  else
    ln -s "${link_target}" "${destination}"
  fi
done
rollback_active=1
atomic_generation_switch ".sifr-generations/${generation_name}"
write_install_manifest
cp "$manifest_path" "$generation_root/install.json"
rollback_active=0

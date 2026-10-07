# One-time migration is explicit because old compilers resolve a mutable root.
# The owner must first stop every compiler/LSP using that root. Ordinary updates
# continue to refuse mutable installations. The install lock prevents competing
# installers, but is not a substitute for the owner's quiescence declaration.
prepare_legacy_migration() {
  legacy_present=0
  for relative in .cargo vendor crates lib Cargo.toml Cargo.lock sysroot.toml bin/sifr; do
    destination="${sysroot_dir}/${relative}"
    if [ -e "${destination}" ] && [ ! -L "${destination}" ]; then legacy_present=1; fi
  done
  [ "${legacy_present}" = "1" ] || return 0
  if [ "${SIFR_MIGRATE_LEGACY:-0}" != "1" ]; then
    fail "mutable installation at ${sysroot_dir}; select an empty install root for immutable toolchain generations, or stop its compilers/LSP and explicitly set SIFR_MIGRATE_LEGACY=1"
  fi
  [ "${install_dir}" = "${sysroot_dir}/bin" ] || fail "legacy migration requires the standard root/bin layout"
  [ -f "${manifest_path}" ] && [ ! -L "${manifest_path}" ] || fail "legacy migration requires the original regular install receipt"
  [ ! -e "${sysroot_dir}/.sifr-current" ] && [ ! -L "${sysroot_dir}/.sifr-current" ] || fail "legacy migration refuses a mixed installation layout"
  legacy_field() {
    sed -n 's/^.*"'"$1"'"[[:space:]]*:[[:space:]]*"\([^"]*\)".*$/\1/p' "${manifest_path}" | head -n 1
  }
  grep '"schema_version"[[:space:]]*:[[:space:]]*2[[:space:]]*[,}]' "${manifest_path}" >/dev/null || fail "legacy migration requires an installer schema-v2 receipt"
  [ "$(legacy_field name)" = "sifr" ] || fail "legacy receipt does not identify Sifr"
  [ "$(legacy_field target)" = "${target}" ] || fail "legacy receipt target differs from requested installation"
  [ "$(legacy_field binary_path)" = "$(canonical_path "${installed_binary}")" ] || fail "legacy receipt belongs to another executable"
  [ "$(legacy_field sysroot_path)" = "$(canonical_path "${sysroot_dir}")" ] || fail "legacy receipt belongs to another sysroot"
  [ "$(manifest_version "${manifest_path}")" = "${installed_version}" ] || fail "legacy receipt version differs from installed compiler"
  [ "$("${installed_binary}" --version)" = "sifr ${installed_version}" ] || fail "legacy compiler version differs from its receipt"
  [ "$(toml_string_field sifr-version "${sysroot_dir}/sysroot.toml")" = "${installed_version}" ] || fail "legacy sysroot version differs from its receipt"
  [ "$(toml_string_field target-triple "${sysroot_dir}/sysroot.toml")" = "${target}" ] || fail "legacy sysroot target differs from its receipt"
  for relative in .cargo vendor crates lib Cargo.toml Cargo.lock sysroot.toml bin/sifr; do
    destination="${sysroot_dir}/${relative}"
    case "${relative}" in
      .cargo|vendor|crates|lib) [ -d "${destination}" ] || fail "legacy migration requires a complete flat toolchain directory: ${destination}" ;;
      *) [ -f "${destination}" ] || fail "legacy migration requires a complete flat toolchain file: ${destination}" ;;
    esac
    [ -e "${destination}" ] && [ ! -L "${destination}" ] || fail "legacy migration requires a complete flat toolchain: ${destination}"
    if find "${destination}" ! -type f ! -type d -print -quit | grep . >/dev/null 2>&1; then
      fail "legacy migration refuses toolchain symlinks or special files: ${destination}"
    fi
  done
  legacy_backup_root="$(mktemp -d "${sysroot_dir}/.sifr-generations/legacy.XXXXXX")"
  cp "${manifest_path}" "${legacy_backup_root}/install.json"
  # Arm rollback before the first rename. Rollback discovers moved entries from
  # their actual presence, so a signal between rename and bookkeeping is safe.
  rollback_active=1
  for relative in .cargo vendor crates lib Cargo.toml Cargo.lock sysroot.toml bin/sifr; do
    mkdir -p "$(dirname "${legacy_backup_root}/${relative}")"
    mv "${sysroot_dir}/${relative}" "${legacy_backup_root}/${relative}"
  done
}

restore_legacy_installation() {
  for relative in .cargo vendor crates lib Cargo.toml Cargo.lock sysroot.toml bin/sifr; do
    backup="${legacy_backup_root}/${relative}"
    destination="${sysroot_dir}/${relative}"
    if [ -e "${backup}" ]; then
      if [ -L "${destination}" ]; then
        case "${relative}" in
          bin/sifr) expected="../.sifr-current/bin/sifr" ;;
          *) expected=".sifr-current/${relative}" ;;
        esac
        [ "$(readlink "${destination}")" = "${expected}" ] || fail "legacy rollback refuses an unexpected link: ${destination}"
        rm "${destination}"
      fi
      [ ! -e "${destination}" ] || fail "legacy rollback refuses an unexpected path: ${destination}"
      mv "${backup}" "${destination}"
    fi
  done
  if [ -L "${sysroot_dir}/.sifr-current" ]; then rm "${sysroot_dir}/.sifr-current"; fi
  # Restore the exact old receipt even if a later failure followed publication.
  if ! cmp -s "${legacy_backup_root}/install.json" "${manifest_path}"; then
    restored_receipt="$(mktemp "${manifest_dir}/.legacy-receipt.XXXXXX")"
    cp "${legacy_backup_root}/install.json" "${restored_receipt}"
    mv "${restored_receipt}" "${manifest_path}"
  fi
}

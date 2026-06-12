Reviewing the M3 changes against the phase contract for `ad-hoc-self-update-m3`.

## Verdict: READY

The M3 implementation faithfully delivers the installer-delegation contract. Below is what I confirmed and a handful of non-blocking observations.

## Contract compliance ✓

- **Download path**: only the immutable installer is fetched; URL derived from `INSTALL_BASE_URL` constant in `self_update_metadata.rs:8` and `PreviewVersion::installer_url`; no runtime override path exists.
- **TLS**: `curl -fsSL` keeps default cert verification; no `-k`/`--insecure`.
- **Atomic rename**: partial → `installer.sh` via `fs::rename` (`self_update_runner.rs:94`).
- **Size + shebang validation** before execution: `MIN_INSTALLER_BYTES = 1024` (strict `<`) and `first_line.starts_with(b"#!")` (`self_update_runner.rs:158-193`).
- **Lock**: `<install_dir>/.sifr-update.lock` acquired via atomic `mkdir`, dropped on `Drop` (`self_update_runner.rs:226-255`). Released on success AND on failure-Err paths.
- **Installer env**: `SIFR_INSTALL_DIR`, `SIFR_INSTALL_LOCK_HELD=1` (always after lock acquisition), `SIFR_NO_MODIFY_PATH=1` (only when `modify_path==false`), `SIFR_INSTALL_MANIFEST_DIR` (only when canonical receipt path differs from default), and `--force` (only when requested). Default-home special case for `~/.sifr/install.json` is preserved.
- **Output preservation**: `Command::status()` inherits stdout/stderr; failure path surfaces installer exit code as the CLI exit code with a diagnostic appended afterward (`self_update_runner.rs:50-65`).
- **No archive download from CLI**: confirmed; runner only fetches the installer script.
- **Installer template handoff**: `INSTALL_LOCK_HELD` short-circuits both `acquire_install_lock` and `release_install_lock`, and fails closed if the marker is set but the lock directory is missing (`generate_version_installer.sh:344-362`).
- **External-lock verification**: `artifact_self_update_receipt_contract.sh:82-101` pre-creates the lock, runs the installer with `SIFR_INSTALL_LOCK_HELD=1`, then asserts the caller-owned lock survives and the binary still installs.
- **Docs**: `internal_docs/distribution_pipeline.md:184` adds the lock-acquisition + handoff paragraph. Issue checklist updated to mark M2 merged and M3 in-progress.

## Non-blocking observations

1. **Diagnostic codes are intentionally coarse.** Every runner error is `SELF_UPDATE_UNMANAGED_RECEIPT` (SIFR-BUILD-0901), so "installer exited with status …" and "downloaded installer too small" share a code whose label says "unmanaged receipt." The phase contract explicitly reserves the family but does not require per-case codes ("A dedicated CLI diagnostic family is out of scope for this ad hoc phase"), so this is contract-compliant. Worth opening a follow-up to split SIFR-BUILD-09xx into download / execution / lock subcodes during M5 or a later planning PR — current behavior is acceptable.
2. **Stale-lock recovery.** If a previous run is SIGKILLed mid-update, the lock directory persists and the next run busy-waits forever at 100ms intervals. Not required by the M3 contract, but worth queueing as M4/M5 polish (e.g., refresh-timestamp + age-based reclamation).
3. **Concurrent manual + self-update test gap.** Contract's Integration tests list ("manual installer invocation and self-update invocation serialize on the same install lock and cannot produce a binary/receipt mismatch") is mechanically guaranteed because both call sites use the same lock path + atomic `mkdir`, and `artifact_self_update_receipt_contract.sh` exercises the handoff. But no test actually races a manual installer against a runner. Consider adding one alongside the existing `serializes_concurrent_updates_on_install_lock` to make the property explicit.
4. **Defense-in-depth for curl redirects.** `curl -fsSL` follows redirects without `--proto =https --proto-redir =https`. The trusted base is HTTPS, so a misconfigured redirect is the only failure mode; adding the proto pinning is cheap insurance.
5. **`reviews/self-update-m3-review-pass-1.md`** exists as a 1-line file — looks like a placeholder you'll fill in with this review.

None of the above blocks the M3 PR. The runner module, CLI wiring, installer-template handoff, and verification script all match the contract; the test suite (38 self-update tests + distribution validation) covers the explicit M3 definition-of-done items.

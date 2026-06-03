I have everything I need.

## Verdict: SATISFIED

Every M3 contract item is implemented and locally validated:

- **Immutable-installer-only download** — `self_update_runner.rs:75` uses `plan.target_version.installer_url()`, which is built from the compile-time `INSTALL_BASE_URL` constant (`self_update_metadata.rs:8,86`). No archives or metadata-supplied URLs are reachable.
- **TLS + redirect pinning** — `curl -fsSL --proto =https --proto-redir =https` (`self_update_runner.rs:78-82`); no `-k`, no insecure fallback. Pass-1 advisory addressed.
- **Temp download + atomic rename** — partial path → `installer.sh` via `fs::rename` (`self_update_runner.rs:73-108`); `TempWorkDir` cleans up on Drop.
- **Pre-execution gating** — `validate_installer` enforces `< 1024 bytes` and `#!` shebang (`self_update_runner.rs:166-202`) before `make_executable` and before lock acquisition.
- **Lock acquisition + scope** — `InstallLock::acquire` (`self_update_runner.rs:234-256`) uses atomic `mkdir`; lifetime spans only `run_installer`, and `Drop` releases on both success and error paths.
- **Generated-installer handoff fails closed** — `acquire_install_lock` errors when `SIFR_INSTALL_LOCK_HELD=1` but the lock dir is absent (`generate_version_installer.sh:344-353`), and `release_install_lock` short-circuits the same marker so the caller-owned lock survives.
- **Env passthrough** — `SIFR_INSTALL_DIR`, `SIFR_INSTALL_LOCK_HELD=1`, conditional `SIFR_NO_MODIFY_PATH`, conditional `SIFR_INSTALL_MANIFEST_DIR` (with the documented `~/.sifr/install.json` default-home exception), and `--force` only when requested (`self_update_runner.rs:111-153`).
- **Stdout/stderr + exit status preserved** — `Command::status()` inherits std streams; failure path surfaces installer exit code as `error.exit_code` and `cmd_update` propagates it (`self_update_cli.rs:129-136`).
- **Structured diagnostics** — every error returns a `RenderedDiagnostic` via `runner_error_with_exit`.
- **Tests cover DoD** — env+force+manifest override, default-home manifest omission, tiny-download rejection, no-shebang rejection, installer-failure exit-code mapping, concurrent-update serialization, no-op skipping download/lock (`self_update_runner.rs:471-617`); plus the external-lock-handoff scenario in `artifact_self_update_receipt_contract.sh:82-101`.

## Non-blocking residual risks / follow-ups

1. **Coarse diagnostic family.** All runner errors reuse `SELF_UPDATE_UNMANAGED_RECEIPT` (download-too-small, no-shebang, installer-exit, lock-acquire). The phase contract explicitly defers a dedicated family, so this is compliant, but worth splitting in M5 so the label matches the failure.
2. **Stale-lock recovery is indefinite busy-wait.** If a previous process is SIGKILLed mid-update, `InstallLock::acquire` polls forever at 100 ms. Add age-based reclamation or a max-wait diagnostic in M4/M5.
3. **No explicit manual-installer-vs-self-update race test.** The lock-path identity is mechanically guaranteed and the handoff is covered by the verification script, but the contract's "manual installer invocation and self-update invocation serialize" bullet has no test that literally races the two. Consider extending `serializes_concurrent_updates_on_install_lock` to start one runner and one direct `sh installer.sh` invocation.
4. **`validate_installer` has no upper size cap.** Trusted HTTPS + immutable URL makes a runaway-size attack unrealistic, but a max-bytes guard before reading the first line would be cheap defense-in-depth.
5. **`reviews/self-update-m3-review-pass-2.md` is empty (1 line).** Looks like an unfilled placeholder — should be populated with this review or deleted before the M3 PR is opened.
6. **Curl-missing path leaks message via the `SELF_UPDATE_UNMANAGED_RECEIPT` label.** Same observation as (1); the message itself is clear ("could not run curl …"), only the diagnostic code is mismatched.

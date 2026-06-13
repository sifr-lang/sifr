Second review pass for M3 (`ad-hoc-self-update-m3`) after the pass-1 refinements.

## Verdict: READY

The three pass-1 refinements are correctly applied, no contract regressions, and the 38-test self-update suite plus distribution validation continues to enforce the M3 properties.

## Pass-1 → pass-2 refinements verified

1. **Direct shebang-validated installer execution.** `self_update_runner.rs:117` invokes `Command::new(installer_path)` directly instead of routing through `sh`. The OS resolves the interpreter from the installer's shebang line, which is the `#!/usr/bin/env sh` the generated template emits (`generate_version_installer.sh:114`). Pre-execution this is gated by:
   - shebang validation at `self_update_runner.rs:195` (`first_line.starts_with(b"#!")`),
   - `make_executable` chmod `|= 0o700` at `self_update_runner.rs:217` (Unix only; Windows is out of scope per the phase contract).

   This is strictly tighter than `sh installer.sh` was: the kernel honors the file's declared interpreter (which we just validated exists) instead of going through whichever `sh` is on `PATH`. No regression — every existing test fixture writes a shebang-prefixed file and continues to pass.

2. **NoOp short-circuit inside the runner.** `self_update_runner.rs:39-41` returns `Ok(0)` before any temp dir, network, or lock work when `plan.action == UpdateAction::NoOp`. The CLI at `self_update_cli.rs:119-127` already prints the "already installed" message and returns `EXIT_SUCCESS` for the NoOp case, so the runner branch is defense in depth, not a behavior change. The new `no_op_plan_skips_download_and_lock` test (`self_update_runner.rs:601-617`) points the runner at a deliberately-missing curl binary and confirms it never reaches the download step.

3. **curl protocol pinning.** `self_update_runner.rs:77-86` adds `--proto =https --proto-redir =https` alongside `-fsSL`. The `=https` syntax restricts the allowed set to exactly HTTPS for both the initial request and any redirect, so a misconfigured trusted-base redirect cannot downgrade to HTTP. Args are passed as discrete slice entries so curl receives `--proto`/`=https` and `--proto-redir`/`=https` as separate tokens. No regression; default TLS verification is unchanged and no `-k`/`--insecure` is added. This resolves the pass-1 advisory.

## Contract re-check

- **Trusted URL derivation only.** `installer_url()` at `self_update_metadata.rs:86-88` is the sole source for the runner's URL and is built from the compile-time `INSTALL_BASE_URL = "https://sifr.sh/install"` plus the resolved version string. No production runtime override exists; metadata is resolution-only and never produces URLs.
- **Normal TLS, no insecure bypass.** curl runs with `-fsSL --proto =https --proto-redir =https`; no `-k`, `--insecure`, or `cacert` override.
- **Temp download + atomic rename + pre-execution validation.** Order: write to `installer.download`, `fs::rename` to `installer.sh` (atomic on the single `TempWorkDir`), then size ≥ 1024 bytes (the `<` predicate makes "smaller than 1024" the rejected case as worded in the contract), then shebang prefix, then chmod, then exec (`self_update_runner.rs:44-46, 102-108, 166-202`).
- **Lock acquisition + generated-installer handoff.** Runner acquires `<install_dir>/.sifr-update.lock` via atomic `fs::create_dir`, env-passes `SIFR_INSTALL_LOCK_HELD=1`, and releases the lock via `Drop` on both Ok and Err paths. The installer template at `generate_version_installer.sh:344-362` short-circuits both `acquire_install_lock` and `release_install_lock` when that marker is set and fails closed if the lock dir is missing — no deadlock and no caller-lock destruction. `artifact_self_update_receipt_contract.sh:82-101` exercises the external-lock path end-to-end.
- **Receipt-derived env + `--force`.** `SIFR_INSTALL_DIR` always; `SIFR_INSTALL_LOCK_HELD=1` always after acquisition; `SIFR_NO_MODIFY_PATH=1` only when `modify_path == false`; `SIFR_INSTALL_MANIFEST_DIR` only when the canonicalized receipt path differs from the default (preserving the `~/.sifr/install.json` special case at `self_update_runner.rs:155-163`); `--force` only when `plan.force == true`.
- **Output and exit code preservation.** `Command::status()` inherits stdout/stderr; success returns the installer's exit code; failure surfaces the installer's exit code via `RunnerError.exit_code` and only falls back to `EXIT_USER_DIAGNOSTIC` (1) if the installer was signal-terminated.
- **No archive download from the CLI.** Confirmed — only `installer.sh` is fetched.
- **Tests, docs, and status.** 38-test self-update suite passes locally; distribution validation passes; `internal_docs/distribution_pipeline.md:184` documents the lock handoff; `internal_docs/typescript_go_architecture_transfer_m1_guardrails.md` adds the runner fixture-read inventory row; `issues/ad-hoc-sifr-self-update.md` marks M2 merged and M3 in-progress.

## Non-blocking observations carried from pass 1

These remain valid follow-ups but do not block M3:

1. Diagnostic codes are still coarse — all runner errors reuse `SELF_UPDATE_UNMANAGED_RECEIPT` regardless of failure mode. Contract permits this for the ad-hoc phase; split into download/execution/lock subcodes in M5.
2. `InstallLock::acquire` busy-waits indefinitely at 100 ms if a previous process is SIGKILLed mid-update. Add age-based reclamation or a max-wait diagnostic later.
3. No test literally races a manual `sh installer.sh` invocation against the runner. Mechanically guaranteed by shared lock path + handoff verification, but an explicit test would make the property visible.
4. `validate_installer` has no upper size cap. Trusted HTTPS + immutable URL makes this unrealistic; a max-bytes guard is cheap defense-in-depth.

Pass-1's `--proto` defense-in-depth recommendation is resolved.

## Validation reproduced from the user-supplied trace

- `cargo fmt --check`, `cargo clippy --workspace -- -D warnings` — pass.
- `cargo test -p sifr -- self_update` — 38 tests pass.
- `scripts/run_distribution_validation.sh`, file-size guardrail, manual fake-installer smoke — pass.
- `scripts/run_all_tests.sh --profile quick` — pass (advisories only).

No blocking issues introduced by the pass-2 refinements; M3 is READY.

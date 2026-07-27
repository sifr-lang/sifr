# Phase 40 / Milestone 40.5 — Qualification-Isolation Review (Pass 2)

No files modified. Working tree identical to the start snapshot.

## Pass-1 finding closure (independently verified)

**1. HIGH — override live on the real install path → CLOSED.**
`self_update_metadata_source.rs:19-23` rejects the var before any path handling; `self_update_cli.rs:89` now threads `args.dry_run` into `fetch_channel_metadata`. Verified with the **release** binary extracted from the built archive against a real `install.json` receipt:
`SIFR_TEST_CHANNEL_METADATA_PATH=<fixture> sifr self update --version 0.1.0-beta.1301` → `error[SIFR-BUILD-0901]: SIFR_TEST_CHANNEL_METADATA_PATH is permitted only with self update --dry-run`, **exit=1**, no installer download, no lock acquisition. The rejection happens at metadata fetch, strictly before `resolve_update_plan`, so a fixture can no longer supply `installer_sha256` or re-declare a withdrawn release for a real update. `args.dry_run` comes only from the clap `--dry-run` flag (`self_update_cli.rs:67-68`); `fetch_channel_metadata` has exactly one call site repo-wide.
Docs consequence closed: `internal_docs/distribution_pipeline.md:383-390` now states the actual authority — dry-run-only, controls "dry-run release status and digest planning", cannot download an installer or mutate the installation, real updates and protected publication smoke reject or omit it. That matches observed behavior. `docs/self_update.md:17-19` is no longer conditionally false, since the public asset is the only source that can reach a real install.

**2. MEDIUM — direct `runner.py` execution → CLOSED.** `runner.py:18-27` inserts the area dir on `sys.path` before importing the sibling by bare module name. `python3 verification/areas/sysroot_release/runner.py --help` → exit 0, and the full suite ran to completion as `__main__`.

**3. LOW — trust boundary not inventoried → CLOSED.** `plans/releases/stable_gate_inventory.json:85-93` adds `self-update-metadata-source` with `activation_boundary: stable-qualification` and a disposition that pins real updates to the public asset. 27 gates, all `location` paths exist; governance selftest clean.

**4. LOW — tracking/artifact → CLOSED.** `plans/issues/active/phase-40-stable-channel-ga-execution.md:308-318` records this wave under `milestone_40_5` with the archived pass-1 reference and the remediation list. Pass-1 artifact is 8141 bytes at `plans/reviews/archive/phase-40-milestone-40-5-qualification-isolation-review-pass-1.md`.

**5. LOW — `is_file()` symlink follow + coverage gap → CLOSED.** `self_update_metadata_source.rs:34-41` uses `fs::symlink_metadata` + `file_type().is_file()`. Release binary: symlink → `does not name a regular file`, exit 1. New Rust tests cover dry-run-only, symlink, directory, non-UTF-8, and unset→public (`resolve_test_fixture(None, false) == None`).

## Independent reruns

| Check | Result |
|---|---|
| `sysroot_release --suite host-installed-smoke` (direct `runner.py`) | **PASS** — `elapsed_ms=263236 status=pass`, variants=1, failures=0. Full chain: self version, self-update dry run, healthy/broken doctor, out-of-repo emit, LSP stdio, both path-leakage checks. |
| Dry-run artifact from the release binary | `current=0.1.0-beta.1300 → target=0.1.0-beta.1301`, `action=update`, `would_run_installer=true`, `installer_url=https://github.com/sifr-lang/sifr/releases/download/0.1.0-beta.1301/sifr-installer-0.1.0-beta.1301` (constant-derived). |
| Fail-closed matrix, release binary | symlink / directory → `regular file`; relative → `absolute test fixture`; empty string → `absolute test fixture`; missing → `cannot be inspected`; non-UTF-8 → `stream did not contain valid UTF-8`. All `SIFR-BUILD-0901`, all exit 1. |
| Override unset, dry-run | Falls through to the public asset and **rejects it**: `self-update channel metadata contains unsupported fields` — the live asset is still schema-v1. Confirms public source selection and no v1 acceptance. |
| Inherited-env sanitization | `runner.py:748` pops the var in `installed_env()`, the builder for all installed-binary call sites; the override is added only to a per-command copy (`self_update_certification.py:67-68`). |
| No schema-v1 reader/fallback/migration | `schema_version != 2` hard-rejected (`self_update_metadata.rs:185`); new module is schema-agnostic. Only `sysroot_schema_version: 1` remains — unrelated pre-existing sysroot receipt schema. |
| No Rust-interop change | Diff touches 5 code/doc/plan files + 2 new files; nothing under interop. |
| Immutable URL/digest trust | `self_update_runner.rs` and `self_update_receipt.rs` untouched; `validate_installer(&installer_path, &plan.installer_sha256)` intact at `self_update_runner.rs:46`; `installer_url()` still derived from `GITHUB_RELEASE_DOWNLOAD_BASE_URL` (`self_update_metadata.rs:88-90`). |
| `cargo test -p sifr --bin sifr self_update` | 53 passed (was 51 in pass 1; +2 new). |
| `cargo fmt --check`, `cargo clippy --workspace -- -D warnings` | Clean. |
| File-size guardrail | PASS (2890 files, limit 900). |

## Non-blocking observations (not actionable for this wave)

- `verification/areas/sysroot_release/runner.py:96` — `result_path.relative_to(REPO_ROOT)` raises `ValueError` when `--result-json` points outside the repo. Pre-existing on `origin/main` (confirmed via `git show`), sits after the pass/timing line, and does not affect the result JSON or in-process `sifr_verify` execution.
- `cargo clippy --workspace --all-targets` fails on `redundant_closure_for_method_calls` in `sifr_ipc` test `ipc_process_pipe_fixture` — untouched file, pre-existing; the documented gate form is clean.
- The `milestone_40_5` checkbox at `plans/issues/active/phase-40-stable-channel-ga-execution.md:307-308` is still unchecked, which is the normal pre-merge state now that the evidence entry exists.
- `plans/reviews/active/phase-40-milestone-40-5-qualification-isolation-review-pass-2.md` is a 0-byte placeholder — that is this review's own artifact, which I was instructed not to write.

No remaining actionable findings.

VERDICT: SATISFIED

# Phase 40 / Milestone 40.5 — Qualification-Isolation Review (Pass 1)

## What I verified independently

| Check | Result |
|---|---|
| `sysroot_release --suite host-installed-smoke` (full, offline) | **PASS** — `case=host-installed-smoke elapsed_ms=319192 status=pass`, `variants=1, failures=0`, `exit=0`. No network fetch of the live index. |
| Dry-run artifact produced by the **release** binary | `target/verification/actual/sysroot_release/installed-self-update-dry-run.json`: `current=0.1.0-beta.1300`, `target=0.1.0-beta.1301`, `action=update`, `would_run_installer=true`, `installer_url=https://github.com/sifr-lang/sifr/releases/download/0.1.0-beta.1301/…` (constant-derived). |
| Fixture schema-v2 validity | Valid against `ChannelMetadata::parse` — 5 top-level keys, `schema_version:2`, `generation:1`, `ga_status:preview`, exactly alpha+beta, 2 active release records, 4 targets, non-zero lowercase hex. Confirmed end-to-end, not just by reading. |
| Fail-closed matrix (real binary, `/tmp` fixture) | relative → rejected; directory → rejected; empty string → rejected; missing file → rejected; non-UTF-8 → rejected. All `SIFR-BUILD-0901`. |
| Inherited-env sanitization | `installed_env()` pops the var (runner.py:744) and is the env builder for **all three** installed-binary call sites (runner.py:202, 302, 419). `base_env()` sites (245, 568) only run cargo. Override is added to a per-command copy (self_update_certification.py:67-68) so `self version`, doctor, LSP run clean. |
| No schema-v1 reader/fallback/migration | None. New module is schema-agnostic; `schema_version == 2` remains hard-required (self_update_metadata.rs:185). Only `sysroot_schema_version: 1` persists, which is the unrelated pre-existing sysroot schema. |
| No Rust-interop change | Diff is 5 tracked files + 3 untracked; nothing under interop. |
| `cargo test -p sifr --bin sifr self_update` | 51 passed. |
| `cargo fmt --check` / `cargo clippy --workspace -- -D warnings` | Clean. |
| `distribution_release --suite full` | `variants=54, failures=0`. |
| File-size guardrail | PASS (2890 files, limit 900). runner.py 822, self_update_certification.py 190, self_update_metadata_source.rs 76. |

## Actionable findings, severity order

### 1. HIGH — the override is live on the real install path, not only `--dry-run`; it lets a local env var replace governance state in the shipped GA binary
`crates/sifr/src/self_update_metadata_source.rs:8-13` applies the override unconditionally, and `crates/sifr/src/self_update_cli.rs:89` fetches metadata **before** the `args.dry_run` branch at `:106`. So a plain `sifr self update` honors it too. The consumed metadata is not inert data: `resolve_exact` supplies `plan.installer_sha256` (self_update_metadata.rs:371), which is the only integrity pin `self_update_runner.rs:46` checks, and release `status` is what marks a version withdrawn/rolled-back. Anyone who can set env for a `sifr self update` invocation can therefore re-declare a withdrawn or incident-rolled-back release as `active`, supply a matching digest, and have the real installer run. The URL base stays constant, so blast radius is bounded to versions published under `sifr-lang/sifr`, but that is exactly the population the withdrawal/incident machinery exists to gate — and this reintroduces the "local path overrides the canonical fetched index" shape the phase plan explicitly bans at `plans/phases/40_…md:663-666`. I confirmed the **release** binary from the built archive honors it (the smoke artifact above), so this is not debug-only.

Qualification only ever needs it under `--dry-run` (self_update_certification.py:67-85), so the fail-closed fix costs nothing: thread the dry-run flag into `load()` and reject the override for a real update.

Two consequences to fix with it:
- `internal_docs/distribution_pipeline.md:383-389` — "This override changes only the metadata input for isolated validation; immutable installer URLs remain derived from trusted repository constants" is technically true but materially understates it: installer-digest pinning and release-withdrawal status are also metadata inputs.
- `docs/self_update.md:17-19` — "Channel resolution uses the `channels.json` asset on the … release tag `channels`" is now conditionally false for the production binary. This file is stable-gate `self-update-contract-docs`.

### 2. MEDIUM — the runner refactor breaks direct execution of `runner.py`
`verification/areas/sysroot_release/runner.py:18` uses an absolute package import that requires repo root on `sys.path`, and it sits **above** the `REPO_ROOT` definition at `:25`, so no bootstrap is possible in the current ordering. Verified: `python3 verification/areas/sysroot_release/runner.py --help` from repo root → `ModuleNotFoundError: No module named 'verification'` (worked on `origin/main`). It survives today only because `sifr_verify` loads it in-process via `spec_from_file_location` (areas.py:85-93) with cwd = repo root. The manifest still declares `runner.py` as the case `entry`, and `area_adapter.run_area_check_case` executes entries as `[sys.executable, str(entry)]`. Every other area runner does the sys.path bootstrap instead: `python_interop/runner.py:15-19`, `regression/runner.py:12`, `runtime_platform/runner.py:27`, `algorithmic_compatibility/runner.py:28`. Fix: define `REPO_ROOT` first, `sys.path.insert(0, str(REPO_ROOT))`, then import — or import the sibling by bare module name with `AREA_ROOT` on the path.

### 3. LOW — new trust-boundary module has no stable-gate inventory entry
`plans/releases/stable_gate_inventory.json` carries two gates on `crates/sifr/src/self_update_metadata.rs` (`self-update-index-state`, `self-update-channel-selection`), but the file that now decides **where index bytes come from** has none. The selftest only checks location existence (`stable_gate_inventory_selftest.py:38`), so nothing fails today — but a test-only override compiled into the GA binary is precisely the thing the inventory's `activation_boundary` / `disposition` fields exist to pin before GA activation.

### 4. LOW — milestone tracking not updated for this wave; empty review artifact
The only issue-doc change (`plans/issues/active/phase-40-stable-channel-ga-execution.md:293-302`) records the **previous** milestone's 40.4 pass-5 evidence. The 40.5 item "Isolate installed-sysroot self-update qualification…" (`:297-298`) is still unchecked with no evidence/PR entry, and `plans/reviews/active/phase-40-milestone-40-5-qualification-isolation-review-pass-1.md` is a 0-byte file in the working tree. AGENTS.md requires checklist/status updates per item.

### 5. LOW — `is_file()` follows symlinks, so "regular file" is not what is enforced
`self_update_metadata_source.rs:26`. Verified: a symlink to the fixture is accepted, contradicting the `does not name a regular file` message and the doc wording at distribution_pipeline.md:386-387. Not a boundary in this threat model (the var is already fully caller-controlled), but the message should match the check or the check should match the message. Related coverage gap: the unit tests cover only absolute-rejection and successful read — the non-file and non-UTF-8 branches have no Rust test, and nothing asserts the public path is taken when the var is unset.

## Non-blocking observations

- The fixture's digests are synthetic (`a`/`b`/`c`/`d` repeats) and `0.1.0-beta.1301` is not a real release. That is correct for an isolated fixture — it is never downloaded, and it is explicitly not a schema-v1 "synthesized digest" of the kind the phase plan forbids.
- Passing `run_checked` in as a parameter is a reasonable way to avoid a runner↔certification import cycle; the responsibility split (receipt writing, fixture writing, snapshot driving, validators) is clean and both files are well under the cap.
- The archived PR #3038 final-head review is internally consistent with what I observed: it records the live asset as `{"schema_version": 1, …}` and two `sysroot_release:host-installed-smoke` failures, which is exactly the failure this wave removes.

VERDICT: NOT SATISFIED

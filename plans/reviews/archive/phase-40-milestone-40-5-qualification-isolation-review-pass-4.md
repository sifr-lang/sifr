## Review scope

- **Reviewed git HEAD:** `d78cfb756c6378cb2a6c7e1d2fe5030585cfc066` (`codex/phase-40-milestone-40-5-qualification-isolation`, single commit `d78cfb756` over `origin/main`).
- **The two uncommitted remediation files were explicitly included** in this review:
  - `internal_docs/typescript_go_architecture_transfer_guardrails.md` (modified, uncommitted)
  - `plans/issues/active/phase-40-stable-channel-ga-execution.md` (modified, uncommitted)
  - Also present: `plans/reviews/active/phase-40-milestone-40-5-qualification-isolation-review-pass-4.md` (0 bytes — this review's own artifact slot; not written per instruction).
- No files were edited by this review.

## Commands run and results

| Command | Result |
|---|---|
| `python3 verification/areas/developer_tooling/check_typescript_go_transfer_guardrails.py` | **PASS**, exit 0 |
| Scanner replay (`direct_fs_sites()` + `validate_direct_fs_inventory()` against both doc versions) | remediated doc: **0 failures**; `origin/main` doc: **2 failures** (`self_update_metadata_source.rs:37`, `:42`) — remediation is exact and load-bearing |
| `cargo test -p sifr --bin sifr self_update` | **53 passed**, 0 failed |
| `cargo clippy --workspace -- -D warnings` | clean |
| `cargo fmt --check` | clean |
| `python3 scripts/check_file_size_guardrails.py` | **PASS** (2890 files, limit 900) |
| `python3 verification/areas/sysroot_release/runner.py --help` | exit 0 (direct-runner execution restored) |
| `python3 verification/areas/distribution_release/governance/stable_gate_inventory_selftest.py` | exit 0 |
| Independent fail-closed matrix, real `sifr` binary + fabricated eligible receipt (9 cases) | see below |

**Fail-closed matrix (my own run, not taken from prior passes):** dry-run + valid fixture → exit 0 with schema-v2 JSON; **real update + fixture → exit 1, `SIFR_TEST_CHANNEL_METADATA_PATH is permitted only with self update --dry-run`**; symlink → `does not name a regular file`; directory → same; relative → `must name an absolute test fixture`; empty → same; missing → `cannot be inspected`; non-UTF-8 → `stream did not contain valid UTF-8`; no override → falls through to the public asset and rejects it (`unsupported fields`, live asset is still schema-v1 — no v1 reader or fallback). All rejections `SIFR-BUILD-0901`, exit 1.

## Verification of the remediation itself

The scanner at `check_typescript_go_transfer_guardrails.py:42-44` matches `fs::read_to_string|fs::read_dir|.is_file()|.is_dir()`. In the new module those are exactly `self_update_metadata_source.rs:37` (`if !metadata.file_type().is_file()`) and `:42` (`fs::read_to_string(path)`) — precisely the two references added at `internal_docs/typescript_go_architecture_transfer_guardrails.md:73`. Neither more nor fewer. The row's classification (non-semantic release-qualification command surface, keep inventoried) matches the adjacent `self-update-receipt-state` and `self-update-runner-fixture-reads` rows. The issue-doc bullet at `plans/issues/active/phase-40-stable-channel-ga-execution.md:325-329` states the finding and its resolution accurately and does not retro-edit the archived pass-2 SATISFIED verdict — the ordering (pass 2 clean → create-PR gate finds gap) is disclosed honestly.

## Security / trust boundary

`fetch_channel_metadata` has exactly one call site (`self_update_cli.rs:89`); `args.dry_run` originates only from the clap flag. Rejection occurs strictly before `resolve_update_plan` (`:95`), so a fixture can no longer supply `installer_sha256` or re-declare a withdrawn release for a real update — pass 1's HIGH finding is genuinely closed, reproduced above. `--dry-run` returns at `self_update_cli.rs:106-110` before `SelfUpdateRunner::production().run(...)` at `:122`, so it acquires no install lock and mutates nothing. `self_update_runner.rs` and `self_update_receipt.rs` are untouched. Test-only override containment is correct: `runner.py:748` pops the var in `installed_env()`, the builder for all installed-binary call sites, and the override is added only to a per-command copy (`self_update_certification.py:68`). No `.github/**` workflow sets the var. `runner.py` has no unused imports after the extraction, and `CertificationError` has no external importers.

## Actionable findings

**1. LOW — review pass 3 has no archived artifact and no ledger entry, breaking the milestone's review-round record.**
`plans/reviews/active/phase-40-milestone-40-5-qualification-isolation-review-pass-3-final-pr-head.agent.log` exists (created 01:31, between pass 2 at 01:22 and pass 4 at 01:42), but there is no `plans/reviews/archive/phase-40-milestone-40-5-qualification-isolation-review-pass-3*.md` anywhere in the repo, working tree, git history, or stashes. The milestone ledger at `plans/issues/active/phase-40-stable-channel-ga-execution.md:311-329` cites pass 1 and pass 2 and then jumps directly to the create-PR gate finding, never mentioning pass 3. This breaks a convention the same file applies uniformly everywhere else — `:165-166` (40.4 pass 3), `:363-364` (40.0), `:445-446` (40.1), `:700-701` (40.3 pass 3, archived even though *not approved*), and the 40.4 `pass-5-final-pr-head` artifact archived in this very PR. `milestone_40_5`'s own checklist item "Record review rounds, PR, validation, and merge" (`:336`) is not satisfied for this wave while a round is missing from the chain. Fix: archive the pass-3 report, or add one ledger sentence stating that the pass-3 final-PR-head round was superseded by the create-PR gate failure and what it found.

## Non-blocking observations

- `fs::symlink_metadata(path)` at `self_update_metadata_source.rs:34` is the actual filesystem syscall, and it is *not* in the inventory — because `DIRECT_FS_PATTERN` does not match `symlink_metadata`, `metadata`, or `exists`. This is a pre-existing scanner scope limitation affecting the whole inventory, not something this PR introduces, and the remediation correctly covers exactly what the gate requires.
- The inventory pins absolute line numbers (`:37`, `:42`), so any edit above them in that file silently breaks the gate. Consistent with all 130+ existing rows.
- `SIFR_TEST_CHANNEL_METADATA_PATH` remains compiled into the shipped GA binary. Post-fix its only authority is falsifying advisory `--dry-run` output; it cannot download, mutate, or acquire the lock, and it is now pinned by the `self-update-metadata-source` gate. Acceptable.
- The `milestone_40_5` isolation checkbox (`:309-310`) is correctly still unchecked: its second clause ("retaining separate protected public-endpoint smoke") depends on the not-yet-delivered publication workflow. The phase DoD clause "The isolated installed sysroot qualification reproduces that contract without public network access" is served by this wave.
- I did not re-run the full `sysroot_release --suite host-installed-smoke` (~5 min plus release-archive build). Passes 1 and 2 each ran it green, and the only delta since is the two documentation files, which cannot affect it. My independent binary-level matrix covers the behavioral substance.
- `docs/self_update.md:16-18` is accurate again post-fix: the public asset is the only source reachable by a real install. No change needed to that stable-gate file.

VERDICT: NOT SATISFIED

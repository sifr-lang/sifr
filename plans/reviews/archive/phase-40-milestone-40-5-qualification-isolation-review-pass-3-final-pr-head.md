## Phase 40 / Milestone 40.5 — Qualification-Isolation Review (Pass 3, exact pushed PR head)

Identity confirmed: `gh pr view 3039` → `headRefOid = d78cfb756c6378cb2a6c7e1d2fe5030585cfc066`, `refs/pull/3039/head` = same, local `HEAD` = same. `origin/main...d78cfb756` is one commit, 12 files, +515/−111. Working tree carries only the 0-byte `plans/reviews/active/…pass-3-final-pr-head.md` placeholder — no post-review drift, no files modified by me.

### Remediation checklist against the committed head

| Item | Status at head |
|---|---|
| Dry-run-only fixture override | `self_update_metadata_source.rs:19-23` rejects the var before any path handling; `self_update_cli.rs:89` threads `args.dry_run` (clap-only, one call site repo-wide) |
| Real-update rejection before plan/installer | Release binary: `self update --version …`, bare `self update`, and `--force --channel beta` all → `error[SIFR-BUILD-0901]: … permitted only with self update --dry-run`, exit 1. Rejection is at metadata fetch, strictly before `resolve_update_plan` and `SelfUpdateRunner`; no `.sifr-update.lock` created, no download |
| Public-source fallback | Var unset + `--dry-run` → curl to the public asset, which is rejected as `contains unsupported fields` (live asset is still v1). Confirms public selection and no v1 acceptance |
| Absolute / regular / non-symlink / UTF-8 | Release binary: symlink → `does not name a regular file`; directory → same; relative → `must name an absolute test fixture`; empty → same; missing → `cannot be inspected`; non-UTF-8 → `stream did not contain valid UTF-8`. All `SIFR-BUILD-0901`, all exit 1. `fs::symlink_metadata` + `file_type().is_file()` at `:34-41` |
| Inherited-env sanitization | `runner.py:748` pops `SIFR_TEST_CHANNEL_METADATA_PATH` in `installed_env()`, the builder for all three installed-binary sites (206, 306, 423); the override is added only to a per-command copy (`self_update_certification.py:67-68`) |
| Direct runner execution | `runner.py:18-20` inserts the area dir before the bare-name sibling import — the same pattern as `python_interop/runner.py:16`. `python3 verification/areas/sysroot_release/runner.py --help` → exit 0; full suite ran as `__main__` |
| Stable-gate inventory ownership | `stable_gate_inventory.json:85-93` adds `self-update-metadata-source`, `activation_boundary: stable-qualification`. Selftest passes; 27 gates, all locations exist |
| Responsibility-based split | 190-line `self_update_certification.py` owns receipt writing, fixture authoring, snapshot driving, validators; `run_checked` injected to avoid an import cycle. No stale references to moved symbols |
| Docs / ledger accuracy | `distribution_pipeline.md:383-390` states the actual authority (dry-run-only, controls dry-run status/digest planning, cannot download or mutate). `docs/self_update.md:17-19` is true again for real updates. Issue ledger `:308-318` records the wave, both archived passes, and the remediation list; `milestone_40_5` boxes correctly stay unchecked. The 40.4 pass-5 bullet matches the archived report (`b09845a86`, 20-file replay, SATISFIED) and `21bd64d7c` is indeed `#3038`; the `readonly-check-doctor` timeout is genuinely pre-indexed (`adhoc_performance_budget_host_variance.md:51`, issue `:189`) |

### Independent reruns

- **`sysroot_release --suite host-installed-smoke`, invoked directly as `python3 runner.py`** — **PASS**: `case=host-installed-smoke elapsed_ms=272375 status=pass`, `variants=1, failures=0, blocking_failures=0`, exit 0. Full chain from the release binary extracted out of the built archive: self version, self-update dry run, healthy/broken doctor, out-of-repo emit, LSP stdio, and both path-leakage checks.
- Dry-run artifact from that release binary: `current=0.1.0-beta.1300 → target=0.1.0-beta.1301`, `action=update`, `would_run_installer=true`, `installer_url=https://github.com/sifr-lang/sifr/releases/download/0.1.0-beta.1301/sifr-installer-0.1.0-beta.1301` — constant-derived, fixture-independent. Fixture path does not leak into any artifact.
- `cargo test -p sifr --bin sifr self_update` → 53 passed. `cargo fmt --check` clean. `cargo clippy --workspace -- -D warnings` → exit 0. File-size guardrail PASS (2890 files, cap 900). HIR guardrail exit 0. Stable-gate inventory selftest clean.
- Prohibited content: `schema_version != 2` still hard-rejected (`self_update_metadata.rs:185`); the new module is schema-agnostic; the only `1` is the pre-existing, unrelated `sysroot_schema_version` in the moved receipt writer. No interop path touched.

### Actionable findings

None.

### Non-blocking observations

- Nothing automated asserts that `installer_url` in the dry-run artifact starts with the trusted GitHub base while fixture metadata is in play — the wave's central claim is doc- and inventory-asserted plus code-derived (`installer_url()` at `self_update_metadata.rs:88-90`), and I verified it by hand. A one-line prefix assertion in `validate_self_update_dry_run_json` would make it test-enforced; not a defect in this wave.
- `distribution_pipeline.md:388-389` describes "protected publication smoke" in the present tense; that smoke is still an unchecked milestone-40.5 item (`issue:325`, phase scope `:957-961`). Vacuously true today (it omits the var) and consistent with the authorized plan.
- `runner.py:96` `result_path.relative_to(REPO_ROOT)` still raises for an out-of-repo `--result-json`; pre-existing on `origin/main`.
- `plans/reviews/active/…pass-3-final-pr-head.md` is a 0-byte placeholder — this review's own artifact, which I was instructed not to write.

VERDICT: SATISFIED

Pass-2 review of the Wave 4 workspace manifest/source-root compact baselines slice on branch `codex/wave-4-workspace-compact-baselines`.

## Findings (ordered by severity)

### 1. BLOCKER — Pass-1 follow-up edits hit the wrong code rows: SIFR-BUILD-0002..0005 received the workspace-graph rationale; SIFR-WORKSPACE-0101..0104 still carry the original generic boilerplate

Severity: high (governance honesty / metadata correctness)

The pass-1 review (`plans/reviews/active/ad-hoc-world-class-verification-wave-4-workspace-source-root-compact-baselines-review-pass-1.md:5-10`) explicitly asked to rewrite the four legacy-workspace deferral reasons "WORKSPACE-0101..0104" with the lower-level-harness / coverage-policy rationale. The tracker entry restates that this is what was applied (`plans/issues/active/ad-hoc-world-class-verification-standard-and-gate-closure.md`, Seventh Wave 4 slice "Review" bullet: "replacing WORKSPACE-0101..0104 boilerplate deferral reasons with the lower-level-harness / coverage-policy rationale").

The actual diff in `verification/areas/diagnostics/data/code_baseline_coverage.json` mis-targets four entirely different codes. The new rationale is now attached to:

- `verification/areas/diagnostics/data/code_baseline_coverage.json:80` — `SIFR-BUILD-0002` (Build file materialization failed)
- `verification/areas/diagnostics/data/code_baseline_coverage.json:93` — `SIFR-BUILD-0003` (Temporary build workspace creation failed)
- `verification/areas/diagnostics/data/code_baseline_coverage.json:106` — `SIFR-BUILD-0004` (Cargo manifest generation failed)
- `verification/areas/diagnostics/data/code_baseline_coverage.json:119` — `SIFR-BUILD-0005` (Rustc or Cargo execution failed)

Those four BUILD codes are emitted by `sifr_driver::build::workspace` for build-orchestration failures (`crates/sifr_diagnostics/src/codes/registry/registry_entries/project_and_backend.rs:132-175`). They are NOT legacy workspace import-graph codes, they have no SIFR-IMPORT replacement on public CLI paths, and they are not listed in `LEGACY_WORKSPACE_IMPORT_CODES` (`crates/sifr_driver/src/bin/diagnostic_contract_harness.rs:13-18`). The new reason text — "Legacy workspace graph diagnostic remains active, but current public project import paths intentionally render source-spanned SIFR-IMPORT replacements. Wave 4 follow-up must either add a lower-level rendered harness for this legacy code or make an explicit coverage-policy decision." — is materially wrong for them and will mislead future contributors about what these BUILD codes mean and why they are deferred.

The intended targets still hold the original boilerplate text that the pass-1 review flagged as dishonest:

- `verification/areas/diagnostics/data/code_baseline_coverage.json:1844` — `SIFR-WORKSPACE-0101`
- `verification/areas/diagnostics/data/code_baseline_coverage.json:1857` — `SIFR-WORKSPACE-0102`
- `verification/areas/diagnostics/data/code_baseline_coverage.json:1870` — `SIFR-WORKSPACE-0103`
- `verification/areas/diagnostics/data/code_baseline_coverage.json:1883` — `SIFR-WORKSPACE-0104`

All four still say "Rendered baseline fixture expansion is staged inside Wave 4; this code is active and tracked by registry/docs/e2e coverage until its rendered baseline lands." — which contradicts the contract harness behavior the tracker correctly describes.

Schema validators do not catch this because deferral.reason is just free text. Diagnostics contracts and baselines still pass; `git diff --check` still passes. The error is purely semantic.

Required fix before PR:
1. Revert the four `deferral.reason` strings at lines 80, 93, 106, 119 of `code_baseline_coverage.json` back to the prior boilerplate ("Rendered baseline fixture expansion is staged inside Wave 4; this code is active and tracked by registry/docs/e2e coverage until its rendered baseline lands."). If the BUILD-0002..0005 deferral rationale actually deserves a more accurate description, that is a separate follow-up — do not leave the misattributed text in place.
2. Apply the workspace-graph rationale to the four entries at lines 1844, 1857, 1870, 1883 (SIFR-WORKSPACE-0101..0104) — the rows the pass-1 review and the tracker actually identified.
3. Re-run `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite contracts` and `git diff --check` to confirm no regression. (Tracker "Review" bullet wording is then accurate without further change.)

### 2. PASS — Each new SIFR-WORKSPACE-0001..0004 baseline truly covers its intended code and is reachable through the public `check` command

Verified by directly invoking `cargo run -q -p sifr -- --diagnostic-format compact check <fixture>/main.sifr` on each of the four new fixtures from REPO_ROOT:

- `workspace_malformed_manifest` (`sifr.toml` = `[source\n`) hits `parse_manifest_error` in `crates/sifr_driver/src/workspace/mod.rs:181-188`, emitting a single `SIFR-WORKSPACE-0001 <unknown>` compact diagnostic with the TOML crate's natural framing, exit 1, empty stdout. Baseline file at `verification/areas/diagnostics/fixtures/diagnostics/workspace_malformed_manifest/baselines/check-compact.stderr.txt` matches after `<WORKSPACE>` normalization; the trailing blank line in the recorded baseline corresponds to the toml crate's terminating `\n\n`.
- `workspace_source_root_escapes` (`["../outside"]`) trips `Component::ParentDir` at `crates/sifr_driver/src/workspace/mod.rs:151-156`, single `SIFR-WORKSPACE-0002` compact line.
- `workspace_source_root_not_directory` (`["missing"]`) survives normalization, then `provider.is_dir(absolute)` is false at `crates/sifr_driver/src/workspace/mod.rs:172-177`, single `SIFR-WORKSPACE-0003` compact line.
- `workspace_invalid_source_root` (`[""]`) trips `source_root.is_empty()` at `crates/sifr_driver/src/workspace/mod.rs:139-143`, single `SIFR-WORKSPACE-0004` compact line.

Each fixture has empty `check-compact.stdout.txt`, `check-compact.exit-code.txt` containing `1`, and the manifest declares `command: "check"`, `expect_exit_code: 1`. The verification harness invokes `cargo run … check <abs-entry>` from REPO_ROOT with absolute paths via `case_entry_path` → `resolve_repo_path` (`verification/runner/sifr_verify/area_adapter.py:377-388`), which is why the recorded `workspace_malformed_manifest` baseline correctly contains `<WORKSPACE>/verification/...` after `normalize_string` (`verification/runner/sifr_verify/area_adapter.py:541-553`).

### 3. PASS — Manifest, coverage, and baseline metadata counts are internally consistent

- `verification/areas/diagnostics/manifest.json` baselines suite holds 116 cases (4 new workspace + 112 prior); contracts suite still 5.
- `verification/areas/diagnostics/data/code_baseline_coverage.json` reports 170 codes total: 118 active, 52 deferred. Family breakdown of deferrals matches the tracker exactly: BUILD 6, ENCODING 1, FMT 1, INTERNAL 1, IO 2, PACKAGE 34, STDLIB 3, WORKSPACE 4. The four new `baseline_fixture_id` + `renderer_formats: ["compact"]` entries for SIFR-WORKSPACE-0001..0004 are correctly cleared of `deferral`.
- `verification/areas/diagnostics/data/baseline_metadata.json` carries 147 entries = 144 baselines (116 compact + 14 human + 14 json) + 3 synthetic_baselines, matching the bless output `116 cases / 144 renderer variants`. All four new metadata rows share `source_hash` `sha256:cde0429b…`, which `shasum -a 256` confirms is the hash of all four `main.sifr` files (each containing exactly `def main():\n    pass\n`). Owner (`compiler/frontend`), renderer (`compact`), normalizer set (`workspace-path`, `tmp-path`, `crlf`, `artifact-cache-lines`), and `bless_reference` (`wave-4-workspace-source-root-compact-baselines-pr`) follow existing slice conventions.

### 4. PASS — WORKSPACE-0101..0104 deferral remains technically honest for this slice

`crates/sifr_driver/src/bin/diagnostic_contract_harness.rs:157-211` runs all PROJECT/CYCLE/PACKAGE fixtures through `assert_contract` with `LEGACY_WORKSPACE_IMPORT_CODES` declared as forbidden codes (`assert_contract`'s `forbidden_codes` argument at `:298-321`). Any public path that surfaces 0101..0104 fails the harness with "retired workspace import code leaked". So a public-CLI rendered baseline for these codes is unreachable by design, exactly as the tracker says — the slice is right to defer them; the only governance issue is finding #1 (the rationale text landed on the wrong rows).

### 5. INFO — Cross-area duplicate of `workspace_malformed_manifest`

`verification/areas/project_workspace/fixtures/project/workspace_malformed_manifest/` already emits SIFR-WORKSPACE-0001 with compact/human/json baselines. The new diagnostics-area fixture is a near-identical compact-only copy. This is acceptable because each area's `code_baseline_coverage` check is scope-bound (`verification/areas/diagnostics/checks/code_baseline_coverage.py:270`), but the two fixtures will need to be kept in sync if the toml crate's error text changes. No action required for this slice.

### 6. INFO — `source_hash` only tracks `main.sifr`, not `sifr.toml`

For these fixtures the behavior under test is driven by `sifr.toml`, but `verification/areas/diagnostics/checks/code_baseline_coverage.py:296-305` hashes the entry `main.sifr` only. This is a pre-existing schema limitation already in use for `source_import_*` fixtures and is consistent with the rest of the diagnostics area — but it means a contributor editing only the `sifr.toml` would not trip the source-hash check. Out of scope for this slice; flagging for awareness.

## Verifications performed in this pass

- `git status` + `git diff --stat` confirmed only the expected four files (tracker, manifest, coverage, metadata) plus the four new fixture directories and the two review markdowns are touched.
- Direct CLI re-runs of all four fixtures via `cargo run -q -p sifr -- --diagnostic-format compact check <fixture>/main.sifr` confirmed exit code 1, empty stdout, and exactly one intended SIFR-WORKSPACE-000x compact diagnostic each (path text differs from baseline because the harness invokes with absolute paths, which is what produces the `<WORKSPACE>/...` token after normalization).
- `shasum -a 256` on all four new `main.sifr` files matches the metadata `source_hash`.
- `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite contracts` passed (`variants=5, failures=0, blocking_failures=0, non_blocking_failures=0`).
- `git diff --check` passed.
- Confirmed the contract harness's `LEGACY_WORKSPACE_IMPORT_CODES` list and `assert_contract` forbidden-codes enforcement, which validates that the WORKSPACE-0101..0104 deferral rationale is factually correct (just attached to the wrong rows).

## Verdict

**Blocked.** Finding #1 is a governance-honesty regression that must be fixed before PR submission: the pass-1 follow-up edits were applied to SIFR-BUILD-0002..0005 instead of SIFR-WORKSPACE-0101..0104, leaving both code groups misdescribed and making the tracker's "Review" bullet materially false. The fix is a four-line revert in `code_baseline_coverage.json` (lines 80, 93, 106, 119) plus four targeted edits at lines 1844, 1857, 1870, 1883, followed by a quick diagnostics contracts + `git diff --check` re-run. Findings #2–#6 are clean; no other blockers, stale baselines, accidental coverage regressions, missing validations, or hidden policy gaps were observed. **Another review round is required after the fix** to confirm the rationale ends up on the intended rows.

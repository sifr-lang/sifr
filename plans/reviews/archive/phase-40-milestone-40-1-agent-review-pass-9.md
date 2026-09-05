# Independent Review — Sifr Phase 40 / milestone_40_1, pass 9 (final PR-head review)

**Target:** PR #3028, branch `codex/phase-40-milestone-40-1`, HEAD `aeff4d07aca52154a20e1a328ec2d3288e247dea` vs `origin/main`.
**Mode:** read-only. No repository file was created, modified, or deleted. The only working-tree entry is the pre-existing, untracked, **zero-byte** `plans/reviews/active/phase-40-milestone-40-1-agent-review-pass-9.md`, which was already present and was left untouched (confirmed 0 bytes; `git status --porcelain` identical before and after this review; HEAD unchanged).

---

## 1. PR identity, head, and scope

| Claim | Verified |
|---|---|
| `headRefOid` = `aeff4d07aca52154a20e1a328ec2d3288e247dea` | ✅ matches local HEAD exactly |
| base = `main`, state OPEN, `mergeable: MERGEABLE` | ✅ |
| 13 commits, 48 files, +5576/−160 | ✅ (`git diff --stat origin/main..HEAD`) |
| Title `feat(release): qualify stable release candidates` matches content | ✅ |

Scope is coherent with the milestone: qualification workflow, target qualifier, artifact collector, non-mutating planner, governance/schema/test surfaces, profiles, capability demo, docs, tracker, review artifacts. **No Rust-interop implementation file is touched** — the only `crates/` change is `self_update_receipt.rs` (receipt channel acceptance), which is in-scope for stable-installation gating. No unrelated user changes were reverted.

## 2. Prior reviews and the pass-8 approval

All eight prior reviews are tracked in `plans/reviews/archive/phase-40-milestone-40-1-agent-review-pass-{1..8}.md`; **none remain in `plans/reviews/active/`** as tracked files. I read all eight. The finding chain is closed end-to-end: pass 1 (custody/materialization/demo) → 2 (digest confound, symlink container, id binding) → 3 (non-UTF-8 tracebacks, dispatch commit, report shapes) → 4 (sibling decodes, installer assignment evasion, collector symlink) → 5 (verifier decode, shell-parse replaced by byte equality, two-claim fixture) → 6 (documentation omission) → 7 (contract literal truncated before the closing `--out` quote) → 8 APPROVED.

**Pass 8's approval still applies to the exact head.** `git diff --stat 6b261cf3b..HEAD` (the commit pass 8 reviewed) is limited to:

- `plans/issues/active/phase-40-stable-channel-ga-execution.md` (+29/−9)
- the eight review artifacts (7 pure renames `active/`→`archive/`, +93 for the pass-8 body)

**Zero production, workflow, script, governance, schema, test, or documentation bytes changed since pass 8.** I verified this by `--name-status`, not by trusting the stat.

## 3. The three subsequent tracker-only commits

| Commit | Change | Accurate? |
|---|---|---|
| `4c8beab53 docs(release): close qualification review` | rewrites 7 `plans/reviews/active/…` paths to `archive/…`, adds the pass-8 entry | ✅ every rewritten path exists in `git ls-files plans/reviews/archive`; the pass-8 summary ("verified the full invocation anchor … returned `APPROVED` with no actionable findings") matches the artifact's actual body and verdict |
| `879983804 docs(release): record qualification validation` | status → "implementation, local qualification, and independent review complete"; adds the authoritative `create-pr` evidence line | ✅ status is now true; the evidence line is appropriately qualified ("cold-cache 21.8-minute wall time exceeded only the **advisory** warm target") rather than claiming a clean budget |
| `aeff4d07a docs(release): link qualification pull request` | status → "ready [PR #3028] is open" | ✅ PR #3028 is open at this head |

No tracker sentence overclaims. The milestone_40_1 record (`:200-293`) is a faithful history: every pass has its finding, its remediation, and the required-next-pass note; the passing-evidence list matches what I could re-verify below.

## 4. PR body accuracy

| PR body claim | Status |
|---|---|
| distribution release full suite — 43 variants, zero failures | ✅ **re-ran**: `variants=43, failures=0, blocking_failures=0, non_blocking_failures=0`; 14 governance self-tests |
| qualification suite — 8 self-tests plus workflow variant | ✅ **re-ran**: `qualification self-tests ok: tests=8`, 1 variant, 36.5 s; the workflow-contract variant lives in the full suite (also passed) |
| file-size and HIR maintainability guardrails — pass | ✅ **re-ran**: `PASS (2857 files, limit 900)`; `PASS`. Largest new files: `qualification_selftest.py` 889, `qualification_fixture.py` 864, `planner.py` 757 — all under the cap |
| Rust interop matrix/tiers/compatibility-matrix/stale-drafts/stable-candidate | ✅ these are exactly the `rust_interop` suites in `verification/profiles/create-pr.json`, so the create-pr run covers them |
| Eight review rounds archived; pass 8 APPROVED | ✅ verified against the artifacts themselves |
| `scripts/run_all_tests.sh --profile create-pr` — pass, 131/131 E2E | **recorded, not re-run.** I did not re-run the ~22-minute authoritative facade; nothing in the diff since pass 8 could change its outcome (tracker/markdown only), and the tracker records the run with its cold-cache caveat. Stated here for transparency, not as a finding. |
| Also re-ran: `cargo test -p sifr --bins -- self_update` | ✅ 42 passed, 0 failed — covers the receipt-channel change |

## 5. Independent review of the complete diff

I reviewed the full diff at this head rather than deferring to prior passes. Points I verified myself:

**Workflow (`.github/workflows/release-qualification.yml`).** `permissions: contents:read, actions:read` only; `workflow_dispatch` with exactly `source_commit`/`version`; inputs regex-pinned to 40-hex and stable SemVer, with `github.sha == source_commit` so the dispatch ref cannot float; every checkout pinned to the resolved commit with `submodules: recursive` and re-asserted via `git rev-parse HEAD`; four uploads, all `overwrite: false` + `retention-days: 30`; no `gh release`, `vsce publish`, `repository_dispatch`, write permissions, or `environment:`. The two `${{ }}` interpolations inside a `run:` block (`:389-390`) carry only regex-validated 40-hex/SemVer values — not injectable.

**Contract case is load-bearing.** `release_qualification_workflow_contract.sh` **PASS** at this head (exit 0). The pass-7 defect is genuinely fixed: `:78-83` now terminates the pinned literal with both the closing `"` and `\n`, so an appended argument or altered `--out` breaks the substring. It also pins `cargo build --locked --release -p sifr` in `build_release_artifacts.sh` (present at `:283`), `count(...) == 1` for each of the four exact download names, `overwrite: false`×4 / `retention-days: 30`×4, permissions, job topology, and the target/runner matrix.

**Installer binding is producer-bound, not text-parsed.** `planner.py:348-399` rejects a missing/symlinked generator, copies the four archives + checksums into a temp dir, re-runs the governed generator, and requires `read_bytes()` equality. I re-confirmed the property this depends on: `OUT` appears in `generate_version_installer.sh` only at `:126/:127/:801/:802` and never in the heredoc body, so `--out` does not perturb output bytes — the binding is achievable in production, not vacuously fail-closed.

**Stable-channel derivation fixes a latent defect.** The old `APP_CHANNEL="${VERSION#*-}"; %%.*` would have yielded `"0"` for `0.1.0`; `:80-86` now derives `stable` explicitly, and `cases/artifact_stable_candidate_generation.sh:34,50` asserts `APP_CHANNEL="stable"` in the installer and `"channel": "stable"` in the receipt for all four targets.

**Publication remains gated.** `create_new_version.sh:186` still hard-fails on stable-looking versions, so widening the shared builder/installer to stable SemVer does not open a publication path. `plan-stable-release` refuses any `--out` inside the repository and refuses to overwrite. `self_update_receipt.rs` widens only read-only receipt *discovery*; `rejects_stable_and_rc_versions` / `rejects_stable_metadata` still pass, and `stable_gate_inventory.json:86-92` records the split precisely ("read-only receipt discovery accepts alpha, beta, and stable; stable update resolution remains gated").

**Custody.** Both the collector (`collect_qualification_artifacts.py:208-227`) and the planner (`planner.py:239-263`) reject container symlinks, member symlinks, nested directories, and any resolved path outside the custody root; sizes and digests are checked against the index. `artifact_index.py` binds every id to its exact kind/target/upload-name/file-name, requires the exact 20-id set, full four-target coverage per target kind, singleton counts, and per-artifact expiry ≥ the workflow boundary, with `require_unexpired` wired through the collector and planner. Retention is derived from `expires_at − created_at == 30 days`, not asserted from config.

**Tests are not vacuous.** `test_materialized_planner_contract` and `test_plan_digest_sensitivity` require the planner to *accept* a fully valid bundle (`run_planner` raises on rejection), so the 17 drift negatives in `test_planner_rejects_drift_cases` cannot pass by blanket fail-closed behavior. `expect_planner_rejected` also asserts no raw `Traceback`.

**Non-UTF-8 governance is complete.** `grep -rn 'decode('` over `scripts/distribution/**` returns exactly one site (`verify_release_archive.py:172`), and it is wrapped into a governed `SystemExit`. `read_utf8`/`read_evidence_text` cover the checksum, sysroot-manifest, aggregate-checksums, and release-profile reads; the shell case asserts the governed message and the absence of a traceback for both non-UTF-8 sysroot and non-UTF-8 checksum inputs.

**Cross-cutting consistency.** `build_preview_artifacts.sh` → `build_release_artifacts.sh` is renamed at every live call site (`create_new_version.sh`, `sysroot_release/runner.py`, `cases/common.sh`, the preview demo README, `distribution_pipeline.md`); the only residual old-name references are in `plans/issues/archive/**` historical records, which are immutable history. The `qualification` suite is registered consistently in `manifest.json`, `runner.py` (both execution and `validate_suite_case`), `release_report.py:REQUIRED_SUITES`, `schema_contracts.py`, `selftest.py`, `sifr_verify/selftest.py`, and the merge/nightly/release profiles. `create-pr.json` has no `distribution_release` entry at all (pre-existing), so nothing regressed there and the merge gate covers the new suite.

**Capability naming.** `grep -rniE 'phase[_ -]?40|milestone[_ -]?40'` over `scripts/distribution/`, `verification/areas/distribution_release/`, the qualification workflow, the demo, `self_update_receipt.rs`, and `internal_docs/distribution_pipeline.md` returns **no matches**. The two `milestone 40.2` strings are confined to `plans/releases/stable_gate_inventory.json` dispositions — a planning artifact under `plans/`, which is the correct home for milestone-scoped forward commitments.

**Documentation.** `internal_docs/distribution_pipeline.md` accurately describes the workflow's read-only authority, the collector's index, the planner's non-mutating contract including governed-producer regeneration and byte equality, the `--locked` build, the demo, and the receipt-channel split. I re-verified each clause against the code rather than against pass 8's summary.

## 6. Non-blocking observations (no change requested, carried forward)

- **The installer-invocation pin is a substring test, so it does not assert uniqueness.** A workflow retaining the governed call *and* appending a second `generate_version_installer.sh` call would still satisfy it. This is narrower than the pass-7 defect — realistic drift (replacing or extending the existing call) is now rejected — and the failure mode is a clean plan-time rejection (`$.installer_sha256: transported installer bytes do not match the governed generator`), never a bad installer shipping. The file's own `count(...) != 1` idiom would close it if maintainers ever want belt-and-braces. The tracker does not claim uniqueness.
- `expect_planner_rejected` asserts exit ≠ 0 and no traceback, not the specific rejection reason, so a negative could in principle be rejected for a different governed reason than intended. Low value to tighten: byte-equality binding makes the installer negatives reject for the right reason by construction.
- `qualify_stable_target.py:227-230` imports `hashlib` inside `sha256_bytes`. Cosmetic.
- `collect_qualification_artifacts.py:283-291` accepts whatever single `.vsix` filename is transported; not a gap, since `planner.py:330-345` binds `vsix_sha256`, `package_path`, `package_version`, and `compiler_compatibility` to the editor report.

---

## Findings

**None.** No actionable correctness, security, testing, documentation, tracking, scope, or capability-naming issue remains at `aeff4d07a`.

The PR head is exactly what pass 8 approved plus three tracker-only commits whose every claim I verified against the artifacts and the PR. All suites, guardrails, and the workflow contract pass at this head; publication and stable resolution remain gated; the tracking record and PR body are accurate.

**APPROVED**

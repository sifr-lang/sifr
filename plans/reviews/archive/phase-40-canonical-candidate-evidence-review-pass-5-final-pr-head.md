I modified no files. Final exact-head review below.

## Reviewed head

- **PR #3051** — `codex/phase40-canonical-rust-evidence` → `main`, state OPEN, `isDraft: false`, `mergeable: MERGEABLE`, `mergeStateStatus: CLEAN`
- **Exact reviewed remote head: `90cda61b9e8ce68bc38b5347de0ed5faeca69362`** — `gh pr view` `headRefOid` == `git rev-parse origin/codex/phase40-canonical-rust-evidence` == local `HEAD`, all three identical
- Base `origin/main` = `53cc9c4bf36762d39a0b372402d202589f920c2e`; diff = **13 files, +657/−14**; GitHub's file list matches the local diff exactly
- Three commits: `8048de434` (implementation), `1841576ce` (ledger), `90cda61b9` (remediation of the pass-4 findings: +105/−6 across 7 files)
- Working tree carries one untracked, **0-byte** file (`plans/reviews/active/…pass-5-final-pr-head.md`), not part of the PR. I did not touch it.

## Checks run

| Check | Result |
|---|---|
| `python3 -m verification.runner.sifr_verify --self-test` | **11/11 pass**, incl. `release report production self-test` (the harness carrying the new symlink case) |
| `areas run --area distribution_release` (full, independent rerun at this head) | **125/125, 0 failures**, 0 blocking, 0 non-blocking |
| `areas run --area distribution_release --suite qualification --suite evidence-custody` | **2/2**; `qualification self-tests ok: tests=9` incl. `test_plan_digest_sensitivity` |
| `check_file_size_guardrails.py` | PASS (2952 files, limit 900); largest touched file `planner.py` **832** |
| `check_hir_maintainability_guardrails.py` | PASS |
| `git diff --check origin/main..90cda61b9` | clean |
| Custom discrimination probes (finding 1: 4 cases; finding 2: 4 cases) | see below |

**Probe results.** For `canonicalize_custodied_results`: a *regular* file with pretty bytes is ACCEPTED and rewritten to exactly `canonical_json_bytes` (verified byte-equal against the repo's own helper, trailing newline included); a symlink to a file **outside** the result root is REJECTED, the outside target's bytes are **unchanged**, and the symlink is not clobbered; an inside-pointing symlink and a dangling symlink are also REJECTED with the target untouched. For `validate_staged_support_claims`, holding staged bytes and source *content* identical and varying only the source node type: regular → ACCEPT, symlink → REJECT, missing → REJECT, directory → REJECT. Symlink-ness alone flips the verdict, so the guard is load-bearing rather than incidentally tripping on content drift.

## Disposition of the four pass-4 findings

**1 — RESOLVED (with the requested mutation proof).** `release_evidence.py:133` is now `if not path.is_file() or path.is_symlink():`. The new test at `release_evidence_selftest.py:203-214` builds a symlink to `outside-rust-result.json`, asserts the `GovernanceError`, and then asserts `symlink_target.read_bytes() == symlink_bytes` — an explicit "release custody rewrote a symlink target" mutation assertion, exactly the coverage asked for. It is reached: `_assert_canonicalization_rejections` is called at `release_evidence_selftest.py:180`, inside the function wired as `release report production self-test` (`selftest.py:36,53`), which passed. My independent probe reproduces the reject-and-preserve behavior on out-of-root, in-root, and dangling links.

**2 — RESOLVED, and on the enforcing path.** `planner.py:226-231` adds the regular-non-symlink guard to `validate_staged_support_claims` — the gate `materialize_stable_plan` actually calls (`planner.py:159-162`), not just the staging helper. Direct coverage at `evidence_custody_selftest.py:205-211`. I confirmed that assertion is genuinely discriminating: at that point in the test `output` holds `canonical_json_bytes(payload)` and the symlinked source resolves to a file with **that same payload**, so pre-fix the digest comparison would have passed and the case would have been a false green. The symmetry with `stage_stable_support_claims` (`planner.py:203-207`) is now exact — identical message and predicate on both sides. No false-positive risk for the real flow: the check is on the file node, not its parents, so a symlinked *checkout directory* still validates, and the real repo path is a regular non-symlink file.

**3 — RESOLVED.** `internal_docs/distribution_pipeline.md:232-241` now gives the concrete `cp <clean-source-checkout>/target/verification/areas/rust-interop-release-results.json <release-work-dir>/rust-validation-report.json` followed by "Pass that copied result to the planner's `--rust-validation-report` argument." The source path matches the writer's actual emission site (`result_root = REPO_ROOT/target/verification/areas`, `release_evidence.py:91`, joined with `CRITICAL_RESULTS["rust_interop"] = "rust-interop-release-results.json"`), the flag name matches `release_governance.py:146` (`required=True`), and the placeholder vocabulary matches the two blocks above it. The block now parallels the `--documentation-report` handoff it was measured against. Code fences balance (50, even).

**4 — RESOLVED.** The inaccurate clause is gone. `phase-40-stable-channel-ga-execution.md:1339-1345` now reads that claims are staged from the exact source checkout and "the staged bytes must be canonical JSON and exactly equal the source claims' canonical representation" — no longer asserting the source bytes are canonical, which is the whole premise of blocker B. Wrong-area rejection is correctly reattributed to "The release evidence writer" (`release_evidence.py:138`), and the residual list is hedged as "the writer and planner reject **applicable** noncanonical, drifted, symlinked, and duplicate-key inputs," which is accurate: duplicate-key rejection is real on both sides via `load_json_bytes_strict`'s `object_pairs` hook (`common.py:197-203`), and the writer canonicalizes rather than rejects noncanonical bytes — which "applicable" correctly accommodates. The new bullet at `:1357-1369` accurately records pass-4's head, its 125/125 and 1/1 reruns, and its verdict.

## Disposition of blockers A–C

All three remain **CLOSED** at this head; the remediation commit touches none of their mechanics. **A** — `write_release_profile_report` still calls `canonicalize_custodied_results(result_root)` at `release_evidence.py:92` before `build_release_profile_payload`, so `sha256_file` binds canonical bytes and the custody contract's `require_canonical=True` agrees; the new symlink predicate only narrows the accepted input set and cannot let a non-canonical result through. **B** — `planner.py:195-212` still emits `canonical_json_bytes(source)` and `:222-235` still requires exact equality; the added source guard strengthens it. **C** — `validate_rust_candidate_result` and the `verify_artifacts=True` re-hash are untouched, and `planner.py:153` still loads the candidate report with `require_canonical=True`. The full-area 125/125 independently confirms no regression in any of the 125 governance cases.

## Findings

**No actionable finding at any severity.** All four pass-4 LOWs are closed with real, discriminating coverage, and the remediation introduces no new code path beyond two guard predicates and two test cases.

Two INFO residuals carry forward unchanged, neither a hole nor a blocker:
- `release_evidence.py:246` — `collect_critical_results` still uses plain `json.loads` for all four critical areas, so duplicate keys in the three *non-rust* results would be tolerated when deriving `suite_results` case ids. Those results are not candidate-planner entrances, and the one that is (`rust_interop`) is now strict via `load_json_strict` at `release_evidence.py:137`.
- `release_evidence.py:133-140` — a check-then-write TOCTOU on the result path. Not meaningful in a single-operator local release run, and the digest is taken on the bytes actually written.

PERF-HOST remains reasonably non-blocking: this diff touches no timeout, threshold, baseline, or profile selection, and `python_interop` is untouched. I did not re-run the isolated `readonly-check-doctor` case; pass 4 independently passed it 1/1 at 131.98 s and nothing in the remediation commit can affect it.

Scope holds: the diff touches only Phase 40 release governance, its self-tests, operator docs, and review/ledger artifacts. No Rust-interop implementation, capability claim, suite selection, or demo is added or required — Rust suites are consumed as evidence only.

## Verdict

**SATISFIED**

**PR #3051 is ready to merge.** All three original blockers stay closed, all four pass-4 low-severity findings are correctly and verifiably resolved at the exact pushed head `90cda61b9`, and I independently reproduced the full distribution area at 125/125 alongside the runner self-tests, both guardrails, and the diff check. Nothing outstanding gates the merge.

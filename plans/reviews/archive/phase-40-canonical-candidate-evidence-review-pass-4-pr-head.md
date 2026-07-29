## Reviewed head

- **PR #3051** — `codex/phase40-canonical-rust-evidence` → `main`, state OPEN, `mergeStateStatus: CLEAN`, `mergeable: MERGEABLE`
- **Exact reviewed remote head: `1841576ce667a23b680b2c248f178025e49d0930`** (`gh pr view` `headRefOid` == `git rev-parse origin/codex/phase40-canonical-rust-evidence` == local `HEAD`)
- Base `origin/main` = `53cc9c4bf36762d39a0b372402d202589f920c2e`; diff = 12 files, +558/−14
- Two commits: `8048de4343` (entire implementation + tests + docs + 3 review archives, 529 lines) and `1841576ce` (ledger-only, 29 lines)
- Working tree carries one untracked file (`plans/reviews/active/…pass-4-pr-head.md`), not part of the PR. I modified nothing.

## Commands run

| Check | Result |
|---|---|
| `sifr_verify --self-test` | 11/11 pass, incl. `release report production self-test` |
| `areas run --area distribution_release` | **125/125**, 0 failures (independently reproduces the claimed count) |
| `areas run --area distribution_release --suite qualification --suite evidence-custody` | 2/2; `qualification self-tests ok: tests=9` incl. `test_plan_digest_sensitivity` |
| `areas run --area python_interop --suite readonly-check-doctor` (isolated) | **1/1 pass, elapsed_ms=131984** (2:12 wall) |
| `check_file_size_guardrails.py` | PASS (2952 files, limit 900); largest touched file `planner.py` 826 |
| `check_hir_maintainability_guardrails.py` | PASS |
| `git diff --check origin/main..HEAD` | clean |
| Custom adversarial probe of every new branch (14 cases) | see below |

Probe results (real repo + temp fixtures): real `stable_support_claims.json` is **non-canonical** (5776 B; raw digest `4913b281…` vs canonical `b62f5b93…`), staging produces canonical bytes and 29 ids; `canonicalize_custodied_results` REJECTS non-dict, wrong-area, missing, directory, duplicate-key, and is idempotent; `validate_staged_support_claims` REJECTS missing source, noncanonical staged, source-drift, symlinked staged; `stage_stable_support_claims` REJECTS symlinked source, contract-violating source (and writes nothing), `--out` inside source, `--out` == source root, `--out` reaching source through a symlinked parent, duplicate source keys.

## Disposition of previous blockers A–C

**A (release report hashed pretty bytes, custody demanded canonical) — CLOSED.** `release_evidence.py:91-92` canonicalizes the rust result *before* `build_release_profile_payload`, so `collect_critical_results`' `sha256_file` (`release_evidence.py:249`) binds canonical bytes and custody's `require_canonical=True` (`evidence_custody.py:262`) and the digest tie now agree. Confirmed the runner emits `json.dumps(payload, indent=2, sort_keys=True)` with no trailing newline (`area_adapter.py:94`) — never canonical, so the step always applies. Ordering is safe: `write_release_profile_report` runs after `reports.summarize` (`profile_runner.py:872-889`), so no in-run consumer sees pre-canonical bytes, and the operation is idempotent.

**B (staged claims non-canonical) — CLOSED, and the digest move is safe.** `planner.py:195-212` emits `canonical_json_bytes(source)`; `planner.py:215-233` requires the staged file to equal exactly that. I re-verified pass-2's key premise at this head: nothing compares `stable_support_claims_sha256` to the raw source file — `stable_prepare.py:757-781` binds only the compatibility matrix, facts schema, and facts generator to the checkout. The claims digest is still auditable after the fact, since canonicalization of the source file at `plan.source_commit` is deterministic.

**C (staged rust report came from a standalone rerun) — CLOSED.** `validate_rust_candidate_result` (`planner.py:738-756`) requires the candidate report's digest to be the single `result_artifacts` entry named `rust-interop-release-results.json` *and* the single `stable-candidate` `suite_results` digest; `validate_release_profile_report(..., verify_artifacts=True)` re-hashes that path on disk in the release checkout (`release_report.py:347-352`). A rerun differs in `duration_ms`, so it cannot satisfy the tie.

**Plan-digest sensitivity** is no longer degenerate: `qualification_plan_digest_selftest.py:98-113` asserts inside the `rust-claims` branch that both `stable_support_claims_sha256` and `advertised_claim_ids` differ from baseline, and `stable_claims(variant="rust-claims")` (`qualification_fixture.py:820-828`) genuinely appends `diagnostic_fixture`, so the assertion is substantive rather than riding on the changed fixture commit.

## Findings (all non-blocking)

1. **LOW — `canonicalize_custodied_results` accepts a symlinked result path and writes through it.** `release_evidence.py:133-140` uses `path.is_file()` (follows links) and `path.write_bytes`. Probe: symlink to a file outside `target/` → ACCEPT, symlink preserved, outside file rewritten. Inconsistent with the sibling convention in this same PR (`planner.py:204`, `planner.py:220`) and with `planner.py:435`. Integrity is preserved (the digest is taken on the resolved file) and `clear_critical_result_files` (`release_evidence.py:77-80`) unlinks the path at prepare time, so only the write location can escape. Fix: add `path.is_symlink()` to the guard.

2. **LOW — asymmetric symlink policy on the *source* claims file.** `stage_stable_support_claims` refuses a symlinked source (`planner.py:204-208`) but the enforcing gate `validate_staged_support_claims` does not (`planner.py:226` loads `source_root / STABLE_SUPPORT_CLAIMS` unchecked). Probe: stage REJECT, validate ACCEPT. Requires a committed symlink at that path, so low risk — but the check belongs on the side that gates the plan.

3. **LOW (docs) — the block never names the file to pass as `--rust-validation-report`.** `internal_docs/distribution_pipeline.md:229-232` states the principle ("Candidate custody copies that same result file; a standalone Rust-suite rerun is not interchangeable evidence") but, unlike the documentation-report block directly above it which gives a command plus "Pass that exact report to the planner's `--documentation-report` argument", never names `target/verification/areas/rust-interop-release-results.json` → `<work-dir>/rust-validation-report.json`. That is exactly the step the operator got wrong in pass 1. Fail-closed at the planner now, so this is completeness, not correctness.

4. **LOW (tracking accuracy — introduced by the post-implementation commit `1841576ce`).** The new ledger bullet at `plans/issues/active/phase-40-stable-channel-ga-execution.md:1339-1341` says "both the source and staged bytes are validated as canonical JSON." The source bytes are explicitly *not* canonical — that is Finding B's premise, and `stage_stable_support_claims` loads the source with `load_json_strict(source_path)` with no `require_canonical`. The same bullet attributes "wrong-area" rejection to the planner; it lives in the evidence writer (`release_evidence.py:138`). The corresponding bullets in `8048de4343` are accurate; only this restatement is loose. **Otherwise the tracking-only commit introduces nothing** — it touches one markdown file, no code, no test, no schema.

5. **INFO — coverage residuals, none of them a real hole.** (a) No planner-level negative case for a non-canonical `--rust-validation-report`; the digest chain already forecloses it, so the new `require_canonical=True` at `planner.py:153` is belt-and-braces. (b) The planner fixture writes canonical source claims (`qualification_fixture.py:143`) and a directly-written staged copy (`:366-368`) rather than calling the governed staging function, so the pretty→canonical transformation in the planner path is covered only by `_test_stable_support_claim_staging` (`evidence_custody_selftest.py:121-205`) — which does write a pretty source, so the class is covered. (c) `collect_critical_results` still uses plain `json.loads` (`release_evidence.py:246`) for all four areas, so duplicate keys in the three non-rust results would be tolerated when deriving `suite_results` case ids; those are not candidate-planner entrances and their JSON semantics never reach custody, and the one that does is now strict. Pass-3's carried cosmetic (dangling `--out` symlink writes to the resolved target) reproduces and remains fail-closed.

**PERF-HOST:** reasonably nonblocking. `python_interop` is untouched by this diff (its last commits predate the branch), the diff changes no timeout, threshold, baseline, or profile selection, and PERF-HOST is a pre-existing indexed deferred follow-up (`plans/phases/index.md:54`). My isolated replay passed 1/1 at 131.98 s, consistent with the reported 160.114 s under different host load.

No regression, no unsafe fallback, and no incomplete Phase 40 contract found. The diff touches no Rust-interop implementation, capability claim, suite selection, or demo — Rust suites are consumed as evidence only.

## Verdict

**SATISFIED**

**PR #3051 is ready to merge.** All three prior blockers are closed and independently reproduced at the exact pushed head; the five findings are LOW/INFO and none gate the release-governance contract. Items 1–2 (symlink guards) and 3–4 (docs/ledger wording) are worth a follow-up, and item 4 in particular is a one-sentence ledger correction that would be cheap to fold in before merge if you want the record exact.

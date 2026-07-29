## VERDICT: SATISFIED

All six hardening requirements are implemented and independently reproduced. No blocking findings.

### Independent verification

**1 · Rust-claims sensitivity is no longer degenerate — CLOSED.** `qualification_plan_digest_selftest.py:101-113` now asserts, inside the `rust-claims` branch, that both `rust_interop.stable_support_claims_sha256` **and** `advertised_claim_ids` differ from baseline (`or`-joined equality → fails if either matches). The assertion is substantive, not tautological: `stable_claims(variant="rust-claims")` (`qualification_fixture.py:820-828`) appends `diagnostic_fixture`, so ids and digest genuinely diverge, and `write_source_contracts(..., variant=variant)` keeps source and staged claims consistent so the planner still binds them. `test_plan_digest_sensitivity` passes in-suite; the `nochange`/fresh-control commit-stability invariants (lines 38-62, 79-80) still hold with the new source file present.

**2 · Canonicalized release-run bytes + digest binding — CLOSED, and format-faithful.** `release_evidence_selftest.py:99-121` writes results with `indent=2, sort_keys=True` — byte-for-byte the real producer's format (`verification/areas/rust_interop/runner/report.py:13`), so the fixture is not a straw man — then asserts `before != after`, `after == canonical`, and `load_json_strict(..., require_canonical=True)`; `:155-162` asserts the payload's `result_artifacts` digest equals `sha256_file` of the canonicalized file. `_assert_canonicalization_rejections` covers missing, wrong-area, and duplicate-key with message matching (`fail()` → `GovernanceError` carrying "duplicate object key", confirmed at `common.py:201`). Ordering re-verified: `write_release_profile_report:91-92` canonicalizes before payload build and both validation passes; nothing else in `reports.py`/`profile_results.py`/`results.py` records a pre-canonical digest.

**3 · Staging/validation mutations + real CLI — CLOSED.** `_test_stable_support_claim_staging` covers canonical staging, real `subprocess` CLI wiring, overwrite refusal, noncanonical staged bytes, source drift, in-checkout output, and symlinked source. I ran the CLI myself: `rc=0`, 56 ms, output canonical. I also confirmed the untested-but-present branches fail closed against the **real** repo claims file (29 claims, 4517 B canonical): symlinked staged path → reject, top-level array → reject, missing source → reject.

**Mutation efficacy (each guard individually neutralized in-memory, no repo files touched):**

| weakening | test result |
|---|---|
| drop source-byte binding (keep `require_canonical`) | caught → "source-drifted … passed" |
| drop `require_canonical` + binding | caught → "noncanonical … passed" |
| drop in-checkout `--out` check | caught → "in-source … passed" |
| drop symlink-source check | caught → "symlinked source … passed" |
| no-op `canonicalize_custodied_results` | caught → "did not canonicalize the exact Rust result bytes" |
| drop area-identity check | caught → "mutation passed: identity mismatch" |

**4 · Planner earliest-load canonical gate — CLOSED.** `planner.py:153` now loads the rust validation report with `require_canonical=True`; it is the only planner load of that path (earlier lines 148-151 are digest-only, no load). No new breakage: custody already required canonical bytes at `evidence_custody.py:261`.

**5 · Blockers A–C intact, scope clean.** Ordering, source-derived staging (`validate_staged_support_claims` returning the ids the plan must match), and the digest tie to the release-run result all hold. Zero files under `crates/`, `demos/`, or `verification/areas/rust_interop/` — only `rust_interop` in custody is the rust *result*, and canonicalizing only that key is correct: it is the sole `CRITICAL_RESULTS` entry that enters candidate custody (`evidence_custody.py:190-207, 247-263`).

**6 · Docs/ledger/guardrails truthful.** The new doc block's claims all check out, including "the planner later verifies source cleanliness" (`planner.py:270`) and the `stable-support-claims.json` name matching custody's required filename — this also resolves pass-2 finding 7. Ledger entries match the archives on disk. File-size guardrails PASS (2952 files); `git diff --check` clean; import ordering alphabetical.

**Gates re-run independently:** `sifr_verify --self-test` all pass (incl. release report production); `distribution_release` full area **125/125, 0 failures**; qualification suite 9/9; evidence-custody pass; `--suite full` 67/67 with `test_evidence_custody_mutations` passing in `governance-contracts`.

### Non-blocking findings

1. **LOW — `canonicalize_custodied_results` accepts a symlinked critical result and writes through it.** Verified: a `rust-interop-release-results.json` symlink is followed and the target is rewritten. The dir is runner-owned (`target/`), so risk is nil, but sibling code (`validate_staged_support_claims`, `validate_installer_bytes`) refuses symlinks; `path.is_symlink()` would match convention.
2. **LOW — `stage_stable_support_claims` writes through a dangling `--out` symlink.** `output_path.resolve()` means the in-checkout and refuse-existing checks apply to the *resolved* target, so it stays fail-closed — but the bytes land somewhere other than the literal `--out`. Cosmetic.
3. **LOW — the planner's new `require_canonical` on the rust report has no negative test.** Only the happy path is exercised; a noncanonical case in the existing `test_planner_rejects_drift_cases` framework would be a few lines.
4. **INFO — `plans/reviews/active/phase-40-canonical-candidate-evidence-review-pass-3.md` is a 0-byte placeholder** (same shape as pass-2 finding 6). It must carry a real verdict or not be committed, since the ledger pattern cites these files.
5. **INFO — pass-2 finding 8 is moot.** `GovernanceError` subclasses `ValueError`, so the NaN path (plain `ValueError` from `canonical_json_bytes`) is indistinguishable to `profile_runner.py:894`, which fails closed with status 2.
6. **INFO — headroom shrank.** `planner.py` 826/900, `qualification_fixture.py` 853/900. The next addition to either should split by responsibility rather than append.

No files were modified.

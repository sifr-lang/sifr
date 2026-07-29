# VERDICT: SATISFIED

All three pass‑1 blockers are closed, and I reproduced each closure independently against real code paths. No blocking findings. Nine non‑blocking items follow, the first two of which I'd fix before merge.

## Blocker closure — independently verified

**Finding A (release‑report digest vs. canonical custody) — CLOSED.** `release_evidence.py:130-140` canonicalizes the exact release‑run Rust result in place, and `write_release_profile_report:91-112` calls it *before* `build_release_profile_payload` (which hashes via `sha256_file` at `:249`) and before both `validate_release_profile_report` passes. Ordering is correct. Driving the real `rust_candidate_result()` fixture through it:

```
pretty digest      : 18c3bfae2c78fdc9...      (pre-normalization)
post-write digest  : 7dcf81696f2030a4...
canonical digest   : 7dcf81696f2030a4...      tie holds: True
custody require_canonical on post-write bytes: ACCEPT
```

The formerly impossible state is now the only satisfiable one. Fail‑closed branches all fire: missing result → `REJECT`, wrong `area` identity → `REJECT`, duplicate keys → `REJECT`.

**Finding B (noncanonical source claims) — CLOSED.** The real `verification/areas/rust_interop/data/stable_support_claims.json` is confirmed noncanonical; `stage_stable_support_claims` (`planner.py:195-207`) emits `canonical_json_bytes(source)` (29 claims, 4517 B), and `validate_staged_support_claims:210-223` requires exactly those bytes. Verified rejections: noncanonical staged bytes, canonical‑but‑drifted staged bytes, staged bytes after the source drifts underneath, missing source, overwrite, out‑path inside the source root, and out‑path reaching the source root through a symlinked parent. The digest is safe to move off the source file's raw bytes — I confirmed no validator compares `stable_support_claims_sha256` to the in‑repo file (`stable_prepare.py:753-784` binds only the compatibility matrix, facts schema, and facts generator to source).

**Finding C (standalone rerun) — CLOSED.** Candidate custody's rust report must equal the release report's `result_artifacts` digest (`planner.py:728-746`, `evidence_custody.py:252-260`, `stable_prepare.py:801-810`), and the planner additionally re‑verifies that artifact on disk under `source_root` (`verify_artifacts=True` → `release_report.py:347-352`). Since that digest is now the canonicalized release‑run bytes, a standalone rerun (which differs in `duration_ms`) cannot satisfy it. The doc's "a standalone Rust‑suite rerun is not interchangeable evidence" is enforced, not aspirational.

**Post‑run ordering / interactions.** `write_release_profile_report` runs at `profile_runner.py:885-894`, after the full run and after `reports.summarize`, so no consumer sees rewritten bytes; `prepare_release_report_output` deletes the critical results up front, so no stale double‑normalization. Dirty‑source staging fails closed downstream via `validate_source_identity`. Re‑ran locally: `distribution_release` area 125/125 variants, 0 failures; `sifr_verify --self-test` all pass including release report production; file‑size guardrails PASS (2952 files); `git diff --check` clean. Zero files under `crates/`, `demos/`, or `rust_interop/` touched — no Rust‑interop implementation and no demo‑naming violation.

## Findings

**1 · MEDIUM — the `rust-claims` digest‑sensitivity assertion is now degenerate.** `qualification_plan_digest_selftest.py:65,83` moves `rust-claims` from the same‑source input group into the new‑source group. That variant now rewrites the fixture source's claims file, which changes the fixture commit — verified: baseline `4efeff2cfa67…`, rust‑claims `0d31dbd046ac…`. The plan digest therefore changes via `$.source_commit` alone, so the assertion passes even if claim content were ignored entirely. Phase DoD line 563‑564 ("changing any … Rust-claim input … changes the fixture plan digest") and the ledger's "Rust‑claim … digest sensitivity" evidence line are now trivially satisfied. The move itself is *necessary* — a same‑source claims variant would now be rejected by the source binding — but the isolation was lost with it. Fix: inside the `rust-claims` branch, also assert the produced plan's `rust_interop.stable_support_claims_sha256` and `advertised_claim_ids` differ from baseline's.

**2 · MEDIUM — the branches that actually close A and B have no regression test.** Untested: `validate_staged_support_claims`'s source‑mismatch branch and its `require_canonical` gate; `stage_stable_support_claims`'s in‑source‑output refusal; `canonicalize_custodied_results`'s missing‑file and area‑identity branches. `_test_stable_support_claim_staging` covers only canonical output and overwrite refusal. Nothing exercises the two layers end‑to‑end — pretty runner bytes → canonicalized digest → custody `require_canonical` + digest tie — which is precisely Finding A's defect class; each side asserts its half with the same helper. I verified all of it by hand, so this is coverage, not correctness. Recommend one `qualification_selftest` drift case (noncanonical / source‑drifted staged claims) and one custody case seeded from pretty‑printed rust bytes passed through `canonicalize_custodied_results`.

**3 · LOW — new CLI subcommand untested.** `stage-stable-support-claims` is exercised only at function level; an argparse wiring error would be invisible. Verified manually: it stages canonically and refuses overwrite with a governed message.

**4 · LOW — symlinked source claims accepted.** `stage_stable_support_claims` does not reject a symlinked `stable_support_claims.json` (verified `ACCEPT`), unlike the sibling convention at `planner.py:425` (`validate_installer_bytes` refuses a symlinked generator). Low risk — the planner requires a clean checkout, so the link would have to be committed — but a `source_path.is_symlink()` refusal would match the surrounding code. Same for the staged path in `validate_staged_support_claims` (candidate custody catches it later, at `evidence_custody.py:175-181`).

**5 · LOW — planner still loads the rust report without `require_canonical`.** `planner.py:153` is now the only planner evidence load that omits the flag. It's enforced transitively by the digest tie, but a report from a pre‑remediation writer would pass the planner and fail only later in custody. Adding the flag moves the failure to the earliest point.

**6 · INFO — two housekeeping items before commit.** `plans/reviews/active/phase-40-canonical-candidate-evidence-review-pass-2.md` is a 0‑byte placeholder, and the pass‑1 archive the ledger cites is still untracked. Both must land in the commit for the ledger reference at `phase-40-stable-channel-ga-execution.md:344-345` to be truthful.

**7 · INFO — doc vs. flag default.** `internal_docs/distribution_pipeline.md:223` shows `--source-root <clean-source-checkout>`, but the flag defaults to the invoking repo root and staging performs no cleanliness check. It fails closed downstream, so this is documentation nuance only. Every other claim in the new doc block and the new ledger entries checks out against the code.

**8 · INFO — NaN handling.** A NaN in a runner result surfaces as a plain `ValueError` from `canonical_json_bytes` rather than `GovernanceError` (verified). `profile_runner.py:894` catches `ValueError`, so it fails closed with status 2 and no traceback leak. This is strictly stronger than the pre‑diff behavior, which would have hashed the bytes and moved on.

**9 · INFO — headroom.** `planner.py` 816/900, `qualification_fixture.py` 853/900. The next change to either should split rather than append.

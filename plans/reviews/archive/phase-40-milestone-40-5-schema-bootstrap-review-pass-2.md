# Review: Phase 40 M40.5 schema-v2 preview epoch bootstrap (pass 2)

I read the full `git diff`, all 14 untracked files, the phase/issue docs, and the archived pass-1 report, and independently re-ran the gates.

## Pass-1 re-audit

| # | Pass-1 finding | Status |
|---|---|---|
| 1 | Prepare→publish artifact handoff breaks on re-run | **Resolved.** `release-publication-prepare.yml:197,215` emits and uploads under the same computed name, exported at `:25-26,39`; publish downloads `needs.prepare.outputs.summary_artifact_name` (`release-publication.yml:105`). Rerun-failed reuses the succeeded job's attempt-1 name (artifact still present under the unchanged `run_id`); rerun-all re-uploads at attempt 2. `overwrite: false` retained. |
| 2 | Two distinct approvers hard-fail | **Resolved.** `resolve_distinct_approvers` (`schema_bootstrap.py:87-121`) collects all distinct non-initiating `stable-release` approvers case-insensitively and fails only on an empty set; evidence stores arrays (`_validate_approvers`, `:227-243`). |
| 3 | Generation-1 evidence unbound to alpha stage / prepare | **Resolved.** `prepare_summary_sha256` is required at both stages, and `alpha_evidence.{sha256,run_id,run_attempt,initiator,approvers,prepare_summary_sha256}` at `preview-index` (`:167-198,337-348`). |
| 4 | Schema weaker than validator, no negatives | **Substantially resolved.** `uniqueItems` + per-id `minContains/maxContains` (schema `:68-112`), exact-9 channel-shaped asset maps via `minProperties`/`not minProperties` + `propertyNames` (`:140-157`), stage `if/then/else` with `not required` (`:114-127`), and two negatives in `schema_contracts.py:66-81`. I verified the repo's schema engine actually supports `contains/minContains/maxContains/propertyNames/if/then/else/not` (`verification/json_schema_202012.py:11-42`). I probed 14 instances against both surfaces: all structural cases agree; the only divergences are the four cross-field semantics 2020-12 cannot express (asset map vs sibling `version`, approver vs sibling `initiator` incl. case-folded uniqueness). Those are validator-only by necessity and are covered semantically (`schema_bootstrap_selftest.py:132,139-141`). |
| 5 | Producer untested | **Mostly resolved** — see finding 2 below. |
| 6 | Public-smoke override scrub too late | **Resolved.** Hard `test -z` before any request (`run_schema_bootstrap_public_smoke.sh:31-34`), gated by an ordering assert (`schema_epoch_bootstrap_workflow_contract.sh:113-116`). |
| 7 | Poller abandons a matched run | **Resolved.** Sticky `poll_error` + `break` reaches cancel (`poll_site_release_run.sh:54-56,94-96`), gated at `preview_release_workflow_yaml_parses.sh:110-114`. |
| 8 | Empty review artifact | **Recurs** — see finding 4. |
| 9 | `release-publication.yml` at 851 lines, unenforced | **Resolved.** 795 lines; guardrail covers `.github/workflows/*.{yml,yaml}` with a self-test (`check_file_size_guardrails.py:123,133-134,262,273-276`). |
| 10 | Reformatting noise | **Resolved.** `release_governance.py`/`runner.py` diffs are additions plus alphabetical insertion only; the one E731 conversion is explained. |
| 11 | Unvalidated `--ruleset-id` | **Resolved.** `verify_site_workflow_identity.sh:31`, gated at `site_release_workflow_contract.sh:87`. |

No schema-v1 parser, fixture, migration, or fallback exists: the only `schema-v1` hit is the pre-existing `schema_version: 1` *rejection* fixture at `cases/site_publication_facts_generated.sh:42`. Digest-chain, write-once, ordering, and approval behavior all check out, and `--clobber` count is pinned at 1.

Gates I re-ran here: epoch-bootstrap suite PASS; full distribution suite PASS, **variants=56**; the three touched/new contract cases PASS; file-size self-test + repository gate PASS; `bash -n` over all `scripts/distribution/*.sh` PASS. (Ruff is not installed under the active interpreter, so I could not independently reproduce that one.)

## Actionable findings

### 1. The smoke-output filename contract between bash and Python is ungated, and drift lands *after* the irreversible mutation — Medium

`schema_bootstrap.py:360-367` hashes `smoke_dir / f"{smoke_id}.txt"` for each of the four frozen `SMOKE_IDS` (`:33-38`). The producing side spells those four paths literally and independently in bash — `run_schema_bootstrap_public_smoke.sh:57` (`governance-index.txt`), `:67` (`dispatcher-default.txt`), `:77` (`dispatcher-stable-rejection.txt`), `:97-98` (`installed-self-update.txt`) — and the directory is coupled only by `--out publication/bootstrap-smoke` / `--smoke-dir publication/bootstrap-smoke` at `release-publication.yml:751,782`.

`schema_epoch_bootstrap_workflow_contract.sh:104-116` asserts six URL/message fragments in the smoke script but none of the four output paths. So renaming, relocating, or dropping one output keeps every gate green: the smoke script still exits 0, and the first failure is `FileNotFoundError` inside `materialize_schema_bootstrap_evidence.py` — at which point the version release is published, `channels-generation-1.json` is reserved, `channels.json` is replaced, and the site has deployed. The epoch is live, permanently un-evidenced, and the run cannot be re-run (`release-publication.yml:231-233` is write-once by design). Assert each `SMOKE_IDS` member appears as its `${out}/<id>.txt` literal, and that the workflow's `--out` and `--smoke-dir` are the same path.

### 2. The staged-alpha binding branch — the fix for pass-1 finding 3 — is unreached by the self-test — Low-Medium

`schema_bootstrap.py:326-336` is the invariant that makes generation-1 evidence trustworthy: the staged evidence must be `stage == "alpha-assets"` and its `alpha` block must equal the alpha block this run reproduced. `test_materializer` (`schema_bootstrap_selftest.py:288-358`) always passes the single matching `alpha_evidence_path`; the "unexpected alpha asset" case (`:330-335`) fails earlier inside `_materialize_release_evidence`, so the comparison never executes. Deleting lines 329-336 leaves the suite green. Add two cases: a second alpha evidence materialized from a different alpha version/assets, and the final evidence itself passed as `--alpha-evidence` (exercising the `stage` check).

### 3. The opaque pre-epoch identity is duplicated across four enforcement surfaces with no cross-surface equality gate — Low

`71b3243925…4bf9ef`/`105` is asserted independently at `release-publication-prepare.yml:47,133`, `release-publication.yml:82,275,570`, `schema_bootstrap.py:29-30`, and `schemas/schema_epoch_bootstrap_evidence.schema.json:33-34`. `schema_epoch_bootstrap_workflow_contract.sh:94-96` checks only that the *identifier* `LEGACY_INDEX_SHA256` is present, never that the four values agree. Drift fails closed (the Python constant rejects before mutation), but it is undetectable until a bootstrap attempt. Assert the literal digest and `105` in all four files from the contract case.

### 4. `plans/reviews/active/phase-40-milestone-40-5-schema-bootstrap-review-pass-2.md` is a 0-byte untracked file — Low

Same class as pass-1 finding 8. Populate it with this report and add the ledger entry in `plans/issues/active/phase-40-stable-channel-ga-execution.md` (the pass-1 archive is referenced at `:361-373`; pass 2 is not), or remove it before commit.

## Not findings

- The four schema/validator divergences above are inherent to JSON Schema 2020-12 sibling-value comparison and are covered by semantic tests; no parity regression.
- Post-`gh release create` failures are unrecoverable by re-run in every mode. That is the pre-existing write-once contract, not new to this slice.
- `prepare` checks artifact *presence* per target rather than the exact set; publish's `verify_release_publication_assets.sh:56-79` enforces exactness before any mutation, so the asymmetry only changes which job reports it.
- The external execution requirements from pass 1 (environment + reviewer configuration, prevent-self-review, live `channels.json` identity at bootstrap time, `sifr.sh` convergence budget) still stand and remain outside the implementation.

VERDICT: NOT SATISFIED

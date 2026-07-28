Verification and review complete. No files modified.

## Identity (independently verified)

| Ref | SHA |
|---|---|
| local `HEAD` | `a671c913116e6fa30073d6220abe639154b51e72` |
| `origin/codex/phase-40-milestone-40-5-stable-publication` (after fetch) | `a671c913116e6fa30073d6220abe639154b51e72` |
| PR #3043 `headRefOid` | `a671c913116e6fa30073d6220abe639154b51e72` |
| merge base / base | `f9837adb105f048ed56624c148ee83ecbd2a3d03` (`main`) |

All three identical. PR #3043 OPEN, base `main`. Diff: 1 commit, 31 files, +2270/−218. Working tree carries only the pre-existing untracked pass-2 slot file.

## What I confirmed at this exact head

**Fail-closed provenance chain.** `SOURCE_COMMIT` has no explicit regex in the stable step, but it is transitively pinned: `run_metadata.head_sha == SOURCE_COMMIT` (a real SHA) forces `SOURCE_COMMIT` to be a 40-hex, and `_require_checkout(source_root, plan["source_commit"])` (`stable_prepare.py:399-405`) forces the `ref: source_commit` checkout HEAD to equal the plan's commit — so `head_sha == SOURCE_COMMIT == plan.source_commit == qualification.source_commit` (the last checked at `stable_prepare.py:164-169`). Run/attempt binding checks `.id`, `.run_attempt`, `.head_sha`, `.conclusion == success`; artifact binding checks `.id`, `.name`, `.expired`, `.workflow_run.id`. `.expired` correctly uses `jq -r` not `-er` (`:334`) — with `-e` a healthy `false` would have exited 1 and inverted the guard.

**No injection.** `run_id`, `run_attempt`, `workflow_artifact_id/name` are validated by `validate --kind qualification-artifact-index --require-canonical` **before** interpolation into API URLs and paths. Names are fully pinned to `sifr-stable-candidate-<version>-<source_commit>-<suffix>` and rejected if they contain `/` (`artifact_index.py:166-206`), and file names are pinned per artifact id and must be single components (`:245`).

**`group_by(.id)` ambiguity is closed.** The workflow's jq takes `.[0].name` for a group; the new one-to-one id↔name map plus `len(...) != 6` in both `artifact_index.py:171-178,254-255` and `stable_prepare.py:433-446` make that safe, and it runs before the jq read.

**ZIP safety — exercised, not just read.** Rejects `..`, absolute, backslash, symlink/device members, duplicates, overwrite, non-empty/symlinked destination. I built adversarial archives: byte-count mismatch rejected *before any write* (`d2` left empty after a 10 MB highly-compressible member declared as 5 bytes); `zipfile` bounds decompression by the declared `file_size`, so the sum check is a real decompression bound, and `verify_transported_artifacts` (`planner.py:255-264`) then re-checks each file's actual size and SHA-256. Nested-directory content extracts but would break the exact byte sum, so undeclared content fails closed.

**Submodule binding is real** (I re-derived it rather than trusting pass 1): `validate_release_profile_report(..., source_root=…)` → `collect_submodules` uses `git submodule status --recursive` and rejects `-`/`+`/`U` (`release_report.py:354-366`), then `stable_prepare.py:139` requires report submodules == plan submodules.

**Mutation-free / credential-free.** `actions: read` + `contents: read` only; no `secrets:`; the sole caller (`release-publication.yml:62-70`) omits all five new inputs, and if `governance_mode=normal` ever reached it, prepare fails at the regex gate and `publish` (`needs: prepare`) is skipped. Only output is `prepare/summary.json` via `write_canonical_json(refuse_existing=True)`, pinned by the CLI double-run test. Preview/bootstrap behavior preserved: outputs unchanged, the new outputs degrade to `""` via `// ""` on absent keys (no jq error — indexing `null` is legal), and stable inputs are rejected outright (`:107-110`).

**Reproduced locally at this head:** stable-prepare selftest 6/6 · `--suite stable-prepare` variants=1/failures=0 · `full`+`stable-prepare` **60/60** with both `stable_publication_prepare_workflow_contract` and `stable-publication-prepare` present exactly once · evidence-custody 1/1 · coverage matrix 5/5 · `sifr_verify --self-test` all pass (schema count 16) · `validate_schema_contracts` pass · all three workflow-contract cases pass · file-size guardrails PASS (2908 files, largest new file 621) · `git diff --check` clean. Schema enum ↔ `EXPECTED_ARTIFACT_IDS` verified equal (20). No circular import from evidence_custody's new top-level `.planner`/`.artifact_index` imports.

**Pass-1 SATISFIED still holds.** Its byte-identity claims, the 6-upload/one-to-one closure, the submodule-transitivity correction, and the `jq -er` observation all reconcile with this head; the 0-byte placeholder it flagged is **not** committed. Ledger and doc text are truthful — `internal_docs/distribution_pipeline.md:591-606` describes only what exists, and "the later publish job must consume the exact summary digest" is stated as an obligation with the publish checklist item still unchecked.

**Scope.** No `crates/`, no `demos/`, no Rust interop implementation, no phase-numbered demo names, no wait thread, no production mutation path.

## Findings

None actionable.

**Nonblocking (no change required):**
- `.github/workflows/release-publication-prepare.yml:334` consults the live artifact's `.expired` but not its `.expires_at`; the seven-full-day window (`stable_prepare.py:_require_publication_window`) derives from the evidence's absolute timestamp. Correct as written for truthful evidence (absolute, with `retention_days == 30` pinned), but comparing the API's authoritative `expires_at` would close the forged-evidence case for free.
- `schema_contracts.stable_publication_prepare()` is schema-valid but would be *rejected* by `validate_stable_prepare_summary` (20 distinct workflow ids vs the required 6; `vsix_sha256` = `SHA_D` ≠ `artifacts.vsix.sha256`). Harmless — only the Python validator is authoritative — but the fixture no longer doubles as a realistic example.
- `stable_prepare_selftest.test_safe_artifact_extractor` covers traversal and symlink but not the byte-count mismatch branch. I verified that branch manually; a third fixture would pin it.
- Carried from pass 1 and still true: the candidate-evidence file set is hard-coded in three places (`stable_prepare.py:48-58`, `evidence_custody.py:26-30`, `:190-215`) with no shared constant, and `mode` (`initial`/`resume`) is recorded but has no behavioral effect until the publish job exists.

VERDICT: SATISFIED

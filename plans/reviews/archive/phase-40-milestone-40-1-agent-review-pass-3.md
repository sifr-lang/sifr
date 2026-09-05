## Review — Phase 40 / milestone_40_1, pass 3

Branch `codex/phase-40-milestone-40-1` (5 commits) vs `origin/main`; 40 files, +4208/−137.

### Pass-2 findings: re-tested

| # | Pass-2 finding | Status |
|---|---|---|
| 1 | Vacuous digest-sensitivity fixture | **Fixed** — `qualification_selftest.py:614-660` now shares one fixture source across the six evidence-only variants and adds a `nochange` control. Independently verified: two evidence bundles built from the same source produce byte-identical plans (`same digest: True`), so the control can actually fail |
| 2 | Symlinked upload container escapes custody | **Fixed** — `planner.py:227-236` resolves the path and requires `is_relative_to(resolved_root)` plus a non-symlink container. Probes: symlinked container → exit 2 governed; symlinked *file* pointing outside → exit 2 governed; a legitimately symlinked `--artifact-root` still accepted (no false positive). Also now a permanent regression case (`symlink-container` in `test_planner_rejects_drift_cases`) |
| 3 | `id` not bound to `target`/upload container | **Fixed** — `artifact_index.py:257-309` (`artifact_contract`) pins kind, target, upload suffix, and file name per governed id. All 18 mutations I threw at it were rejected, including the three pass-2 accepted ones plus name swaps, kind swaps, container swaps, extra/duplicate rows, and absolute upload names |
| 4 | Outside-repo output assertion confounded | **Fixed** — `qualification_selftest.py:435-441` now runs the guard against a valid bundle *before* the missing-artifact mutation. Independently verified with a fresh valid bundle: exit 2, `stable release evidence output must be outside the repository`, file not written |
| 5 | `retention_days`/`overwrite` asserted, not derived | **Fixed** — `collect_qualification_artifacts.py:148-152` derives retention from the API `created_at`/`expires_at` delta and requires exactly 30 days. Probes: 31d, 29d, naive timestamps, expired-flag, run-id mismatch, fork repo, `push` event, workflow-name drift, run-attempt drift, extra upload → all rejected |
| — | Workflow contract as a `validate` prerequisite | **Added** — `release-qualification.yml:65-67`; `build`/`editor` need `validate`, `assemble` needs `build`, `collect` needs all. See finding 2 for what this control does *not* cover |

Capability naming and tracking check out: `demos/stable_candidate_qualification_demo.sh` (executable, capability-named, no phase/milestone identifier), referenced from `internal_docs/distribution_pipeline.md:221` and `plans/issues/active/phase-40-stable-channel-ga-execution.md:226`; the execution issue now carries pass-1/pass-2 records and an evidence list.

---

### P1 — Blocking

**1. Non-UTF-8 transported evidence produces a raw traceback instead of a governed diagnostic.**
`planner.py:349` (`installer_path.read_text(encoding="utf-8")`) and `planner.py:373` (`checksums_path.read_text(...)`) are unguarded; `release_governance.py:164` catches only `GovernanceError`. Verified against a coherently resealed bundle:

```
installer is non-UTF8 binary  -> exit=1 TRACEBACK
checksums non-UTF8            -> exit=1 TRACEBACK
```

Exit 1 with a Python stack trace, not exit 2 with a governed message. This is the exact defect class pass-1 finding 3 removed for `KeyError`, and `expect_planner_rejected` already asserts `"Traceback" not in stderr` — there is simply no case that feeds binary evidence. Wrap both reads (or add a governed `read_text_strict` helper alongside `load_json_strict`) and add the two negative cases to `test_planner_rejects_drift_cases`.

**2. The workflow-contract check validates the wrong workflow file.**
`release-qualification.yml:65-67` runs `release_qualification_workflow_contract.sh`, which reads `${REPO_ROOT}/.github/workflows/release-qualification.yml` (`cases/release_qualification_workflow_contract.sh:7`, `cases/common.sh:5`) — i.e. the file from the **checked-out `inputs.source_commit`** (`release-qualification.yml:47-51`). But `workflow_dispatch` executes the YAML resolved from the *dispatch ref*, which is independent of `source_commit`. A run dispatched from a branch whose definition has `contents: write`, an extra upload, a mutation step, or a different matrix will still pass the contract check as long as `source_commit` points at a clean commit. Nothing in `qualification-artifact-index.json` records which workflow definition governed the run, so a later reviewer cannot detect it either.

The evidence needed is already fetched and discarded: `release-qualification.yml:322-324` writes `run-metadata.json` (which carries `head_sha` and `path`), and `collect_qualification_artifacts.py:89-102` checks `id`, `run_attempt`, `event`, `name`, and `repository.full_name` — but not `head_sha`. Either require `run_metadata["head_sha"] == source_commit` in the collector (forcing dispatch from a ref at the exact candidate commit), or compare the checked-out workflow bytes against `gh api repos/.../contents/.github/workflows/release-qualification.yml?ref=${GITHUB_SHA}`; both are reachable under `contents: read`.

---

### P2 — Should fix before merge

**3. The editor and documentation qualification reports have no exact-shape or schema-epoch contract.**
`planner.py:325-345` consumes the editor report via `require_object` + `.get()`; `planner.py:151-160` does the same for the documentation report. Neither validates `schema_version`, `kind`, or the key set — unlike `validate_target_report` (`planner.py:379-397`), which pins all three for the per-target reports produced by the same workflow. Verified with coherently resealed bundles (all digests internally consistent):

```
editor schema_version=1        -> exit=0 accepted
editor schema_version removed  -> exit=0 accepted
editor kind forged             -> exit=0 accepted
editor extra unknown field     -> exit=0 accepted
doc report extra field         -> exit=0 accepted
```

The workflow emits `schema_version: 2` / `kind: "stable-editor-qualification"` (`release-qualification.yml:183-184`) and the phase's single-epoch rule requires every consumer to reject a missing or non-`2` value before using any other field. Every semantically-used field *is* cross-checked, so this is not an exploitable escape today — but it leaves two of the milestone's governed evidence artifacts as the only unpinned shapes in the plan. Give the editor report a `validate_editor_report` with the same exact-key/epoch/kind treatment as `validate_target_report`, and pin the documentation report's key set.

---

### P3 — Test coverage

**4. `validate_source_identity`'s mismatched-ref branch is untested.** `planner.py:196-199` rejects `resolved_commit != HEAD`, and the DoD negative list names "mismatched refs" — but no case reaches it. `test_planner_evidence_contract:299` only exercises the *floating*-ref regex via `resolve_source_once(REPO_ROOT, "main")`, and my probe with a malformed SHA also stops at resolution. Add a second fixture commit and pass the parent SHA as `--source-ref`.

**5. `source`/`submodule`/`lock` digest variants remain confounded.** `qualification_selftest.py:661-676` builds a fresh git repository per variant, so the commit SHA differs regardless of the input under test. For `lock` the confound is inherent (the lockfile is committed), and the same-source control now anchors the other six, so this is minor — but pinning `GIT_AUTHOR_DATE`/`GIT_COMMITTER_DATE` in `configure_git` would let a baseline-vs-baseline source rebuild serve as a control for these three too.

---

### Validation performed (read-only; no repository files modified)

| Check | Result |
|---|---|
| `areas run --area distribution_release --suite qualification` | pass (8 self-tests, 1 variant, 21.8s) |
| `areas run --area distribution_release --suite full` | pass (43 variants, 0 failures) |
| `areas run --area distribution_release --suite evidence-custody` | pass |
| `cases/release_qualification_workflow_contract.sh` | PASS |
| `cases/artifact_stable_candidate_generation.sh` | PASS |
| `cargo test -p sifr --bin sifr self_update_receipt::` | 16 passed |
| Same-source rebuild → identical plan bytes (pass-2 #1 control) | confirmed |
| Symlinked container / symlinked file / symlinked artifact-root (pass-2 #2) | rejected, rejected, correctly accepted |
| 18 artifact-index custody mutations (pass-2 #3 + extensions) | all 18 rejected |
| Valid bundle + in-repo `--out` (pass-2 #4) | exit 2, governed, not written |
| 12 collector metadata/retention forgeries (pass-2 #5) | all rejected |
| 20 coherently-resealed planner forgeries (installer identity, editor semantics, checksums binding, target-report semantics) | 19 rejected, 1 accepted → finding 3 |
| 5 editor/doc report shape probes | 4 accepted → finding 3 |
| 15 malformed-input / raw-exception probes | 13 governed, 2 tracebacks → finding 1 |
| Collector output vs checked-in `qualification_artifact_index.schema.json` | conforms (schema/validator parity) |
| `rust_interop` manifest suites vs create-pr/merge/nightly/release | `stable-candidate` registered and selected in all four |
| Real `stable_support_claims.json` → `stable_claim_ids` | accepted, 23 ordered ids |
| `check_file_size_guardrails.py` / `check_hir_maintainability_guardrails.py` | PASS / PASS |

Not run: `scripts/run_all_tests.sh --profile create-pr`/`merge`, the GitHub-hosted matrix jobs, and `demos/stable_candidate_qualification_demo.sh` (it requires a clean checkout and the worktree carries an untracked pass-3 review file; verified executable and `bash -n` clean, and its planner path is the same one exercised end-to-end by `test_materialized_planner_contract`).

---

**CHANGES_REQUESTED**

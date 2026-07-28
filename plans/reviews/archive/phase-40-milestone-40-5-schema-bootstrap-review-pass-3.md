# Review: Phase 40 M40.5 schema-v2 preview epoch bootstrap (pass 3)

I read the full diff, all 15 untracked files, the phase/issue docs, and the archived pass-1 and pass-2 reports, and independently re-ran the gates plus a schema/validator parity probe.

## Pass-2 re-audit

| # | Pass-2 finding | Status |
|---|---|---|
| 1 | Smoke-output filename contract ungated, drift lands after irreversible mutation | **Resolved.** `schema_epoch_bootstrap_workflow_contract.sh:117-120` now freezes all four `${out}/<id>.txt` literals, and `:57-58` proves `--out publication/bootstrap-smoke` equals `--smoke-dir publication/bootstrap-smoke`. I confirmed all four literals exist at `run_schema_bootstrap_public_smoke.sh:57,67,77,98` and that the ids match `SMOKE_IDS` (`schema_bootstrap.py:33-38`) exactly. |
| 2 | Staged-alpha binding branch unreached by the self-test | **Resolved.** `schema_bootstrap_selftest.py:331-337` passes the final `preview-index` evidence as `--alpha-evidence` (exercises the `stage != "alpha-assets"` half of `schema_bootstrap.py:329-336`), and `:338-370` materializes a genuinely different alpha (`0.1.0-alpha.3`, own record + assets + run 44) to exercise the exact alpha-block equality half. Both raise before the earlier asset checks. |
| 3 | Opaque pre-epoch identity duplicated across four surfaces with no cross-surface gate | **Resolved.** `schema_epoch_bootstrap_workflow_contract.sh:127-136` asserts the literal digest in all four surfaces and the size in each surface's own idiom (`= "105"`, `LEGACY_INDEX_SIZE_BYTES = 105`, `"size_bytes": {"const": 105}`). |
| 4 | Empty pass-2 review artifact | **Resolved.** Populated, archived, and ledgered at `plans/issues/active/phase-40-stable-channel-ga-execution.md:370-376`. |

All eleven pass-1 remediations remain in place. Parity probe (18 instances against both the JSON Schema and `validate_bootstrap_evidence`): every structural case agrees in both directions, including the `alpha-assets` positive and all four stray-key rejections; the only divergences are the three cross-field semantics 2020-12 cannot express (asset map vs sibling `version`, approver vs sibling `initiator`, case-folded approver uniqueness), all validator-stricter and covered semantically. No schema-v1 parser, fixture, migration, or fallback exists.

Gates re-run here: `governance.schema_bootstrap_selftest` PASS; epoch-bootstrap suite PASS; full distribution area PASS (**variants=111, failures=0**); `schema_epoch_bootstrap_workflow_contract.sh`, `preview_release_workflow_yaml_parses.sh`, `site_release_workflow_contract.sh` PASS; file-size self-test + repository gate PASS (2898 files); `git diff --check` PASS; `bash -n` over all `scripts/distribution/*.sh` PASS; `ruff check` clean on every changed Python file (the 8 remaining diagnostics are all in untouched files). `generate_version_installer.sh` output depends only on version + archive digests, so the new prepare→publish installer-digest equality in `verify_release_publication_assets.sh:94-99` is sound.

## Actionable findings

### 1. The pre-dispatch site-workflow re-verification is no longer gated — Medium-Low

Before the pass-1 extraction, `site_release_workflow_contract.sh` asserted two *distinct* diagnostics — `"site workflow tag immutability ruleset is not active and exact"` (input validation) and `"site workflow tag lost immutable protection before dispatch"` (pre-dispatch) — which proved two independent call sites. Both were replaced by one shared script emitting one message set, and the contract now only asserts membership plus one ordering relation:

- `site_release_workflow_contract.sh:77` — `"scripts/distribution/verify_site_workflow_identity.sh"` appears somewhere
- `:96` — its first index precedes `"Publish write-once version release and verify assets"`

`release-publication.yml` invokes it twice (`:212-220` and `:678-686`), but nothing asserts `count == 2`, and nothing asserts an invocation occurs at or after `"Dispatch exact site workflow"`. Deleting `:678-686` keeps `site_release_workflow_contract.sh`, `preview_release_workflow_yaml_parses.sh`, and `schema_epoch_bootstrap_workflow_contract.sh` all green — and reopens exactly the TOCTOU window that second check exists to close: the site workflow tag could be moved or its ruleset weakened between input validation and dispatch, and the protected job would dispatch to unreviewed bytes. This is the same class as pass-2 finding 1 (a gate that a refactor silently stopped covering). Assert `publication.count("verify_site_workflow_identity.sh") == 2` and that the second index falls between `"Dispatch exact site workflow"` and `"Poll exact site run"`.

### 2. `governance.schema_bootstrap_selftest` executes twice per default full-area run — Low

`runner.py:126-133` runs the module for the `epoch-bootstrap` suite, and `:143-149` appends it unconditionally to the `full` suite. Because `select_suites` selects every manifest suite when unfiltered, both fire. Measured: `case=schema-v2-preview-epoch-bootstrap` appears **2** times, while `case=incident-recovery` appears **1** — the adjacent `include_incident=not incident_suite_selected` dedup (`runner.py:47-50,105-108,151`) exists for precisely this reason and was not mirrored. The duplicate is also double-counted in the reported `total_variants` (111 includes it twice), which is the number used as merge evidence. Mirror the incident pattern with an `include_epoch_bootstrap` flag.

### 3. `bootstrap_evidence_bytes` is dead code — Low

`schema_bootstrap.py:246-247` defines it; a repo-wide grep finds no caller in any script, workflow, test, or module. It is also the only consumer of the `canonical_json_bytes` import at `:10`. AGENTS.md's "no fallback paths or solutions unless explicitly requested" and the pass-1 finding-10 posture argue for deleting both.

### 4. The public smoke sets a variable nothing reads — Low

`run_schema_bootstrap_public_smoke.sh:80,94` set `SIFR_SYSROOT_DIR`. No code in the repo reads that name: the generated installer reads `SIFR_SYSROOT_INSTALL_DIR` (`generate_version_installer.sh`, `sysroot_dir="${SIFR_SYSROOT_INSTALL_DIR:-${default_sysroot_dir}}"`), and the compiler reads `SIFR_SYSROOT` (`crates/sifr_sysroot/src/resolve.rs:6`). The intended temp-root isolation holds today only by accident — `install_dir` ends in `/bin`, so `default_sysroot_dir` resolves to the same path. If that default ever changes, the smoke silently writes its toolchain outside the sandbox on the protected runner, and the intent expressed in the code would not be what is enforced. Use `SIFR_SYSROOT_INSTALL_DIR`.

### 5. The `alpha-assets` stage is never validated against the new JSON Schema — Low

`schema_contracts.py:30-33` enforces exactly one fixture per schema file, and `schema_bootstrap_evidence()` returns a `preview-index` instance. So the newly added `if/then/else` else-branch (`schema_epoch_bootstrap_evidence.schema.json:114-127`) — the branch governing the durable alpha-stage evidence that `bootstrap-index` consumes — has no positive coverage: a schema bug there would reject valid alpha evidence with no gate noticing (the runtime path uses only the semantic validator, so it would not break, but the schema/validator parity contract this area maintains would be silently false). I confirmed manually that the branch is correct today. Register the alpha-stage instance as a second checked instance, or add it to the negative/positive probes alongside the two existing bootstrap negatives at `:66-81`.

### 6. Duplicate import statement — Nit

`schema_bootstrap.py:8-25` and `:26` are two separate `from .common import` statements; fold `load_json_strict` into the first.

## Not findings

- Post-`gh release create` failures remain unrecoverable by re-run in every mode; for `bootstrap-index` the durable evidence upload is necessarily last (it must bind the smoke digests), so a `sifr.sh` convergence flake leaves generation 1 live and un-evidenced. That is the correct fail-loud choice over recording an unverified pass, and it is the pre-existing write-once contract.
- The smoke writes a fifth file, `${out}/stable-dispatcher.sh`, into the evidence directory; the materializer hashes only the four `SMOKE_IDS` files, so this is harmless (unlike the asset directories, an exact-set check would be wrong here).
- `poll_site_release_run.sh` does not guard `GH_TOKEN` non-emptiness the way `verify_site_workflow_identity.sh:30` does, but an empty token yields three governed query failures and a governed exit 2.
- Prepare checks artifact presence per target while publish enforces exactness; publish's `.assets` equality against the prepare summary now makes any prepare-side extra file fail closed before mutation.
- Nested reusable-workflow depth (preview-release → release-publication → prepare) is 3, within GitHub's limit of 4; `prepare` declares no `concurrency`, so it cannot deadlock against `group: sifr-release-index`.
- The `.github/workflows` guardrail extension does not cover `.github/actions/**/action.yml`; out of scope for this slice.

## Commit mechanics and execution requirements (not implementation defects)

- `plans/reviews/active/phase-40-milestone-40-5-schema-bootstrap-review-pass-3.md` is a 0-byte untracked file. It must receive this report, be archived, and be ledgered in `plans/issues/active/phase-40-stable-channel-ga-execution.md` before commit. I did not modify it, per instruction.
- Pass-1's external requirements still stand: `stable-release` environment with ≥1 `release/distribution` reviewer and GitHub "prevent self-review" enabled; confirmation that no reviewers are attached to the auto-created `preview-release` environment; the live `channels.json` still being exactly 105 bytes / `71b3243925…4bf9ef` at `bootstrap-index` time; and `sifr.sh` serving new dispatcher bytes inside the 180-second budget.
- One behavior I cannot reproduce locally: whether `actions/download-artifact@v4` resolves an attempt-1 artifact during "Re-run failed jobs". Pass 2 adjudicated this resolved and the naming is correct on both re-run paths; validate it on the first live re-run rather than treating it as settled.

VERDICT: NOT SATISFIED

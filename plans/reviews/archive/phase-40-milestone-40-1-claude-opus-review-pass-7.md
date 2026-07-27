# Independent Review — Sifr Phase 40 / milestone_40_1, pass 7

Branch `codex/phase-40-milestone-40-1` (9 commits) vs `origin/main`; 45 files, +5369/−160. Read-only: no repository files were modified (the untracked, zero-byte `plans/reviews/active/phase-40-milestone-40-1-claude-opus-review-pass-7.md` was already present and was left untouched).

New since pass 6: `7848d613c docs(release): record installer regeneration binding` — 4 files, +172/−1 (`internal_docs/distribution_pipeline.md`, the issue tracker, the pass-6 artifact, and `cases/release_qualification_workflow_contract.sh`).

## Focus item 1 — documentation of the regeneration binding: **accurate and complete**

`internal_docs/distribution_pipeline.md:186-193` now reads:

> The planner also regenerates the immutable installer with `scripts/distribution/generate_version_installer.sh` from the pinned `source_commit` and the transported per-target archives and checksums, then requires byte-for-byte equality with the transported installer. The installer digest is therefore bound to the governed producer rather than to textual self-attestation inside the shell script.

Checked clause by clause against `verification/areas/distribution_release/governance/planner.py:348-397`:

| Doc claim | Code | Verdict |
|---|---|---|
| regenerates with `generate_version_installer.sh` | `subprocess.run([str(generator), ...], cwd=source_root)` at `:373-386` | accurate |
| "from the pinned `source_commit`" | `generator = source_root/scripts/distribution/generate_version_installer.sh`; `validate_source_identity` (`:203-231`, invoked at `:63`) requires HEAD == plan `source_commit` == `--source-ref` with a clean tree, before `bind_aggregate_artifacts` at `:122` | accurate |
| "from the transported per-target archives and checksums" | copies `binary-archive-<target>` and `checksum-<target>` for all four `TARGETS` into a temp `artifacts/` dir (`:361-372`) | accurate |
| "requires byte-for-byte equality" | `if regenerated.read_bytes() != transported_installer.read_bytes(): fail(...)` at `:392-396` | accurate |
| "bound to the governed producer rather than … textual self-attestation" | `validate_installer_identity`'s line-anchored shell parsing is gone; `:355-357` also rejects a missing or symlinked generator | accurate |

No overstatement: the doc does not claim reproducibility guarantees the generator does not provide, and the surrounding section (`:171-186`) still correctly describes index custody. The pass-6 P3 finding is closed.

## Focus item 2 — the new workflow-contract pin: **does not pin what it claims (finding below)**

`cases/release_qualification_workflow_contract.sh:78-85` adds a substring pin for the assemble-job invocation, and the case **PASSES** against the current workflow. But the pinned literal is terminated one character early, so the pin is unanchored at exactly the argument boundary it exists to protect. Details in the finding.

No contract was weakened: every pre-existing `required` fragment, the `forbidden` mutation-capability list, the `overwrite: false`/`retention-days: 30` counts (4 each), the exact per-target download names (×1), the Ruby permissions/topology/matrix/`environment` assertions, and the `cargo build --locked --release -p sifr` pin are all unchanged. The addition is purely additive.

## Re-run validation (this pass)

| Check | Result |
|---|---|
| `areas run --area distribution_release --suite qualification` | **pass** — 8 self-tests, 1 variant, 47.4 s |
| `areas run --area distribution_release --suite full` | **pass** — 43 variants, 0 failures, 0 blocking |
| `cases/release_qualification_workflow_contract.sh` | **PASS** |
| `scripts/check_file_size_guardrails.py` | **PASS** (2857 files, limit 900) |
| capability naming: `grep -rniE 'phase[_ -]?40|milestone[_ -]?40'` over `internal_docs/distribution_pipeline.md`, `verification/areas/distribution_release/`, `scripts/distribution/`, the demo, and the qualification workflow | **no matches** |

Not run (unchanged from pass 6, and out of reach locally): `scripts/run_all_tests.sh --profile create-pr`/merge, and the GitHub-hosted matrix/collector jobs, which need GitHub runners and the Actions artifact API.

I also independently re-read `scripts/distribution/collect_qualification_artifacts.py` end to end (run-identity binding to `workflow_dispatch`/`sifr-lang/sifr`/`head_sha`, exact artifact-name set equality, `expired is not False`, tz-aware timestamps with exactly 30-day retention, per-container symlink/nesting/empty-file rejection, `min(expiries)` as the index expiry) and found no defect. Pass-6's confirmed closures (non-UTF-8 `sysroot.toml` governed on both live paths, byte-equality installer binding, two-claim fixture plus order-reversal negative, evidence-custody mutation coverage) are all still in place and covered by the passing `full` suite.

## Remaining finding

### P2 — testing/contract. The new installer-invocation pin is terminated one character early, so it does not detect the argument drift it was added to prevent.

File: `verification/areas/distribution_release/cases/release_qualification_workflow_contract.sh:78-85`

```python
installer_invocation = """scripts/distribution/generate_version_installer.sh \\
            --version "${VERSION}" \\
            --artifact-dir target-artifacts \\
            --out "qualification-assemble/sifr-installer-${VERSION}"""
```

The literal is intended to end with `...${VERSION}"`, but Python's lexer closes the triple-quoted string at the first `"""` it reaches, so the trailing `"` of the `--out` value is consumed by the delimiter. The pinned text therefore ends mid-token at `sifr-installer-${VERSION}` — with no closing quote and no trailing newline — leaving the command's tail unpinned.

Failure scenario (verified against the real workflow text, `.github/workflows/release-qualification.yml:272-275`):

```
pin tail repr:  '     --out "qualification-assemble/sifr-installer-${VERSION}'
baseline                                   pin-match=True   (case PASSES — correct)
append --artifact-base-url after --out     pin-match=True   (case still PASSES — WRONG)
--out "...-${VERSION}-tampered"            pin-match=True   (case still PASSES — WRONG)
duplicate invocation with --artifact-base-url  pin-match=True (case still PASSES — WRONG)
```

The first drift variant is precisely the coupling this check was added for: `generate_version_installer.sh:79-81` derives `ARTIFACT_BASE_URL` from `VERSION` only when `--artifact-base-url` is absent, and it is embedded verbatim in the generated installer (`:136`). Planner regeneration (`planner.py:373-386`) never passes `--artifact-base-url`, so if the assemble job gained one, the byte-equality check would break. The workflow contract case — the artifact whose commit message states it "pins the exact production invocation that must remain identical to planner regeneration" — would keep reporting PASS.

Impact is fail-closed, not an escape: the drifted installer would be rejected later at plan time as `$.installer_sha256: transported installer bytes do not match the governed generator`. So this is not a security hole. It is actionable because the check as written does not enforce its stated contract, and the tracking record in `plans/issues/active/phase-40-stable-channel-ga-execution.md:254-256` asserts that it does.

Fix (one-character semantic change; verified):

```python
installer_invocation = (
    'scripts/distribution/generate_version_installer.sh \\\n'
    '            --version "${VERSION}" \\\n'
    '            --artifact-dir target-artifacts \\\n'
    '            --out "qualification-assemble/sifr-installer-${VERSION}"\n'
)
```

I confirmed this form matches the current workflow (`True`) and rejects the `--artifact-base-url` drift variant (`True`). The trailing `\n` also anchors the command's end, so no fifth argument can be appended on a further continuation line. Either the escaped-quote heredoc form (`...${VERSION}\""""`) or the concatenated form works; the concatenated form is clearer about the intent. The issue-tracker sentence at `:254-256` should stay as written once the pin actually holds.

### Non-blocking observations (no change requested)

- Pass 6's second non-blocking observation still stands: nothing pins `parse_channel("stable")` erroring. It remains redundancy — `PreviewVersion::parse("0.1.0")` fails first and *is* pinned by `rejects_stable_and_rc_versions`.
- `collect_qualification_artifacts.py:283-291` accepts whatever single `.vsix` filename is transported (the expected-set comparison is self-referential for that one name). It is not a gap: `planner.py:330-345` binds `vsix_sha256`, `package_path`, `package_version`, and `compiler_compatibility` to the editor qualification report, so the name cannot float free of governed evidence.

Everything pass 7 was asked to verify on the documentation side is confirmed accurate; the milestone's correctness and security posture is unchanged and sound. The one item above is a real defect in the check added by `7848d613c`.

**CHANGES_REQUESTED**

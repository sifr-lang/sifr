# Independent Review — Sifr Phase 40 / milestone_40_1, pass 8

Branch `codex/phase-40-milestone-40-1` (10 commits) vs `origin/main`; 46 files, +5472/−160. Read-only: no repository files were modified (the untracked, zero-byte `plans/reviews/active/phase-40-milestone-40-1-claude-opus-review-pass-8.md` was already present and was left untouched — confirmed 0 bytes and still the only entry in `git status --porcelain`).

New since pass 7: `6b261cf3b test(release): anchor installer invocation contract` — 3 files, +107/−4 (the issue tracker, the pass-7 artifact, and `cases/release_qualification_workflow_contract.sh`).

---

## Focus item 1 — the pass-7 finding: **fixed exactly as specified**

`cases/release_qualification_workflow_contract.sh:78-85` now reads:

```python
installer_invocation = (
    'scripts/distribution/generate_version_installer.sh \\\n'
    '            --version "${VERSION}" \\\n'
    '            --artifact-dir target-artifacts \\\n'
    '            --out "qualification-assemble/sifr-installer-${VERSION}"\n'
)
```

The triple-quoted literal that Python's lexer truncated at the `--out` closing quote is gone; the concatenated form terminates with `"` **and** `\n`, so both the argument value and the end of the command are anchored.

I re-ran the probe matrix from pass 7 directly against `.github/workflows/release-qualification.yml:272-275` (first asserting the four-line block is present verbatim, then mutating it):

| Workflow variant | Pin present | Case outcome | Correct? |
|---|---|---|---|
| baseline (current workflow) | `True` | PASS | ✅ |
| `--artifact-base-url` appended on a 5th continuation line | `False` | FAIL | ✅ |
| `--artifact-base-url` appended on the same line as `--out` | `False` | FAIL | ✅ |
| `--out "…-${VERSION}-tampered"` | `False` | FAIL | ✅ |
| `--out "other-dir/sifr-installer-${VERSION}"` | `False` | FAIL | ✅ |
| `--artifact-dir other` | `False` | FAIL | ✅ |
| `--version "${SOURCE_COMMIT}"` | `False` | FAIL | ✅ |
| argument order permuted (`--artifact-dir` before `--version`) | `False` | FAIL | ✅ |
| **second invocation appended, original retained** | `True` | PASS | ⚠️ see observation |

`verification/areas/distribution_release/cases/release_qualification_workflow_contract.sh` → **PASS** (exit 0) against the real workflow. The trailing-newline anchor is load-bearing: it is what turns the two `--artifact-base-url` append variants from `True` (pass-7 behavior) into `False`.

**No contract was weakened.** Every pre-existing `required` fragment, the six-entry `forbidden` mutation-capability list, `overwrite: false` ×4 / `retention-days: 30` ×4, the four exact per-target download names (`count == 1` each), the Ruby permissions (`contents:read`/`actions:read` only), job topology, target/runner matrix, `environment`-absence assertions, and the `cargo build --locked --release -p sifr` builder pin are all byte-identical to pass 7. The change is purely a strengthening of one literal.

## Focus item 2 — pass-6 documentation closure: **still valid**

`internal_docs/distribution_pipeline.md:186-193` still carries the regeneration paragraph verbatim, and I re-verified it clause-by-clause against `planner.py:348-397` rather than trusting pass 7:

- `validate_installer_bytes` (`planner.py:348-397`) rejects a missing or symlinked generator (`:356-358`), copies `binary-archive-<target>` + `checksum-<target>` for all four `TARGETS` into a temp `artifacts/` dir (`:361-372`), runs the generator with `cwd=source_root` (`:373-386`), and requires `regenerated.read_bytes() != transported_installer.read_bytes()` → `fail` (`:392-396`).
- I additionally checked a property the doc's byte-equality claim depends on and that no prior pass stated explicitly: **the generator's output does not depend on `--out`.** `grep -n 'OUT' generate_version_installer.sh` shows `OUT` used only at `:126` (`mkdir -p "$(dirname …)"`), `:127` (redirect target), `:801` (`chmod`), and `:802` (echo). It never enters the heredoc body, so the planner regenerating into `<tmp>/sifr-installer-<version>` can be byte-equal to the workflow's `qualification-assemble/sifr-installer-<version>`. The binding is achievable in production, not vacuously fail-closed.

## Earlier correctness closures — re-verified

| Closure | Origin | Status this pass |
|---|---|---|
| Non-UTF-8 `sysroot.toml` governed, no traceback | pass 5 #1 | `verify_release_archive.py:151-157` guard intact; `grep -rn 'decode('` over `scripts/distribution/**` and `governance/**` returns **exactly that one site**, and it is wrapped |
| Installer identity bound to governed producer, not shell parsing | pass 5 #2 | `validate_installer_identity` absent; `validate_installer_bytes` in place; the five `binary-installer`/`installer-*` negatives pass in the `qualification` suite |
| Collector resolved-path/symlink custody | pass 4 #3 | `test_artifact_collector`, `test_artifact_collector_rejects_drift`, `test_artifact_index_exact_custody` pass |
| Two-claim fixture + Rust-claim order reversal | pass 5 #3 | `test_planner_rejects_drift_cases`, `test_plan_digest_sensitivity` pass |
| Evidence-custody / index / report mutation coverage | passes 2–4 | `test_evidence_custody_mutations`, `test_artifact_index_mutations`, `test_release_report_mutations`, `test_surface_contract_mutations`, `test_strict_loader_rejects_duplicate_keys` pass |
| Stable self-update remains gated (Phase-40 scope) | pass 6 | `rejects_stable_and_rc_versions` and `rejects_stable_metadata` both pass alongside the new `accepts_stable_receipt_for_read_only_version_evidence`; `stable_gate_inventory.json:86-92` records the split accurately |

The demo (`demos/stable_candidate_qualification_demo.sh`) was not re-run this pass, and does not need to be: `git diff --stat 96019549f..HEAD` shows the only changes since the demo-validated commit are `internal_docs/distribution_pipeline.md`, the issue tracker, two review artifacts, and the contract case. No production script, workflow, planner, collector, or Rust source changed.

## Validation performed (read-only)

| Check | Result |
|---|---|
| `areas run --area distribution_release --suite full` | **pass** — 43 variants, 0 failures, 0 blocking; 14 governance self-tests |
| `areas run --area distribution_release --suite qualification` | **pass** — 8 self-tests, 1 variant, 42.1 s |
| `cases/release_qualification_workflow_contract.sh` | **PASS** |
| `cargo test -p sifr --bins -- self_update` | **pass** — 42 tests |
| `scripts/check_file_size_guardrails.py` | **PASS** (2857 files, limit 900) |
| `scripts/check_hir_maintainability_guardrails.py` | **PASS** |
| Capability naming: `grep -rniE 'phase[_ -]?40\|milestone[_ -]?40'` over `internal_docs/distribution_pipeline.md`, `verification/areas/distribution_release/`, `scripts/distribution/`, the demo, the qualification workflow, and `self_update_receipt.rs` | **no matches** |
| Working tree clean apart from the pre-existing zero-byte pass-8 stub | confirmed |

Not run (unchanged from passes 6–7, out of local reach): `scripts/run_all_tests.sh --profile create-pr`/merge, and the GitHub-hosted matrix/collector jobs, which need GitHub runners and the Actions artifact API.

## Tracking accuracy

`plans/issues/active/phase-40-stable-channel-ga-execution.md:259-265` records pass 7 and its remediation precisely, and — importantly — does not overclaim. It states the pin now anchors "the full four-line invocation, including its closing quote and newline, so appended arguments or an altered output path cannot satisfy the contract." Both of those specific claims are verified true above. The pass-6 sentence at `:254-256` ("pins the exact production invocation that must remain identical to planner regeneration") is now backed by a pin that actually holds for every single-invocation drift form I could construct. The evidence list at `:266-274` remains accurate.

## Findings

**None blocking.** No actionable correctness, security, test, documentation, scope, tracking, or capability-naming finding remains.

### Non-blocking observations (no change requested)

- **The pin is a substring test, so it does not assert uniqueness.** A workflow that keeps the governed invocation *and* appends a second `generate_version_installer.sh` call (e.g. one passing `--artifact-base-url`, overwriting the same `--out`) still satisfies the pin. This is a narrower residue than the pass-7 defect: it is not argument *drift* — the realistic edit, replacing or extending the existing call, is now rejected — and the outcome is a clean fail-closed rejection at plan time (`$.installer_sha256: transported installer bytes do not match the governed generator`), never a bad installer shipping. `generate_version_installer.sh` currently appears exactly once in the workflow, so if the maintainers want belt-and-braces the file's own existing idiom applies: `if text.count("scripts/distribution/generate_version_installer.sh") != 1` (the same `count(...) != 1` form already used for the four per-target download names). I do not consider this worth another round; the tracking record does not claim it.
- Pass 6/7's carry-over: nothing pins `parse_channel("stable")` erroring. Redundant — `PreviewVersion::parse("0.1.0")` fails first and *is* pinned by `rejects_stable_and_rc_versions`.
- `collect_qualification_artifacts.py:283-291` accepts whatever single `.vsix` filename is transported. Not a gap: `planner.py:330-345` binds `vsix_sha256`, `package_path`, `package_version`, and `compiler_compatibility` to the editor qualification report.

The pass-7 defect is closed by the exact remediation it specified, verified independently against the real workflow with eight mutation variants. All prior closures hold, every suite and guardrail passes, the architecture documentation is accurate, and the tracking record matches reality.

**APPROVED**

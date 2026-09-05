## Milestone review — Phase 40 / `milestone_40_4` (pass 4)

Read-only. Range `origin/main...HEAD` = 15 commits, 63 files, +3376/−443. Working tree and all recursive submodules clean before and after every check. Pointer `editor_integrations = d7577d492`, `editor_integrations/vscode = 273fd5d3e`, matching `vscode_extension_rules.json:22-23`. Zero files under `crates/**` or `verification/areas/rust_interop/**`.

### Checks re-run

| Check | Result |
|---|---|
| `documentation --suite structure --suite ga-release` | PASS, variants=2, failures=0 (`claims=25`) |
| `developer_tooling --suite editor-release` | PASS, variants=6, failures=0, blocking=0 |
| `distribution_release --suite qualification --suite full` | PASS, variants=55, failures=0 |
| `check_ga_release_docs.py --self-test` | PASS |
| `release_qualification_workflow_contract.sh` | PASS |
| `stable_editor_qualification_contract.sh` | PASS (editor + docs self-tests + renderer `--check`) |
| `sifr_verify --self-test` | PASS (incl. documentation/release-report self-tests) |
| `check_file_size_guardrails.py` | PASS (2880 files, limit 900) |

### Pass-3 findings and the six named items — verified closed

1. **Rollback workflow binding** — closed. `ROLLBACK_VERSION: ${{ needs.validate.outputs.rollback_version }}` now sits in the *editor* step's `env:` (`release-qualification.yml:178`) and is gone from the `build` step; `release_qualification_workflow_contract.sh:42-48` parses the YAML, finds the exact editor step by name, and asserts the binding structurally rather than by grep.
2. **Actual packaged-candidate test discovery** — genuinely non-vacuous. `run_tests` returns `Ok(true)` when nothing is discovered (`crates/sifr_driver/src/test_runner/orchestrator.rs:31-33`), so I executed the demo's exact fixture shape: `test_editor_candidate.sifr` matches `is_test_module_name` (`discovery.rs:371-372`) and produced `Found 1 test file(s) … 1 passed`. The demo's `sifr test "${test_dir}"` (`demos/editor_candidate_qualification_demo.sh:97`) therefore proves the advertised test action.
3. **Exact four-target allowlisting** — closed by a positive allowlist, not a denylist: `check_ga_release_docs.py:282-286` diffs every triple found in the all-docs sweep against `TARGETS`, so `aarch64-unknown-linux-musl` and friends now fail.
4. **Truthful VSIX package evidence naming** — closed. Field renamed `vsix_install_smoke` → `vsix_package_smoke` consistently across `qualify_stable_editor.py:333`, `editor_qualification.py:44,60`, `qualification_fixture.py:283`, the contract case, and the demo. Marketplace evidence remains `status: "planned"` with `execution_owner: "stable-publication-workflow"`.
5. **File-size headroom** — closed. Largest touched files: `sifr_verify/selftest.py` 860, `governance/selftest.py` 854 (−30 via the new `stable_gate_inventory_selftest.py`), `qualification_fixture.py` 828. ≥40 lines headroom everywhere.
6. Pass-3 observations also closed: operator command for `qualify_stable_documentation.py` documented (`internal_docs/distribution_pipeline.md:205-215`) and the planner now validates its `suites`/`result_sha256` (`planner.py:480-497`); planner cross-binds editor evidence to the qualified target report and binary digest (`planner.py:333-364`); `documentation ga-release` is in `verification/profiles/release.json` and enforced by the runner self-test.

Ledger, phase DoD, and the ad hoc record are mutually consistent; all `milestone_40_4` checkboxes correctly remain unchecked, so candidate-evidence materialization is properly left to post-merge.

### Findings

**1. MEDIUM — `docs/cli/lsp.mdx:84` (and `:175`) documents a VS Code setting the qualified extension does not contribute.**
The GA editor-setup instruction is `"sifr.serverPath": "/path/to/sifr"`. The recorded `sifr.sifr-vscode 0.2.0` package contributes `sifr.lsp.path`, `sifr.lsp.trace.server`, `sifr.diagnostics.mode`, `sifr.format.enable`, `sifr.lint.enable` — no `serverPath`; `config.ts:36` reads `config.get("lsp.path")` and `lsp.ts:57` tells users to "Configure sifr.lsp.path". The same diff's `docs/troubleshooting.mdx:56` and `docs/support.mdx:30` correctly say `sifr.lsp.path`, so GA docs contradict each other. Failure scenario: a `0.1.0` user whose `sifr` is outside `PATH` follows `lsp.mdx`, sets `sifr.serverPath`, VS Code silently ignores the unknown key, and the server still fails to launch. This violates the DoD "docs … and extension metadata agree", and no check covers it — `lsp.mdx` is outside `CANONICAL_DOCUMENTS` and no required-fact or forbidden-pattern entry names configuration keys. *Remediation:* replace `serverPath` with `sifr.lsp.path` at `:84` and the `serverPath` reference at `:175`, and add the contributed configuration keys to the `REQUIRED_BY_DOCUMENT`/sweep surface (or a forbidden pattern for non-contributed `sifr.*` settings) so key drift fails mechanically.

**2. MEDIUM — `docs/cli/lsp.mdx:68-73` directs GA users to a VSIX the governed release cannot supply.**
The pass-3 reword now reads "Install the qualified `sifr-vscode-0.2.0.vsix` supplied with the governed `0.1.0` release" plus `code --install-extension sifr-vscode-0.2.0.vsix`. Nothing supplies that file to users: `.github/workflows/release-publication.yml:196-219` computes an exact expected version-asset set of four `sifr-<version>-<target>.tar.gz`, four `.sha256`, and `sifr-installer-<version>`, and *fails on any unexpected file*; the phase's write-once asset list is likewise archive/checksum/installer/plan only, and the VSIX exists solely as a 30-day `sifr-stable-candidate-…-editor` qualification artifact plus the `milestone_40_5` Marketplace publication. No page in `docs/**` gives a VSIX download URL (only mentions are these four lines). Failure scenario: a `0.1.0` user follows the GA LSP guide, has no `sifr-vscode-0.2.0.vsix` on disk and no documented place to obtain one, while the sentence at `:75` tells them the Marketplace is not yet an install source. *Remediation:* state the actual `0.1.0` acquisition path — either add the VSIX to the governed version-release asset set (workflow allowlist + plan/sign-off asset binding) and reference its release URL, or describe the Marketplace listing published by the protected workflow at activation and adjust the `install … from the Marketplace` forbidden pattern (`check_ga_release_docs.py:145`) to permit the post-activation wording deliberately.

**3. LOW — `sifr self update` help text still calls stable channels and pins "preview", which the docs gate forbids in prose.**
`crates/sifr/src/self_update_cli.rs:36,39` render as `--channel <CHANNEL>  Resolve the latest version for a preview channel` and `--version <VERSION>  Resolve one immutable preview version` (verified against the built binary). GA docs now document `sifr self update --channel stable` / `--version 0.1.0` (`docs/cli/overview.mdx:93-94`, `docs/installation.mdx:117-131`), and `check_ga_release_docs.py:143` bans the literal phrase *"one immutable preview version"* in documentation. Failure scenario: a `0.1.0` user runs `sifr self update --help` and is told stable resolution is a preview facility, in the exact words the GA gate rejects on the docs side. Pre-existing text, but this milestone owns the truthful GA CLI surface. *Remediation:* reword both doc comments to the canonical `alpha|beta|stable` / "one immutable governed version" phrasing and extend the internal/public drift check to the CLI help strings so the two surfaces stay bound.

### Non-actionable observations

- `TARGET_TRIPLE_RE` (`check_ga_release_docs.py:47-49`) only matches triples beginning `aarch64`/`x86_64`; a `riscv64-…` claim would evade the new allowlist. Narrow residual class.
- `qualify_stable_editor.py` writes canonical JSON with default `ensure_ascii=True` while `governance.common.canonical_json_bytes` uses `ensure_ascii=False`; inert while every field is ASCII.
- `stable_site_release_facts.json` remains a synthetic fixture (`aaaa…`/`ffff…`); real plan/index binding arrives with the `milestone_40_5` site adapter, as designed.
- The extension still contributes `sifr.showGeneratedRust` while GA docs place generated Rust outside the qualified surface — consistent with the recorded, user-authorized `plans/issues/active/adhoc_packaged_candidate_generated_rust.md` deferral.
- The demo requires a clean checkout then dirties the submodule via `npm ci`/`npm run package`; benign because `dist/` and `node_modules/` are ignored.

Findings 1–3 are actionable.

NOT APPROVED

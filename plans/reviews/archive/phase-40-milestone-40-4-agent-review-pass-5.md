## Milestone review — Phase 40 / `milestone_40_4` (pass 5)

Read-only. Range `origin/main...HEAD` = 17 commits, 66 files. Working tree and all recursive submodules clean before and after every check. Pointer `editor_integrations = d7577d492`, `editor_integrations/vscode = 273fd5d3e`, matching `vscode_extension_rules.json`. Rust interop untouched (0 files under `verification/areas/rust_interop/**`); the only `crates/**` change is the two self-update help doc comments pass 4 required.

### Checks re-run

| Check | Result |
|---|---|
| `areas run --area documentation --suite structure --suite ga-release` | PASS, variants=2, failures=0 (`claims=25`) |
| `areas run --area developer_tooling --suite editor-release` | PASS, variants=6, failures=0, blocking=0 |
| `areas run --area distribution_release --suite qualification --suite full` | PASS, variants=55, failures=0 |
| `check_ga_release_docs.py --self-test` | PASS (16 mutation cases) |
| `stable_editor_qualification_contract.sh` | PASS (editor + docs self-tests + renderer `--check`) |
| `release_qualification_workflow_contract.sh` | PASS |
| `check_file_size_guardrails.py` | PASS (2880 files, limit 900) |
| `cargo build -p sifr` + `sifr self update --help` | builds; help renders the new text |

### Pass-4 findings — verified closed

**1. Contributed setting (`docs/cli/lsp.mdx`).** `sifr.serverPath` is gone from `docs/**` entirely (`grep -rn serverPath docs/` → none); `:84` and `:175` now use `sifr.lsp.path`, which the real package contributes (`editor_integrations/vscode/package.json` → `sifr.diagnostics.mode, sifr.format.enable, sifr.lint.enable, sifr.lsp.path, sifr.lsp.trace.server`). I swept every `sifr.<key>` token in `docs/**`: only `sifr.lsp.path` and `sifr.format.enable` are editor settings, both contributed. No drift.

**2. Real acquisition path, exact identity, no rebuild.** `lsp.mdx:68-76` now states the protected activation publishes and verifies the exact qualified `sifr.sifr-vscode` `0.2.0` "without rebuilding it," then gives `code --install-extension sifr.sifr-vscode`. That is backed by the `milestone_40_5` scope line "Publish and verify the recorded VS Code extension from the main-repository protected workflow using `vsce publish` only when the recorded version is absent," and the ordering holds — 40_5 publishes the extension *before* dispatching the site deploy, so the page never goes live ahead of the listing. The stale "not yet an install source" sentence is gone; `Marketplace` now appears in `docs/**` only at `lsp.mdx:69-70`.

**3. Gate binds to actual extension metadata.** `check_ga_release_docs.py` reads the real submodule `package.json` (`VSCODE_PACKAGE_PATH`), asserts `publisher/name/version/sifrCompilerCompatibility` = `sifr`/`sifr-vscode`/`0.2.0`/`>=0.1.0,<0.2.0`, requires `contributes.configuration.properties["sifr.lsp.path"]`, and cross-binds those exact strings into `lsp.mdx`. `REQUIRED_BY_DOCUMENT["lsp"]` additionally pins `code --install-extension sifr.sifr-vscode`, `protected stable activation`, and `without rebuilding`. Two new mutation cases prove non-vacuity: `extension-setting-drift` (docs `sifr.lsp.path`→`sifr.serverPath`) and `extension-metadata-drift` (delete the contributed property). Missing submodule checkout fails closed via `DocumentationError`. A generic "any `sifr.*` in docs must be contributed" rule is infeasible here — stdlib module names share the prefix — so positive binding is the right shape.

**4. Truthful, guarded self-update help.** `self_update_cli.rs:36,39` now render as `Resolve the latest version for an alpha|beta|stable channel` / `Resolve one immutable governed version` (confirmed against the built binary above). Truthful against `self_update_metadata.rs:337-339` (`alpha|beta|stable` accepted) and `:71-75` (only `alpha`/`beta` prerelease labels, so a stable `--version` is an unprereleased immutable SemVer). Mechanically guarded by `validate_cli_help_contract`, which requires both exact phrases and rejects `preview channel` / `immutable preview version`, with mutation case `cli-help-preview-drift`.

**5. Target allowlisting covers other architectures.** `TARGET_TRIPLE_RE` is now arch-agnostic and the check is a positive diff against `TARGETS`. Verified empirically: `riscv64gc-unknown-linux-gnu`, `powerpc64le-unknown-linux-gnu`, `armv7-unknown-linux-gnueabihf`, `aarch64-apple-ios`, `x86_64-unknown-linux-musl`, `aarch64-pc-windows-msvc` all detected.

**6. Editor JSON canonical.** `qualify_stable_editor.canonical_json_text` uses `ensure_ascii=False, allow_nan=False, sort_keys=True, separators=(",", ":")` + `\n` — byte-identical to `governance.common.canonical_json_bytes`, which is what the producer `qualify_stable_target.py` writes. All four former inline `json.dumps` sites, including the `validate_target_report` canonical-byte comparison, route through it.

### Earlier passes re-spot-checked

Pass-3 finding 1 is closed structurally, not by grep: `ROLLBACK_VERSION: ${{ needs.validate.outputs.rollback_version }}` sits in the editor step's `env:` and `release_qualification_workflow_contract.sh:42-48` parses the YAML, locates the step by name, and asserts the binding (plus `required: true`, `default: none`, `editor needs: [build, validate]`). Docs↔inventory mutation registration matches exactly (16 cases both sides), `documentation ga-release` is wired into `runner.py`, `manifest.json`, and `verification/profiles/release.json`, and the planner validates the documentation report's exact fields, `pass` status, and `("structure", "ga-release")` suite order.

### Non-actionable residuals

- `TARGET_TRIPLE_RE` matches only `apple|unknown|pc` vendors; `aarch64-linux-android` or `sparcv9-sun-solaris` would evade it. Architecture coverage — the named requirement — is complete; this is a vendor-shaped remainder.
- `code --install-extension sifr.sifr-vscode` resolves latest rather than pinning `0.2.0`; the adjacent prose names the exact version and `0.2.0` is the activation version.
- `stable_site_release_facts.json` is still a synthetic fixture (`aaaa…`/`ffff…`); real plan/index binding arrives with the `milestone_40_5` site adapter, as designed.
- All `milestone_40_4` checkboxes remain unchecked; candidate-evidence materialization and the evidence-only PR are correctly left post-merge, as the review brief specifies.

No actionable findings remain.

APPROVED

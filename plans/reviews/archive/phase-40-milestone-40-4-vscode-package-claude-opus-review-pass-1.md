## Review: sifr-lang/sifr-vscode — Phase 40 / milestone_40_4 (package portion)

**Repository:** `sifr-lang/sifr-vscode` at `editor_integrations/vscode` (nested repo, `origin` = `https://github.com/sifr-lang/sifr-vscode.git`)
**Range reviewed:** `346a93cc7c7b6b063de62e4ba83e1e19926bd21b..3d304b257f1b92f24831ffed55b145e18c749386`
**Exact HEAD:** `3d304b257f1b92f24831ffed55b145e18c749386` ("Prepare VS Code extension for stable Sifr")
**Working tree:** clean before and after review; no files or repository state modified. All scratch work was done under `/tmp`.

Diff: `CHANGELOG.md`, `README.md`, `package.json`, `src/extension.ts`, `src/lsp.ts`, `package-lock.json` (6 files, +108/−292).

### Read-only checks run

| Check | Result |
|---|---|
| `npm run lint` | PASS |
| `npm run typecheck` (`tsc --noEmit`) | PASS |
| `npm test` (compile + unit) | PASS |
| `npm run test:extension` | PASS |
| `npm ci --dry-run` | lockfile consistent with `package.json` ("up to date") |
| `npm audit` at HEAD | **0 vulnerabilities** |
| `npm audit --package-lock-only` on base lockfile (in `/tmp`) | **5 high** (js-yaml, linkify-it, undici ×7 advisories) → audit resolution confirmed real |
| Committed VSIX / `dist` artifacts | none tracked at HEAD; `git log --all --diff-filter=A -- '*.vsix'` returns nothing (never committed). `.gitignore` covers `dist/` and `*.vsix` |
| VSIX inventory (`dist/sifr-vscode-0.2.0.vsix`, untracked build output) | exactly one VSIX; 369 entries; ships `out/src/*.js`, `node_modules/vscode-languageclient@10.1.0`, `readme.md`, `changelog.md`, grammar, language config, icon, LICENSE |
| LC10 API contract | `node_modules/vscode-languageclient/lib/common/client.d.ts:251-253` — `outputChannel`/`traceOutputChannel` are `LogOutputChannel` in 10.x, so the `src/lsp.ts` + `src/extension.ts` change is required and minimal |
| Isolated typecheck of `src/`+`test/`+LC 10.1 `.d.ts` against `@types/vscode@1.90.0`, `skipLibCheck: false` | clean — no type-level incompatibility with the declared 1.90 floor |
| Live LSP handshake: `vscode-jsonrpc@9.0.1` ↔ `sifr lsp --stdio` | `initialize` OK, `serverInfo={"name":"sifr-lsp","version":"0.0.0"}`, 22 capabilities, `shutdown`/`exit` clean. Note: only a local dev build (`sifr 0.0.0`) was available, not the stable candidate |
| Build/test/package-only boundary | `.github/workflows/ci.yml` runs `npm ci`, lint, typecheck, test, test:extension, package — no publish step. `vsce publish` / `VSCE_PAT` appear nowhere in this repo or the main repo's workflows/scripts |
| `npm run package` | **not executed** — it would overwrite the `dist/` VSIX whose SHA-256 may already be recorded as candidate evidence. Assessed statically |

### Findings

**1. HIGH — the advertised compiler range is in a syntax the governance range validator rejects, and nothing binds the two** (`package.json:6`)

`package.json:6` declares `"sifrCompilerCompatibility": ">=0.1.0 <0.2.0"` (space-separated semver AND), repeated in `README.md:33-34` and `CHANGELOG.md:5-6`. The only code in the repo that *parses* a compiler range is the milestone_40_3 rollback-target validator, `verification/areas/distribution_release/governance/incident_fixture.py:506-512`:

```python
match = re.fullmatch(r">=([0-9]+\.[0-9]+\.[0-9]+),<([0-9]+\.[0-9]+\.[0-9]+)", expression)
if match is None:
    fail("compiler_compatibility", "must use >=X.Y.Z,<X.Y.Z")
```

Comma-only. Fed the packaged string, it fails before any containment check. The validator instead reads a hand-written fixture, and the only producer hard-codes the comma form (`incident_recovery_selftest.py:722-726`: `">=0.1.0,<0.2.0"`), while the qualification/plan path carries the space form verbatim (`.github/workflows/release-qualification.yml:173-175` → `qualification_fixture.py:252`, `schema_contracts.py:291`). Plan-side validation never parses it: `release_plan.py:314` only requires a non-empty string and `planner.py:338-345` only requires equality with the editor report.

Failure scenario: an `incident-roll-forward`/rollback with a non-`none` target. The DoD item "the packaged extension range passes the rollback-target validator introduced in `milestone_40_3`" is unmet — the packaged range is both syntactically rejected by that validator and not derived into the artifact the validator reads, so the negative-validation case "a non-`none` rollback target outside the advertised compiler range fails the gate" cannot be exercised against the value the extension actually ships. Note the space form is the correct/only valid semver spelling, so the resolution likely belongs in the main-repo parser (or a normalizer that derives `extension-metadata.json` from `package.json`) — but the divergence is live and undetected today, and the string under review is the package's.

**2. MEDIUM — no gate binds package version or compiler range across `package.json`, `README.md`, and `CHANGELOG.md`** (`test/extensionSmoke.test.ts:1-48`, `scripts/lint.js`)

The "extension smoke test" is static metadata assertion only — commands, grammar scope, trace enum, `vscode-languageclient` presence, `.vscodeignore` not excluding `node_modules`, and `$npm_package_version` in the package script. It never asserts `version`, never asserts `sifrCompilerCompatibility` exists or is well-formed, and never cross-checks the README/CHANGELOG range or version text. `scripts/lint.js` checks neither. The main repo only checks the key is a non-empty string (`release-qualification.yml:173-175`).

This is not theoretical: at the base commit `README.md` claimed "Version `0.1.5` supports the Sifr CLI/LSP `0.1.0-beta.12`" while `package.json` was at `0.1.7` and the CHANGELOG's top entry named `0.1.0-beta.14` — two-version drift in the Marketplace-visible page, passed by the same gates that pass today. The milestone's negative validation requires "compiler-range drift, extension-version drift … fail the gate"; in the authoritative checkout nothing does.

**3. LOW — declared VS Code engine floor is below its runtime dependency's declared floor, and the type gate cannot detect it** (`package.json:23-25`)

`engines.vscode: "^1.90.0"` while `vscode-languageclient@10.1.0` declares `engines.vscode: "^1.91.0"`, so 1.90.x users can install a combination the library declares unsupported. Compounding it, `"@types/vscode": "^1.90.0"` resolves to **1.120.0** in the lockfile, so `npm run typecheck` validates against an API surface 30 minors past the declared floor. I did verify by isolated typecheck against `@types/vscode@1.90.0` with `skipLibCheck: false` that there is no *current* incompatibility, so no demonstrated break — the issue is that the floor is under-declared and unverified. Remedies: raise `engines.vscode` to `^1.91.0` (coordinated with `verification/areas/developer_tooling/vscode_extension_rules.json`'s `minimum_vscode_engine`, and `check_vscode_extension_rules.py:101-102` which asserts exact equality), and/or pin `@types/vscode` to `~1.90.0`.

**4. LOW — only `dist/` is cleaned; `out/`, whose contents actually ship, is not** (`package.json:198`)

The new step clears `dist` before `vsce package`, which correctly fixes a real bug (previously `mkdir -p dist && vsce package --out dist/…` with a leftover `dist/sifr-vscode-0.1.7.vsix` would embed the old VSIX in the new one, since `.vscodeignore` does not exclude `dist/`). But `compile` is plain `tsc -p ./`, which never prunes `out/`, and `.vscodeignore` excludes only `out/test/**` and `**/*.map` — so a stale `out/src/*.js` from an earlier build ships. This matters because the local `editor-release` suite packages in-place in the developer's checkout (`verification/areas/developer_tooling/check_vscode_extension.py`, `validate()`), which undercuts the byte-for-byte reproducibility DoD. `out/` is clean right now (7 sources ↔ 7 emitted files), so this is latent.

Informational, pre-existing: the VSIX ships `extension/.gitignore` (`.vscodeignore` does not exclude it).

### Confirmed correct

- Extension SemVer independence at `0.2.0` — `package.json:5`, `CHANGELOG.md:3`, and `README.md:36-40` ("intentionally independent from the Sifr compiler version").
- Explicit range `>=0.1.0 <0.2.0` present in `package.json:6` (consumed and fail-closed by `release-qualification.yml:173-175`, bound into the plan), `README.md:33-34` (Marketplace page), and `CHANGELOG.md:5-6` (release notes). Contains the candidate `0.1.0`.
- No exact mutable active-version claim: the previous "supports the Sifr CLI/LSP `0.1.0-beta.12`" wording is replaced by a range; `description` carries no version; historical CHANGELOG entries are appropriately left as history.
- Exactly one VSIX, and `release-qualification.yml:176-178` fails on `len(built) != 1`.
- No committed VSIX artifacts at HEAD or anywhere in history.
- Dependency audit resolution verified end to end (5 high → 0), driven by `@vscode/vsce` `^3.2.1` → `^3.9.2`.
- `vscode-languageclient` 10.1 migration is correct, necessary, and minimal.
- `npm install` → `npm ci` in `README.md:21`, matching the milestone's required command.
- Build/test/package-only boundary holds; no Marketplace publication surface exists in this package.

Findings 1 and 2 are actionable package issues.

NOT APPROVED

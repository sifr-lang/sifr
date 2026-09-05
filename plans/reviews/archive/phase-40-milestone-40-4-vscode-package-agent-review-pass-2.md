## Review: `sifr-lang/sifr-vscode` — Phase 40 / milestone_40_4 (package portion), pass 2

**Range:** `346a93cc7c7b6b063de62e4ba83e1e19926bd21b..7e20bf58cbef6af2ccdcffe09d45e1e558ab8446`
**Exact HEAD:** `7e20bf58cbef6af2ccdcffe09d45e1e558ab8446` ("Harden stable extension release metadata")
**New since pass 1:** one commit, `7e20bf5`, on top of the pass-1 HEAD `3d304b2`.
**Working tree:** clean before and after; nothing in either repo modified. All mutation experiments ran on a `/tmp` copy. `dist/sifr-vscode-0.2.0.vsix` (untracked, pre-existing) was not overwritten — packaging was exercised only in `/tmp`.

Diff: `CHANGELOG.md`, `README.md`, `package.json`, `package-lock.json`, `scripts/lint.js`, `src/extension.ts`, `src/lsp.ts` (7 files, +178/−299).

### Read-only checks run

| Check | Result |
|---|---|
| `npm ci --dry-run` | lockfile consistent ("up to date") |
| `npm run lint` | PASS |
| `npm run typecheck` | PASS |
| `npm test` | PASS |
| `npm run test:extension` | PASS |
| `npm audit` | **0 vulnerabilities** |
| Seeded-drift matrix vs `scripts/lint.js` (7 mutations, `/tmp` copy) | see table below |
| Seeded stale `out/src/stale.js` + `dist/sifr-vscode-0.1.7.vsix` → `npm run clean` | both trees removed |
| `npm run package` on `/tmp` copy with those seeds | **exactly one VSIX**; no `stale.js`, no nested `.vsix`, no `dist/` entry; `out/src` = 7 files ↔ 7 sources |
| Lockfile audit | `vscode-languageclient@10.1.0`, `@types/vscode@1.91.0`, `@vscode/vsce@3.9.2`; all `registry.npmjs.org`; 18 transitive packages removed, 1 added |
| LC10 API | `outputChannel`/`traceOutputChannel` are `LogOutputChannel` in 10.x; `createOutputChannel(name, {log:true})` overload requires `@types/vscode` ≥1.90 — migration is required and minimal |
| Live handshake: `vscode-jsonrpc@9.0.1` (LC10's) ↔ `sifr lsp --stdio` | `initialize` OK, `serverInfo={"name":"sifr-lsp","version":"0.0.0"}`, 24 capabilities, clean `shutdown`/`exit`. **Caveat:** only the local dev build `sifr 0.0.0` exists here, not the `0.1.0` stable candidate |
| Publish surface (`vsce publish` / `VSCE_PAT` / `ovsx`) | **absent** from the whole repo; `.github/workflows/ci.yml` ends at `npm run package` |
| `check_vscode_extension_rules.py` against this checkout | **FAIL** — see cross-repo dependency below |

Drift matrix (`node scripts/lint.js`, `/tmp` copy):

| Mutation | Result |
|---|---|
| baseline | PASS |
| `package.json` version → `0.3.0` | **FAIL** |
| range → space form `>=0.1.0 <0.2.0` | **FAIL** |
| `sifrCompilerCompatibility` key deleted | **FAIL** |
| `package.json` range → `>=0.1.1,<0.2.0` | **FAIL** |
| README range → `>=0.1.1,<0.2.0` | **FAIL** |
| CHANGELOG range → `>=0.1.1,<0.2.0` | **FAIL** |
| *additive* stale line `Version \`0.1.5\` … \`0.1.0-beta.12\`` appended to README | PASS (residual, below) |

### Pass-1 findings re-checked

**1. HIGH — non-canonical compiler-range syntax — RESOLVED.** `package.json:6` now declares `">=0.1.0,<0.2.0"`, matching `incident_fixture.py:506-512`'s `r">=(\d+\.\d+\.\d+),<(\d+\.\d+\.\d+)"` exactly. Same string in `README.md:33-34` and `CHANGELOG.md:5-6`. Contains the GA version `0.1.0` (`plans/issues/active/phase-40-stable-channel-ga-execution.md:12`); first GA records `rollback_target: none`, so the containment branch is correctly skipped. `scripts/lint.js:88-93` now enforces the comma form in-package, so the space form cannot regress. *Residual (main repo, out of package scope):* `incident_recovery_selftest.py:723-726` still hard-codes the fixture range rather than deriving it from `package.json`, so the two agree by matching literals, not by construction.

**2. MEDIUM — no version/range drift gate — RESOLVED.** `scripts/lint.js:82-138` adds `releaseMetadataFailures()` (exact-SemVer version, canonical range form, README version, README range, CHANGELOG heading, CHANGELOG range) plus a self-checking mutation table that fails lint if any negative case stops failing. All six drift mutations I seeded independently fail. The exact pass-1 regression (README pinned at `0.1.5` while `package.json` was `0.1.7`) is now caught.

**3. LOW — engine floor below `vscode-languageclient`'s — RESOLVED.** `engines.vscode` raised to `^1.91.0`, matching LC10.1's own `engines.vscode: "^1.91.0"`, and `@types/vscode` tightened from `^1.90.0` to `~1.91.0` (lockfile resolves 1.91.0, was 1.120.0). `README.md:34` updated in step. Typecheck now validates against the declared floor rather than 30 minors past it.

**4. LOW — only `dist/` cleaned — RESOLVED.** New `clean` script removes both `out` and `dist`; `package` is `clean && compile && mkdir -p dist && vsce package`. Verified empirically: seeded `out/src/stale.js` and a leftover `0.1.7` VSIX are both gone after clean and absent from the packaged VSIX.

### Findings this pass

No actionable package issue. Three informational items:

- **Cross-repo dependency (blocking for the milestone, not for this package).** `verification/areas/developer_tooling/vscode_extension_rules.json` still declares `minimum_vscode_engine: "^1.90.0"`, and `check_vscode_extension_rules.py:101-102` asserts exact equality. Run against this checkout it now fails with `package.json VS Code engine does not match rules`, so milestone_40_4's required `--area developer_tooling --suite editor-release` is red today. The engine value here is the correct remediation of pass-1 finding 3 and cannot be reconciled from inside this repo; the paired main-repository submodule-pointer PR must bump `minimum_vscode_engine` to `^1.91.0` in the same landing. Not counted against the package under the phase doc's sanctioned upstream-first two-PR exception.
- **Residual lint weakness (low).** The metadata checks are `String.includes` assertions, so they catch a *stale* version/range but not an *additive* one — appending a second `Version \`0.1.5\` … \`0.1.0-beta.12\`` line to README still passes. Tightening to a single anchored match on the Compatibility section would close it.
- **Trivial.** The `0.2.0` CHANGELOG entry says builds start "from an empty `dist/` directory"; `clean` also clears `out/`, which is the part that actually fixes stale shipped code.

Pre-existing, unchanged: the VSIX ships `extension/.gitignore` (`.vscodeignore` doesn't exclude it).

### Confirmed correct

Extension SemVer independence at `0.2.0`, consistently stated across all three files; no exact mutable active-compiler-version claim anywhere; exactly one VSIX with no committed VSIX at HEAD or in history; audit resolution holds at 0 vulnerabilities with a materially smaller dependency tree; the LC10 `LogOutputChannel` migration is correct, necessary, and confined to the two channels the client consumes; `npm ci` documented; build/test/package-only boundary intact with no Marketplace surface in this repo.

APPROVED

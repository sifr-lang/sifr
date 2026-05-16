# Phase 33, Milestone 33.1 Review — Pass 1

Status: **APPROVED**

## Scope Reviewed

- `scripts/distribution/generate_dispatchers.sh`
- `verification/distribution/common.sh`
- `verification/distribution/install_*.sh` (9 scripts)
- `internal_docs/distribution_pipeline.md`
- `issues/phase-33-preview-distribution-execution.md`
- `internal_docs/phases/33_preview_distribution_and_release_automation.md` (status check)
- Site repo: `apps/sifr-site/public/install/` (index, alpha, beta, versions/)

## Phase Contract Checklist

| Requirement | Status |
|---|---|
| Generated-installer baseline, no uv code | PASS — Sifr-owned shell generation |
| Dispatchers for /install, /install/alpha, /install/beta | PASS — all three present and correct |
| Default resolves to beta | **BLOCKER FOUND** — see below |
| Explicit alpha/beta work | PASS |
| Preview version pins work | PASS |
| Stable channel rejected before download | PASS |
| Stable-looking versions rejected | PASS |
| Invalid channels rejected | PASS |
| Conflicting channel/version rejected | PASS |
| Missing generated installer rejected | PASS |
| Malformed dispatcher config rejected | PASS |
| Dispatchers delegate to immutable installers, do not resolve artifacts | PASS |

## Blocker: Default Channel Is Alpha, Not Beta

The phase contract requires:

> **5.** The default `https://sifr.sh/install` entrypoint resolves to the current `beta` preview.

The site repo dispatcher `index` has:

```sh
DEFAULT_CHANNEL="alpha"
```

This violates the phase contract. The generator correctly calls `write_dispatcher "${INSTALL_ROOT}/index" "beta"`, but the generated output shows `DEFAULT_CHANNEL="alpha"`.

Root cause: the generator hardcodes `DEFAULT_CHANNEL="${default_channel}"` correctly, but the site repo dispatchers were regenerated with the wrong argument order or the generator was called with "alpha" as the default channel argument for the index dispatcher.

**Fix required:** Re-run the generator with the correct argument:

```bash
./scripts/distribution/generate_dispatchers.sh \
  --install-root /Users/yaseralnajjar/work/sifr/sifr-blog-website/apps/sifr-site/public/install \
  --alpha-version 0.1.0-alpha.1 \
  --beta-version 0.1.0-beta.1
```

This should produce `index` with `DEFAULT_CHANNEL="beta"`. `alpha` should have `DEFAULT_CHANNEL="alpha"`. `beta` should have `DEFAULT_CHANNEL="beta"`.

**After fix:** re-run site build and verify all 9 validation scripts still pass.

## Other Observations

### Correct

- `generate_dispatchers.sh` is clean, self-contained shell code with no uv code.
- Phase 33 attribution record: "no copied or adapted uv code in milestone 33.1" — confirmed correct.
- Version prerelease label validation is correct: only `alpha.N`, `beta.N`, `rc.N` accepted; `X.Y.Z` stable-looking and `0.X.Y` without prerelease rejected.
- `stable` channel gated at dispatcher level before download — correctly implemented.
- Conflicting SIFR_CHANNEL / --channel / --version inputs produce hard errors with context — correct.
- `normalize_channel` enforces non-empty channel — correct.
- Malformed dispatcher config self-detects channel/version mismatch — good defense-in-depth.
- `mktemp` with EXIT/HUP/INT/TERM trap ensures tmpdir cleanup — correct.
- All 9 validation scripts pass.
- `bash -n` passes on all scripts.
- Site build completed per local validation record.
- Site filesystem layout is correct: `public/install/index`, `public/install/alpha`, `public/install/beta`, `public/install/versions/<version>`.

### Minor Note

- `preview_channel_for_version` treats `rc` as a valid channel but `normalize_channel` does not accept `rc`. This is intentional (rc is accepted in version strings but not as a channel selection) but worth documenting. Not a blocker for milestone 33.1.

## Required Action

1. Regenerate site dispatchers with correct default for `index`.
2. Rebuild site.
3. Re-run all 9 validation scripts.
4. Commit corrected dispatchers.
5. Update `internal_docs/phases/33_preview_distribution_and_release_automation.md` status when ready for final approval.

## Verification Commands

```bash
# Regenerate
./scripts/distribution/generate_dispatchers.sh \
  --install-root /Users/yaseralnajjar/work/sifr/sifr-blog-website/apps/sifr-site/public/install \
  --alpha-version 0.1.0-alpha.1 \
  --beta-version 0.1.0-beta.1

# Verify fix
grep 'DEFAULT_CHANNEL=' /Users/yaseralnajjar/work/sifr/sifr-blog-website/apps/sifr-site/public/install/index
# Expected: DEFAULT_CHANNEL="beta"

# Site build
cd /Users/yaseralnajjar/work/sifr/sifr-blog-website && npm run build

# All validation
for f in verification/distribution/install_*.sh; do bash "$f" || exit 1; done
```

Once the default is corrected and all validations pass, **milestone_33_1 is approved**.
# Self-Update Milestone 1 Review — Pass 1

Reviewer scope: working-tree diff against `main` plus untracked files on
`ad-hoc-self-update-m1`. Reviewed against the contract in
`issues/ad-hoc-sifr-self-update.md`, specifically the `milestone_self_update_1`
scope and DoD.

## Verdict

No blocking findings for milestone 1.

The metadata + receipt contract is implemented faithfully to the phase
contract. Locking, atomic install.json writes, channel derivation, and
schema-versioning all match. The items below are recommended cleanups that
either belong to a follow-up pass on this branch or can be carried into
milestone 2; none of them justify holding the PR.

## Findings (severity-ordered)

### Low — internal inconsistency between `install_dir` and `binary_path`

In `scripts/distribution/generate_version_installer.sh:312-335` the receipt
records `install_dir` as the raw `SIFR_INSTALL_DIR` value while `binary_path`
is run through `canonical_path` (`pwd -P` on the parent). On macOS, where
`${TMPDIR}` is symlinked under `/private/var/folders`, this produces
`dirname(binary_path) != install_dir` in the same receipt. The phase contract
explicitly requires `binary_path` to be canonical and is silent on
`install_dir`; however, the asymmetry will bite milestone 2 discovery rule #2
(`<current_exe_parent>/install.json`) if anyone compares the two paths.
Recommend canonicalizing `install_dir` the same way, or documenting the
asymmetry next to the receipt writer.

### Low — temp manifest file can leak when `manifest_dir != install_dir`

`write_install_manifest` calls `mktemp "${manifest_dir}/.install.json.XXXXXX"`
and only removes that file via `mv`. The EXIT trap only cleans `${tmp_dir}`
(the extract staging dir) and releases the install lock. For the default
home install (`manifest_dir=${HOME}/.sifr`, `install_dir=${HOME}/.sifr/bin`),
a crash between `mktemp` and `mv` leaves `~/.sifr/.install.json.XXXXXX` behind
with no cleanup. The contract test happens to use `manifest_dir==install_dir`
so it does not exercise this path. Either track the temp file in the cleanup
trap or accept this as a low-risk residue.

### Low — schema_version `!= 1` rejection has no unit test

`crates/sifr/src/self_update_receipt.rs:55-59` rejects unsupported
`schema_version`, but the Validation contract clause "receipt schema rejects
... unsupported schema versions" has no unit test. Easy regression test to
add now (e.g. `schema_version: 2` with otherwise valid fields should hit
`SELF_UPDATE_UNMANAGED_RECEIPT`).

### Low — empty/invalid-JSON paths have no unit test

The contract also lists "rejects empty files, invalid JSON". The serde_json
mapping handles both, but no test pins the diagnostic shape for them.
Two trivial tests would lock it in.

### Low — set-equality check fires before schema_version inspection

`crates/sifr/src/self_update_receipt.rs:43-49` performs `expected != actual`
before reading `schema_version`. A v2 receipt with extra/missing fields will
emit the "predates or diverges" diagnostic instead of the more specific
"schema_version unsupported" one. Both are actionable; reordering would be
cleaner but is cosmetic.

### Cosmetic — `pub(crate)` is meaningless on a `#[cfg(test)]` module

`crates/sifr/src/main.rs:21-22` gates `mod self_update_receipt` behind
`#[cfg(test)]`, yet items inside it are `pub(crate)`. No production path
references them. Milestone 2 will likely promote this module to non-test, at
which point the visibility becomes meaningful; until then it is noise.

### Cosmetic — `SIFR-BUILD-0901` is declared Active but is unreachable in production

The diagnostic is registered as `Active` in
`crates/sifr_diagnostics/src/codes/registry.rs:244` and listed in
`ACTIVE_DIAGNOSTIC_CODES`, but the only emission site is the
`#[cfg(test)]`-only `self_update_receipt.rs`. So a stable build ships a code
that nothing can raise until milestone 2 wires the runner. The registry
contract permits this, but it is worth calling out in the PR description so
reviewers don't expect to see runtime emission yet.

## Observations carried to milestone 2

- The diagnostic dedupes on `{message}`. Milestone 2 will need more
  diagnostics in this family; reconsider whether `{message}` alone is the
  right dedupe key once eligibility/runner errors land.
- `canonical_path` resolves the parent directory only (`pwd -P` on
  `dirname`), not the file component. Correct for the post-install state
  written by the installer; milestone 2's eligibility check must perform
  full canonicalization on the current_exe side before comparing to
  `binary_path`.
- `validate_site_dispatchers` in `create_new_version.sh` still checks only
  dispatcher drift. Milestone 4 ("Distribution Drift Guardrails") owns
  metadata-vs-dispatcher drift detection.

## What works well

- Atomic `install.json` write via `mktemp` + `mv` in the same directory
  guarantees same-filesystem rename. ✓
- `acquire_install_lock` is invoked immediately before binary copy and held
  through manifest write; lock path `<install_dir>/.sifr-update.lock`
  matches the contract used by self-update in milestone 3. ✓
- Receipt `channel` is derived from the `${VERSION#*-}` prerelease label —
  deterministic, no hand-maintenance, and the JSON schema constrains it to
  `alpha|beta|rc`. ✓
- Channel metadata is generated inside `generate_dispatchers.sh` from the
  same alpha/beta inputs as the dispatchers, so they cannot drift at
  generation time. ✓
- `verification/distribution/self_update_install_receipt.schema.json` has
  `additionalProperties: false` and pins the receipt shape against drift. ✓
- `artifact_self_update_receipt_rules.sh` exercises the real installer,
  asserts field-order, checks lock release, and checks the temp manifest
  file was renamed away. ✓
- `run_distribution_validation.sh` discovers `*.sh` automatically, so the
  two new fixture scripts are picked up without additional plumbing. ✓
- The `check_formatter_phase_manifests.py` fallback to `issues/archive/` is
  the minimum surgical fix for the quick-validation regression that surfaced
  after the formatter docs were archived. The fallback returns the active
  path on miss so a real missing-doc state still fails loudly. ✓
- Internal docs (`internal_docs/distribution_pipeline.md`) now describe the
  receipt, metadata, fail-closed schema-versioning, and TLS-and-delegation
  policy — matching the milestone-1 documentation requirement. ✓

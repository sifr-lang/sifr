# Self-Update Milestone 1 Review — Pass 3

Reviewer scope: working-tree state on `ad-hoc-self-update-m1` (modified files
plus untracked artifacts), reviewed against
`issues/ad-hoc-sifr-self-update.md` `milestone_self_update_1` scope and DoD,
and against `reviews/self-update-m1-review-pass-2.md` to confirm the
pass-2 follow-up landed.

## Verdict

**Satisfied — no blocking findings for milestone 1.**

The pass-2 coverage gap (the contract test only exercised the
`modify_path: false` branch of the generated installer) is now closed.
`verification/distribution/artifact_self_update_receipt_rules.sh:82-101`
runs the installer a second time with neither `SIFR_NO_MODIFY_PATH` nor
`--no-modify-path`, using a tmp-scoped `HOME` (`${tmp_dir}/home`), a clean
`SHELL=/bin/sh`, and a stripped `PATH=/usr/bin:/bin`, then asserts
`modify_path is True`. With this addition, both branches of the
`write_install_manifest` `modify_path` selector at
`scripts/distribution/generate_version_installer.sh:316-320` are covered
end-to-end by a real installer run, not just by unit-test JSON literals.

The pass-2 fix is correctly scoped:

- The `HOME=${tmp_dir}/home` redirect ensures the side effects of
  `configure_path` (writes to `~/.profile` and `~/.sifr/env`) land inside
  the test tempdir and are reaped by the existing `trap cleanup EXIT HUP
  INT TERM` at line 11.
- `PATH=/usr/bin:/bin` keeps `configure_path` from being short-circuited
  by an `install_dir` that happens to already be on the test runner's
  PATH, so the modify_path=true code path is actually exercised.
- `manifest_dir` derivation at `generate_version_installer.sh:507-514`
  picks `manifest_dir=${install_dir}` (not `${HOME}/.sifr`) because the
  second invocation uses `${tmp_dir}/path-managed/bin`, which is not
  `${HOME}/.sifr/bin`. So the receipt under assertion is exactly
  `${path_install_dir}/install.json`, matching the test.

Local gates re-run here per the task report:

- `cargo fmt --check` → clean
- `cargo test -p sifr -- self_update` → 7 passed
- `scripts/run_distribution_validation.sh` → pass
- `scripts/run_all_tests.sh --profile quick` → pass (advisory only for
  group skew)

## Pass-2 follow-up status

| Pass-2 finding | Severity | Status in pass 3 |
|---|---|---|
| Variable-init order deserves a comment near `trap cleanup` | Low | Not addressed. Carried as cosmetic — both `install_lock_path` and `manifest_tmp` are initialised before the trap at `generate_version_installer.sh:517-518`, and the cleanup handler null-checks both. The foot-gun only fires if a future edit moves the trap above the init lines while running under `set -u`. Non-blocking. |
| Contract test only exercised `modify_path: false` | Low | **Resolved** — second installer invocation at lines 82-101 asserts `modify_path is True` end-to-end. |
| JSON Schema document not used as an executable validator | Low | Not addressed. Pass-2 already classified this as M4's natural owner (distribution drift guardrails); pass 3 leaves it carried. |

## New pass-3 findings (severity-ordered)

### Low — second installer invocation does not re-assert lock/temp cleanup

The new second invocation at
`verification/distribution/artifact_self_update_receipt_rules.sh:82-101`
asserts only `receipt["modify_path"] is True`. It does not re-run the
`compgen -G "${install_dir}/.install.json.*"` temp-file check or the
`.sifr-update.lock` assertion at lines 72-80 against `${path_install_dir}`.
The first invocation still pins those properties for the
`SIFR_NO_MODIFY_PATH=1` code path, and the lock acquisition and atomic
manifest rename are unconditional (they don't branch on `NO_MODIFY_PATH`),
so the gap is theoretical — the same code paths handle both invocations.
Defensible to defer; a three-line repetition of the post-first-invocation
checks would close it. Non-blocking.

### Cosmetic — second invocation re-validates one field only

The first invocation feeds the receipt through the full schema-ordered
field check (order, set equality, every value). The second invocation
only checks `modify_path`. Drift in any other field on the modify-path
true branch would not be caught here — but it would be caught by the
first invocation, which exercises the same `write_install_manifest`
code path with the same field-by-field rendering. No new test surface
needed; flagging only for completeness.

### Cosmetic — `reviews/self-update-m1-review-pass-1.md` and pass-2 carry items unchanged

The four items pass-1 already classified Low/Cosmetic and pass-2 carried
forward (install_dir canonicalization asymmetry, set-equality firing
before schema_version inspection, `pub(crate)` on a `#[cfg(test)]`
module, SIFR-BUILD-0901 unreachable in production until M2) remain
unaddressed. None of them are M1 DoD blockers and all are correctly
owned by milestone 2.

## What works well in pass 3

- The pass-2 fix uses the minimum surgical surface: one additional
  installer invocation, one additional python receipt check, both
  scoped inside the existing tempdir cleanup. No new fixtures, no new
  helpers. ✓
- The second invocation explicitly establishes a clean
  `HOME=${tmp_dir}/home`, so `configure_path`'s shell-profile writes
  land in the tempdir and are reaped. The test does not pollute the
  test runner's home directory. ✓
- `manifest_dir` resolution intentionally avoids the
  `${HOME}/.sifr` Phase-33 manifest path for this test's
  `install_dir`, so the receipt under assertion is the
  `install_dir`-local `install.json`. The test exercises the
  generic-install branch of the manifest-dir selector, complementing
  the implicit default-install branch elsewhere. ✓
- All seven `self_update`-prefixed unit tests still pass; the receipt
  parser DoD ("rejects empty files, invalid JSON, wrong field types,
  unknown fields in schema-versioned receipts, and unsupported schema
  versions") remains fully covered. ✓
- `scripts/run_distribution_validation.sh` continues to discover both
  `artifact_self_update_receipt_rules.sh` and
  `channel_metadata_generated.sh` automatically, so no plumbing
  changes were needed to wire pass-2's fix into CI. ✓

## Carried to milestones 2 / 4

Unchanged from pass 2:

- M2: eligibility-side canonicalization must symmetrically match the
  installer-side `binary_path` canonicalization, or `install_dir`
  asymmetry must be documented in `internal_docs/distribution_pipeline.md`.
- M2: the runner becomes the first production emitter of
  `SIFR-BUILD-0901`; reconsider whether `{message}` is the right
  dedupe key for what will be a family of self-update diagnostics.
- M4: wire `self_update_install_receipt.schema.json` into the
  distribution validation suite as an executable schema validator,
  not just a required-field comparison.

## Conclusion

Milestone 1 is ready to merge. The metadata + receipt contract is fully
implemented, both branches of `modify_path` are exercised end-to-end, the
distribution drift fixtures are in place, the receipt parser rejects all
the malformed shapes listed in the validation contract, and the local
closure gate (`cargo fmt --check`, `cargo test -p sifr -- self_update`,
`scripts/run_distribution_validation.sh`, `scripts/run_all_tests.sh
--profile quick`) is green.

# Self-Update Milestone 1 Review — Pass 2

Reviewer scope: working-tree state on `ad-hoc-self-update-m1` (modified files
plus untracked artifacts), reviewed against
`issues/ad-hoc-sifr-self-update.md` `milestone_self_update_1` scope and DoD,
and against `reviews/self-update-m1-review-pass-1.md` to confirm pass-1
follow-ups landed.

## Verdict

**Satisfied — no blocking findings for milestone 1.**

The pass-1 items the author claimed to address are addressed correctly: the
generated installer's EXIT trap now reaps a partially-created manifest temp
file, and `self_update_receipt.rs` now has dedicated unit tests for
unsupported `schema_version`, empty receipt JSON, and invalid receipt JSON.
A `rejects_wrong_field_types` test was also added — not in the author's
summary but listed by the M1 validation contract, so it's a bonus.

Local gates re-run here:

- `cargo test -p sifr -- self_update` → 7 passed, 0 failed
- `cargo fmt --check` → clean
- `cargo clippy -p sifr --no-deps -- -D warnings` → clean
- `scripts/run_distribution_validation.sh` → exit 0
  (`channel_metadata_generated.sh` and `artifact_self_update_receipt_rules.sh`
  run as part of the suite).

## Pass-1 follow-up status

| Pass-1 finding | Severity | Status in pass 2 |
|---|---|---|
| Temp manifest leak outside `tmp_dir` | Low | **Resolved** — `cleanup()` now removes `${manifest_tmp}` when set, and `write_install_manifest` clears the variable after `mv`. Safe under EXIT/HUP/INT/TERM. |
| `schema_version` unsupported rejection lacked a unit test | Low | **Resolved** — `rejects_unsupported_schema_version`. |
| Empty / invalid-JSON paths lacked unit tests | Low | **Resolved** — `rejects_empty_receipt_json`, `rejects_invalid_receipt_json`. |
| `install_dir` not canonicalized while `binary_path` is | Low | Not addressed; carried to M2 (eligibility-check side). |
| Set-equality fires before `schema_version` inspection | Low | Not addressed; cosmetic, both diagnostics route through `SELF_UPDATE_UNMANAGED_RECEIPT`. |
| `pub(crate)` meaningless on a `#[cfg(test)]` module | Cosmetic | Not addressed; becomes meaningful when M2 wires the runner. |
| `SIFR-BUILD-0901` Active but unreachable in production | Cosmetic | Not addressed; M2 will produce the first non-test emission site. |

The four unaddressed items were already classified Low/Cosmetic in pass 1
and none of them are M1-DoD blockers.

## New pass-2 findings (severity-ordered)

### Low — EXIT cleanup is correct but the variable-init order deserves a comment

`cleanup()` references `install_lock_path` and `manifest_tmp`. Both are
initialised to empty strings at
`scripts/distribution/generate_version_installer.sh:517-518` **before**
`trap cleanup EXIT HUP INT TERM` at line 563, so the trap is always safe to
fire. `release_install_lock` null-checks `${install_lock_path:-}`, and the
`manifest_tmp` branch guards on `[ -n ... ] && [ -f ... ]`. This is correct
today but fragile: if a future edit moves the trap above the `=""` lines, or
adds an early `exit` that fires before line 517 with `set -u`, cleanup will
reference unset variables. A one-line comment near the variable-init or trap
line stating "these must be defined before `trap cleanup`" would prevent that
foot-gun. Non-blocking.

### Low — receipt contract test only exercises `modify_path: false`

`verification/distribution/artifact_self_update_receipt_rules.sh:26-30`
runs the installer with both `SIFR_NO_MODIFY_PATH=1` *and*
`--no-modify-path`, then asserts `modify_path is False`. The
`modify_path: true` branch of `write_install_manifest` is only covered by
the unit-test JSON literal in `self_update_receipt.rs`, not by an end-to-end
installer run. M1 DoD says "Receipt validation proves … `modify_path`
reflect[s] the installer version and request" — the request half is half-
covered. Adding a second invocation with neither env nor flag set, asserting
`modify_path is True`, would close the gap. Cheap to add now; defensible to
defer to M2 since the unit test still proves the parser handles both shapes.

### Low — JSON Schema document exists but is not used as a validator anywhere

`verification/distribution/self_update_install_receipt.schema.json` carries
`additionalProperties: false`, `const`/`enum`/`pattern` constraints, and the
authoritative required-fields list. The Bash contract test reads
`schema["required"]` and compares to `list(receipt.keys())` for ordering and
set equality, but never feeds the receipt through a JSON-Schema validator —
so the `pattern` on `version`, the `enum` on `channel`, the `const` on
`name`/`schema_version`, and `additionalProperties: false` would not catch a
generated-installer drift on their own; they're only enforced by the
hand-written Python assertions next to them. The Rust parser independently
re-implements the same constraints. Two sources of truth claiming to be one.
Either run the receipt through `jsonschema` in `artifact_self_update_receipt_rules.sh`
(stdlib-only via a small inlined validator, or accept the optional dep),
or note explicitly in `internal_docs/distribution_pipeline.md` that the
schema is documentation, not an executable contract. M1 calls the schema
"authoritative"; today the Bash + Rust hand-checks are the actual gate.
Carried — milestone 4 ("distribution drift guardrails") is the natural
owner.

### Cosmetic — `rejects_unknown_fields` and `rejects_wrong_field_types` route through the generic "predates or diverges" message

Both new tests assert on `code` only (and `wrong_field_types` asserts that
the message contains `modify_path`). The user-facing message for an unknown
extra field is identical to the message for a missing field — both say
"predates or diverges from the schema-versioned self-update contract". This
matches pass-1's observation about the set-equality short-circuit being
cosmetic; it's not regression. If M2 wants to differentiate "extra field
present" from "field missing" for better user remediation, that requires
splitting the set check into two passes. Not a milestone-1 blocker.

## What works well in pass 2

- The EXIT trap addition is minimal and idempotent. `write_install_manifest`
  clears `manifest_tmp` after `mv`, so the trap is a no-op on the happy
  path and a single `rm -f` on the failure path. ✓
- All five "rejects" cases listed by the M1 validation contract
  ("empty files, invalid JSON, wrong field types, unknown fields in
  schema-versioned receipts, and unsupported schema versions") now have
  dedicated unit tests pinning both the diagnostic code and a substring of
  the human message. The receipt parser DoD reads as fully covered. ✓
- The contract test continues to assert no `.install.json.*` temp file
  survives and no `.sifr-update.lock` remains; pass-2's trap addition does
  not change that surface, and the assertion still passes. ✓

## Carried to milestone 2 / milestone 4

- M2: eligibility-side canonicalization must match the installer-side
  `binary_path` canonicalization (full path, not just `pwd -P` on the
  parent). The `install_dir` asymmetry from pass 1 still warrants either
  symmetric canonicalization on the installer side or a documented
  asymmetry in `internal_docs/distribution_pipeline.md`.
- M2: when the runner becomes the first production emitter of
  `SIFR-BUILD-0901`, reconsider whether `{message}` is the right dedupe key
  for what will become a family of self-update diagnostics.
- M4: wire `self_update_install_receipt.schema.json` into the distribution
  validation suite as an executable schema check, not just a required-field
  comparison.

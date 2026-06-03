# Self-Update M2 — Review Pass 1

Reviewed branch `ad-hoc-self-update-m2` against `issues/ad-hoc-sifr-self-update.md` (milestone_self_update_2).

Scope reviewed:
- `crates/sifr/src/self_update_cli.rs` (new, 445 lines)
- `crates/sifr/src/self_update_metadata.rs` (new, 532 lines)
- `crates/sifr/src/self_update_receipt.rs` (extended in M2)
- `crates/sifr/src/cli_model_and_entrypoint.rs` (dispatch glue)
- `crates/sifr/src/main.rs` (module wiring)

## Findings

### Medium — `sifr self version` fails closed for unmanaged installs

`self_update_cli.rs:130-133` calls `discover_production_receipt` and returns `EXIT_USER_DIAGNOSTIC` if any of receipt missing / belongs to other executable / unsupported target. That means a Cargo-installed or Homebrew-installed user gets no version output at all from `sifr self version`. The phase contract (issues/ad-hoc-sifr-self-update.md:146-153) does not explicitly cover this case, and `sifr --version` already gives raw build version, so this is defensible — but worth a deliberate decision rather than implicit fall-through. If kept as-is, recommend mentioning it in milestone_self_update_5 docs.

Expected fix (if changed): in `cmd_version`, allow the unmanaged path to render a JSON/text response with `current_executable`, `current_version` from `SIFR_BUILD_VERSION`, `matches_receipt=false`, and other receipt-derived fields as `null`. This would require adding nullable variants to the rendered JSON, which changes the documented JSON shape — so this is a contract clarification, not a pure code fix. Probably correct to defer.

### Medium — Single diagnostic code reused for every self-update error

`self_update_receipt.rs:286-313` and `self_update_metadata.rs:339-344` both build diagnostics with `DiagnosticCode::SELF_UPDATE_UNMANAGED_RECEIPT` (`SIFR-BUILD-0901`, registry.rs:244). Every distinct failure case in the contract (issues/ad-hoc-sifr-self-update.md:308-326 — 18 enumerated cases including "stable channel gated", "metadata malformed", "downgrade requires --force", etc.) reports the same code. The contract says "Self-update diagnostics use `SIFR-BUILD-09xx` in this phase" (line 305), which is plural; the only differentiation today is the message string.

Each error message does carry remediation text inline (e.g. "use --channel alpha|beta", "requires --force", "re-run curl ... | sh"), so the user-facing remediations from contract lines 328-335 are present. The risk is downstream tooling (CI parsers, IDE integrations) cannot programmatically distinguish "receipt missing" from "downgrade refused".

Recommendation: split into at least the families called out in M2 scope — `SELF_UPDATE_UNMANAGED_RECEIPT`, `SELF_UPDATE_RECEIPT_MISMATCH`, `SELF_UPDATE_CHANNEL_GATED`, `SELF_UPDATE_FORCE_REQUIRED`, `SELF_UPDATE_METADATA_MALFORMED`, `SELF_UPDATE_METADATA_UNAVAILABLE`. Non-blocking if M5 docs explicitly say one code covers all cases this phase, but I'd prefer it split before merge.

### Medium — Missing hardlink eligibility test

Contract validation list (issues/ad-hoc-sifr-self-update.md:345-346):
> symlinked or hardlinked current-executable eligibility where the platform supports same-file metadata

`self_update_receipt.rs:595-620` covers the symlink case. There's no hardlink test. The underlying `same_metadata` function (line 195-201) compares `dev() && ino()`, which is correct for hardlinks by definition, so this is a test-only gap, not a behavior gap. Add a `(cfg(unix)) test` using `std::fs::hard_link` to lock the behavior in.

Expected fix: add a parallel test in the same module to `accepts_symlinked_current_executable_using_same_file_metadata` that uses `std::fs::hard_link(&binary, &hardlink_path)` and asserts `matches_receipt`.

### Low — Current executable canonicalization wording mismatch

Contract (issues/ad-hoc-sifr-self-update.md:222-225):
> - canonicalize the current executable path during self-update and reject eligibility if canonicalization fails,
> - on Unix, compare device and inode metadata after following symlinks; path-string equality alone is not enough,

`ReceiptDiscoveryEnv::production` (self_update_receipt.rs:42-53) uses `std::env::current_exe()` without an explicit `.canonicalize()` call. `validate_receipt_eligibility` then uses `same_file` → `same_metadata` (dev/ino) on Unix, which does follow symlinks via `fs::metadata`, so the effective check is correct on supported targets. On Windows (non-Unix `cfg`), `same_file` does canonicalize both sides at line 190-191, so the fallback also matches the contract.

The behavior is correct; only the literal "canonicalize the current executable path … reject eligibility if canonicalization fails" wording is unmet — there's no explicit `current_exe.canonicalize()?` call. If a non-Unix path ever reaches `validate_receipt_eligibility` without canonical paths, the dev/ino fallback at line 175 (`same_metadata(left, right).unwrap_or_else(|_| left == right)`) silently degrades to path-string equality. Currently this only matters on non-Unix, which isn't a supported target this phase (contract Non-Goals line 73 explicitly defers Windows), so non-blocking.

Recommendation: add an explicit canonicalization call (or document the dev/ino-based behavior as the canonicalization-equivalent in a code comment) so future Windows enablement doesn't silently weaken the check.

### Low — Channel-switch-without-force fetches metadata before failing

`cmd_update` (self_update_cli.rs:88-100) unconditionally fetches metadata for `TargetRequest::Channel(_)` even when the requested channel differs from the receipt channel and `--force` is absent — that failure path (`resolve_update_plan` line 281-288) is purely local and could be evaluated before network. Not a contract violation: the contract requires "missing or mismatched install receipts" and "unknown channels" to be rejected before network (line 116, 117), not channel-switch-needs-force. Just wastes one network round-trip for an always-failing dry-run.

Recommendation: move the "requested_channel != receipt_channel && !force" check ahead of `fetch_channel_metadata`. Non-blocking.

### Low — `resolve_update_plan` re-parses pre-validated receipt fields

`self_update_metadata.rs:254-255`:
```rust
let current_version = PreviewVersion::parse(current_version)?;
let receipt_channel = parse_channel(receipt_channel)?;
```

The receipt was already validated by `parse_install_receipt_json` + `validate_receipt_eligibility`. If for any reason a receipt holds a stable-looking version (the receipt parser doesn't enforce version shape — only channel allow-list), the user would see "stable-looking versions are disabled until Phase 39 … use --channel alpha|beta" — confusing wording for a receipt-state problem.

Recommendation either:
1. Add a `PreviewVersion::parse` call inside `parse_install_receipt_json` so receipt-level version invariants are enforced once, or
2. In `resolve_update_plan`, on parse failure re-wrap the error with a "receipt version is not a valid preview semver" diagnostic.

Non-blocking; edge case.

### Low — Metadata requires both `alpha` and `beta` present

`self_update_metadata.rs:212-216`:
```rust
if !parsed.contains_key("alpha") || !parsed.contains_key("beta") {
    return Err(self_update_diagnostic(
        "self-update metadata must contain alpha and beta channels",
    ));
}
```

The contract (issues/ad-hoc-sifr-self-update.md:263-269) lists what metadata must *reject*; it does not require both channels to be *present*. If release automation ever publishes a metadata file with only `beta` (e.g., a cycle with no alpha), an alpha-channel user would get a less-informative error than the targeted "requested channel is missing from metadata" they'd get from `resolve_channel`. Probably fine given release-plan invariants, but flag the assumption.

### Low — Manual JSON construction in `render_dry_run_json` and `render_version_json`

`self_update_cli.rs:218-275` builds JSON by `format!` with a `json_string` helper that runs each string through `serde_json::to_string`. Booleans use Rust `Display` (`true`/`false`, JSON-valid). Integer literal `1` is hardcoded. Safe today, but a future field of a non-string non-bool type would silently break the schema, and the deterministic field-ordering contract (line 144) is encoded only via the format string.

Recommendation: prefer a serde struct with `#[serde(rename_all = "snake_case")]` and explicit field order (use `IndexMap` or struct ordering). Functional today; refactor risk only.

### Informational — File-size guardrail

```
crates/sifr/src/self_update_cli.rs       445 ✓ <900
crates/sifr/src/self_update_metadata.rs  532 ✓ <900
crates/sifr/src/self_update_receipt.rs   621 ✓ <900
crates/sifr/src/cli_model_and_entrypoint.rs 854 ✓ <900 (still close to cap — contract line 277-281 noted this)
```

All within budget. Self-update logic correctly kept out of `cli_model_and_entrypoint.rs` (only the enum variant + `SelfArgs` import + 1-line dispatch).

### Informational — Module boundary compliance

Contract line 278-283:
- `self_update_cli.rs`: clap arg structs + dispatch + output formatting ✓
- `self_update_receipt.rs`: receipt schema, discovery, eligibility checks ✓
- `self_update_metadata.rs`: channel/version metadata parsing + target resolution ✓
- `self_update_runner.rs`: M3 scope, correctly absent

### Informational — Network ordering verified

Walking `cmd_update`:
1. Argument-combination validation — local (line 76-78)
2. `target_request` — parses `--channel`/`--version` strings, rejects unknown channels and stable/rc/stable-looking versions — local (line 80-83, contract line 116-117 ✓)
3. `discover_production_receipt` — receipt discovery + eligibility (canonicalize/same-file) — local (line 84-87, contract line 116 ✓)
4. `fetch_channel_metadata` — first network access — only for `ReceiptChannel`/`Channel(_)` requests (line 88-100)
5. `resolve_update_plan` — pure (line 101-110)
6. `render_dry_run` or stub error — local (line 112-123)

The contract's "missing or mismatched install receipts rejected before network" and "unknown channels rejected before network requests" both hold.

### Informational — Dry-run vs force rules

Contract line 118:
> Dry-run obeys the same force rules as a real update, so same-version reinstall, downgrade, or channel switch plans that require `--force` fail before output when `--force` is absent.

`resolve_update_plan` (self_update_metadata.rs:281-306) returns `Err` for unforced downgrades and unforced channel-switches before producing an `UpdatePlan`, and `render_dry_run` (self_update_cli.rs:112-116) only runs after a successful plan — so unforced downgrade/switch dry-runs error out as required. Same-version reinstall without force returns `UpdateAction::NoOp` (no error) with `would_run_installer: false` — also correct.

### Informational — `sifr self version --short` behavior

`render_version` (self_update_cli.rs:236-242) emits only `SIFR_BUILD_VERSION` for `Text + short`; rejection of `--short --format json` is in `version_args_diagnostic` (line 153-159) and tested at line 437-444. Matches contract line 154-156.

### Informational — Dry-run JSON snapshot

`dry_run_json_is_deterministic` (self_update_cli.rs:358-379) asserts the exact byte sequence including schema_version, field ordering, `requested_channel: null` rendering, `would_run_installer: true`, and `warnings: []`. Matches contract line 138-144.

What is NOT snapshot-tested at full fidelity:
- A `requested_channel: "beta"` (non-null) JSON output. Contract line 141 specifically says "absent-vs-null behavior is snapshot-tested" — only the null variant is asserted byte-exact.
- An `action: "no_op"` JSON output with `would_run_installer: false`.
- A `channel_switch` action JSON output.

Existing tests cover the *logic* (same_version_is_no_op, channel_switch_requires_force) but not the rendered JSON for those actions. Recommend adding one snapshot per action variant.

## Test coverage summary vs contract validation list (lines 339-353)

| Required case | Covered |
|---|---|
| receipt parsing schema-versioned shape | ✓ `parses_schema_versioned_receipt_shape` |
| schema rejection (empty/invalid/wrong types/unknown fields/unsupported version) | ✓ 5 tests |
| receipt discovery order | ✓ `discovers_manifest_dir_before_adjacent_manifest`, `discovers_default_home_manifest_only_for_default_binary` |
| current-executable mismatch rejection | ✓ `rejects_receipt_for_different_executable` |
| symlinked current-executable eligibility | ✓ `accepts_symlinked_current_executable_using_same_file_metadata` |
| hardlinked current-executable eligibility | ✗ **missing** |
| channel metadata parsing | ✓ `parses_channel_metadata` |
| preview semver validation | ✓ via parse tests |
| stable-looking version rejection | ✓ `rejects_stable_and_rc_versions` |
| rc channel & `-rc.N` rejection before Phase 39 | ✓ `rejects_rc_channel_before_phase_39`, `rejects_stable_and_rc_versions` |
| update-needed comparison | ✓ `same_version_is_no_op_without_force`, `same_version_reinstall_requires_force`, `downgrade_requires_force`, `channel_switch_requires_force`, `receipt_channel_request_uses_receipt_channel_metadata` |
| dry-run output text + JSON | ✓ JSON byte-exact; text only `contains` — see above |
| `--format` without `--dry-run` rejection | ✓ `update_rejects_json_format_without_dry_run_before_receipt_discovery` |
| `--short --format json` rejection | ✓ `version_rejects_short_json_before_receipt_discovery` |
| metadata stable rejection | ✓ `rejects_stable_metadata` |
| metadata unknown channel rejection | ✓ `rejects_unknown_metadata_channel` |

Integration tests listed in the contract (lines 355-368) are mostly M3 scope (installer delegation, install lock, environment plumbing). Items M2 should already cover from the integration list — "mismatched receipt fails before network access" and "missing receipt fails before network access" — are exercised at the unit level by `rejects_receipt_for_different_executable` and the `missing_receipt_diagnostic` discovery path; full network-fixture tests can land with M3.

## Verdict

No blocking correctness issues. The CLI command shape, receipt discovery order, eligibility via dev/ino on Unix, stable/rc gating, force rules, dry-run-no-mutation, deterministic JSON, invalid-combination rejection, and network-ordering all align with the contract.

Recommend addressing **before merge**:
1. **Add hardlink eligibility test** (contract validation list item).
2. **Decide on `sifr self version` for unmanaged installs** — keep as-is and document in M5, or relax to allow degraded output.

Recommend addressing **before M3** (non-blocking for M2):
3. Split diagnostic codes within `SIFR-BUILD-09xx` family — currently one code covers ~16 distinct cases.
4. Add explicit `current_exe.canonicalize()` call to satisfy the literal contract wording (behavior is already correct on supported targets).
5. Snapshot non-null `requested_channel` and other action variants in dry-run JSON.

Another review pass is **not required** if the two pre-merge items above are addressed — they are mechanical changes that don't alter architecture. The remaining items can be tracked as M3 prep.

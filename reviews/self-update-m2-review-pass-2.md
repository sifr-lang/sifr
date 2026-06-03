# Self-Update M2 — Review Pass 2

Reviewed branch `ad-hoc-self-update-m2` against `issues/ad-hoc-sifr-self-update.md` and the prior review at `reviews/self-update-m2-review-pass-1.md`.

Scope reviewed (working-tree only):
- `crates/sifr/src/self_update_receipt.rs` — eligibility canonicalization, discovery wiring, new hardlink test, `install_dir` vs `binary_path` cross-check.
- `crates/sifr/src/self_update_cli.rs` — argument-combination tests rewritten to inspect diagnostics directly, new dry-run JSON tests for `no_op` and `channel_switch` actions.
- `crates/sifr/src/self_update_metadata.rs` — unchanged this pass.
- `crates/sifr/src/cli_model_and_entrypoint.rs`, `crates/sifr/src/main.rs`, `crates/sifr/src/diagnostic_rendering_and_run.rs` — visibility plumbing and `mod` wiring.
- `issues/ad-hoc-sifr-self-update.md` — added M2 decision on `sifr self version` for unmanaged installs and execution-status block.
- `internal_docs/typescript_go_architecture_transfer_m1_guardrails.md` — new inventory row for self-update receipt reads.

## Pass-1 pre-merge items — verification

### Pass-1 medium #1 — `sifr self version` for unmanaged installs
**Addressed via contract update.** `issues/ad-hoc-sifr-self-update.md:155` now records the deliberate M2 decision that `sifr self version` requires the same managed install receipt as `sifr self update`, and points unmanaged users to `sifr --version` until M5 troubleshooting docs land. `cmd_version` (`self_update_cli.rs:126-137`) correctly goes through `discover_production_receipt` first, so behavior matches the documented decision. Fall-through to `sifr --version` is intentional and now contractual.

### Pass-1 medium #3 — Hardlink eligibility test
**Addressed.** `self_update_receipt.rs:625-651` adds `accepts_hardlinked_current_executable_using_same_file_metadata`, gated on `cfg(unix)`, which constructs a hard link via `std::fs::hard_link`, writes a receipt pointing to the original binary, and asserts `matches_receipt`. Symmetric with the existing symlink test at `self_update_receipt.rs:597-622`. Contract validation list item (issues/ad-hoc-sifr-self-update.md:347) now covered.

### Pass-1 low #4 — Explicit `current_exe` canonicalization
**Addressed.** `ReceiptDiscoveryEnv::production` (`self_update_receipt.rs:42-58`) now calls `std::env::current_exe()?.canonicalize()?` and surfaces a structured diagnostic on either failure. Wording matches the contract literally ("reject eligibility if canonicalization fails", issue line 225). The Unix dev/ino-based `same_metadata` path remains the actual identity check; canonicalization is the additional fail-closed gate before that comparison.

### Pass-1 informational — Dry-run JSON variant coverage
**Addressed.** `self_update_cli.rs:409-430` adds two tests:
- `dry_run_json_no_op_has_false_installer_flag` — locks `action: "no_op"`, `force: false`, `would_run_installer: false`, `requested_channel: null`.
- `dry_run_json_channel_switch_renders_requested_channel` — locks `target_version: "0.1.0-alpha.2"`, `requested_channel: "alpha"` (non-null), `resolved_channel: "alpha"`, `action: "channel_switch"`, `force: true`.

These exercise the absent-vs-null requested_channel rendering and the `would_run_installer == false` no-op case that the contract calls out at line 142-143. Note: only the `update` variant is asserted byte-exact (`dry_run_json_is_deterministic` at line 378-399); the no-op and channel-switch variants use `contains` substring matches and so are less strict about field ordering or whitespace. See residual gaps below.

### User-stated change — Argument-combination tests inspect diagnostics directly
**Confirmed.** `self_update_cli.rs:447-487` now calls `update_args_diagnostic` and `version_args_diagnostic` directly and asserts on `diagnostic.message`. Prior approach of routing through `cmd_*` and capturing stderr is gone. This isolates the test from stderr side effects and tightens the assertion to the exact diagnostic produced.

## Pass-2-specific observations

### Non-blocking — New `install_dir` / `binary_path` containment check has no negative test

`validate_receipt_eligibility` (`self_update_receipt.rs:108-148`) is new since pass-1 and adds:
1. `canonicalize_for_receipt(install_dir)` and `canonicalize_for_receipt(binary_path)` — fail closed when either path does not resolve.
2. `paths_same_after_canonicalization(install_dir, binary_path.parent())` — reject receipts whose `binary_path` does not live inside `install_dir`.

Behavior is consistent with the contract's eligibility intent (issue line 222-230), but the failure path is not covered by a test. A receipt with `install_dir: /opt/sifr` and `binary_path: /usr/local/bin/sifr` would never trip the existing test suite. Recommend adding a `rejects_receipt_with_binary_outside_install_dir` test for parity with `rejects_receipt_for_different_executable`. Non-blocking — the positive path is exercised by every other discovery test.

The unreachable `if !receipt_path.is_file()` after canonicalization (`self_update_receipt.rs:148-153`) is dead defensively-coded code (the receipt was already `is_file` checked in `discover_receipt_path`), but it is harmless — leave or remove at maintainer preference.

### Non-blocking — `unmanaged_receipt_diagnostic` remediation is misaligned for `current_exe` / canonicalize errors

`ReceiptDiscoveryEnv::production` reuses `unmanaged_receipt_diagnostic` for failures of `current_exe()` and `canonicalize()`. The diagnostic carries a `help: "self-update is available only for official standalone installs ..."` and the unmanaged-receipt note pointing the user at `curl -LsSf https://sifr.sh/install | sh`, which is the wrong remediation for "kernel could not tell us our own path." Same diagnostic-code-overload concern flagged in pass-1 medium #2 — non-blocking for this milestone, but the case list grows.

### Non-blocking — Channel-switch force check still runs after metadata fetch

Pass-1 low #5 remains unchanged. `cmd_update` (`self_update_cli.rs:88-110`) still fetches metadata for `Channel(_)` before `resolve_update_plan` evaluates `requested_channel != receipt_channel && !force`. The contract's "before network" gates (issue line 116-117) cover only receipt-missing/mismatched and unknown-channel cases, so this is still spec-compliant. Wastes one round-trip on a flag-only failure path.

### Informational — File-size guardrail

```
crates/sifr/src/self_update_cli.rs        488 ✓ <900
crates/sifr/src/self_update_metadata.rs   532 ✓ <900
crates/sifr/src/self_update_receipt.rs    652 ✓ <900
crates/sifr/src/cli_model_and_entrypoint.rs 854 ✓ <900
```

All within budget. `cli_model_and_entrypoint.rs` is still close to the cap; the M2 dispatch glue added only ~10 lines.

### Informational — Module wiring change

`main.rs` flipped `self_update_receipt` from `#[cfg(test)]` to unconditional and added `self_update_cli` / `self_update_metadata`. Visibility on `EXIT_SUCCESS`, `EXIT_USER_DIAGNOSTIC`, `EXIT_USAGE_OR_CONFIG`, `diagnostic_with_code`, and `render_diagnostics` widened from `pub(super)` to `pub(crate)` to support the new sibling modules. Tightly scoped to the new dependencies; nothing leaks outside the crate.

### Informational — Diagnostic-format consistency across cmd_self error paths

`cmd_self` (`self_update_cli.rs:68-73`) threads `diagnostic_format` through to both `cmd_update` and `cmd_version`. All error-rendering helpers (`render_usage_diagnostic`, `render_user_error`) honor the format. The dry-run text output path (`render_dry_run_text`) prints to stdout via `writeln!(io::stdout(), ...)` rather than through any structured renderer, which is correct — the dry-run "text" format is human-readable plan output, not a diagnostic.

## Residual test gaps and risks

1. **No negative test for receipt `binary_path` outside `install_dir`** — see pass-2 observation above.
2. **`current_exe` / `canonicalize` failure path in `production()` is not unit-tested** — `production()` reads real process state and is not parameterizable; this is inherent rather than fixable in-module. Behavior is exercised implicitly by integration paths.
3. **Non-null dry-run JSON variants are tested with `contains`, not byte-exact** — pass-1 informational about snapshotting non-null `requested_channel` is partially addressed (the field now appears in a dedicated test) but field ordering and whitespace for the `channel_switch` and `no_op` variants are not locked. If determinism for those variants matters as much as the contract suggests at line 144 ("field names, field ordering, field types, warning ordering, and absent-vs-null behavior are snapshot-tested"), promote them to byte-exact assertions.
4. **Pass-1 non-pre-merge items not yet acted on** — diagnostic-code splitting within `SIFR-BUILD-09xx`, `resolve_update_plan` re-parsing pre-validated receipt fields, both-channels-required metadata invariant, manual JSON formatting. All explicitly tracked as M3-or-later by pass-1 and remain non-blocking for M2.
5. **`fetch_channel_metadata` uses `Command::new("curl")`** — this was already the case in pass-1 but wasn't explicitly called out as a residual. The contract's TLS requirement applies to the runner (M3) but is satisfied transitively by curl's default verification here. Worth flagging if a Rust-native HTTP client is planned, since this shape would need to change.

## Validation status

The user reports:
- `cargo fmt --check`, `cargo test -p sifr -- self_update`, `cargo clippy --workspace -- -D warnings` — passing focused after pass-2 edits.
- `scripts/run_distribution_validation.sh`, `python3 scripts/check_file_size_guardrails.py`, `python3 scripts/check_hir_maintainability_guardrails.py`, `scripts/run_all_tests.sh --profile quick` — passed with advisory-only warnings (warm wall-time budget exceeded; e2e group skew high) at the prior pre-pass-2 checkpoint.

I did not re-run validation as part of this pass. The diff since the previous validation is small (one test added in `self_update_receipt.rs`, ~14 lines in `ReceiptDiscoveryEnv::production`, two tests in `self_update_cli.rs`, doc-only changes elsewhere) and is unlikely to perturb timing or guardrails. If `cargo test -p sifr -- self_update` passes locally including the new hardlink test, pass-2 validation is sufficient.

## Verdict

**SATISFIED.**

Both pass-1 pre-merge items are addressed:
1. Hardlink eligibility test landed at `self_update_receipt.rs:625-651`.
2. `sifr self version` behavior for unmanaged installs is now a documented M2 contract decision at `issues/ad-hoc-sifr-self-update.md:155`.

Additional pass-1 low/informational items were addressed beyond the minimum (`current_exe` canonicalization, non-null dry-run JSON variant coverage, diagnostic-inspecting argument tests). New `install_dir` containment check is a defensible safety addition.

Remaining items are all non-blocking and align with the pass-1 disposition for M3+. No further M2 review pass required; the items in "Residual test gaps and risks" can be folded into M3 review or addressed opportunistically.

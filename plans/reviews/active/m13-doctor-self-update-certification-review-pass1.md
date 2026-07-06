## M13 Wave 2 Review — Doctor and Self-Update Certification

**Verdict:** No blockers. Wave 2 is satisfactory for PR readiness.

The `sifr doctor` implementation is honestly scoped, the installed-smoke additions actually exercise the receipt/version/dry-run pairing offline, the broken-sysroot snapshot is meaningful, and the docs/manifest updates match what the code and runner now do. Local validation ran to completion in the sysroot_release area (109s elapsed).

Non-blocking suggestions, ordered by likely benefit:

1. **`validate_self_update_dry_run_json` does not assert `binary_path` pairing** — `verification/areas/sysroot_release/runner.py:565-575`. It checks `sysroot_path` resolves to `install_root` but not that dry-run JSON's `binary_path` resolves to `install_root/bin/sifr`. The pairing is already enforced by `discover_install_receipt` and covered explicitly in `validate_self_version_json`, so pairing IS proven end-to-end — but tightening the dry-run check for parity would remove that indirection. Cheap follow-up.

2. **`write_install_receipt` swallows a missing `sysroot_content_sha256`** — `verification/areas/sysroot_release/runner.py:506`. `str(sysroot_payload.get("sysroot_content_sha256"))` yields the literal `"None"` if the field disappears from `--print sysroot --json`; the failure then surfaces late as an unhelpful `is_sha256_hex` diagnostic from `self_update_receipt.rs`. A defensive assertion in `validate_sysroot_json` (which is now returning the payload anyway) would fail closer to the cause.

3. **`cli_model_and_entrypoint.rs` is at 881 / 900 lines** — `crates/sifr/src/cli_model_and_entrypoint.rs`. Wave 2 added 9 lines. Still within cap, but the next CLI addition in that file may push it over the guardrail; consider peeling off the `Commands` enum or the dispatch table into its own module before the next wave.

4. **JSON-mode doctor error branch is uncovered** — `crates/sifr/src/sysroot_cli.rs:66-88`. The broken-sysroot smoke exercises only `broken_sifr doctor` (text mode). The `doctor --json` error emitter (with `error_kind`, `attempted_sysroot`, `asset_path` fields) is untested in the release suite. Adding a `broken_sifr doctor --json` step and asserting `status == "error"` plus `asset_path` would close that gap and pin the JSON schema.

5. **`cmd_doctor` and `print_sysroot` share resolve→render structure** — `crates/sifr/src/sysroot_cli.rs:20-90` vs `92-138`. Not a duplicate resolution path (both call the single `sifr_sysroot::resolve_sysroot(None)` entrypoint, so wave 2 does not introduce a second sysroot resolver), but the render layer is duplicated. Extractable later if a third caller lands; not worth churn now.

6. **UX inconsistency: `sifr doctor --json` bool vs `sifr self ... --format {text,json}`** — `crates/sifr/src/cli_model_and_entrypoint.rs:123-127`. Two flag shapes for the same "give me JSON" idea. Mirrors the pre-existing `sifr --json --print sysroot` bool style, so wave 2 didn't create the inconsistency, but future harmonization would be nice.

7. **Home-path leakage scan omits the new snapshots** — `verification/areas/sysroot_release/runner.py:246-258`. The repo-path scan (line 224-243) covers the five new files; the home-path scan is limited to archive + emit. Because install_root is a `tempfile.TemporaryDirectory` outside `$HOME`, doctor and self-update outputs would only leak home paths if the compiler baked them at build time — which the emit + archive scan already covers. Cosmetic parity only.

**Answers to the specific review questions**

1. Yes — `cmd_doctor` goes through the same `sifr_sysroot::resolve_sysroot(None)` entrypoint as `--print sysroot`; `--sysroot` is applied globally via `set_process_sysroot_override`, so no second resolution path.
2. Yes — every check reported `ok` in the JSON `checks` array corresponds to an assertion that `layout.rs::validate` actually enforces (manifest read, stdlib public/private source dirs, runtime and stdlib crate manifests, cargo lock, vendor).
3. Yes — `--dry-run --version 0.1.0-beta.1301` takes the `TargetRequest::Version` path in `self_update_cli.rs:184-189`, which `cmd_update` explicitly does NOT fetch metadata for (`self_update_cli.rs:89-101`). Receipt discovery + version validation both assert `sysroot_path`/`binary_path` pairing.
4. Yes — the broken scenario deletes exactly the file whose absence trips `require_file(...runtime_crate_manifest...)` before validate_workspace_manifest runs; the assertion pins both `missing or invalid asset` and `sifr_runtime/Cargo.toml`. Leakage scanning of the new snapshots is meaningful because those outputs contain resolved paths generated at runtime.
5. Yes — the architecture doc, issue plan, and manifest description all reflect what the code and runner now do; no aspirational claims.
6. No file-size violations. The `cli_model_and_entrypoint.rs` proximity to 900 is the only maintainability signal worth watching.

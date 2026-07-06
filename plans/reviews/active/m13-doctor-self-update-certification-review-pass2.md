I inspected the delta against pass 1. All three tightenings are correctly implemented and I have no remaining blockers for PR readiness.

## Confirmation of the three tightenings

**1. Dry-run `binary_path` pairing** — `verification/areas/sysroot_release/runner.py:581-582`
```python
if Path(str(payload.get("binary_path"))).resolve() != (install_root / "bin" / "sifr").resolve():
    raise CertificationError("self update dry-run json did not preserve the installed binary path")
```
Symmetric with the existing `sysroot_path` assertion above it and with `validate_self_version_json`. Good.

**2. Early `sysroot_content_sha256` assertion** — `verification/areas/sysroot_release/runner.py:493-494`, called before `write_install_receipt` at `runner.py:192-193`
```python
if not isinstance(payload.get("sysroot_content_sha256"), str):
    raise CertificationError("installed sysroot json omitted sysroot_content_sha256")
```
Fails with a specific message at the point the field is missing, instead of surfacing later as an `is_sha256_hex` diagnostic. Good.

**3. Broken doctor `--json` snapshot + leakage scan coverage** — `verification/areas/sysroot_release/runner.py:627-631, 654-663`
- Runs `broken_sifr doctor --json`, writes `installed-doctor-broken.json` (stdout+stderr, `runner.py:628`)
- Asserts exit code ≠ 0, `status == "error"`, and `"sifr_runtime/Cargo.toml" in asset_path`
- Added to the repo path-leakage `check_no_path_leakage.py` invocation at `runner.py:239`

The CLI's JSON-error branch it exercises is `crates/sifr/src/sysroot_cli.rs:66-80`, which serializes `error.asset_path` — so the runner assertion pins the field that closes the gap flagged in pass 1.

## Additional checks

- **File-size guardrail**: `cli_model_and_entrypoint.rs` is still 881 lines (no change from wave 2's original 9-line addition); `sysroot_cli.rs` is 139; `runner.py` grew to 784. All under 900.
- **JSON parse safety**: `broken_json.stdout` is the sole content of the JSON pretty-print block; `render_diagnostics` writes to stderr, which is why concatenating stdout+stderr to file works and stdout alone parses cleanly.
- **Ordering**: `validate_sysroot_json` now returns the payload (`runner.py:482,495`) and `write_install_receipt` consumes it downstream, so tightening 2 provably fires before the receipt is written.
- **Validation record**: your reported run (elapsed_ms=104168, 10 artifacts scanned, home scan passed) is consistent with the six new snapshot paths joining the four pre-existing ones in the repo-scan list at `runner.py:232-241`.

## Non-blocking observations (do not gate the PR)

- `write_install_receipt` at `runner.py:511` still wraps the sysroot sha in `str(...)`; now that the validator guarantees it's a string, the `str()` is a no-op. Purely cosmetic.
- Pass 1 items 3 (881/900 proximity), 4 was closed by tightening 3, 5, 6, and 7 remain as follow-up polish, not wave 2 concerns.

**Verdict:** Ready for PR. No blockers.

## Wave 0 review — pass 3

### Pass-2 blocker

**RESOLVED.** `verification/runner/e2e/run_e2e_pass.sh:198` now reads:

```
cargo test --locked -p sifr --test e2e test_e2e_pass -- --nocapture
```

`--locked` is present before the `--` separator, so cargo will fail rather than silently regenerate `Cargo.lock`. Combined with `CARGO_NET_OFFLINE=true` inherited from `profile_runner.py:128-130`, the e2e pass step is now fully hermetic on the profile-execution path.

### Hermetic-contract sweep (re-verified)

All cargo invocations reached from any profile step include `--locked`:

- `profile_runner.cargo_command` (line 90) — injects `--locked` before any `--` separator. Used by every crate test in `run_crate_tests` (lines 343-369).
- `area_adapter.py:233` — validation contract matrix.
- `area_adapter.py:450` — sifr-variant runner (baseline / contract cases).
- `audit_fixtures.py:143` — fixture audit.
- `hardening/core.py:155` — hardening suites.
- `run_e2e_pass.sh:198` — e2e pass (the pass-2 fix).
- `doctor.py:102` — `cargo metadata --locked`.

### Remaining blocking Wave 0 issues

**None.** Pass-2's five-item blocker list is now closed in full.

### Non-blocking carryovers (unchanged from pass-2)

- `scripts/check_codegen_rawcode_gate.sh:15` and `verification/areas/performance/tools/ci_e2e_throughput.sh:18` still invoke `cargo test` without `--locked`. Neither is referenced from any profile path (verified by grep — no callers in `verification/`, `scripts/`, or `.github/`), so the contract is not breached today. If either gets wired into a profile later, it will need `--locked` at that point.
- `area_adapter._contract_matrix_env()` (line 597) copies `os.environ` for direct `areas run` invocations. When `ProfileRunner.__init__` hasn't run, `CARGO_NET_OFFLINE` isn't set — direct area runs aren't covered by the offline half of the contract. Pass-2 already flagged this as worth centralizing; still non-blocking because the profile gate sets it correctly.
- `distribution_release_full` / `distribution_release_representative` still share the same `merge_suite`; cosmetic, not gating.

### Bottom line

Pass-2 blocker is resolved. No new blockers introduced. Wave 0 can close on the verification track.

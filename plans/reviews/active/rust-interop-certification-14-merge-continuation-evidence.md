# Rust Interop Certification 14 Merge-Continuation Evidence

Date: 2026-07-30

Reviewed source head:
`ef34d2267` (the only change after the fully tested `017c1df41` source was the
merge-evidence ledger entry).

The authoritative merge profile is fail-fast and stopped at the governed
performance comparison. The unmodified profile runner then executed every
later merge step successfully. This artifact preserves compact reruns for the
three results whose original console output had no unique durable report.

## Project Validation Matrices

Command:

```text
uv run --project verification --locked python -m sifr_verify areas run \
  --area project_workspace \
  --suite frontend_mode_parity \
  --suite project_graph_isolation \
  --result-json target/verification/areas/rust-interop-cert14-project-validation-results.json
```

Result:

```text
Frontend mode parity matrix: PASS
project graph isolation regression matrix: PASS
project workspace verification ok: variants=2, failures=0,
blocking_failures=0, non_blocking_failures=0
```

The two suites executed seven rows: positive project, negative project type
error, single-file layout, multi-file import closure and test, reachable parse
error rules, cycle diagnostic stability, and parallel invocation isolation.
The uniquely named result JSON SHA-256 was
`a5d8c3a8b2a364ff4ed6825c793434b7ef6b9d7e76159895911039b47cb9d81a`.

## Package Management Offline Merge Smoke

Command:

```text
uv run --project verification --locked python -m sifr_verify areas run \
  --area package_management \
  --suite offline-merge-smoke
```

Result:

```text
offline package merge smoke ok
offline package merge smoke self-test: PASS
package management verification ok: variants=2, failures=0,
blocking_failures=0, non_blocking_failures=0
```

This is the package-management step after the fail-fast performance comparison
in the emitted merge-profile plan.

## CLI Generated Builds

Command:

```text
cargo test -q -p sifr --bin sifr -- --ignored --test-threads=1
```

Result:

```text
running 6 tests
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured;
114 filtered out; finished in 164.08s
```

The output included successful frozen, locked, and offline release builds with
matching values, plus the expected missing-lockfile, stale-version, checksum,
dependency-source, and feature-selection rejection diagnostics.

## Driver Generated Builds

Command:

```text
cargo test -q -p sifr_driver --lib -- --ignored --test-threads=1
```

Result:

```text
running 65 tests
.................................................................
test result: ok. 65 passed; 0 failed; 0 ignored; 0 measured;
450 filtered out; finished in 1794.34s
```

## Full Merge E2E

Command:

```text
bash verification/runner/e2e/run_e2e_pass.sh \
  --profile merge \
  --sifr-jobs 4 \
  --rust-jobs 3 \
  --run-jobs 3 \
  --cargo-build-jobs 1 \
  --max-group-fixtures 12
```

Result:

```text
[sifr-e2e] timing: compile=27578ms plan=106ms build=48ms build-sum=13ms
run=3018ms cache_hits=178/178
[sifr-e2e] group_stats: groups=178 largest_group_fixtures=12
median_group_fixtures=2
[sifr-e2e] report_signature=5e45a6a7b96f2688
678 pass tests completed (678 passed, 0 failed)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured;
36 filtered out; finished in 30.99s
```

The warmed rerun reproduced the exact report signature from the earlier cold
merge continuation.

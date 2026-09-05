I've verified the merged facts against the repo state and the M6 acceptance criteria in the milestone doc.

## Verification against M6 acceptance criteria

All four acceptance criteria are satisfied and corroborated by the on-disk state:

1. **Async resources have a checked declaration/lifecycle model** — `async_runtime_core` row exists in the compatibility matrix (line 221), covering async declaration contracts, async-close lifecycle, current-thread affinity, and hidden-blocking rejection. M6b added the accept/reject contract tests including `close=async_close` with sync-close-only rejection. ✓
2. **Cancellation/drop deterministic and documented** — matrix notes record runtime task cancellation/drop in executable Rust tests; M6b added Handle async close/double-close plus cancelled-join determinism coverage. ✓
3. **Certification distinguishes sync vs async resource evidence** — sync `opaque_resource_core` (line 122) and async `async_runtime_core` (line 221) are separate rows, both in the sysroot gate allowlist. ✓
4. **sleep/monotonic no longer require compiler-native dispatch** — M6a deleted the compiler time registry/fallback signatures and routed both behind `stdlib/_sifr/time.sifr` + `sifr_stdlib::time`; `_sifr.time` is `closing` with `certification_rows = ["async_runtime_core"]` (line 160), repointed off the future-owned `async_runtime_reqwest`. The M1d guard rejecting `closing` rows with a surviving fallback table means validation passing confirms dispatch is truly gone. ✓

Scope isolation holds: `async_runtime_reqwest` remains `future-owned-by-separate-phase` (line 208) for reqwest loopback only, correctly out of M6 scope. The `_sifr.time` `closing` row is legitimate at milestone level (final deletion is M13's job).

Validation evidence is clean: create-pr passed, 129/129 E2E, no advisories, both sub-PR reviews READY (M6a round1, M6b round7).

Non-blocking note (docs bookkeeping, expected to land with this closeout, not a gate): the Implementation Status table (line 98) still lists only M6a and marks M6 "in progress" — flipping to "merged" and adding the M6b `PR #2875 · sha=f3ce312` row is the closeout doc update itself.

No blocking findings.

READY

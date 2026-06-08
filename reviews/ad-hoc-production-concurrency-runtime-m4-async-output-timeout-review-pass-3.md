RESULT: PASS

All four required ledger values in `issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md:902` verified against `target/validation_lane_reports/create-pr.latest.json` and `.log`:

- `real_seconds: 1026.02` → `1026.02s` ✓
- `cache_hits: 20`, `group_count: 26` → `cache_hits=20/26` ✓
- log line 1494 `report_signature=9212e77abfa82acc` ✓
- log line 1495 `101 passed, 0 failed` ✓
- advisories (warm wall-time + warm-cache hit rate) match JSON ✓

Notes:
- Pass-2 blocker fix is correct: previous `513.27s`/`cache_hits=23/26` replaced with `1026.02s`/`cache_hits=20/26` and the line is now prefixed `PASS after merging current origin/main`, which matches the post-merge re-run rationale.
- Other modified files (`issues/ad-hoc-production-network-http-platform-substrate-execution.md`, `…-substrate.md`) are unrelated WIP for a different (network/http) workstream and not part of PR #2361's M4 async output timeout scope; they don't affect this blocker.

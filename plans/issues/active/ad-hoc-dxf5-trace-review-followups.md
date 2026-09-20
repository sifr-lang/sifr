# DXF.5 trace review follow-ups

Status: deferred observations; no DXF.5 blocker or added implementation scope.

Source: [scoped Opus review of PR #3883](https://github.com/sifr-lang/sifr/pull/3883#issuecomment-5749892882),
candidate `e2a12e316beeaa950249b18e456b614fc77cd4df`, verdict **SATISFIED**.
See the [merged record](ad-hoc-compiler-dx-followup-execution.md#dxf5-merged-record--2026-09-20)
for validation and immutable external evidence.

- Suggestion, test-harness owner: the Unix child asserts libtest stdout includes
  `1 passed`. Retain this as an intentional nonzero-selection guard for now;
  consider a structured child completion assertion if libtest output changes.
- Suggestion, trace-schema owner: final bounding serialization is outside
  `trace_overhead_us_before_final_write`. The implementation comment explicitly
  records the snapshot boundary; retain that explanation if fields are renamed.
  No claim that it measures complete instrumentation overhead is added.
- Pathological-counter observation, trace-sink owner: checked JSON drop-counter
  increment errors if a synthetic seed plus removals exceeds `u64::MAX`.
  The retained-record cap does not itself mathematically bound the lifetime
  dropped count. No practical overflowing invocation was evidenced, and the
  widest supported count is tested. Do not add saturation or a fallback as part
  of this closed item; any expanded numeric contract needs separate scope.

No code change, repeated validation, or additional external review is required
for this record-only disposition.

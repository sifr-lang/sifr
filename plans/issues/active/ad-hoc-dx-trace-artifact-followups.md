# Optional trace-artifact follow-ups

status: partially resolved by DXF.5; remaining optional scope deferred
owner: CLI observability

Source: [scoped Opus review of #3871](https://github.com/sifr-lang/sifr/pull/3871#issuecomment-5748783307),
candidate `3d0f42b2ef3c83d279a45a0775b941920f4ae5e3`; verdict **SATISFIED**,
no blocking findings. These are retained separately to prevent scope drift.

- Resolved by [DXF.5 / PR #3883](https://github.com/sifr-lang/sifr/pull/3883):
  final timing serialization reapplies bounded optional-record dropping with
  truthful counters. Exact boundary/width tests and installed acceptance pass.
  See the [merged evidence record](ad-hoc-compiler-dx-followup-execution.md#dxf5-merged-record--2026-09-20).
- Suggestion: distinguish bare CLI, `--explain` and `--print` from the current
  safe `metadata` command label.
- Resolved by DXF.5 / PR #3883: owned Unix directories are created privately
  with mode 0700 (subject to further restrictive umask bits), files remain 0600,
  and unrelated existing destinations are unchanged. Permissive-umask testing
  runs in a child process on Unix.
- Pre-existing scope: frontend `WorkspaceTraceLog` projections are available
  through `sifr trace`, while build/check use their existing owner reports.
  Expanding frontend-event coverage requires a separately approved owner change.

Further implementation is not authorized by this record. The original whole-phase review's
stale DX.16 status-prose finding stays with the documentation closer, not here.

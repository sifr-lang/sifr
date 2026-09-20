# Optional trace-artifact follow-ups

status: deferred; separate scope required; not a DX.16 closure blocker
owner: CLI observability

Source: [scoped Opus review of #3871](https://github.com/sifr-lang/sifr/pull/3871#issuecomment-5748783307),
candidate `3d0f42b2ef3c83d279a45a0775b941920f4ae5e3`; verdict **SATISFIED**,
no blocking findings. These are retained separately to prevent scope drift.

- Suggestion: reserve a small byte margin before final timing-field serialization
  so wider timing numbers at the exact 32 KiB boundary do not cause a truthful
  sink-finalization error. The evidenced runs do not encounter this boundary.
- Suggestion: distinguish bare CLI, `--explain` and `--print` from the current
  safe `metadata` command label.
- Suggestion: consider a private directory mode in addition to the current 0600
  artifact-file mode; the directory currently uses the process umask.
- Pre-existing scope: frontend `WorkspaceTraceLog` projections are available
  through `sifr trace`, while build/check use their existing owner reports.
  Expanding frontend-event coverage requires a separately approved owner change.

No implementation is authorized by this record. The original whole-phase review's
stale DX.16 status-prose finding stays with the documentation closer, not here.

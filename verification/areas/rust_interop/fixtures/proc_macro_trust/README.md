# proc_macro_trust

This fixture family tracks proc-macro and codegen trust evidence for
`serde_derive` and `prost-build`.

- Positive evidence: `trusted_proc_macro` remains planned for a fixture proving
  trusted proc-macro and deterministic codegen execution are recorded in the
  build plan and cache key.
- Negative evidence: `untrusted_proc_macro_rejected_pre_execution` remains
  planned for a fixture proving untrusted build-time code is rejected before it
  runs.
- Compatibility category: `future-owned-by-separate-phase`. The trust model is
  documented and wired, but representative proc-macro ecosystem certification is
  not listed as verified support.

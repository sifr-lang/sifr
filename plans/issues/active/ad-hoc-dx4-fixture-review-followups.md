# DX.4 fixture and inventory review follow-ups

status: open; separate future work, not DX.4 blockers
source: [merged DX.4](ad-hoc-compiler-dx-and-toolchain-reuse.md)
implementation: [PR #3846](https://github.com/sifr-lang/sifr/pull/3846)

These observations do not authorize implementation in the completed DX.4 session.
Final scoped review is [SATISFIED, no blockers](https://github.com/sifr-lang/sifr/pull/3846#issuecomment-5712346406).
Initial expected-exit and input-coverage findings were repaired, including the
later consumed-doc/config and root-README observations. Their original review
classifications remain preserved.

- Derive the remaining compatibility/taxonomy source-sweep consumer registrations
  from area manifests. Currently their paths are explicit constants; a renamed
  or moved checker can fail the existence guard and silently drop that source.
  Keep actual consumed input coverage and phase-record exclusion while reducing
  this hand-maintained registration list.
- Isolate transitive module imports during declared-input discovery. Temporary
  sys.path and synthetic module registration are restored, but imported plain
  module names remain cached; future duplicate names could collide.
- Measure and reduce per-area inventory cost without unsafe evidence reuse.
  The 52k-input inventory and declaration imports cost roughly 4.3 seconds warm
  in the preceding review; this is an observation, not a qualified performance
  result. Focused single-case runs currently pay this whole cost.
- Make future non-Python guardrail-entrypoint errors explicit, and distinguish
  missing directory declarations from missing file declarations if they acquire
  consumers requiring that distinction. Preserve clear errors for unmerged
  indexes and initialized submodules lacking HEAD.
- Require nonempty native selections and write evidence/report output after the
  per-entry loop. Current profiles always select known nonempty companion IDs;
  the empty-selection gap is unreachable through current profiles.
- Qualify output headroom for the 1 MiB per-stream bound on Cargo JSON and verbose
  validation-suite output. Truncation already fails loudly; no silent pass exists.
- Include suppressed non-early guardrails in invalid-inventory blocked-step
  records, alongside already-reported setup, area and toolchain dependents.
- Integrate generated-code-quality area temporary roots with owner-scoped,
  pressure-based cleanup. Never replace this with a target-size-only threshold.
- Correct cosmetic literal backslash-n error text and stale CompletedProcess
  annotation, remove duplicate synthetic input writes, and consider a local
  absolute-path synthetic declaration test in addition to the existing real-root
  assertion. These have no current behavioral impact.

All raw responses, earlier failed evidence and scoped adjudications remain under
yaser5@yaser.tailaa73b4.ts.net:/home/yaser5/projects/sifr/dx4-evidence/.
The final exact-candidate response is in
`fa8083ccfea3ec1b8d0fe8a5290c9a3d21efe834/review-response.md`.

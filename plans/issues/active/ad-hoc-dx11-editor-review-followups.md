# DX.11 editor review followups

Status: open followup work; not a blocker for the scoped DX.11 closure.

Source: [initial scoped Opus review of dd282f9d](https://github.com/sifr-lang/sifr/pull/3860#issuecomment-5726155969).
The [final exact-candidate review](https://github.com/sifr-lang/sifr/pull/3860#issuecomment-5726309175) is SATISFIED.
The one blocking regression (workspace progress emitted for incremental edits)
was corrected and qualified separately. These suggestions do not expand DX.11.

| ID | Owning boundary | Observation and next bounded action |
| --- | --- | --- |
| F1 | DX.12–DX.14 editor performance qualification | Change reconciliation republishes open documents without same-version deduplication. Measure settled edits with multiple push-diagnostics documents and work-done capability before deciding on dependency/version-aware scheduling. Existing large-session budgets use diagnostics off and do not qualify this push cost. Preserve those original scopes. |
| F2 | Editor syntax request UX | Folding/selection on transient parse errors currently terminate with an internal error; the former semantic path returned empty facts. Evaluate graceful empty results as a separate compatibility/UX change with malformed-buffer tests. The DX.11 terminal-response and syntax-without-metadata contracts are satisfied. |
| F3 | Compiler context API contract | Explicit re-resolution currently constructs a normal context without carrying explicit metadata overrides or ResolvedSysroot. The present LSP cannot supply either. If that caller surface is added, define and test whether refresh preserves or rejects such overrides. |
| F4 | DX.12–DX.14 active-state memory baseline | Active/peak metadata memory has no demonstrated reduction; identical demanded metadata index and 13 modules remain resident. Preserve Q09's active-state measurements and separate them from the measured post-close ownership release. |
| F5 | Editor notification-error reconciliation | Rejected mutations still reconcile with full workspace progress. This is outside the normal typing path and was explicitly nonblocking; evaluate using the incremental progress scope for consistency, with a rejected-edit regression. |

The stale-reconciliation comment noted by the reviewer was corrected with the
blocking progress fix. The separate DX.10 F6 whole-file size observation remains
**failed (+6720 bytes)** in [its owning issue](ad-hoc-dx10-profile-review-followups.md);
DX.11 neither reruns nor relaxes that assertion.

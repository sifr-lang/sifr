# DX.16 whole-phase nonblocking review observations

status: open; separate maintenance, not required trace-artifact remediation

Origin: [whole-phase review of 1901074b62e2aad614d285f1c35c95aa16218490](https://github.com/sifr-lang/sifr/pull/3870#issuecomment-5748613670).
The required trace-artifact finding is resolved in its [merged remediation issue](ad-hoc-dx-trace-artifact-remediation.md).
No implementation is authorized by this observation list.

| Owner | Observation |
| --- | --- |
| CLI tests | Resolved by [DXF.6 / #3885](https://github.com/sifr-lang/sifr/pull/3885): focused parser and 34 isolated process contracts cover cache help, arguments, globals, JSON and cleanup counters. |
| Documentation | Resolved by DX.16: prune reserve-bytes/prune-project, profile examples and historical source-migration prose are reconciled. |
| CLI context | Consider aligning the unreachable unset-profile fallback with the Development default. |
| Driver cache | Explain why process-global interop-plan identity is safe while compiler identity is process-constant. |
| Public embedding API | Resolved by [DXF.4 / #3881](https://github.com/sifr-lang/sifr/pull/3881): LSP embedding requires caller context at construction; analysis/bootstrap/SQL cfg guards and the internal verification harness are audited in the canonical follow-up plan. Shipped CLI identity remains explicit. |
| Driver formatter | Consider explicit rustfmt context/process ownership; current contract names Cargo/rustc/Sifr. |
| Project-cache tests | Resolved by [DXF.6 / #3885](https://github.com/sifr-lang/sifr/pull/3885): both valid and diagnostic-producing relocated sources miss and match fresh diagnostics. |
| Documentation owners | Resolved by [DXF.6 / #3885](https://github.com/sifr-lang/sifr/pull/3885): four moved issue links target canonical archives; the absent parity report now has an honest historical reference. The architecture `item: T` match remains an inline-code false positive. |
| DX.10 followups | Existing F8 profile-authority wording remains with its original issue. |

DX.16 reconciles the command heading and acceptance preamble after the required
trace surface was implemented. No suggestion waived a required capability.

DXF.4 scoped review suggestions (optional maintenance, not new authorized scope):
the retained run_stdio_with_identity wrapper could be consolidated if a future
CLI caller supplies its context directly; public entrypoint doc comments could
cover the ownership contract already recorded in the architecture table.
The test-mode CLI helper policy remains DX13-F5. No production identity defect
was established in the verification-only diagnostic rendering harness.

# DX.16 whole-phase nonblocking review observations

status: open; separate maintenance, not required trace-artifact remediation

Origin: [whole-phase review of 1901074b62e2aad614d285f1c35c95aa16218490](https://github.com/sifr-lang/sifr/pull/3870#issuecomment-5748613670).
The required trace-artifact finding is resolved in its [merged remediation issue](ad-hoc-dx-trace-artifact-remediation.md).
No implementation is authorized by this observation list.

| Owner | Observation |
| --- | --- |
| CLI tests | Add cache help/argument/JSON coverage to the CLI contract matrix; current storage behaviour has library/process evidence. |
| Documentation | Resolved by DX.16: prune reserve-bytes/prune-project, profile examples and historical source-migration prose are reconciled. |
| CLI context | Consider aligning the unreachable unset-profile fallback with the Development default. |
| Driver cache | Explain why process-global interop-plan identity is safe while compiler identity is process-constant. |
| Public embedding API | Review non-test constructors using test identity; shipped CLI uses the embedded identity. |
| Driver formatter | Consider explicit rustfmt context/process ownership; current contract names Cargo/rustc/Sifr. |
| Project-cache tests | Add explicit moved-workspace miss coverage; current canonical-root hashing is conservative. |
| Documentation owners | Five pre-existing broken links in roadmap/index point to moved interop/sysroot/stdlib issues and a missing parity report. The architecture item:T match is only a Markdown-regex false positive. Exact locations are preserved in the external audit. |
| DX.10 followups | Existing F8 profile-authority wording remains with its original issue. |

DX.16 reconciles the command heading and acceptance preamble after the required
trace surface was implemented. No suggestion waived a required capability.

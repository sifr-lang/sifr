# Review Artifact: semantic diagnostic code taxonomy & structured HIR diagnostics — diag 9 protocol primary ranges

**Slice**: milestone_diag_9
**Review pass**: pass-1
**Date**: 2026-05-03

## Verdict

**SATISFIED**

## Findings

none

## Verified primary ranges

| Diagnostic | Anchor | Primary range |
|---|---|---|
| SIFR-PROTO-0001 | generic call | `call.range()` |
| SIFR-PROTO-0001 | builtin reversibility | `call.arguments.args[0].range()` |
| SIFR-PROTO-0002 | method signature | `func.name.range()` |
| SIFR-PROTO-0002 | element mismatch | `class_def.name.range()` |
| SIFR-PROTO-0003 | context manager | `item.context_expr.range()` |

## Validation

All eight e2e column anchors and validation commands passed.

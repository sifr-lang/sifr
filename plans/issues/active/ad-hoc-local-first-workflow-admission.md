# Local-first workflow admission failure

Status: open
Owner: CI / verification workflow admission

DXF.7 observed `.github/workflows/local-first-validation.yml` failing before any
jobs were created. This was already present at its unchanged base
`4f9fc5c0d8ac82d5a9e01c4dc4b20450ebf23e09`:
[base run 35512834988](https://github.com/sifr-lang/sifr/actions/runs/35512834988).
The same failure occurred on the original DXF.7 package candidate:
[run 35513756708](https://github.com/sifr-lang/sifr/actions/runs/35513756708).
Check suite 96158878536 contained no check runs. Workflow bytes are unchanged
between the base and final validation candidate
`db38e61773b351399a0217e502d8011b1212def7`.

The underlying admission failure has not been established. Do not describe this
as a failed compiler test or guess a YAML/matrix cause without the admission
error. Inspect the workflow/check-suite admission diagnostics, identify the
owning configuration failure, and qualify its correction in a separate item.
No workflow repair or broad compiler gate restart was absorbed into DXF.7.

Read-only evidence:
`/home/yaser5/projects/sifr/dxf-evidence/f6865cdfb8aa8d0f7d62759ec4fe313e5edc3708/unrelated-workflow-failure.json`.
The follow-up validation policy and completed affected checks are in the
[DXF.7 record](../archive/ad-hoc-compiler-dx-followup-execution.md#dxf7-merged-record--2026-09-20).

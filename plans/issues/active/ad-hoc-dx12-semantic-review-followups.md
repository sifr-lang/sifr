# DX.12 semantic contract review followups

Status: open follow-up work; not a blocker for the scoped DX.12 closure.

Source: [scoped Opus review of cc2874a31](https://github.com/sifr-lang/sifr/pull/3862#issuecomment-5726861173).
The exact-candidate verdict is **SATISFIED**, without blocking findings.
The reviewer accepted DX.12 as the complete capture/transport contract boundary;
ordinary cross-process use is DX.13. Preserve these suggestions when wiring that
consumer, rather than expanding the merged DX.12 item.

| ID | Owning boundary | Observation and next bounded action |
| --- | --- | --- |
| DX12-F1 | DX.13 family consumers | CodegenHandoff.checked_module_identity contains the checked input stamp and is compared with CheckedModule.input_identity. Clarify the field name before consumer wiring so a caller cannot confuse it with module_identity; preserve exact-input binding. |
| DX12-F2 | DX.13 diagnostics / later editor memory integration | CanonicalDiagnostic::render registers every supplied source, including sources not referenced by that diagnostic. Consider shared or demand-selected remapping when using whole-project source tables, and measure retained ownership without weakening source validation. |
| DX12-F3 | DX.13 bounded record decode | Observation, Family and SourceOutcome enums do not carry deny_unknown_fields like sibling structs. Evaluate uniform rejection of unknown payload fields before accepting on-disk records. |
| DX12-F4 | DX.13 outer semantic context | required_external checks declared external completeness, but components has no separate required set. The outer owner must populate the complete component inventory; consider an explicit required-components contract and negative test while wiring the consumer. |

Pre-existing supplemental lint findings stay in the
[metadata owner issue](ad-hoc-dx-metadata-review-followups.md) and
[profile owner issue](ad-hoc-dx10-profile-review-followups.md).
They remain failed evidence for phase-end work; DX.12 does not waive them.


## DX.13 consumer disposition (2026-09-18)

[PR #3864](https://github.com/sifr-lang/sifr/pull/3864) addresses DX12-F2 with
demand-selected diagnostic source remapping and measured CLI peak RSS, and
DX12-F3 with unknown-field rejection for Observation, Family and SourceOutcome.
DX12-F4 is handled conservatively at this consumer: configured components and
external contexts are ineligible, and successful pure-package publication also
requires the actual empty generated interop plan. A general persisted component
inventory remains separate future work. DX12-F1 remains open: DX.13 does not
restore typed/codegen families, so it does not consume that handoff identity.

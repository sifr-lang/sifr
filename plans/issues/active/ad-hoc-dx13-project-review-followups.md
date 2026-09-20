# DX.13 project reuse review followups

Status: open follow-up work; not a blocker for scoped DX.13 closure.

Source: [scoped Opus review of be67cc8df](https://github.com/sifr-lang/sifr/pull/3864#issuecomment-5727613414).
Verdict: **SATISFIED**, no blocking findings. These observations are separate
bounded work; do not expand the merged DX.13 item.

| ID | Owning boundary | Observation and next bounded action |
| --- | --- | --- |
| DX13-F1 | Phase-end qualification | Include the direct check-entrypoint tests in diagnostics_and_packages_tests.rs and the full lint gate. CLI process coverage passed; supplemental older lint failures remain owned by the metadata/profile followups and are not waived. |
| DX13-F2 | Project storage housekeeping | Resolved by [DXF.3 / #3879](https://github.com/sifr-lang/sifr/pull/3879): explicit pressure cleanup reclaims owned stages, pointer scratch and writer-serialized stage locks while preserving live lock inodes. Ambiguous workspace hint scratch remains protected. |
| DX13-F3 | Project namespace lifecycle | Resolved by [DXF.3 / #3879](https://github.com/sifr-lang/sifr/pull/3879): original absolute canonical paths select recorded deleted owners without recreating workspaces; exclusive namespace/context leases protect live readers/writers. Stable owner/lock tombstones remain. |
| DX13-F4 | Long-lived project history | Resolved by [DXF.2 / #3877](https://github.com/sifr-lang/sifr/pull/3877): deterministic recent retention (128 records / 64 MiB), continued publication past 4096 inputs, leased-reader safety and recent A/B/A. Physical predecessor/orphan reclamation remains DXF.3. |
| DX13-F5 | In-process CLI helper ownership | PROJECT_CACHE_OPTIONS defaults to enabled/quiet before run_cli. Make the policy for direct in-process helper consumers explicit, including test-cache isolation. |
| DX13-F6 | Project prune CLI clarity | DXF.3 / [#3879](https://github.com/sifr-lang/sifr/pull/3879) documents zero-reserve no-op and separates examined/eligible/deleted counters. Exact CLI JSON/help coverage remains DXF.6; no unconditional cleanup was introduced. |
| DX13-F7 | CLI source ownership | check_and_package_commands.rs is 898 lines. Before adding more behavior, split by responsibility to maintain the 900-line guardrail. |

Existing failed supplemental lint evidence remains in the
[metadata followups](ad-hoc-dx-metadata-review-followups.md) and
[profile followups](ad-hoc-dx10-profile-review-followups.md).

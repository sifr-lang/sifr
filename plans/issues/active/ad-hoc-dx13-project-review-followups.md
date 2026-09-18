# DX.13 project reuse review followups

Status: open follow-up work; not a blocker for scoped DX.13 closure.

Source: [scoped Opus review of be67cc8df](https://github.com/sifr-lang/sifr/pull/3864#issuecomment-5727613414).
Verdict: **SATISFIED**, no blocking findings. These observations are separate
bounded work; do not expand the merged DX.13 item.

| ID | Owning boundary | Observation and next bounded action |
| --- | --- | --- |
| DX13-F1 | Phase-end qualification | Include the direct check-entrypoint tests in diagnostics_and_packages_tests.rs and the full lint gate. CLI process coverage passed; supplemental older lint failures remain owned by the metadata/profile followups and are not waived. |
| DX13-F2 | Project storage housekeeping | Unique stage-lock inodes and interrupted latest.stage-* scratch files can accumulate. Design owner-safe reclamation without deleting live lock inodes or weakening reader/writer ordering. |
| DX13-F3 | Project namespace lifecycle | Deleted workspaces leave cache namespaces; explicit prune currently requires a named existing workspace. Design bounded owner-scoped reclamation for orphan namespaces without touching live projects. |
| DX13-F4 | Long-lived project history | Historical inheritance reaches the 4096-record cap and safely becomes write-unavailable. Define bounded eviction preserving active-reader safety, latest-generation integrity and useful A/B/A retention. |
| DX13-F5 | In-process CLI helper ownership | PROJECT_CACHE_OPTIONS defaults to enabled/quiet before run_cli. Make the policy for direct in-process helper consumers explicit, including test-cache isolation. |
| DX13-F6 | Project prune CLI clarity | Zero default reserve makes an unqualified prune-project a no-op; eligible_generations also labels actual deletion counts. Clarify pressure selection and output semantics without adding unconditional cleanup. |
| DX13-F7 | CLI source ownership | check_and_package_commands.rs is 898 lines. Before adding more behavior, split by responsibility to maintain the 900-line guardrail. |

Existing failed supplemental lint evidence remains in the
[metadata followups](ad-hoc-dx-metadata-review-followups.md) and
[profile followups](ad-hoc-dx10-profile-review-followups.md).

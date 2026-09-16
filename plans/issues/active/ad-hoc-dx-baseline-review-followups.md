# DX.1 review observations for later bounded work

status: recorded follow-ups; no DX.1 blocker
source: [Opus review of PR #3840](https://github.com/sifr-lang/sifr/pull/3840#issuecomment-5705061104)
reviewed candidate: `5c7502f7aea3cd887a217c2fb3e289c7a1f2c653`

These observations are separate from completed DX.1. They do not authorize
additional implementation in its closure session or add new acceptance criteria.

| Observation | Owning boundary / next consideration |
| --- | --- |
| Capture currently hardcodes values equal to frozen Q09 contract data; future contract edits could diverge. | Performance/Q09 owner: consider reading or asserting the frozen values when that workload changes. |
| Product results are rejected by the current budget gate because no qualified product comparator exists. | Future product-policy qualification must update the gate along with any new product reference; do not disable current guards. |
| Receipt-less contributor selection still inherits ambient sysroot overrides. | Existing DX.2 explicit toolchain-context scope owns removal of ambient authority. |
| Benchmark runner is 877 lines, below the 900-line limit. | Next substantive runner change must respect the guardrail and split by responsibility if needed. |
| Direct `_read_message` EOF errors outside a client cleanup lifecycle do not attach process/stderr context; normal cleanup preserves it as a note. | Developer-tooling diagnostics owner may improve direct-use context in separate bounded work. |

The review's record-update suggestion was addressed by the DX.1 post-merge
phase, roadmap, index and transport issue updates. No follow-up is a claim that
future DX.2–DX.16 functionality has been implemented.

A closure-only broad Markdown-link diagnostic also found the pre-existing roadmap
link to `verification/areas/stdlib_parity/reports/milestone_psp_7_parity_governance_inventory.md`
missing in this checkout. The unchanged link belongs to stdlib-parity roadmap
records and is not modified here. The scoped check of added/changed record links
passes.

# DXF.3 housekeeping review follow-ups

Status: nonblocking follow-up work, separate from merged DXF.3.

Source: [scoped Opus review of #3879](https://github.com/sifr-lang/sifr/pull/3879#issuecomment-5749706095),
candidate `0bc95340dfe6a5e4e32e0824a7b62455dae24df7`.
Verdict: **SATISFIED**, no blocking findings. Complete read-only review evidence
is preserved outside the Git tree in the candidate's `dxf-evidence/opus-review.md`.

| ID | Classification / owner | Finding and disposition |
| --- | --- | --- |
| DXF3-F1 | Suggestion / affected lint reconciliation | Opus suspected `ReadDir.into_iter()` in `housekeeping.rs` may trigger `clippy::useless_conversion` under `-D warnings`. Clippy was not among DXF.3's mandated checks and this is not a measured failure. Preserve the observation for the existing DXF.7 affected-contract reconciliation before claiming lint qualification; do not turn it into a new full gate. |
| DXF3-F2 | Suggestion / future storage lifecycle work | Permanent generation locks are established before staging so read paths do not create lock inodes. A failure after lock creation but before publication can leave a zero-byte generation lock without a generation. Live cleanup intentionally preserves generation lock inodes; orphan context cleanup can reclaim them under exclusive namespace and context leases. Any additional live lock reclamation requires a separate bounded design that preserves concurrent readers. |
| DXF3-F3 | Documented lifecycle limitation / future owner-reset policy | A deleted and recreated directory at the same path has a different owner identity. Its old namespace remains protected and cannot be reused or pruned as an orphan while the replacement exists. This is deliberate replacement-path preservation, documented in the architecture, not permission to invent automatic owner takeover. A separate explicit reset/recovery policy would need scoped authorization. |
| DXF3-F4 | Documented filesystem limitation / qualification | Directory creation time is required for the owner identity. Filesystems/kernel combinations without usable btime leave optional project persistence unavailable. Current evidence is the remote qualified Linux filesystem only; overlayfs/NFS/older-kernel behavior is not claimed as tested. Retain this limitation when making supported-host/storage claims. |
| DXF3-F5 | Suggestion / CLI contract owner | A busy context produces zero examined/eligible/deleted counters because cleanup cannot inspect it under the writer lease. This is truthful but indistinguishable from an empty scope. An explicit contention indicator is an optional future refinement for the CLI owner, not a new DXF.6 requirement. |

No finding reopens DXF.3, adds a current gate or authorizes unrelated work.
The canonical sequential plan remains authoritative.

# DX.5 metadata schema review observations

status: open follow-up suggestions; nonblocking for DX.5
source candidate: `c7734025c242631a326901ce08c846ac42afd7ae`
implementation: [PR #3848](https://github.com/sifr-lang/sifr/pull/3848)
review: [Claude Opus 5, SATISFIED](https://github.com/sifr-lang/sifr/pull/3848#issuecomment-5712915895)

The [canonical Phase DX](ad-hoc-compiler-dx-and-toolchain-reuse.md) remains the
scope and acceptance authority. These are prospective suggestions, not new DX.5
closure requirements or authorization to extend the current session. The next
owning item should assess each suggestion against its approved scope.

| ID | Owner / suggested timing | Observation and proposed follow-up |
| --- | --- | --- |
| DX5-F1 | sysroot/producer, DX.6 | `metadata/container.rs` permits encoded records up to 2 MiB, while `MetadataStore::open` also requires `4096 + len * 32 <= retained_bytes` (64 MiB by default). The top 128 bytes of the nominal record-size range therefore encode but fail closed when reopened with the same limits. Consider deriving a single effective bound or rejecting at insertion before a real producer publishes artifacts. No producer exists in DX.5. |
| DX5-F2 | IR/type-system producer mapping, DX.6 | `check_dx_metadata_inventory.py` checks current ExternalDefs/IR struct fields and record-family names, but not all enum-variant sets or the Type-to-wire variant transformation. The DX.5 reviewer independently verified the existing variants with no omissions. Consider making that drift check executable before the conversion becomes load-bearing. |
| DX5-F3 | sysroot/provider retention, DX.7 | The cache-pressure path re-sums retained entry charges while holding the retention mutex. It is correct and bounded at the tested sizes. Measure real demanded stdlib volumes before deciding whether incremental accounting is needed. |

No implementation for these observations belongs to the DX.5 record update.

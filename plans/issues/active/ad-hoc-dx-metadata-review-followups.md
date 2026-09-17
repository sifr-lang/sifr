# DX metadata review observations

status: open follow-up suggestions; nonblocking for DX.5 and DX.6
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


## DX.6 disposition and new observations

Source candidate: `5581af0884edcb46d88419b9b7d66d1ce97531df`.
Implementation: [PR #3850](https://github.com/sifr-lang/sifr/pull/3850).
Review: [Claude Opus 5, SATISFIED](https://github.com/sifr-lang/sifr/pull/3850#issuecomment-5717512054).
These are nonblocking suggestions, not additional DX.6 closure requirements or
permission to implement the next item in this session.

- DX5-F1 remains fail-closed: the canonical producer reopens and completely
  validates its output before publication, so an entry outside the decoder's
  effective limits cannot be published. Unifying the nominal encoder/decoder
  limits remains optional wire-format maintenance; no incompatible output was
  accepted in DX.6.
- DX5-F2 is addressed by DX.6's exhaustive typed encoder matches and
  `scripts/generate_metadata_encoders.py --check`, together with the existing
  field/family inventory check. Added live variants fail exhaustiveness; added
  fields are checked against the wire inventory. Nominal/Type transformations
  remain explicit and are compiled against the live variants.
- DX5-F3 remains with the DX.7 provider/retention owner.

| ID | Owner / suggested timing | Observation and proposed follow-up |
| --- | --- | --- |
| DX6-F1 | driver/provider, assess before DX.7 exposes ordinary artifact overrides | Ensure/override/warm/output paths currently use `fs::read` before the decoder checks its 256 MiB file bound. A very large wrongly copied or corrupt private entry can allocate before rejection. Consider a size-first, bounded read (including growth races), or a file-backed store with streaming digest. The reviewer classified this as nonblocking for DX.6's uid-checked private cache and test-only override boundary. |
| DX6-F2 | driver/provider, DX.7 | `with_metadata_override` does not enter the current source-stdlib owner key `(compiler identity, sysroot)`. This is harmless while the override only validates; when consumers read metadata, key the owner by the selected immutable artifact identity so different selections cannot share the wrong owner. |
| DX6-F3 | storage owner, future scoped maintenance assessment | Failed/cancelled `{key}.stage` files are reclaimed on the next production of that key; published metadata entries currently have no pruning. This matches DX.6's documented immutable-byte ownership and native-only pruning boundary. Assess inactive-stage and obsolete-configuration cleanup under the existing pressure policy; do not infer new work authorization from this suggestion. |
| DX6-F4 | documentation closer | Wrap the appended DX.6 paragraph's trailing fragment-validation sentence in `internal_docs/architecture.md`; cosmetic only. |

No implementation for these observations belongs to this record-only update.

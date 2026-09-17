# DX metadata review observations

status: open follow-up suggestions; nonblocking for DX.5–DX.7
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


## DX.7 disposition and new observations

Source candidate: `036e69592164b87d309f31ba43ee406a58ed79ca`.
Implementation: [PR #3852](https://github.com/sifr-lang/sifr/pull/3852).
Review: [Claude Opus 5, SATISFIED](https://github.com/sifr-lang/sifr/pull/3852#issuecomment-5719741485).
These are nonblocking observations, not new DX.7 acceptance requirements or
permission to implement the next item during its record-only closure.

- DX6-F1 is addressed: required metadata/descriptor/source reads are bounded,
  including growth races, before decoding or retaining buffers.
- DX6-F2 is addressed: explicit artifact selection resets the context-owned
  provider; only successful initialization is retained. Tests prove independent
  owners and retry after a corrected missing artifact.
- DX5-F3 was assessed against actual demanded retention: the Q09 candidate loads
  13 semantic modules/13 shared nominal projections and passes its scoped cap.
  Incremental cache-charge accounting remains optional, with no measured blocker.
- DX6-F3 remains with storage maintenance under pressure-based cleanup.
  DX6-F4 remains a cosmetic documentation-closer observation.

| ID | Owner / suggested timing | Observation and proposed follow-up |
| --- | --- | --- |
| DX7-F1 | lowering/external definitions, future scoped maintenance | Baseline-priority reads and overlay-only mutation are intentionally asymmetric. Existing callers respect reserved baseline ownership; a future mutator targeting a baselined key could be silently hidden. Consider enforcing the invariant explicitly at mutable access boundaries without allowing reserved stdlib shadowing. |
| DX7-F2 | lowering/external definitions, future scoped maintenance | Calling `freeze()` after a baseline is already present leaves overlay entries unshared. Current bootstrap freezes once; consider making unsupported repeated freeze explicit. |
| DX7-F3 | frontend/incremental owner, assess before frequent overlay enumeration | `len()`/`is_empty()` enumerate and overlay iteration performs baseline membership lookups. Current sizes are bounded; measure before adding accounting complexity. |
| DX7-F4 | driver metadata/codegen, assess during DX.8 qualification | The provider closure skips absent names, following pre-existing demand/support `continue` behavior. A valid-but-incomplete artifact could therefore surface missing support only in downstream Rust validation. Producer complete validation/template checks remain active; assess a precise missing-required-module diagnostic without adding fallback. |
| DX7-F5 | phase qualification owner | Intermediate validation is intentionally scoped: no full driver corpus or clippy receipt is claimed for DX.7. Preserve the required DX.8 corpus/target work and the single phase-end full gate; this observation does not reinstate intermediate gates. |
| DX7-F6 | performance reporting owner | The empty allocator sample grows by 0.107 MiB (0.070 → 0.177 MiB), while steady, peak and retained totals decrease. Keep this absolute increase visible alongside the larger reductions; no optimization is required by this observation. |
| DX7-F7 | package graph/editor owner, separate scoped assessment | The frozen DX.1 demanded-stdlib fixture already reports `SIFR-PACKAGE-0103` for its Cargo package graph. All paired DX.7 samples preserve the exact diagnostic while demand checking executes. Investigate package graph setup separately; do not erase the diagnostic, change the frozen workload, or claim clean-package editor correctness from this measurement. |
| DX7-F8 | compiler measurement receipt owner, future scoped maintenance | The existing lane receipt still labels embedded compatibility identity as unavailable-before-DX.2. DX.7 evidence separately verifies the actual installed descriptor's compiler identity and binary digest. Update the receipt probe when next editing that owner; do not treat the legacy label as authoritative evidence of absent identity. |

No implementation for these observations belongs to this record-only update.

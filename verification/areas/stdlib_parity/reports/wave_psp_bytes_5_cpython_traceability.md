# wave_psp_bytes_5 CPython Traceability Matrix

Wave: `wave_psp_bytes_5`  
Scope: successor-phase and FFI-readiness governance closeout after raw-byte backend completion

## CPython Harvest Inputs

- `Lib/test/test_io/`
- `Lib/test/test_base64.py`
- `Lib/test/test_hashlib.py`
- `Lib/test/test_bytes.py`

## Adopt / Adapt / Waive (Wave 5 Closeout)

| CPython family | Sifr successor/governance direction | State | Local regression/doc anchor |
| --- | --- | --- | --- |
| `test_io` binary file-handle and stream families | successor runtime/file-object planning now assumes canonical raw-byte-backed `bytes` and forbids reintroducing widened/list stand-ins on typed boundaries | `adapted` (governance closed) | `issues/ad-hoc-runtime-and-file-object-parity-expansion.md`<br>`issues/ad-hoc-first-class-bytes-and-binary-surface-foundation-execution.md` |
| `test_hashlib` binary payload and digest families | successor RNG/crypto planning now assumes bytes-native paths consume typed raw-byte-backed `bytes` directly with no per-element typed-input revalidation | `adapted` (governance closed) | `issues/ad-hoc-stateful-rng-crypto-and-polish-parity-expansion.md`<br>`issues/ad-hoc-first-class-bytes-and-binary-surface-foundation-execution.md` |
| `test_base64` binary payload families | successor governance keeps bytes as canonical binary carrier and preserves explicit conversion-boundary checks only | `adapted` | `verification/areas/stdlib_parity/reports/milestone_psp_7_parity_governance_inventory.md`<br>`verification/areas/stdlib_parity/reports/wave_psp_bytes_4_cpython_traceability.md` |
| `test_bytes` ownership/model and interoperability-adjacent families | interoperability planning now explicitly anchors on owned immutable read-only `bytes` plus deferred mutable/view semantics | `adapted` (ownership contract locked) | `internal_docs/phases/43_interoperability.md`<br>`issues/ad-hoc-first-class-bytes-and-binary-surface-foundation.md` |

## Classified waivers (real remaining binary set after wave 5)

| Surface | State | Rationale |
| --- | --- | --- |
| `bytearray` mutable object-model parity | `unsupported` | Mutable byte buffers remain deferred to an explicit mutable/view phase. |
| `memoryview` and general buffer-protocol families | `unsupported` | Generic borrowed-view protocol remains intentionally deferred. |
| Non-UTF-8 codec matrices for `str.encode` / `bytes.decode` | `unsupported` | Current bytes conversion closure remains UTF-8-only by design. |
| `hashlib` bytes-native update/digest constructor families (`update_bytes`, `digest() -> bytes`, `new_bytes`) | `unsupported` | Deferred to the RNG/crypto successor implementation phase; governance now locks the required bytes-native contract baseline. |
| Direct bytes-oriented base64 entrypoints as primary public parity claim | `unsupported` | Current closure keeps text-friendly public surface with explicit bytes conversion boundaries. |

## Governance closeout anchors (wave 5)

- successor runtime/file-object planning:
  - `issues/ad-hoc-runtime-and-file-object-parity-expansion.md`
- successor RNG/crypto planning:
  - `issues/ad-hoc-stateful-rng-crypto-and-polish-parity-expansion.md`
- interoperability/FFI-readiness baseline:
  - `internal_docs/phases/43_interoperability.md`
- canonical milestone parity/waiver ledger:
  - `verification/areas/stdlib_parity/reports/milestone_psp_7_parity_governance_inventory.md`

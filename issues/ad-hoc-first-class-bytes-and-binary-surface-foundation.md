# Ad Hoc Phase: First-Class Bytes and Binary Surface Foundation

Status: open (documented 2026-03-18)
Context: prerequisite phase after structured/class-surface parity expansion and before runtime/file-object plus RNG/crypto follow-ups
Execution readiness: implementation-ready in sequence after `issues/ad-hoc-structured-data-and-class-surface-parity-expansion.md`; wave 1 still requires recorded typing/lowering/codegen migration evidence from architecture lock validation

## Objective

Land a first-class immutable `bytes` type and one explicit text/binary boundary so later parity phases do not keep routing binary APIs through `list[int]` or string-only stand-ins.

This phase is not a broad binary-stdlib sweep. It is a language/runtime and parity-foundation phase whose job is to make later `io`, `zipfile`, `tempfile`, `hashlib`, `base64`, `random.randbytes`, and related work honest and coherent.

Primary targets:

- first-class immutable `bytes`
- explicit construction and conversion rules
- migration path for the current `sifr.bytes` helper surface
- downstream binary-surface contract for later runtime and crypto phases

## Source of Truth

This phase must use the following as authoritative references:

- canonical parity inventory:
  - `verification/stdlib/milestone_psp_7_parity_governance_inventory.md`
- relevant closure and waiver ledgers:
  - `verification/stdlib/wave_psp_a2_cpython_traceability.md`
  - `verification/stdlib/wave_psp_c2_cpython_traceability.md`
  - `verification/stdlib/wave_psp_d1_cpython_traceability.md`
  - `verification/stdlib/wave_psp_e1_cpython_traceability.md`
- historical and architectural baseline:
  - `internal_docs/architecture.md`
  - `internal_docs/phases/03_standard_library.md`
  - `internal_docs/phases/07_stdlib_parity.md`
- predecessor and successor planning docs:
  - `issues/ad-hoc-structured-data-and-class-surface-parity-expansion.md`
  - `issues/ad-hoc-runtime-and-file-object-parity-expansion.md`
  - `issues/ad-hoc-stateful-rng-crypto-and-polish-parity-expansion.md`
- current shipped binary-adjacent surfaces:
  - `lib/sifr/bytes.sifr`
  - `lib/sifr/io.sifr`
  - `lib/sifr/base64.sifr`
  - `lib/sifr/hashlib.sifr`
- CPython source and tests:
  - `/Users/yaseralnajjar/work/sifr/cpython`
  - `/Users/yaseralnajjar/work/sifr/cpython/Lib/test`

Primary upstream families:

- `Objects/bytesobject.c`
- `Objects/bytearrayobject.c`
- `Lib/test/test_bytes.py`
- `Lib/test/test_base64.py`
- `Lib/test/test_hashlib.py`
- `Lib/test/test_io/`

## Why This Needs Its Own Phase

The current repo still mixes three different binary stories:

- a custom `sifr.bytes` helper module,
- `list[int]` as an adapted binary carrier,
- and string-only stand-ins for some crypto surfaces.

That is manageable for narrow shipped subsets, but it is the wrong foundation for the next parity frontier.

The runtime/file-object phase needs one canonical binary carrier for streams, archives, and temporary files. The RNG/crypto phase needs one canonical binary carrier for digests, codec payloads, and random byte generation. Mixing the `bytes` object-model decision into either of those phases would force type-system, codegen, ownership, and stdlib boundary work to land at the same time.

This phase exists to close that root cause first.

The architecture target is fixed in this document. What remains before wave 1 is not a fresh design pass; it is execution evidence for the chosen type-system, HIR, lowering, codegen, and intrinsic-migration path.

## Depends on

- `issues/ad-hoc-structured-data-and-class-surface-parity-expansion.md`
- milestone-7 canonical closure inventory remains the baseline
- Phase 27 non-regression invariants remain mandatory
- Phase 29 local-first validation contract remains mandatory

## Current Repo Reality

- Parser and AST support for Python-style bytes literals already exists in:
  - `crates/sifr_python_parser/src/string.rs`
  - `crates/sifr_python_ast/src/nodes.rs`
- The missing work is not bytes-literal parsing from zero. The missing work is:
  - first-class `bytes` typing in `sifr_type_system`,
  - HIR and stdlib signature migration away from `list[int]`,
  - lowering and codegen for first-class `bytes` operations,
  - and runtime/intrinsic migration of current binary helpers and file APIs.
- Current binary intrinsics and file APIs still expose `list[int]` boundaries in:
  - `crates/sifr_hir/src/stdlib/collections_bytes_time.rs`
  - `crates/sifr_hir/src/stdlib/sys_fs.rs`
  - `lib/sifr/bytes.sifr`
  - `lib/sifr/io.sifr`

## Implementation Approach

This phase should be executed as a focused compiler-and-runtime migration, not as a parser project.

1. Keep existing parser and AST bytes-literal support as the frontend input surface.
2. Add a first-class `bytes` type to the type system and propagate it through HIR signatures and stdlib contracts.
3. Lower first-class `bytes` values to one immutable owned byte-buffer runtime model.
4. Migrate binary intrinsics and file-system intrinsics from `list[int]` to `bytes`.
5. Rebuild `lib/sifr/bytes.sifr` as a compatibility wrapper over the first-class implementation.
6. Only after the core type and runtime path are stable, rewire successor phase docs and ledgers to consume the new binary carrier.

## Public Surface Contract

### `bytes`

- `bytes` becomes a first-class immutable value type.
- The canonical runtime representation is an owned contiguous byte buffer compatible with Rust `Vec<u8>` semantics.
- `bytes` supports:
  - equality and inequality
  - concatenation
  - `len(...)`
  - iteration yielding `int`
  - indexing returning `int`
  - slicing returning `bytes`
- Construction and conversion must be explicit. Approved entry points:
  - `bytes() -> bytes`
  - `bytes(size: int) -> Result[bytes, ValueError]`
  - `bytes.from_ints(data: list[int]) -> Result[bytes, ValueError]`
  - `bytes.from_hex(s: str) -> Result[bytes, ParseError]`
  - `to_ints() -> list[int]`
- No implicit coercion between `str`, `list[int]`, and `bytes`.
- `bytes(size)` allocates a zero-filled immutable buffer of the requested length.
- negative sizes must fail with `ValueError`.

### Text / binary conversion

- `str.encode(encoding: str = "utf-8") -> Result[bytes, ParseError]`
- `bytes.decode(encoding: str = "utf-8") -> Result[str, ParseError]`
- This phase supports UTF-8 only.
- Non-UTF-8 codec matrices remain out of scope.

### Transitional `sifr.bytes` compatibility layer

- `lib/sifr/bytes.sifr` remains temporarily available for compatibility,
- but it must delegate to the first-class `bytes` implementation rather than define the canonical binary model itself,
- and future parity claims attach to the first-class `bytes` type, not to the helper module.

### Downstream adoption contract

- Later phases must use `bytes` as the canonical binary carrier for:
  - `io` binary read/write surfaces
  - `BytesIO`
  - `zipfile` read handles
  - binary tempfile modes
  - `hashlib` digest and update surfaces
  - `base64` binary codec surfaces
  - `random.randbytes`
- `list[int]` may remain as an explicit conversion boundary for legacy compatibility helpers, but it is no longer the parity target for binary APIs.

### `bytearray`, `memoryview`, and buffer protocol

- `bytearray` is deferred in this phase.
- `memoryview` is deferred in this phase.
- No general CPython buffer protocol or bytes-like duck typing is introduced in this phase.
- Zero-copy view semantics are therefore out of scope; later phases must use explicit owned `bytes` values and honest copying boundaries where needed.

## Permanent Sifr-Safe Diffs

The following are intentionally not part of this phase’s execution target:

- full CPython buffer protocol parity,
- `memoryview`,
- `bytes` / `bytearray` subclass ecosystems,
- implicit coercions between text and binary values,
- non-UTF-8 codec families,
- mutable `bytearray` parity unless a later dedicated follow-up closes it explicitly.

If these remain unsupported at phase exit, they must be explicit and narrow in the waiver inventory.

## Scope

This phase owns:

- first-class immutable `bytes` type support in the compiler and runtime,
- explicit constructor, indexing, slicing, iteration, and conversion behavior,
- UTF-8 encode/decode and hex conversion on the first-class `bytes` surface,
- migration of `sifr.bytes` onto the first-class implementation,
- parity-ledger updates that replace the current “custom helper only” bytes classification with the new target.

This phase does not own:

- full `bytearray` parity,
- buffer protocol parity,
- broad `io`, `zipfile`, `tempfile`, `hashlib`, or `base64` closure,
- runtime host abstractions,
- stateful RNG object parity.

## Non-goals

- sneaking binary-surface redesign into later phases without first-class `bytes`,
- preserving `list[int]` as the long-term public parity target for binary APIs,
- weakening ownership or panic-free guarantees to mimic CPython mutability quirks,
- reopening all binary-adjacent stdlib modules in this same phase.

## Priority Targets

### priority_1: Core bytes object model

Targets:

- first-class `bytes`
- compiler/runtime representation
- indexing, slicing, iteration, concatenation, equality

Required closure direction:

- `bytes` is a real public type rather than a helper-module convention,
- binary values no longer need `list[int]` to exist in user-visible APIs,
- the type system, lowering, and codegen agree on one immutable byte-buffer model.

### priority_2: Text/binary conversion and helper migration

Targets:

- UTF-8 encode/decode
- hex conversion
- `lib/sifr/bytes.sifr`

Required closure direction:

- text/binary conversion is explicit and typed,
- helper compatibility surfaces delegate to the new canonical type,
- no conversion path introduces panic-prone or lossy behavior.

### priority_3: Downstream parity unblockers

Targets:

- runtime/file-object phase contracts
- RNG/crypto phase contracts
- parity governance ledgers

Required closure direction:

- later phases no longer need to invent a binary carrier,
- stale “custom helper only” bytes wording is removed from the active plan,
- surviving binary-related waivers point to concrete remaining blockers rather than to the absence of a bytes type.

## Waves

### wave_psp_bytes_0: Architecture Lock

Scope:

- `bytes` object model
- UTF-8 boundary
- downstream binary adoption contract

Definition of done:

- the public surface contract in this document is reflected in traceability and waivers,
- deferred mutable/view/buffer families are explicitly classified before implementation proceeds,
- later phases can consume `bytes` without inventing conversion or ownership semantics.

### wave_psp_bytes_1: Core `bytes` Type and Compiler Support

Scope:

- first-class `bytes`
- lowering and codegen support
- immutable value behavior

Definition of done:

- `bytes` is supported as a real public type,
- indexing, slicing, iteration, concatenation, and equality are shipped or explicitly waived,
- local tests prove panic-free bounds and conversion behavior,
- parser/AST support is reused rather than reimplemented,
- the type-system and HIR signatures no longer route core bytes operations through `list[int]`.

### wave_psp_bytes_2: Conversion Surfaces and Compatibility Migration

Scope:

- UTF-8 encode/decode
- hex conversion
- `sifr.bytes` compatibility wrappers

Definition of done:

- the typed encode/decode/hex surfaces are shipped,
- compatibility helpers delegate to the first-class `bytes` implementation,
- negative-path coverage proves explicit failure semantics for invalid byte values, invalid UTF-8, and invalid hex data.

### wave_psp_bytes_3: Downstream Contract Adoption and Governance Closeout

Scope:

- runtime/file-object successor contract alignment
- RNG/crypto successor contract alignment
- waiver and traceability ledgers

Definition of done:

- downstream phases are rewired to use `bytes` as their binary carrier,
- stale `list[int]`-as-parity-target wording is removed from active planning docs,
- the canonical ledgers record the real remaining binary waiver set.

## CPython Test Porting Targets

- `Lib/test/test_bytes.py`
- selected binary-path coverage from:
  - `Lib/test/test_base64.py`
  - `Lib/test/test_hashlib.py`
  - `Lib/test/test_io/`

## Quality Contract

- `bytes` must be a real first-class type, not syntax sugar over helper calls.
- No implicit `str` <-> `bytes` coercion may be introduced.
- No user-triggerable panic paths are introduced.
- Every wave must update the canonical waiver ledgers before merge.
- No wave may claim binary parity closure while still depending on undocumented `list[int]` fallbacks.

## Architecture Lock Validation

Before `wave_psp_bytes_1` begins implementation, the phase must add:

- one execution-note entry confirming that bytes-literal parsing and AST support already exist, so the spike scope starts at typing/lowering/codegen rather than lexing,
- one Sifr demo covering first-class `bytes` construction, indexing, slicing, and iteration,
- one Sifr demo covering explicit UTF-8 encode/decode and hex conversion,
- one negative-path test for every newly explicit permanent divergence,
- one CPython-family mapping table proving which upstream cases are adopted, adapted, or permanently waived,
- explicit phase test families covering `test_bytes` plus selected binary-path coverage from `test_base64`, `test_hashlib`, and `test_io`,
- one compile-time rejection or negative runtime case for every new typed surface that proves the remaining Sifr-safe divergence is explicit rather than accidental.

## Local Validation Commands

- quick gate:
  - `scripts/run_all_tests.sh --profile quick`
- full gate:
  - `scripts/run_all_tests.sh`

## Exit Criteria

This phase is complete only when:

- first-class immutable `bytes` is shipped or sharply and explicitly re-waived,
- the repo no longer treats `list[int]` as the long-term public parity target for binary APIs,
- successor phase docs and ledgers use the new binary contract consistently,
- local validation is green,
- external review confirms the phase is production-grade and Sifr-safe.

# `stdlib_parity_bytes_0` Architecture Lock (First-Class Bytes and Binary Surface Foundation)

Phase: `issues/ad-hoc-first-class-bytes-and-binary-surface-foundation.md`  
Execution ledger: `issues/ad-hoc-first-class-bytes-and-binary-surface-foundation-execution.md`

## Objective

Lock one canonical binary carrier (`bytes`), one explicit text/binary boundary, and one explicit permanent-diff set before compiler/runtime migration waves begin.

## Locked Public Contract Snapshot

| Surface | Locked direction for this phase |
| --- | --- |
| `bytes` (language type) | Promote to first-class immutable value type with explicit construction/conversion and no implicit text/binary coercion. |
| `str.encode` / `bytes.decode` | Explicit UTF-8-only boundary in this phase (`Result`-based failure semantics). |
| `lib/sifr/bytes.sifr` | Keep as temporary compatibility layer, delegating to first-class bytes implementation once wave 2 lands. |
| `io`/runtime binary surfaces | Target `bytes` as canonical binary carrier; remove `list[int]` as long-term parity target wording in downstream planning during wave 3. |

## Permanent Sifr-Safe Diffs (Locked for This Phase)

| Surface | Classification | Enforcement fixture |
| --- | --- | --- |
| Mutable `bytearray` object-model parity | `unsupported` | `crates/sifr/tests/e2e/fail/bytes_bytearray_unsupported.sifr` |
| `memoryview` object-model parity | `unsupported` | `crates/sifr/tests/e2e/fail/bytes_memoryview_unsupported.sifr` |
| CPython buffer protocol / bytes-like duck typing | `unsupported` | `crates/sifr/tests/e2e/fail/bytes_buffer_protocol_unsupported.sifr` |
| Implicit `str` <-> binary coercions | `unsupported` | `crates/sifr/tests/e2e/fail/bytes_implicit_str_bytes_coercion_unsupported.sifr` |
| Non-UTF-8 codec families | `moved-to-text-i18n-m1` | `crates/sifr/tests/e2e/pass/text_i18n_encoding_io.sifr` |
| `bytes`/`bytearray` subclass ecosystems | `unsupported` | `crates/sifr/tests/e2e/fail/bytes_bytes_subclass_unsupported.sifr` |

## Parser/AST Scope Lock (Pre-existing Support)

- Bytes literal parsing and AST nodes are already present:
  - `third_party/ruff/crates/ruff_python_parser/src/string.rs`
  - `third_party/ruff/crates/ruff_python_parser/src/parser/expression.rs`
  - `third_party/ruff/crates/ruff_python_ast/src/nodes.rs`
- Wave implementation scope therefore starts at type-system, HIR signatures, lowering/codegen, and stdlib/intrinsic migration.

## CPython Family Mapping (Wave Ownership)

| CPython family | Direction | Owning wave | Local anchor |
| --- | --- | --- | --- |
| `Lib/test/test_bytes.py` core immutable object model and constructor/index/slice behaviors | `adapted` | `stdlib_parity_bytes_1` + `stdlib_parity_bytes_2` | `crates/sifr/tests/e2e/pass/bytes_basics.sifr`, `crates/sifr/tests/e2e/pass/bytes_constructors.sifr` |
| `Lib/test/test_base64.py` binary payload pathways | `adapted` | `stdlib_parity_bytes_3` | downstream contract rewiring and parity-ledger updates |
| `Lib/test/test_hashlib.py` digest/update binary pathways | `adapted` | `stdlib_parity_bytes_3` | downstream contract rewiring and parity-ledger updates |
| `Lib/test/test_io/` binary file-handle pathways | `adapted` | `stdlib_parity_bytes_3` | downstream contract rewiring and parity-ledger updates |

## Architecture-Lock Validation Fixtures (Wave 0)

- Positive path fixture: `crates/sifr/tests/e2e/pass/bytes_helpers.sifr`
- Wave-0 demos:
  - `demos/bytes_roundtrip/main.sifr`
  - `demos/text_and_bytes/main.sifr`
- Permanent-diff negative fixtures:
  - `crates/sifr/tests/e2e/fail/bytes_bytearray_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/bytes_memoryview_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/bytes_buffer_protocol_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/bytes_implicit_str_bytes_coercion_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/bytes_bytes_subclass_unsupported.sifr`

# `stdlib_parity_rng_0` Architecture Lock (Stateful RNG, Crypto, and Polish Parity Expansion)

Capability: `issues/stateful-rng-crypto-and-polish-parity-expansion.md`
Execution ledger: `issues/stateful-rng-crypto-and-polish-parity-expansion-execution.md`

## Objective

Lock one deterministic state-model direction for `random`, one bytes-native direction for `hashlib`, and one explicit residual-polish boundary for `base64`/`statistics`/`textwrap`/`html` before implementation passes begin.

This lock exists to prevent later implementation passes from reintroducing stateless host-random shortcuts, str-only crypto digests, or vague residual-waiver ownership.

## Locked Public Rules Snapshot

| Surface | Locked direction for this capability |
| --- | --- |
| `random` deterministic model | `RandomState(version, state_words, index, gauss_next)` is the canonical state container. `Random` owns deterministic mutable state; module-level helpers delegate to one module-global `Random` instance. |
| `random` non-deterministic model | `SystemRandom` remains host-backed and explicitly does not support state export/import (`getstate`/`setstate`). |
| `random` bytes API | `randbytes(n)` is in-scope and must return first-class `bytes` via raw-byte-backed storage paths only. |
| `hashlib` bytes-native model | `HashObject.digest() -> bytes` is the canonical binary digest surface, with `digest_bytes()` alias and `update_bytes(bytes)` support. |
| `hashlib` constructor model | `new_bytes(name, data: bytes = bytes())` is the canonical bytes-first constructor entry for Unicode core capability. |
| SHA3/SHAKE policy | SHA3/SHAKE APIs are in-scope only when backed by actual runtime dependency support; SHAKE requires explicit output length and returns `bytes`. |
| `base64` boundary | binary-oriented APIs must consume/return first-class `bytes`; text helpers may remain as compatibility overlays. |
| `statistics` boundary | readiness targets stay float/int deterministic surfaces only; decimal/fraction/context-sensitive semantics remain explicitly unsupported. |
| residual `textwrap` / `html` | only capability-owned residual waivers are eligible (`textwrap` formatter ecosystem subset, package-wide `html` parser family). No broad parser/text redesign enters this capability. |

## Baseline Fractures Recorded at Capability 0 Entry

The following capability-owned gaps are explicitly present at entry and are treated as implementation targets (not hidden compatibility shims):

- `random` remains stateless host-intrinsic wrappers (`seed`, `getstate`, `setstate`, `Random`, `SystemRandom`, `randbytes`) are not shipped.
- `hashlib` digest surface is currently hex-string based (`digest() -> str` alias to `hexdigest()`), and bytes-native object APIs are absent.
- bytes-oriented `hashlib` constructor (`new_bytes`) is absent.
- residual `textwrap` formatter ecosystem fields and package-wide `html` parser family remain unsupported.

## Permanent Sifr-Safe Diffs (Locked for This Capability)

| Surface | Classification | Enforcement anchor |
| --- | --- | --- |
| Full CPython buffer protocol parity | `unsupported` | inherited bytes governance from `stdlib_parity_bytes_0` / `stdlib_parity_bytes_5` |
| `memoryview` and mutable `bytearray` ecosystem parity | `unsupported` | inherited bytes governance from `stdlib_parity_bytes_0` / `stdlib_parity_bytes_5` |
| `SystemRandom` state export/import parity | `unsupported` | implementation pass-owned policy lock (no deterministic state export for host RNG) |
| Decimal/Fraction/context-sensitive `statistics` semantics | `unsupported` | implementation pass-owned policy lock (float/int deterministic surfaces only) |

## CPython Family Mapping (Capability Ownership)

| CPython family | Direction | Owning capability | Local anchor |
| --- | --- | --- | --- |
| `Lib/test/test_random.py` | `adapted` | `stdlib_parity_rng_1` | deterministic state/object model + module-global proxy readiness |
| `Lib/test/test_hashlib.py` | `adapted` | `stdlib_parity_rng_2` | bytes-native digest/object API + algorithm inventory expansion |
| `Lib/test/test_base64.py` | `adapted` | `stdlib_parity_rng_2` | bytes-native carrier parity on shipped codec families |
| `Lib/test/test_statistics.py` | `adapted` | `stdlib_parity_rng_3` | narrow advanced deterministic surface readiness |
| `Lib/test/test_textwrap.py` | `adapted` | `stdlib_parity_rng_3` | residual formatter ecosystem waiver reduction only |
| `Lib/test/test_html.py` | `adapted` | `stdlib_parity_rng_3` | residual top-level polish only; package parser family remains explicitly unsupported |

## Architecture-Lock Validation Artifacts (Capability 0)

- Positive fixture: `crates/sifr/tests/e2e/pass/random_hashing_and_text.sifr`
- Demo:
  - `demos/random_hashing/main.sifr`
- Negative fixtures:
  - `crates/sifr/tests/e2e/fail/html_package_parser_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/random_choices_weights_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/system_random_state_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/sha3_object_model_unsupported.sifr`

Historical note:
- `crates/sifr/tests/e2e/fail/stdlib_parity_rng_0_random_state_object_model_unsupported.sifr` was retired once `stdlib_parity_rng_1` shipped the deterministic state/object model.
- `crates/sifr/tests/e2e/fail/stdlib_parity_rng_0_hashlib_bytes_digest_api_unsupported.sifr` was retired once `stdlib_parity_rng_2` shipped bytes-native `hashlib` object APIs.
- `crates/sifr/tests/e2e/fail/stdlib_parity_rng_0_textwrap_max_lines_unsupported.sifr` was retired once `stdlib_parity_rng_3` shipped `TextWrapper` `max_lines`/`placeholder` formatter options.

# CPython Parity Plan (env, bytes, base64, math, hashlib)

Date: 2026-02-20  
Scope order (required): `sifr.env` -> `sifr.bytes` -> `sifr.base64` -> `sifr.math` -> `sifr.hashlib`

## References Used

- Sifr architecture contract: `internal_docs/architecture.md`
- Sifr modules:
  - `lib/sifr/env.sifr`
  - `lib/sifr/bytes.sifr`
  - `lib/sifr/base64.sifr`
  - `lib/sifr/math.sifr`
  - `lib/sifr/hashlib.sifr`
- Intrinsic signatures: `crates/sifr_hir/src/stdlib.rs`
- Runtime codegen behavior: `crates/sifr_codegen/src/lib.rs`
- CPython references:
  - `Lib/os.py`
  - `Lib/base64.py`
  - `Lib/hashlib.py`
  - `Modules/mathmodule.c`
  - `Lib/test/test_os/test_os.py`
  - `Lib/test/test_os/test_posix.py`
  - `Lib/test/test_bytes.py`
  - `Lib/test/test_base64.py`
  - `Lib/test/test_hashlib.py`
  - `Lib/test/test_math.py`

## Current Baseline (verified)

- Module smoke tests currently pass:
  - `stdlib_env.sifr`
  - `stdlib_bytes.sifr`
  - `stdlib_base64_intrinsics.sifr`
  - `stdlib_hash.sifr`
  - `stdlib_hashlib_intrinsics.sifr`
  - `stdlib_math.sifr`
  - `cpython_math.sifr`
  - `cpython_math_extended.sifr`
- Important: these are smoke/subset tests, not full CPython parity.

---

## 1) Module Audit: `sifr.env` (vs CPython `os.environ`/`getenv`/`putenv`/`unsetenv`)

### Current Sifr Surface

- Exposes only:
  - `env_get(key: str) -> str | None`
  - `env_set(key: str, value: str) -> None`

### Parity Gaps

- Missing CPython-equivalent capabilities:
  - `unsetenv`
  - `getenv(key, default)`
  - mapping behavior (`keys`, `values`, `items`, iteration, clear/update semantics)
  - behavior sync with process environment semantics covered in CPython tests
- Safety gap:
  - `env_get` and `env_set` directly use Rust env APIs that can panic on invalid keys (`=` or NUL) instead of returning `Result`.

### Root Cause

- `sifr.env` is intentionally minimal and function-based, while CPython environment support is mapping-based.
- Intrinsic contract currently lacks error-typed env operations for invalid key/value validation.

### M1 Deviation Notes (Documented)

- CPython exposes env via mapping semantics (`os.environ`); Sifr currently exposes function-first APIs in `sifr.env`.
- Invalid names are safety-guarded (ignored/no panic) instead of raising runtime exceptions, aligned with Sifr no-panic goals.
- `keys()`/`values()`/`items()` are snapshot helpers, not a mutable mapping object.

### Test Port Status

- Sifr has 1 dedicated test (`stdlib_env.sifr`), mostly happy-path.
- CPython has extensive environment tests (mapping protocol, putenv/unsetenv errors, reload behavior, bytes environ).

---

## 2) Module Audit: `sifr.bytes` (vs CPython `bytes`/`bytearray` behavior subset)

### Current Sifr Surface

- `encode_utf8(s: str) -> list[int]`
- `decode_utf8(bytes: list[int]) -> Result[str, ParseError]`
- `bytes_to_hex(bytes: list[int]) -> str`
- `bytes_from_hex(s: str) -> Result[list[int], ParseError]`

### Parity Gaps

- No first-class `bytes` type / `bytearray` type API parity.
- Missing most CPython byte methods (`find`, `count`, `split`, `join`, `startswith`, `endswith`, etc.).
- Behavioral mismatches:
  - integer-to-byte conversions use `as u8` casts, silently wrapping out-of-range values instead of rejecting.
  - `bytes_from_hex` does not match CPython whitespace/error-position semantics.
  - UTF-8 decode pathway assumes list-int transport, not bytes-like objects.
- Potential panic path:
  - hex parsing uses direct Rust string slicing by byte index; non-ASCII multibyte input can panic instead of returning `ParseError`.

### Root Cause

- Sifr type system currently has no `Bytes` primitive, so implementation uses `list[int]` transport.
- Intrinsic implementation optimized for minimal functionality, not CPython-compatible bytes-like protocol.

### Test Port Status

- Sifr has one smoke test plus parse-safety path tests.
- CPython `test_bytes.py` is very broad and battle-tested (constructors, methods, error messages, edge cases).

---

## 3) Module Audit: `sifr.base64` (vs CPython `base64`)

### Current Sifr Surface

- Intrinsics/wrappers currently available:
  - `base64_encode`, `base64_decode`
  - `b64encode`, `b64decode` aliases
  - `urlsafe_b64encode`, `urlsafe_b64decode`
  - `b32encode`, `b32decode`

### Parity Gaps

- Missing CPython API surface from `base64.__all__`:
  - `standard_b64encode/decode`
  - `b32hexencode/decode`
  - `b16encode/decode`
  - `a85encode/decode`
  - `b85encode/decode`
  - `z85encode/decode`
  - `encodebytes/decodebytes`
  - legacy file `encode/decode` helpers
- Signature/behavior mismatches:
  - CPython is bytes-first; Sifr currently string-first.
  - `b64decode` does not support `altchars`, `validate`, `ignorechars`.
  - `b64encode` does not support `wrapcol`.
  - decode returns UTF-8 string, failing binary payload use cases where CPython returns bytes.

### Root Cause

- No first-class bytes type in Sifr forces lossy string-centric semantics.
- Intrinsic signatures fixed to minimal forms (no optional/strictness parameters).

### Test Port Status

- Sifr coverage is smoke-level only.
- CPython `test_base64.py` includes RFC vectors, error behavior, and property-based round-trip tests.

---

## 4) Module Audit: `sifr.math` (vs CPython `math`)

### Current Sifr Surface

- Broad surface exists (trig, inverse trig, logs, constants, combinatorics helpers, numeric helpers).
- Includes many functions from `math` and extra pure-Sifr helpers.

### Parity Gaps

- Missing CPython `math` functions:
  - `cbrt`, `exp2`, `fma`, `fmax`, `fmin`, `isnormal`, `issubnormal`, `remainder`, `signbit`, `sumprod`
- Signature mismatches:
  - `log(x, base)` not supported (Sifr only `log(x)`).
  - `isclose` lacks CPython `abs_tol` support.
  - `frexp` should be `(float, int)` tuple; currently `list[float]` with exponent encoded as float.
  - `modf` should be tuple return.
- Behavioral mismatches:
  - `dist` uses min length of inputs instead of erroring on dimension mismatch.
  - `fsum` is naive summation, not CPython-accurate compensated algorithm.
  - combinatorics edge cases (`factorial`, `comb`, `perm`, `isqrt`) do not model CPython exception semantics with Sifr-safe adaptation.
  - `nextafter`/`ulp` implementations are not CPython-equivalent for many edge cases.
- Note on intentional divergence:
  - domain errors (`sqrt(-1)`, `log(0)`) are intentionally aligned to Rust IEEE behavior (NaN/inf) per architecture.

### Root Cause

- Math module evolved incrementally via intrinsics, with some placeholder approximations.
- Tuple-return ergonomics and mixed-type tuple contracts are not used by current stdlib design.
- CPython parity tests were only partially ported (subset assertions), leaving edge semantics unverified.

### Test Port Status

- Stronger than other modules (two large CPython-inspired Sifr tests), but still incomplete versus full `test_math.py`.

---

## 5) Module Audit: `sifr.hashlib` (vs CPython `hashlib`)

### Current Sifr Surface

- One-shot digest helpers only:
  - `md5`, `sha1`, `sha224`, `sha256`, `sha384`, `sha512`, `blake2b`, `blake2s`

### Parity Gaps

- Missing CPython API core:
  - `new(name, ...)`
  - `algorithms_guaranteed`, `algorithms_available`
  - `file_digest`
  - hash object protocol (`update`, `digest`, `hexdigest`, `copy`, `.name`, `.digest_size`, `.block_size`)
- Missing algorithms / families:
  - SHA3 and SHAKE constructors
  - optional APIs such as `pbkdf2_hmac`, `scrypt` (if included in Sifr parity scope)
- Data model mismatch:
  - CPython accepts bytes-like input and returns digest objects; Sifr currently accepts `str` and returns hex strings.

### Root Cause

- No hash-object abstraction in Sifr stdlib yet.
- No bytes-like first-class type to represent binary digest/update operations cleanly.

### Test Port Status

- Only smoke checks (digest length / happy path).
- CPython `test_hashlib.py` is extensive (vectors, object behavior, threading, file digest, API contracts).

---

## Cross-Module Root Causes (Systemic)

1. No first-class `bytes`/`bytearray` type in Sifr type system; current substitute is `list[int]`.
2. Many intrinsics are thin direct Rust calls without full CPython-compatible argument model.
3. Missing object models for modules that are object-centric in CPython (`hashlib`).
4. CPython test-port strategy is uneven; only `math` has substantial parity tests.
5. Safety adaptation is incomplete in env/bytes edge paths (panic vs `Result`).

---

## Milestone Plan (Sequential, One Module at a Time)

## Milestone 0 (Foundation, required before module parity)

- Goal: unblock CPython-compatible binary APIs and safe env validation.
- Deliverables:
  - Introduce `bytes` core representation strategy for stdlib parity layer (at minimum in stdlib boundary APIs).
  - Add safe env key/value validation wrappers returning `Result`, no panic path.
  - Define tuple-return conventions for math functions that need CPython tuple semantics.
  - Add a CPython parity harness layer for function-vector style tests.
- Demo:
  - `demos/m0_parity_foundation_demo.sifr`
- PRs:
  - PR0-A: env safety wrappers + tests
  - PR0-B: bytes boundary type strategy + minimal adapters
  - PR0-C: parity harness utilities

## Milestone 1 (`sifr.env`)

- Implement:
  - `unsetenv`
  - `getenv(key, default)` wrapper semantics
  - safe validation behavior matching CPython expectations (adapted to `Result` where needed)
  - optional mapping-like helper API (keys/items/values snapshot) for parity portability
- Port tests from CPython:
  - selected tests from `test_os/test_os.py` + `test_os/test_posix.py` focused on environment operations
- Demo:
  - `demos/m30_1a_env_parity_demo/main.sifr`
- PR plan:
  - PR1-A API+intrinsics
  - PR1-B parity tests + safety tests
  - PR1-C docs/deviation notes

## Milestone 2 (`sifr.bytes`)

- Implement:
  - byte validation (no silent wrapping)
  - robust hex parsing with CPython-compatible whitespace/error behavior
  - byte utility methods needed by dependent modules (minimum parity subset)
- Port tests from CPython:
  - `test_bytes.py`: encoding/decode/fromhex/hex and core method subset first
- Demo:
  - `demos/m30_1a_bytes_parity_demo/main.sifr`
- PR plan:
  - PR2-A correctness and safety fixes
  - PR2-B API expansion subset
  - PR2-C CPython parity tests

## Milestone 3 (`sifr.base64`)

- Implement:
  - full `base64.__all__` target subset for MVP parity:
    - b64/standard/urlsafe + b32/b32hex/b16 + encodebytes/decodebytes
  - optional args (`altchars`, `validate`, `ignorechars`, `wrapcol`) where applicable
  - bytes-preserving decode path
- Port tests from CPython:
  - vectors + invalid input + roundtrip property tests from `test_base64.py`
- Demo:
  - `demos/m30_1a_base64_parity_demo/main.sifr`
- PR plan:
  - PR3-A API expansion
  - PR3-B strictness/error behavior
  - PR3-C CPython parity tests

## Milestone 4 (`sifr.math`)

- Implement:
  - missing math functions from CPython list
  - signature fixes (`log` base, `isclose` abs_tol, tuple returns)
  - semantic fixes (`dist`, `fsum`, `nextafter`, `ulp`)
  - edge handling policy aligned with architecture divergence table
- Port tests:
  - expand from current subset to broad coverage from `test_math.py`
- Demo:
  - `demos/m30_1b_math_parity_demo/main.sifr`
- PR plan:
  - PR4-A API/sig completion
  - PR4-B numerical behavior fixes
  - PR4-C parity tests + corpus/safety checks

## Milestone 5 (`sifr.hashlib`)

- Implement:
  - object model (`update`, `digest`, `hexdigest`, `copy`, metadata attrs)
  - `new`, `algorithms_guaranteed`, `algorithms_available`, `file_digest`
  - additional algorithms (SHA3/SHAKE minimum for parity scope)
- Port tests:
  - selected vectors and API behavior from `test_hashlib.py`
- Demo:
  - `demos/m30_1a_hashlib_parity_demo/main.sifr`
- PR plan:
  - PR5-A hash object runtime model
  - PR5-B API/algorithms expansion
  - PR5-C parity tests + stress tests

---

## Definition of Done (Per Milestone)

- CPython parity tests for the module pass (ported subset explicitly tracked).
- Safety contract enforced:
  - no panic on invalid input paths
  - `Result`/`Option` used where CPython raises
- Existing e2e suite remains green.
- Demo file exists in `demos/` and runs successfully.
- Deviation notes documented when intentionally different per architecture.

---

## `/project-workflow` Execution Loop (for each milestone)

1. `/create-task` for each PR slice in the milestone.
2. `/add-ticket` to board (Backlog -> Ready via `/refinement`).
3. `/work-on-ticket` for implementation PR.
4. Run module-focused parity tests + safety tests + regression e2e.
5. Create milestone demo in `demos/<milestone>_demo`.
6. `/review-pr` and address findings.
7. Merge PR, move ticket to Done.
8. Repeat until all milestone tasks are complete.

---

## Master TODO (Implementation Checklist)

- [x] M0: env panic-proof validation path (`env_get`, `env_set`, new `unsetenv`)
- [x] M0: bytes boundary design approved (type + adapters)
- [x] M0: parity harness helpers for CPython vector tests
- [x] M1: env API parity subset implemented
- [x] M1: env CPython tests ported and passing
- [x] M1: env demo created and validated
- [x] M2: bytes safety fixes (no wraparound, robust hex)
- [x] M2: bytes parity subset methods implemented
- [x] M2: bytes tests ported and passing
- [x] M2: bytes demo created and validated
- [x] M3: base64 API expansion (b64/standard/urlsafe/b32/b32hex/b16)
- [x] M3: base64 strict decode options implemented
- [x] M3: base64 tests ported and passing
- [x] M3: base64 demo created and validated
- [x] M4: math missing function surface completed
- [x] M4: math signature and semantic fixes completed
- [x] M4: expanded `test_math` parity port passing
- [x] M4: math demo created and validated
- [x] M5: hashlib object model implemented
- [x] M5: hashlib API/algorithm parity subset completed
- [x] M5: hashlib tests ported and passing
- [x] M5: hashlib demo created and validated
- [x] Final: all milestone PRs reviewed/merged, board moved to Done

---

## Detailed Milestone Workboard (No Shortcuts)

### Milestone 0 (Foundation)

- [x] PR0-A Root-cause safety hardening for env intrinsics
  - [x] Prove panic path (`std::env` invalid key/value) from current implementation
  - [x] Add input guards in codegen to prevent panic
  - [x] Add env unset/list helper intrinsics needed for parity work
  - [x] Add regression e2e tests for invalid key handling
  - [x] Validate with focused test run
- [x] PR0-B Bytes boundary design + adapters
  - [x] Document bytes boundary representation trade-offs
  - [x] Implement minimal conversion adapters used by base64/hashlib work
  - [x] Add tests for out-of-range byte rejection behavior
- [x] PR0-C CPython parity harness helpers
  - [x] Add reusable vector-driven parity test helper pattern
  - [x] Add baseline fixture format for CPython expected outputs
  - [x] Wire first parity harness tests
- [x] Milestone demo
  - [x] `demos/m0_parity_foundation_demo.sifr` runs and shows safety behavior

### Milestone 1 (`sifr.env`)

- [x] PR1-A API and intrinsic parity baseline
  - [x] `unsetenv` support
  - [x] `getenv(key, default)` helper
  - [x] keys/values/items helper APIs
  - [x] Keep backward compatibility with `env_get`/`env_set`
- [x] PR1-B CPython test ports (environment-focused subset)
  - [x] Port `putenv/unsetenv` invalid-name cases (adapted to Sifr safety semantics)
  - [x] Port getenv default behavior cases
  - [x] Port mapping-like listing behavior checks
- [x] PR1-C Documentation/deviation notes
  - [x] Record differences from full CPython mapping semantics
  - [x] Record safety adaptation for invalid key/value handling
- [x] Milestone demo
  - [x] `demos/m30_1a_env_parity_demo/main.sifr` passes

### Milestone 2 (`sifr.bytes`)

- [x] PR2-A Safety correctness fixes
  - [x] Reject out-of-range integers for byte conversion
  - [x] Remove panic paths in hex parsing
  - [x] Align hex decode errors with CPython-like positions/messages where feasible
- [x] PR2-B API expansion subset
  - [x] Add high-value byte utilities used by base64/hashlib parity work
  - [x] Keep behavior predictable with Result/Option safety model
- [x] PR2-C CPython test subset port
  - [x] fromhex/hex encoding-decoding core tests
  - [x] decode error-path tests
- [x] Milestone demo
  - [x] `demos/m30_1a_bytes_parity_demo/main.sifr` passes

### Milestone 3 (`sifr.base64`)

- [x] PR3-A API expansion
  - [x] `standard_b64encode/decode`
  - [x] `b32hexencode/decode`
  - [x] `b16encode/decode`
  - [x] `encodebytes/decodebytes`
- [x] PR3-B strictness and parameter parity
  - [x] altchars support
  - [x] validate/ignorechars behavior
  - [x] wrapcol behavior
- [x] PR3-C CPython parity tests
  - [x] RFC vectors (`cpython_base64_rfc4648_vectors.sifr`)
  - [x] invalid input behavior (`cpython_base64_subset.sifr`, `cpython_base64_strictness_subset.sifr`)
  - [x] round-trip stress/property-like cases (deterministic payload matrix in `cpython_base64_rfc4648_vectors.sifr`)
- [x] PR3-C progress note: added full RFC4648 vector assertions + deterministic roundtrip matrix; also aligned `b16encode` output to uppercase for CPython parity.
- [x] Milestone demo
  - [x] `demos/m30_1a_base64_parity_demo/main.sifr` passes

### Milestone 4 (`sifr.math`)

- [x] PR4-A Missing API surface
  - [x] `cbrt`, `exp2`, `fma`, `fmax`, `fmin`, `isnormal`, `issubnormal`, `remainder`, `signbit`, `sumprod`
  - [x] `log` base support via `log_base(x, base)` adapter (`log(x)` kept for compatibility until language-level optional args/overloads are available)
  - [x] `isclose(..., abs_tol)` support (explicit 4-arg form in current Sifr surface)
- [x] PR4-B Semantic corrections
  - [x] `dist` dimension mismatch behavior (safe `NaN` on length mismatch)
  - [x] robust `fsum` strategy (Neumaier-style compensated summation + NaN/Inf handling)
  - [x] accurate `nextafter` and `ulp` edge behavior
  - [x] tuple-return semantics (`frexp`, `modf`) adaptation via pair helper accessors (`frexp_mantissa`/`frexp_exponent`, `modf_fractional`/`modf_integral`)
- [x] PR4-C Test-port completion
  - [x] Broad `test_math.py` subset migrated (`cpython_math.sifr`, `cpython_math_extended.sifr`, `cpython_math_missing_surface_subset.sifr`, `cpython_math_semantic_corrections_subset.sifr`, `cpython_math_parity_expanded_matrix.sifr`)
  - [x] divergence tests for intentional Sifr differences (safe-`NaN` paths for non-`Result` APIs such as `dist` mismatch and invalid `remainder` cases)
- [x] Milestone demo
  - [x] `demos/m30_1b_math_parity_demo/main.sifr` passes

### Milestone 5 (`sifr.hashlib`)

- [x] PR5-A Hash object model
  - [x] object constructors + incremental `update`
  - [x] `digest`/`hexdigest`/`copy` (with `copy_hash()` adapter due current method return-type constraints)
  - [x] metadata fields parity (`name`, sizes)
- [x] PR5-B API/algorithm parity
  - [x] `new`, `algorithms_guaranteed`, `algorithms_available`, `file_digest`
  - [x] SHA3/SHAKE constructors parity subset (explicit unsupported constructor stubs returning `ValueError` for current runtime)
- [x] PR5-C CPython parity tests
  - [x] known vectors
  - [x] object behavior
  - [x] file digest and error-path tests
- [x] Milestone demo
  - [x] `demos/m30_1a_hashlib_parity_demo/main.sifr` passes

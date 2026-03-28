# Contradictions and Remediation Plan

Generated: 2026-02-17

## Evidence-Backed Contradictions

## 1) Fallible stdlib operations panic instead of returning `Result`/`Option` (High)

**Architecture contract**
- `internal_docs/architecture.md` states:
  - no panics in normal user code,
  - fallible stdlib operations should return `Result`/`Option`,
  - `assert` is the only intended panic escape hatch.

**Current implementation**
- Intrinsic typing exposes many fallible APIs as infallible return types:
  - `read_text(path) -> str`, `json_loads(s) -> str`, `toml_parse(text) -> str`, etc. in `crates/sifr_hir/src/stdlib.rs`.
- Codegen emits direct `unwrap` in intrinsic runtime paths:
  - `read_text`: `std::fs::read_to_string(...).unwrap()`
  - `json_loads`: `serde_json::from_str(...).unwrap()`
  - `regex::Regex::new(...).unwrap()`
  - `toml parse`: `parse().unwrap()`
  - base64/utf8 decode: `decode(...).unwrap()`, `String::from_utf8(...).unwrap()`
  - see `crates/sifr_codegen/src/lib.rs` intrinsic match arms.

**Impact**
- Runtime panic behavior can still occur for normal invalid input in stdlib-facing code.
- This bypasses architecture-level mandatory error handling.

---

## 2) Silent clone insertion contradicts explicit clone policy (High)

**Architecture contract**
- Escape analysis should diagnose escaping references rather than silently cloning.

**Current implementation**
- Codegen adds `.clone()` in some ownership-sensitive returns:
  - TypeVar return case.
  - returning `self` from class methods under borrowed receivers.
- See `crates/sifr_codegen/src/lib.rs` return emission logic.

**Impact**
- Ownership behavior differs from explicit-user-choice model.
- Hidden clones can mask move/borrow design errors and blur performance expectations.

---

## 3) Method receiver contract includes consumptive `self`, implementation does not (Medium-High)

**Architecture contract**
- Receiver modes should include:
  - `&self` for read-only,
  - `&mut self` for mutation,
  - `self` (move) for consuming methods.

**Current implementation**
- Method codegen currently selects only `&self` or `&mut self` based on field assignment detection.
- No emitted consumptive receiver mode for regular class methods.

**Impact**
- Builder/consuming patterns are not represented as designed.
- Some move-centric APIs are forced into clone/borrow workarounds.

---

## 4) For-loop contract vs implementation cost model drift (Medium)

**Architecture contract**
- `for item in collection` is documented as borrow-preserving iteration.

**Current implementation**
- For list/dict iteration, codegen emits cloned elements:
  - list: `.iter().cloned()`
  - dict: `.keys().cloned()`
  - string: `.chars().map(|c| c.to_string())`

**Impact**
- Correct non-consuming behavior is preserved, but cost/ownership details differ from plain borrow-only expectation.

---

## 5) Stdlib ownership feature coverage is narrow (Medium)

**Observation**
- `lib/sifr` exported APIs currently use no explicit `own`/`mut` parameter annotations.
- Many would-be mutating APIs are shaped as copy-return functional operations.

**Impact**
- Borrow checker is not heavily exercised by stdlib public API surface.
- Fewer real-world guardrails for ownership-transfer and mutable-borrow interactions in stdlib usage.

---

## 6) Receiver mutability inference only checks field assignment shape (Low-Medium)

**Current behavior**
- Method mutability inference uses field assignment presence (`self.field = ...`) signal.
- This works for many cases, but can miss more indirect mutation patterns.

**Impact**
- Some edge mutation paths may not be inferred ideally without richer mutation analysis.

## Remediation Plan (Prioritized)

## P0 - Safety contract compliance for fallible intrinsics

1. Update intrinsic signatures in `crates/sifr_hir/src/stdlib.rs` to return `Result`/`Option` where fallible.
2. Replace `unwrap` emission in `crates/sifr_codegen/src/lib.rs` intrinsic arms with error-returning expressions.
3. Ensure diagnostics and mandatory handling apply consistently in lowered/typed code.
4. Add e2e tests for negative paths (bad regex, invalid JSON/TOML, bad file path, invalid hex/utf8).

## P1 - Remove silent clone policy violations

1. Replace auto-clone return rewrites with diagnostics in cases where ownership escapes borrowed context.
2. Keep explicit `.clone()` as user-level requirement where needed.
3. Add e2e fail tests proving diagnostic quality for escape/return cases.

## P1 - Add consumptive method receiver mode (`self`)

1. Extend method lowering/codegen receiver inference to include consume mode.
2. Define deterministic consume detection rule (explicit annotation or body-based consume criteria).
3. Add class API tests for builder/consume semantics and post-move invalidation.

## P2 - Clarify/align for-loop semantics

1. Decide intended semantics:
   - borrow-only without per-element clone, or
   - borrow-collection + clone-element model as language policy.
2. Align architecture docs and codegen accordingly.
3. Add micro tests to lock expected move/borrow behavior in loops.

## P2 - Increase stdlib ownership coverage

1. Introduce selected stdlib APIs with explicit `mut`/`own` where semantically appropriate.
2. Keep functional-style alternatives where desirable, but add ownership-explicit variants.
3. Expand stdlib e2e set to include:
   - ownership transfer into stdlib functions,
   - mutable borrow APIs,
   - use-after-move checks around stdlib calls.

## Suggested Exit Criteria

- No fallible intrinsic path emits panic-by-default behavior.
- No implicit ownership-preserving `.clone()` inserted where architecture requires diagnostics.
- Receiver inference supports and validates consume mode.
- Stdlib includes explicit ownership/mutability APIs and tests that exercise them.
- Architecture and generated behavior are aligned for loop semantics and ownership costs.

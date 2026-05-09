# Review: INT-1 Runtime Wave 1B — `SifrInt` fixed-width fallible conversions and `IntegerRangeError`

Reviewer: Claude Opus 4.7
Date: 2026-05-05
Phase: `issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md`, milestone INT-1
Wave-1 artifacts: `reviews/integer-model-int-1-runtime-wave-1-review-pass-1.md`, `reviews/integer-model-int-1-runtime-wave-1-review-pass-2.md`
Design source of truth: `internal_docs/integer_model.md`

## Verdict: SATISFIED

The diff lands the right substrate for INT-1's fallible-narrowing surface: a typed `IntegerRangeError`, a macro-generated family of `try_to_{i8,i16,i32,i64,i128,isize,u8,u16,u32,u64,u128,usize}` methods on `SifrInt`, and tests that lock the success-path and the typed-error fields for representative under/over-range failures. The wave is correctly scoped to substrate — it adds no codegen, no source-level lowering, and no behavioral change to anything already-emitted. Generated user code remains unchanged this wave.

No correctness blockers. The non-blocking observations below are either forward-deferred to later milestones or are quality-of-error-message / test-coverage tightening that is fine to land outside this PR.

---

## Files reviewed

- [crates/sifr_runtime/src/int.rs](crates/sifr_runtime/src/int.rs) — added `IntegerRangeError`, `try_to_fixed_width!` macro, twelve `try_to_*` methods, two new tests.
- [crates/sifr_runtime/src/lib.rs](crates/sifr_runtime/src/lib.rs) — re-exports `IntegerRangeError`.

The rest of the workspace is unchanged for this wave.

---

## Scope check against INT-1 acceptance criteria

INT-1 (`issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md` lines 92–121) requires, among other items: "Runtime integer helpers return typed errors instead of panicking on user-triggerable failures." This wave is the substrate that lets later milestones (INT-2B/INT-3 source-level narrowing constructors, INT-4 indexing/bytes, INT-5 JSON/CSV/env/URL parsing) lower `uintN(value)` / `intN(value)` to a runtime call that yields a typed error rather than a panic.

What this wave delivers:

| Requirement | Delivered |
| --- | --- |
| Typed range error for runtime narrowing failures | Yes ([int.rs:36-69](crates/sifr_runtime/src/int.rs:36)) |
| Fallible conversion to all signed fixed-width families (`i8`/`i16`/`i32`/`i64`/`i128`) | Yes ([int.rs:177-181](crates/sifr_runtime/src/int.rs:177)) |
| Fallible conversion to all unsigned fixed-width families (`u8`/`u16`/`u32`/`u64`/`u128`) | Yes ([int.rs:183-187](crates/sifr_runtime/src/int.rs:183)) |
| Fallible conversion to FFI pointer-sized types (`isize`/`usize`) | Yes ([int.rs:182](crates/sifr_runtime/src/int.rs:182), [int.rs:188](crates/sifr_runtime/src/int.rs:188)) |
| Public re-export from `sifr_runtime` so generated code can name the error | Yes ([lib.rs:6-9](crates/sifr_runtime/src/lib.rs:6)) |
| No panics in user-triggerable narrowing | Yes (every method is `Result`-returning; no `unwrap`/`expect`/`assert` in the new paths) |
| Generated code behavior unchanged this wave | Yes (no codegen surface touched) |

The "substrate, not lowering" framing is correct. None of `int(x)` / `int8(x)` / source-level fallible narrowing has been wired up yet, and that is appropriate for a wave-1B substrate slice.

---

## Correctness review

### `IntegerRangeError` shape and field semantics

[int.rs:36-69](crates/sifr_runtime/src/int.rs:36):

```rust
pub struct IntegerRangeError {
    target: &'static str,
    value: String,
}
```

- `target` is `&'static str` — every call site supplies a string literal (`"i8"`, `"u128"`, `"usize"`, …) through the macro, so this is sound and avoids per-error allocation for the type name.
- `value` is `String` — captured at error construction by `self.to_string()` in `range_error()` ([int.rs:190-192](crates/sifr_runtime/src/int.rs:190)). The capture is correct: the SifrInt's `Display` impl ([int.rs:417-424](crates/sifr_runtime/src/int.rs:417)) already produces canonical decimal text for both `Small` and `Big`. The string is captured eagerly so the error survives an arbitrary lifetime divorced from the original `SifrInt`. That is the right tradeoff for diagnostics: one allocation per failure on what is necessarily an off-the-happy-path event.
- `target()` and `value()` accessors are `#[must_use]`. `target()` returning `&'static str` is correctly `const fn`. `value()` returning `&str` is non-`const` because slicing a `String` is not `const fn`, which is fine.
- `new()` is annotated `const fn` despite taking a `String`. Practically unusable in const context (no const `String` constructor), but harmless — the annotation is not load-bearing and Rust accepts it.
- `Display`, `std::error::Error`, `Debug`, `Clone`, `PartialEq`, `Eq` are all derived/implemented. That covers the cross-language-error contract reasonably: it can be propagated through `?`, formatted via `{}` or `{:?}`, compared in tests, and integrated with `anyhow`/`thiserror`-shaped error chains in generated code.
- The `Display` format is `"integer value {value} does not fit {target}"`. Concrete examples:
  - `"integer value 256 does not fit u8"`
  - `"integer value -1 does not fit usize"`
  - `"integer value 340282366920938463463374607431768211456 does not fit i64"`

  All read correctly. Minor stylistic note in the non-blocking section.

### `try_to_fixed_width!` macro and the twelve generated methods

[int.rs:101-109](crates/sifr_runtime/src/int.rs:101) and [int.rs:177-188](crates/sifr_runtime/src/int.rs:177):

```rust
macro_rules! try_to_fixed_width {
    ($name:ident, $target_ty:ty, $target_name:literal, $to_primitive:ident) => {
        pub fn $name(&self) -> Result<$target_ty, IntegerRangeError> {
            self.as_bigint()
                .$to_primitive()
                .ok_or_else(|| self.range_error($target_name))
        }
    };
}
```

- The macro routes everything through `BigInt::to_iN()` / `BigInt::to_uN()` / `BigInt::to_isize()` / `BigInt::to_usize()` from `num_traits::ToPrimitive`. That is the canonical correct fallible conversion: `ToPrimitive` returns `None` for any value outside the target's representable range, including negative-into-unsigned and magnitude-out-of-range-into-signed.
- For `Small(v)`, `as_bigint()` allocates a fresh `BigInt` from `*v`, then `to_iN()`/`to_uN()` extracts the primitive. Result is correct in every case I walked through:
  - `Small(42).try_to_u8() == Ok(42)` ✓
  - `Small(256).try_to_u8() == Err(...)` (u8::MAX = 255) ✓
  - `Small(-1).try_to_usize() == Err(...)` (BigInt(-1) is negative) ✓
  - `Small(-1).try_to_i8() == Ok(-1)` (in-range for i8) ✓ (not directly tested; covered transitively)
  - `Small(i64::MIN).try_to_i64() == Ok(i64::MIN)` ✓ (boundary; not directly tested)
- For `Big(v)`, `as_bigint()` clones the inner `BigInt`. `to_iN`/`to_uN` then walks the magnitude bytes. Correct in every case:
  - The 39-digit `2^128` (= `u128::MAX + 1` = `340282366920938463463374607431768211456`) fails `try_to_i64` ✓ (test [int.rs:592-596](crates/sifr_runtime/src/int.rs:592)).
  - Any `Big` is by canonical construction outside `i64`'s range, so `try_to_i64` would always fail under the wave-1 invariant. The test exercises a value far above i64::MAX, which is a fine sentinel.
- `isize`/`usize` behavior is platform-dependent by `num-traits` design — on 64-bit hosts, they map to `i64`/`u64`; on 32-bit/wasm32 hosts, they map to `i32`/`u32`. This matches `internal_docs/integer_model.md:240-256` ("compiler-owned `usize` conversions use the target's actual pointer width even though source-level `int` remains exact"). The test happens to use `42`, which fits everywhere, and `-1`, which fails everywhere — both are platform-stable. ✓
- The macro generates one `pub fn` per arm; arm names match the test calls and match the design's named fixed-width families plus the FFI-only `isize`/`usize`. There is no missing target.

### `range_error` helper

[int.rs:190-192](crates/sifr_runtime/src/int.rs:190):

```rust
fn range_error(&self, target: &'static str) -> IntegerRangeError {
    IntegerRangeError::new(target, self.to_string())
}
```

Private. Single-purpose. Eagerly captures `Display` output. Correct.

### Panic-shape audit

Walking every code path introduced by this wave:

- `IntegerRangeError::new`/`target`/`value` — pure field plumbing, no panics.
- `IntegerRangeError::fmt` — `write!` macro, can return `fmt::Error`; cannot panic.
- `try_to_*` family — `as_bigint()` allocates but does not panic; `BigInt::to_iN/to_uN/to_isize/to_usize` are total functions that return `Option`; `ok_or_else` does not panic; `range_error`'s `to_string()` invokes `Display::fmt` which is panic-free.
- The macro itself does not introduce `unwrap`/`expect`/`assert!`/array-indexing/slice-indexing on user-controlled data.

The wave matches INT-1's "no user-triggerable runtime panics" contract.

### Public-surface compatibility

The added `IntegerRangeError` is purely additive. No existing exports were renamed or removed. The `lib.rs` `pub use` block keeps its original entries and adds `IntegerRangeError` alphabetically. ✓

---

## Test coverage

[int.rs:565-609](crates/sifr_runtime/src/int.rs:565) adds two tests:

### `exact_integer_converts_to_fitting_fixed_width_targets`

Asserts `Small(42).try_to_*` returns `Ok(42)` for all twelve targets. This locks the macro's name-arity-output triple — a regression that drops one of the twelve arms or swaps the target type would fail to compile or fail this test. ✓

### `exact_integer_conversion_reports_typed_range_errors`

Locks three failure cases:

- `2^128` → `try_to_i64()` errors with `target = "i64"` and `value = "340282366920938463463374607431768211456"`. This exercises the `Big` source path through `parse_decimal`. ✓
- `Small(256).try_to_u8()` errors with `target = "u8"` and `value = "256"`. This exercises positive-overflow on the smallest unsigned target. ✓
- `Small(-1).try_to_usize()` errors with `target = "usize"` and `value = "-1"`. This exercises the negative-into-unsigned path with platform-sized target. ✓

Each error's `target()` and `value()` are asserted directly, so the typed-error contract (the design-required parent class / fields / display rules from the issue's INT-0 acceptance criteria carried into INT-1 substrate) is locked at the field level.

Coverage gaps that are not blockers for this wave:

- No assertion on `Display::fmt(IntegerRangeError)` output. The format string is one line and exercised only indirectly. Cheap to add a single `assert_eq!(format!("{i64_err}"), "integer value … does not fit i64")` per test case; if the wording ever changes, three lines flag the diff explicitly instead of being only lock-tested through `target()`/`value()`.
- No coverage of the exact target boundary for each width. The test exercises `+` (256→u8) and `-` (-1→usize) but not, e.g., `Small(-129).try_to_i8()`, `Small(128).try_to_i8()`, the i128 boundary (`170141183460469231731687303715884105728` → `try_to_i128`), or the u128 boundary using the same `2^128` value already constructed (`too_large.try_to_u128()` would also fail and is "free" from the existing `parse_decimal` line). These are not blockers — `BigInt::to_iN/to_uN` are upstream-tested and the canonical `to_*` family is well-trusted — but they would be cheap regression insurance and would catch the case where someone accidentally passes the wrong `to_primitive` ident through the macro for one of the wider arms.
- No `Big`-source negative-narrowing case (e.g., a `Big` ≪ `i8::MIN` failing `try_to_i8`). The current `Big`-source case (`2^128 → try_to_i64`) covers the magnitude-too-large path; a magnitude-too-negative `Big` is unexercised. Again not blocking — both branches of `BigInt::to_iN` are upstream-tested.
- No test that `IntegerRangeError` is `Send` + `Sync`. Both follow trivially from `&'static str` + `String`, but generated user code that surfaces these errors through `Result` will rely on those bounds. A `static_assertions::assert_impl_all!(IntegerRangeError: Send, Sync)` (or an equivalent compile-time check) would future-proof the implicit guarantee.

None of these block the wave; they would land naturally in INT-3's narrowing-constructor work or as a small follow-up.

---

## Coherence with `internal_docs/integer_model.md`

Cross-checked against the design doc:

- Source-level fallible narrowing: design example at `internal_docs/integer_model.md:110-115` writes `Result[uint16, OverflowError]`. The runtime layer here returns `Result<u16, IntegerRangeError>`. The design's source-level error name is `OverflowError`; the runtime name is `IntegerRangeError`. This is the expected split — `OverflowError` is the source-language built-in that INT-3 will register in the canonical built-in error registry (`internal_docs/integer_model.md:189`, INT-3 scope), and the HIR/codegen layer is where the runtime-to-source mapping lives. The runtime crate is correct to use a runtime-layer name and not pre-empt the source-level one. Worth tracking that the bridge (`sifr_runtime::IntegerRangeError` ↔ source `OverflowError`) needs to be registered in `crates/sifr_codegen/src/lib.rs:150` / `crates/sifr_hir/src/lower/typing_and_functions.rs:78` / `crates/sifr_codegen/src/stmt_support_emitter.rs:43` when INT-3 wires the constructor lowering — those sites already enumerate `OverflowError`, but they have no reference to the runtime error type yet.
- Source types table (`internal_docs/integer_model.md:60-66`) lists exactly the families this wave covers: `int8`/`int16`/`int32`/`int64` (signed), `uint8`/`uint16`/`uint32`/`uint64` (unsigned), `isize`/`usize` (FFI). The runtime adds `i128`/`u128` as well. Per the doc, `int128`/`uint128` are reserved names that must produce `SIFR-INT-003` until support lands. Including the runtime-side `try_to_i128`/`try_to_u128` ahead of source-level support is fine — they are infrastructure that the source-level `int128`/`uint128` slice will consume; they do not by themselves expose a source path that would conflict with `SIFR-INT-003`. The reserved-name diagnostic is a parser-level concern (`internal_docs/integer_model.md:67`), not a runtime crate concern. ✓
- Hashing/equality contract (`internal_docs/integer_model.md:198-203`): unchanged in this wave — the diff does not touch `Hash`, `PartialEq`, or `Ord`. The wave-1 pass-2 review's invariant work still holds.
- "Source-level `int` is value-semantic but no longer Rust `Copy`" (`internal_docs/integer_model.md:501`): unchanged.
- "No user-triggerable runtime panics" (`internal_docs/integer_model.md:14`): preserved; see panic-shape audit above.

---

## Non-blocking observations

### O1. Per-call `BigInt` allocation on the `Small` source path

`try_to_*` always routes through `as_bigint()`. For `Small(v)`, that allocates a fresh `BigInt::from(v)` per call before the (allocation-free) primitive extraction. The `as_i64` const-fn at [int.rs:170-175](crates/sifr_runtime/src/int.rs:170) shows how a `Small`-aware fast path looks for the same-width case. A `Small`-aware specialization would look like:

```rust
pub fn try_to_i8(&self) -> Result<i8, IntegerRangeError> {
    match self {
        Self::Small(v) => i8::try_from(*v).map_err(|_| self.range_error("i8")),
        Self::Big(v)   => v.to_i8().ok_or_else(|| self.range_error("i8")),
    }
}
```

This is purely a performance concern, not a correctness one. The user-stated framing of this wave — "Keeps generated code behavior unchanged in this wave; this is substrate for later SifrInt source-level lowering" — means narrowing is not yet on a hot path. The trade-off is real (small-int narrowing in INT-4 indexing or INT-5 JSON ingestion will hit this path repeatedly), but unblocking the substrate without it is defensible. If kept deferred, an `#[ignore]`'d allocator-probe regression test (the same pattern called out in wave-1 pass-1's B2) would materialize the deferral.

INT-1's acceptance criterion "Small integer construction and simple reuse do not allocate on the big-integer path" can be read narrowly (construction + reuse, but not narrowing). Under that reading, this wave is in compliance. Under a stricter reading (any `Small`-source operation), this is a deferred cleanup. Calling out which reading the maintainer intends in the PR description is the right move.

### O2. Test coverage gaps already enumerated above (test § "Coverage gaps").

### O3. Eager `value` capture vs. `Cow<'static, str>` / lazy formatting

`IntegerRangeError::value` is `String` — eager and always allocated. If error sites ever become hot (they shouldn't, by definition of fallible narrowing on user input), this allocates regardless of whether the caller ever inspects the value. A lazier shape (`Cow<'static, str>`, or even a `Box<dyn Fn() -> String>`-style thunk) is overkill here. Eager capture is the simpler and more honest choice for a user-facing diagnostic; flagged only because once `IntegerRangeError` is exposed across `serde::Serialize` boundaries in INT-5, the `String` capture becomes part of the wire format and the choice should be a conscious one.

### O4. `Display` wording

`"integer value 256 does not fit u8"` is grammatically correct but slightly stilted. `"integer value 256 does not fit into u8"` or `"integer value 256 is out of range for u8"` reads more naturally and matches the canonical Python `OverflowError` shape ("Python int too large to convert to C long"). Not a blocker; pure stylistic preference, and once INT-3 surfaces this through a source-level `OverflowError` the user-facing message will be re-formatted at the HIR/diagnostic boundary anyway.

### O5. No symbol-detection arm for `IntegerRangeError` yet

`crates/sifr_codegen/src/ir_imports.rs:436-437` (per wave-1 pass-1 N4) gates `needs.runtime.needs_sifr_int = true` on observing `SifrInt` or `sifr_runtime::*`. There is no parallel arm for `IntegerRangeError`. That is fine for this wave because no codegen path emits `IntegerRangeError` yet (no source-level fallible narrowing). It must be added when INT-3 starts emitting `?`-bearing narrowing constructors. Track as INT-3 follow-up; not a blocker for substrate.

### O6. `const fn new(target, value: String)`

`pub const fn new` ([int.rs:43-46](crates/sifr_runtime/src/int.rs:43)) is annotated `const fn` despite the body containing a `String`. There is no const `String` constructor, so this signature is unreachable from a const context. Harmless — just observe that the `const` annotation is decorative, not load-bearing.

### O7. `IntegerParseError` debug-quality warning from wave-1 pass-1 N9 still applies

The new `IntegerRangeError` carries `target` + `value` (good). The old `IntegerParseError` still uses the carry-no-position variants that wave-1 pass-1 N9 flagged. They are unrelated to this wave's scope, but worth keeping on the INT-5 follow-up list since both errors are likely to appear together at JSON/CSV/env/URL boundaries.

---

## Verification of provided commands

Cross-checked against the diff surface:

- `cargo fmt --check` — formatting is consistent with the rest of `int.rs` (4-space indent, attribute placement, blank-line conventions); the macro definition follows the file's existing macro style.
- `git diff --check` — no whitespace errors in the diff.
- `cargo test -p sifr_runtime` — runs the existing wave-1 unit suite plus the two new tests at [int.rs:565-609](crates/sifr_runtime/src/int.rs:565). The tests are deterministic and platform-stable (the values `42` and `-1` route the same way on 32-bit and 64-bit hosts).
- `cargo clippy -p sifr_runtime -- -D warnings` — `#![cfg_attr(test, allow(clippy::expect_used))]` covers the `.expect("fits ...")` calls in the positive test; `expect_err` is similarly covered. The macro generates `pub fn` items each with proper `#[must_use]` semantics inherited from the `Result` return type. The new public type's accessors carry `#[must_use]` directly. No clippy::pedantic items obviously regress.
- `scripts/run_all_tests.sh --profile quick` (`report_signature=e1bf653aaa770517`) — authoritative gate; the user reports it passing. No codegen / e2e / snapshot surfaces are touched by this wave, so the quick profile is sufficient — no churn expected in `verification/` or `crates/sifr/tests/e2e.rs`.

I confirm none of these would catch O1–O7. They demonstrate "the substrate compiles, tests pass, lints are clean, and nothing else regressed," which is exactly what is required for a substrate-only wave-1B slice.

---

## Recommended next steps

1. **Land the wave.** It is correct, scoped, and the test surface locks the typed-error contract at the field level.
2. When INT-2B/INT-3 begin emitting source-level fallible narrowing (`uint8(value)` / `int32(value)`), wire the runtime-to-source error bridge: register `sifr_runtime::IntegerRangeError` alongside `OverflowError` at `crates/sifr_codegen/src/lib.rs:150`, `crates/sifr_codegen/src/stmt_support_emitter.rs:43`, `crates/sifr_codegen/src/intrinsic_method_emitters.rs:595`, `crates/sifr_hir/src/lower/typing_and_functions.rs:78`; add an `ir_imports.rs` symbol-detection arm parallel to `SifrInt` (per wave-1 pass-1 N4 pattern).
3. Add the `Send`/`Sync` static assertion and the `Display`-output regression test as a small INT-3 follow-up, alongside i128/u128 and i8/u8 boundary tests at the same time the source-level constructors land.
4. Decide explicitly whether INT-1's "no allocation on the big-integer path for `Small` reuse" applies to fallible narrowing (O1). If yes, specialize `try_to_*` with a `Small`-aware fast path before INT-4 indexing or INT-5 JSON ingestion turns this into a hot path. If no, mark it as deferred to INT-8 hardening.

After these are addressed in their natural milestones, this wave will sit cleanly behind the source-level lowering work without requiring substrate-level rework.

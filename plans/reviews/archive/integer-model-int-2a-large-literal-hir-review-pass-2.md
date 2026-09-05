# INT-2A — Large Integer Literal HIR Capture — Review Pass 2

Reviewer: agent (agent), 2026-05-06.
Branch: `int-2a-lossless-large-int-literals`.
Prior review: [reviews/integer-model-int-2a-large-literal-hir-review-pass-1b.md](reviews/integer-model-int-2a-large-literal-hir-review-pass-1b.md).

## Scope under review

Same diff as pass 1b plus the B1 fix. Ten files touched:

- `crates/sifr_hir/Cargo.toml` — adds `num-bigint = { workspace = true }` (workspace pins `0.4.6` in the root `Cargo.toml:55`).
- `crates/sifr_hir/src/lower/integer_literals.rs` (new) — the canonical-decimal normalizer (`canonical_large_int_literal_text`) and a private `parse_unsigned_integer_literal_text` helper that strips a `0x`/`0o`/`0b` prefix (case-insensitive), removes underscores, and parses via `BigUint::parse_bytes`.
- `crates/sifr_hir/src/lower/mod.rs` — registers the new module.
- `crates/sifr_hir/src/lower/expressions.rs` — `lower_number_literal` now branches `i.as_i64()` → `IntLiteral` else `LargeIntLiteral(canonical_large_int_literal_text(i))`. The helper call replaces the previous unguarded `i.to_string()`.
- `crates/sifr_hir/src/hir_nodes.rs` — `HirExpr::LargeIntLiteral(String)` doc updated to "Canonical decimal integer literal that does not fit in the historical small-literal `i64` slot." `HirExpr::ty()` returns `&Type::Int` for this variant.
- `crates/sifr_hir/src/lower/expressions_tests.rs` — `function_let_value` helper plus an expanded `test_large_integer_literals_lower_losslessly_from_source` covering eight values across radix × magnitude (see "B1 verification" below). The leaf arm of `test_iterator_builtins_lower_to_canonical_iterator_call_nodes` includes the new variant.
- `crates/sifr_hir/src/lower/nonlocal_support.rs`, `crates/sifr_codegen/src/error_refs.rs`, `crates/sifr_codegen/src/hir_analysis/traversal.rs`, `crates/sifr_codegen/src/lower_stmt.rs` — leaf-no-recurse / leaf-no-error-refs / leaf-no-result-flow arms updated to include `LargeIntLiteral(_)`. Same shape as pass 1b.
- `crates/sifr_codegen/src/lib.rs` — unchanged from pass 1b (`.or_else(|| f())` → `.or_else(f)` clippy cleanup).

Validation reported by the user (not re-run by me): `cargo fmt`, `python3 scripts/check_hir_maintainability_guardrails.py` (I re-ran this — `PASS`), targeted unit test, `cargo clippy -p sifr_hir -p sifr_codegen -p sifr_driver -- -D warnings`, `cargo test -p sifr_hir`, and `scripts/run_all_tests.sh --profile quick` (`report_signature=e1bf653aaa770517`).

---

## B1 verification — RESOLVED

Pass 1b's blocking finding was that `LargeIntLiteral(String)` carried three different formats depending on which Ruff lex branch produced the `Int` (decimal digits when stored in `Number::Small(u64)`, decimal digits when `Number::Big` came from the decimal lexer, and a prefixed-and-underscore-bearing lexeme when `Number::Big` came from the radix lexer).

The fix is a single-pass normalizer at HIR lowering time:

```rust
pub(super) fn canonical_large_int_literal_text(value: &Int) -> String {
    let text = value.to_string();
    parse_unsigned_integer_literal_text(&text).map_or(text, |integer| integer.to_str_radix(10))
}
```

I traced every cell of the radix × magnitude matrix that produces a `LargeIntLiteral`:

| source                                | Ruff `Int` storage              | `to_string()` output                  | Helper output (canonical decimal) |
| ------------------------------------- | ------------------------------- | ------------------------------------- | --------------------------------- |
| decimal in `[2^63, 2^64-1]`           | `Number::Small(u64)`            | underscore-free decimal digits        | same digits, re-emitted via `BigUint::to_str_radix(10)` |
| decimal `> 2^64-1`                    | `Number::Big("…")` (no prefix)  | underscore-stripped decimal digits    | re-parses radix-10, re-emits decimal |
| hex/oct/bin in `[2^63, 2^64-1]`       | `Number::Small(u64)`            | decimal digits (prefix dropped)       | re-parses radix-10, re-emits decimal |
| hex/oct/bin `> 2^64-1`                | `Number::Big("0x…"/"0o…"/"0b…")` (prefix and underscores intact) | original lexeme | strips prefix, strips `_`, parses `BigUint` at radix 16/8/2, re-emits decimal |

The helper's case-insensitive prefix matching (`0x`/`0X`/`0o`/`0O`/`0b`/`0B`) covers every radix prefix Ruff's lexer accepts. After this pass, the format of `LargeIntLiteral(String)` is unambiguous, self-describing, and consistent across all source spellings — exactly what the issue's "lossless representation" / "constructed-AST and parsed-source produce equivalent HIR" wording requires.

The new test `test_large_integer_literals_lower_losslessly_from_source` exercises eight points, including:

- decimal `[2^63, 2^64-1]` (`9223372036854775808`)
- decimal `> 2^64-1` (`184467440737095516160`)
- underscored decimal (`1_000_000_000_000_000_000_000`)
- hex in `[2^63, 2^64-1]` (`0xffffffffffffffff` — the cell that previously dropped its prefix)
- hex `> 2^64-1` (`0x10000000000000000`)
- underscored hex `> 2^64-1` (`0xFFFF_FFFF_FFFF_FFFF_FFFF`)
- oct `> 2^64-1` (`0o2000000000000000000000`)
- bin `> 2^64-1` (`0b1` followed by sixty-four zeros)

I sanity-checked each expected canonical decimal in the test against the source value:

- `0xffffffffffffffff` = `18446744073709551615` ✓
- `0x10000000000000000` = `0xFFFF_FFFF_FFFF_FFFF_FFFF`'s neighbor `0x10000000000000000` = `18446744073709551616` ✓
- `0xFFFF_FFFF_FFFF_FFFF_FFFF` = `1208925819614629174706175` (= 16^20 − 1) ✓
- `0o2000000000000000000000` = `2 × 8^21` = `2^64` = `18446744073709551616` ✓
- `0b1` + 64×`0` = `2^64` = `18446744073709551616` ✓
- `1_000_000_000_000_000_000_000` underscored → `1000000000000000000000` ✓

The hex `[2^63, 2^64-1]` cell — the one that exposed the discontinuity in pass 1b — now goes through the `Number::Small(u64::MAX)` → decimal-display → re-parse-as-decimal → re-emit-decimal path and correctly produces `"18446744073709551615"`.

**B1 is resolved.**

---

## New blocker check — none introduced

I looked specifically for issues created by the new normalization step or by splitting the helper into its own module:

- **Workspace plumbing**: `num-bigint = { workspace = true }` in `crates/sifr_hir/Cargo.toml` resolves to the existing root pin at `Cargo.toml:55` (`num-bigint = { version = "0.4.6" }`). No new third-party dependency added at the workspace level.
- **Module wiring**: `crates/sifr_hir/src/lower/mod.rs` registers `mod integer_literals;` in alphabetical position alongside existing siblings; `expressions.rs` imports `super::integer_literals::canonical_large_int_literal_text`. HIR maintainability guardrails pass (re-ran `python3 scripts/check_hir_maintainability_guardrails.py` → `PASS`), so the helper split has not pushed `expressions.rs` over any size threshold and the new file is well under it.
- **Exhaustive-match plumbing**: identical to pass 1b. The same six leaf arms (`hir_expr_calls_function`, `collect_expr_error_refs`, `process_expr_node` / `validate_expr_lowering_shape` / `expr_has_result_flow`, plus the test's leaf catch-all) include `LargeIntLiteral(_)`. All other consumers fall through wildcards (verified again — see the matrix in pass 1b which is unchanged).
- **Helper correctness**:
  - `BigUint::parse_bytes` accepts uppercase + lowercase hex digits, so the helper does not need to lowercase before parsing.
  - The `.map_or(text, ...)` fallback is defensive; for valid Sifr source it is unreachable because Ruff has already validated digits at lex time. It does not silently corrupt anything — it preserves the Ruff display, which is the same behavior as before the fix on the unreachable branch.
  - The function is only called when `i.as_i64()` returns `None`, so it is never asked to canonicalize values that already round-trip through `IntLiteral(i64)`.
  - Empty digits after prefix strip cannot occur (Ruff rejects e.g. `0x` with no digits at lex time).
  - Negative literals are wrapped in `UnaryOp("-", LargeIntLiteral(...))`; the unsigned-only helper is correct.

No new blockers.

---

## Pass 1b non-blockers — should any be upgraded?

The user explicitly asked whether N2/N3/N4/N6 should now be promoted to blockers. My read on each, after the B1 fix:

- **N2 (`lower_expr_simple` default-arg parity at `crates/sifr_hir/src/lower/classes.rs:1249`)**: still calls `i.as_i64()?`, so a large default is silently dropped from a parameter. This now creates an asymmetry inside the same lowering pass: `def f(x: int = 1 << 100): ...` drops the default, while `x = 1 << 100` lowers cleanly to `LargeIntLiteral`. With the canonical normalizer in hand, plumbing this through is mechanical — `lower_expr_simple` can mirror `lower_number_literal`'s `Some(IntLiteral)` / `Some(LargeIntLiteral(canonical_…))` branch. **My judgment: keep as non-blocker for THIS slice as the user has scoped, but it deserves to be the first item of the next INT-2A slice — the parity criterion is what motivates B1, and N2 is the only remaining direct violation of it inside HIR lowering.** If the next slice does not pick this up, I would upgrade it to a blocker before INT-2A is declared closed.
- **N3 (`compile_error!` from codegen for `LargeIntLiteral`)**: still applies. Slice scope explicitly excludes large-literal codegen. Stays non-blocking. Flag for INT-2B/INT-3.
- **N4 (tuple compile-time index/slice diagnostics ignore `LargeIntLiteral`)**: still applies. Behavioral degradation only at indices that cannot be valid for any real tuple; non-blocking. Same INT-2B follow-up.
- **N6 (clippy cleanup commit shape in `crates/sifr_codegen/src/lib.rs`)**: cosmetic, non-blocking. Note it in the PR description.

None of these should block this slice. N2 is the one that I would most strongly want to see as the very next item.

## Pass 1b N1 (test coverage) — mostly addressed

Pass 1b asked for four extra cases: hex `[i64::MAX+1, u64::MAX]`, decimal `> 2^64`, underscored literals in both decimal and hex, and a negative literal beyond `i64::MIN`. The new test covers the first three with extra octal/binary `> 2^64` cases on top.

The negative-literal beyond `i64::MIN` case (e.g. `-9_223_372_036_854_775_809`) is **not** covered. This is the case that asserts the unary-wrapping invariant `UnaryOp("-", LargeIntLiteral("9223372036854775809"))`. Worth adding in the next slice, but non-blocking — `lower_number_literal` does not see the sign, and the unary wrap is unrelated to B1's normalization.

(Sub-observation: in the existing AST, source `-9_223_372_036_854_775_808` (literal `i64::MIN`) lowers to `UnaryOp("-", LargeIntLiteral("9223372036854775808"))` rather than `IntLiteral(i64::MIN)`, because `9_223_372_036_854_775_808 = 2^63` exceeds `i64::MAX` even though its negation fits `i64::MIN`. This is pre-existing — the new code is not the cause — and it is the kind of edge that an INT-2B const-fitting pass needs to handle. Not a blocker for this slice.)

---

## Helper-specific minor observations (all non-blocking)

These are about `crates/sifr_hir/src/lower/integer_literals.rs` itself:

1. The helper has no direct unit tests of its own; it is exercised through `lower_source`. A small `#[cfg(test)] mod tests { ... }` covering the radix-prefix branches and an underscore-only case would localize regressions if the lexer's display contract ever changes. Non-blocking.
2. `text` is allocated once via `value.to_string()`; on the success path we discard it and allocate a fresh decimal string from `to_str_radix(10)`. Two allocations per large literal. Acceptable; large literals are rare. Mention only because it is occasionally useful to thread `Cow<'_, str>` through similar helpers.
3. The `.map_or(text, ...)` fallback trades a panic for a silent passthrough. For an "if Ruff produced this it parses" invariant, an `expect("Ruff lexer produced unparseable integer text")` is also defensible — a programmer-invariant assert is how AGENTS.md describes that case. Either choice is fine; the current one is the safer default.
4. `LargeIntLiteral(String)`'s doc could spell out "no underscores, no sign, no radix prefix; sign is wrapped in `UnaryOp`" — this is how the canonical format is actually shaped today, and naming it forecloses divergence from a future constructed-AST helper. Non-blocking.

None of these change the verdict.

---

## Final verdict

**Approve.** B1 from pass 1b is fully addressed by the canonical-decimal normalizer; no new blockers were introduced by either the BigUint parse step or the helper-module split; exhaustive-match plumbing remains safe; HIR maintainability guardrails still pass.

The slice can merge as-is. Carry the existing pass 1b non-blockers (N1 negative-case test, N2 default-arg parity, N3 codegen diagnostic, N4 tuple-index diagnostics, N5/N6 doc and commit-shape polish) into the next INT-2A slice. Of those, N2 is the most important — it is the last remaining case where the `i64`-only assumption can silently drop a value at HIR lowering time, and the canonical normalizer in this slice has already done the hard part of the fix.

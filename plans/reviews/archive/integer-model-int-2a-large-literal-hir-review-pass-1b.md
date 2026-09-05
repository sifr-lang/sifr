# INT-2A — Large Integer Literal HIR Capture — Review Pass 1b

Reviewer: agent (agent), 2026-05-06.

## Scope under review

Diff against `main` on branch `int-2a-lossless-large-int-literals`. Eight files touched, ~61 LOC:

- `crates/sifr_hir/src/hir_nodes.rs` — adds `HirExpr::LargeIntLiteral(String)` and includes it in `HirExpr::ty()`.
- `crates/sifr_hir/src/lower/expressions.rs` — `lower_number_literal` now falls back to `HirExpr::LargeIntLiteral(i.to_string())` when `Int::as_i64()` returns `None`.
- `crates/sifr_hir/src/lower/expressions_tests.rs` — adds `function_let_value` helper, a new positive test `test_large_integer_literals_lower_losslessly_from_source`, and adds the new variant to the leaf-arm of `test_iterator_builtins_lower_to_canonical_iterator_call_nodes`.
- `crates/sifr_hir/src/lower/nonlocal_support.rs` — adds the new variant to the leaf arm of `hir_expr_calls_function`.
- `crates/sifr_codegen/src/error_refs.rs`, `hir_analysis/traversal.rs`, `lower_stmt.rs` (`validate_expr_lowering_shape`, `expr_has_result_flow`) — add the new variant to existing leaf-no-recurse / leaf-no-result-flow arms.
- `crates/sifr_codegen/src/lib.rs` — unrelated clippy-style refactor (`.or_else(|| f())` → `.or_else(f)`).

Phase context:
- Issue: `issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md` (slice = "Carry INT-2A parser literal lexeme capture, malformed/over-budget literal diagnostics, and parsed/constructed HIR parity in the next INT-2A slice").
- Design: `internal_docs/integer_model.md`.
- Stated slice intent (from the review request): only parser-side lexeme capture for literals that exceed the historical `i64` slot. Fixed-width const fitting and SifrInt arithmetic codegen are explicitly out of scope (INT-2B/INT-3).

## Validation reproduced from the request

The user reports that the following passed before this review:
`cargo fmt --check`, `git diff --check`,
`cargo test -p sifr_hir test_large_integer_literals_lower_losslessly_from_source`,
`cargo test -p sifr_hir`,
`cargo check -p sifr_codegen -p sifr_driver`,
`cargo clippy -p sifr_hir -p sifr_codegen -p sifr_driver -- -D warnings`,
`cargo run -q -p sifr -- check` on a temp file with oversized decimal and hex literals,
`scripts/run_all_tests.sh --profile quick` (report_signature=`e1bf653aaa770517`).

I did not re-run these. I read the affected source paths plus the relevant Ruff lexer (`third_party/ruff/crates/ruff_python_parser/src/lexer.rs`) and `Int` type (`third_party/ruff/crates/ruff_python_ast/src/int.rs`) to characterize what `i.to_string()` actually emits per radix.

---

## Blocking findings

### B1. The "lossless integer-literal representation" is not consistent across radix and magnitude

The new representation is `LargeIntLiteral(String)` populated by `i.to_string()` where `i: ruff_python_ast::Int`. Tracing through the Ruff lexer and `Int`:

- `lex_decimal_number` strips underscores in `radix_run`, then calls `Int::from_str(s)`. `Int::from_str` parses as `u64`; on overflow it stores `Int::big(s)` where `s` is the **underscore-stripped digit string**.
- `lex_number_radix` for hex/oct/bin builds an underscore-stripped digit-only `number` AND captures the raw `token = &source[token_range]` (which still contains the `0x`/`0o`/`0b` prefix and any underscores). It calls `Int::from_str_radix(number, radix, token)`. On overflow it stores `Int::big(token)` — the **full original lexeme, prefix and underscores included**.
- `Int(Number::Small(u64))` displays as decimal (`write!("{value}")`), so any literal that fits in `u64` round-trips through `to_string()` as decimal digits regardless of source radix.

The combined effect of these branches is that `LargeIntLiteral` ends up with *three different* string formats depending on radix and magnitude:

| source                                          | numeric value             | stored string                                  |
| ----------------------------------------------- | ------------------------- | ---------------------------------------------- |
| decimal in `[2^63, 2^64-1]`                     | fits `u64`                | underscore-free decimal digits                 |
| decimal `> 2^64-1`                              | exceeds `u64`             | underscore-free decimal digits (Big path)      |
| hex/oct/bin in `[2^63, 2^64-1]`                 | fits `u64`                | **decimal digits** (radix prefix dropped)      |
| hex/oct/bin `> 2^64-1`                          | exceeds `u64`             | **original lexeme** with `0x`/`0o`/`0b` prefix and any underscores preserved |

The phase scope says "Normalize decimal, hex, octal, and binary literals into a *lossless* integer-literal representation for HIR" and the acceptance criterion says "The constructed-AST path and parsed-source path produce equivalent HIR literal representations." This branch-dependent format violates both intents:

1. The representation is not self-describing. A consumer holding `"18446744073709551615"` cannot tell whether the user wrote `0xffff_ffff_ffff_ffff` or `18446744073709551615`. Once INT-2B starts parsing these strings (e.g., into `num_bigint::BigInt`), it will need to autodetect the radix from the prefix — which is fine for the `> 2^64-1` hex/oct/bin band but fails for the `[2^63, 2^64-1]` band where the prefix has already been lost.
2. Any programmatically-constructed `HirExpr::LargeIntLiteral` (for example the constructed-AST or compile-time-eval path) will need to know which of the three formats to mirror. Today it must pick decimal, but the parsed-source path is inconsistent with that choice in the > `2^64-1` hex/oct/bin band.
3. The current test (`test_large_integer_literals_lower_losslessly_from_source`) only covers values at exactly `2^64`, which happens to land in the "Big" branch for hex/oct/bin (preserving the prefix) and is `< 2^64` for decimal (preserving decimal digits). The test passes, but it does not exercise the discontinuity.

Recommended fix (pick one before INT-2B starts consuming this node):

- (preferred) Always normalize at lowering time. Either parse the lexeme into a canonical decimal digit string (via `num_bigint::BigInt::from_str_radix` or equivalent in `sifr_runtime`), or shadow Ruff's `Int` by digit-radix-walking the token text. Yes, this adds a parse step, but it makes the HIR shape unambiguous and matches the "lossless" / "equivalent representation" wording in the issue.
- OR, store a structured node: `LargeIntLiteral { lexeme: String, radix: u8 }`. Consumers parse using `radix`. Avoids decimal conversion at lowering time but needs every consumer to plumb the radix through.

Either choice is fine; the existing one-arg `LargeIntLiteral(String)` is the bug. Locking the wrong shape now is a visible footgun for INT-2B since the type/const-fitting code will be the first real consumer.

(Sub-case worth keeping in mind once you fix this: in the current code, decimal literals that contained underscores have those underscores lossily stripped at lex time — they are gone from `Int::big(s)` even before `lower_number_literal` runs. Hex/oct/bin > `2^64-1` keeps them. Whichever direction you go, decide on a single underscore policy and document it.)

---

## Non-blocking findings

### N1. Test coverage gaps (closely related to B1)

The new test exercises exactly four points, all at `2^64` for the radix bases and `2^63` for decimal. Worth adding:

- Hex/oct/bin literal in the `[i64::MAX + 1, u64::MAX]` band, e.g. `0xFFFF_FFFF_FFFF_FFFF`. This is the band that loses its radix prefix today and would catch the B1 regression after a fix.
- Decimal literal `> 2^64`, e.g. `184467440737095516160`. The current test happens to skip this; without it, the Number::Big decimal display path is uncovered.
- Underscored literals in both decimal (`1_000_000_000_000_000_000_000`) and hex (`0xFFFF_FFFF_FFFF_FFFF_FFFF`). The asymmetry in underscore preservation is real today — explicitly pin one behavior.
- A negative literal beyond `i64::MIN`, e.g. `-9_223_372_036_854_775_809`, asserting the result is `UnaryOp("-", LargeIntLiteral(...))`. This protects the unary-wrapping invariant since `LargeIntLiteral` carries no sign.

### N2. `lower_expr_simple` parity gap (`crates/sifr_hir/src/lower/classes.rs:1249`)

`lower_expr_simple` is used to lower default parameter values during the first pass:

```rust
Number::Int(i) => Some(HirExpr::IntLiteral(i.as_i64()?)),
```

For `def foo(x: int = 9_223_372_036_854_775_808): ...`, the `?` returns `None`, the entire `lower_expr_simple` returns `None`, and the default is silently dropped (or the surrounding code rejects the parameter — depends on the caller). That's the same kind of i64-only assumption this PR is otherwise fixing. The acceptance criterion "The constructed-AST path and parsed-source path produce equivalent HIR literal representations" is partly about this kind of internal lowering helper.

The user's framing ("INT-2A parsed-source large integer literal HIR capture") suggests this gap is intentionally deferred to a sibling slice — fine, but it should be tracked explicitly. The phase issue's outstanding bullet "Carry INT-2A parser literal lexeme capture, malformed/over-budget literal diagnostics, and parsed/constructed HIR parity in the next INT-2A slice" implicitly covers this; consider linking the file:line so the next slice doesn't miss it.

(One related cousin to mention in the same TODO: `crates/sifr_hir/src/lower/classes.rs:1272` and other places that rebuild a negated literal via `HirExpr::IntLiteral(-v)` only operate on the small path.)

### N3. Codegen path for `LargeIntLiteral` produces `compile_error!` Rust output

This is consistent with the slice intent ("does not implement … full codegen … for INT-2A") but worth flagging because it is silently inherited via fall-through:

- `lower_expr.rs::is_leaf_expr_candidate` and `lower_expr.rs::try_lower_leaf_expr` are not updated, so `try_lower_leaf_or_name_expr(LargeIntLiteral)` returns `None`.
- `try_lower_simple_let_value` therefore returns `None`, the structured `Let` path at `lib.rs:1444` calls `lower_rendered_expr_for_ir` and `lower_stmt_expr_for_ir` (both return `Ok(None)`), falls to `Ok(false)`, and `emit_stmt` produces

  ```rust
  compile_error!("structured statement emission missing for production path: ...")
  ```

A user running `sifr run` or `sifr build` against `x = 9_223_372_036_854_775_808` will hit this — they will see a rustc `compile_error!` macro fire, not a Sifr-side diagnostic. The validation log only mentions `sifr -- check` which never reaches codegen, so this is genuinely untested in the listed gates.

The plumbing is *safe* (no panic, no `unwrap`/`expect` on the new variant on any path I traced — see "Codegen exhaustive-match plumbing safety" below). But the user-visible failure is poor. Two reasonable options:

1. Emit a typed `CodegenError` with a specific code (e.g., a placeholder under `SIFR-INT-0004`/`-0005`/etc., or a generic "large integer literal codegen pending INT-2B/INT-3") at the boundary in codegen, so the message is Sifr-flavored and can be recognized in fail fixtures.
2. Add a fail-fixture-style test exercising `sifr build` against a large literal and asserting the failure mode, even if the assertion is just "rustc failed with compile_error containing X". That at least pins the current behavior so INT-2B notices when it changes.

### N4. Loss of compile-time tuple index/slice diagnostics for large literals

Two places in `crates/sifr_hir/src/lower/expressions.rs` and `crates/sifr_hir/src/lower/subscript_type.rs:49` match `HirExpr::IntLiteral` to give compile-time tuple-index / tuple-slice diagnostics:

- `subscript_type.rs:49`: tuple indexing with an `IntLiteral` that's out of range fires `tuple index out of range`.
- `expressions.rs:1852`/`1915`/`1922`: tuple slicing requires `IntLiteral` start/stop, otherwise emits `tuple slicing requires compile-time constant indices`.

After this PR, `t[9_223_372_036_854_775_808]` and `t[: 9_223_372_036_854_775_808]` lower successfully (good), but the compile-time diagnostic that would catch the obvious out-of-range now fires the more generic "compile-time constant indices" path or doesn't fire at all (the `LargeIntLiteral` branch is missing). This is behavioral degradation, not a correctness break — the cases involve impossibly large indices for any real tuple — so non-blocking, but worth noting in the same INT-2B follow-up that touches integer-literal const eval.

### N5. Inline doc comment on `LargeIntLiteral(String)` is too thin

Right now:

```rust
/// Integer literal that does not fit in the historical small-literal `i64` slot.
LargeIntLiteral(String),
```

The string format is the load-bearing detail of this node and (per B1) is currently radix- and magnitude-dependent. At minimum the doc should say something like "format is implementation-defined and may include `0x`/`0o`/`0b` prefixes; consumers should defer parsing until INT-2B introduces the canonical const-fitting path." Better: document the canonical format the moment B1 is fixed.

### N6. Unrelated clippy-style change in `crates/sifr_codegen/src/lib.rs`

```diff
-        .or_else(|| discover_sifr_runtime_path_from_current_dir())
-        .or_else(|| discover_sifr_runtime_path_from_current_exe())
+        .or_else(discover_sifr_runtime_path_from_current_dir)
+        .or_else(discover_sifr_runtime_path_from_current_exe)
```

Looks like a `clippy::redundant_closure` cleanup. Functionally equivalent, but unrelated to the INT-2A scope and adds noise to the review. Consider splitting into its own commit with a cleanup-style title so reviewers can see the integer-model diff without distraction. Non-blocking.

---

## Codegen exhaustive-match plumbing safety

Asked specifically about. I traced every place the diff added `LargeIntLiteral(_)` and every place it did *not* but matches `IntLiteral`. None of the unhandled paths panic on the new variant; they all either return `None`/`false` (skipping an optimization) or fall through a `_ => ...` arm.

| Path | Patched? | Behavior on `LargeIntLiteral` | Verdict |
| --- | --- | --- | --- |
| `error_refs.rs::collect_expr_error_refs` | yes (leaf no-error-refs arm) | no error refs to collect | safe |
| `hir_analysis/traversal.rs` | yes (leaf no-recurse arm) | no children to walk | safe |
| `lower_stmt.rs::validate_expr_lowering_shape` | yes (leaf "Ok(())" arm) | shape valid | safe |
| `lower_stmt.rs::expr_has_result_flow` | yes (leaf "false" arm) | no Result flow | safe |
| `nonlocal_support.rs::hir_expr_calls_function` | yes (leaf "false" arm) | does not call any function | safe |
| `lower_expr.rs::is_leaf_expr_candidate` | no (wildcard `_ => false`) | not a leaf candidate; structured path taken | safe but unoptimized |
| `lower_expr.rs::try_lower_leaf_expr` | no (returns `None` by fallthrough) | structured path taken | safe but unoptimized |
| `lower_stmt.rs::try_lower_match_literal_pattern` | no (returns `None`) | falls to non-literal pattern path | safe |
| `stmt_support_emitter.rs::negative_range_step_magnitude` | no | range step optimization skipped | safe |
| `stmt_support_emitter.rs::lower_stmt_expr_for_ir` (and its internal registry helpers) | no | returns `Ok(None)` | safe but reaches `compile_error!` fallback (see N3) |
| `intrinsic_method_emitters.rs::try_eval_const_int_expr` | no | returns `None` (no const fold) | safe |
| `helpers.rs::is_reusable_place_expr` (subscript index) | no | treats large-literal-as-index as not reusable | safe |
| `lower/subscript_type.rs::49` (tuple index const eval) | no | skips compile-time index check (see N4) | safe but degraded |
| `lower/expressions.rs::1852`/`1915`/`1922` (tuple slice const eval) | no | skips compile-time slice check (see N4) | safe but degraded |
| `lower/decimal_methods.rs::48`/`50` | no | small-int decimal path returns `None` | safe |
| `lower/arithmetic_warnings.rs` | no | const-shift / pow warnings skipped | safe |
| `lower/expression_iter_builtins.rs::109,114` (range zero-start fast path) | no | optimization skipped | safe |
| `lower/nonempty_method_narrowing.rs::87` (`pop(0)` matcher) | no | narrowing skipped | safe |
| `lower/method_call_args.rs::466` | no | constructs `IntLiteral(0)` only — unaffected | safe |
| `lower/numeric_sentinels.rs::116,350` | no | constructs sentinel `IntLiteral` only — unaffected | safe |

Summary: every place that needed to be updated to keep exhaustive matches compiling was updated (otherwise `cargo check` / clippy would have failed, which they did not). Every place that was *not* updated has wildcard-like fallback that returns `None`/`false`/`Ok(None)`. Net effect is "treat `LargeIntLiteral` as opaque and unsupported" — that aligns with the slice's stated scope. The only user-visible degradation is N3 (compile_error! on codegen) and N4 (tuple compile-time index/slice diagnostics).

---

## Recommendation

Hold for B1 before merging, *or* merge as-is provided the response to B1 lands as the very next slice (before INT-2B starts parsing `LargeIntLiteral` strings). The acceptance criterion that names "lossless representation" + "constructed-AST and parsed-source produce equivalent HIR" is the hard constraint here; if you want to keep it, B1 must be fixed before any consumer interprets the string.

Non-blocking items N1–N6 should be tracked in the existing INT-2A follow-up bullet ("Carry INT-2A parser literal lexeme capture, malformed/over-budget literal diagnostics, and parsed/constructed HIR parity in the next INT-2A slice") rather than this PR.

## Additional verifications I'd run before claiming the slice closed

- `cargo run -q -p sifr -- build` against a `.sifr` file containing a large decimal literal AND a large hex literal (>`2^64`) AND a large hex literal in `[2^63, 2^64-1]`. Verify the failure mode (today: `compile_error!`; after a future codegen slice: a Sifr-side diagnostic).
- Add the four new `lower_source` test cases enumerated in N1 to lock the radix/underscore behavior.
- A constructed-AST round-trip test that builds `HirExpr::LargeIntLiteral("18446744073709551616")` directly (no parser) and asserts it survives codegen consistently with the parsed-source case — that's the parity acceptance criterion in test form.

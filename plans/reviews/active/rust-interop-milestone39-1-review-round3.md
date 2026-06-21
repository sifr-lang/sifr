# Round 3 delta — sign-off

All five L-findings from round 2 are correctly addressed. Local validation (`cargo fmt --check`, `cargo check -p sifr_lowering -p sifr_codegen -p sifr_driver`, the two targeted test runs) is clean — 12/12 lowering tests and 2/2 cache-key tests pass.

## Findings, one by one

- **L1 — owner threaded through value parsing.** `parse_value` now takes `owner: RustInteropOwner` and forwards it into both `parse_target_path` and `parse_policy_call` (`rust_interop.rs:194-218,268-307`). New test `rust_interop_allows_self_targets_in_method_keyword_values` exercises `view=Self.PollView` on a method and confirms the previously-rejected path now lowers. Resolved.
- **L2 — negative integer literals.** New `Expr::UnaryOp(USub, NumberLiteral)` arm plus a shared `parse_integer_value` helper that uses `checked_neg` (so `-i64::MIN` errors cleanly rather than panicking) — `rust_interop.rs:205-216,238-266`. Verified by `rust_interop_lowers_negative_integer_values`. Bonus: `parse_policy_call` now accepts `Expr::UnaryOp` as a policy-call argument, so `policy(-1)` works too. Resolved.
- **L3 — negative-branch test coverage.** All five gaps listed in round 2 are covered (`rust_interop_tests.rs:179-242`): opaque-on-function, async-on-class, `@rust.unknown(...)`, `**kwargs`, and bare `@rust`. Resolved.
- **L4 — non-empty interop plan in cache key.** `binary_project_cache_key_includes_interop_build_plan` (`materialize.rs:297-331`) builds two `GeneratedBinaryProject` values that differ only in `interop` and asserts the resulting cache keys diverge. The `base_project()` helper extraction is a clean refactor. Resolved.
- **L5 — `RustInteropOwner` re-export.** `sifr_codegen/src/lib.rs:86-88` exposes the enum publicly, and the driver test imports it through that path, confirming the API surface is now self-contained. Resolved.

## No new blockers

The delta is additive: no behavior changes to round-2-reviewed code, no new public types beyond `RustInteropOwner`, no nondeterminism added to `cache_key_fragment` (still pure `push_str`/literal delimiters). The two `RustInteropOwner` enums (lowering-internal vs. codegen-public) coexist safely via `pub(in crate::lower)` scoping — worth noting as a naming overlap for future readers, but not a regression.

Cleared to merge for M39.1.

# Phase 39 milestone_39_1 review

## Blockers

**1. `{:?}` Debug-format used as cache key — `crates/sifr_driver/src/build/materialize.rs:239,249`**

```rust
"[interop]\n{:?}\n..."
generated_project.interop,
```

`InteropBuildPlan` is hashed into `binary_project_cache_key` via its `Debug` impl. If any field downstream uses `HashMap`/`HashSet` (or the `RustInteropPlan` itself does), Debug iteration order is non-deterministic and the cache key will flap between identical logical inputs, defeating the binary-project cache. The diff doesn't show `InteropBuildPlan`'s shape, but the milestone requires stable, content-derived metadata. Either: serialize to a deterministic string (sorted JSON / BTreeMap-based fields throughout), or assert in tests that two equal plans produce equal Debug output across runs. Until that's locked in, this is a latent cache-correctness bug.

## Serious gaps (visible from these excerpts)

**2. Diagnostic surface vs. milestone scope — `crates/sifr_diagnostics/src/codes/registry.rs:450-501`**

Ten new families are reserved (`RUST-CONFIG`, `RUST-RESOLVE`, `RUST-TRUST`, `RUST-TYPE`, `RUST-HANDLE`, `RUST-ASYNC`, `RUST-ZC`, `RUST-CB`, `RUST-PANIC`, `RUST-CARGO`) but only one code (`RUST_CONFIG_MALFORMED_DECORATOR`) is active. That matches the stated milestone scope ("stable malformed-decorator diagnostics") but reserving nine empty families up-front is a future-shape commitment that should be cross-checked against the Phase 39 design — partial misnaming here gets baked into user-facing identifiers. Worth a single sign-off pass on the family bases.

**3. Family-name grammar widening — `crates/sifr_diagnostics/src/codes/registry_tests.rs:148-180`**

The canonical-code test was relaxed to accept hyphenated families (`SIFR-RUST-CONFIG-0001`). The new `parse_family` uses `parts[1..parts.len() - 1].join("-")`, which means *any* number of internal hyphens is now accepted. There's no upper bound on segment count and the 16-char family cap is on the joined string, so `A-B-C-D-E-F-G` would pass. Not a blocker for shipping malformed-decorator diagnostics, but the grammar is now under-specified — pin it to "family is `RUST` or `RUST-<SEGMENT>`" with an explicit assertion before more families land.

## Parser change

**4. `rust.async` attribute carve-out — `crates/ruff_python_parser/src/parser/expression.rs:1103-1131`**

The hook only triggers when the LHS is `Expr::Name("rust")` and the next token is `async`. That's tight enough that it won't leak into general expressions, and the test in `parser/tests.rs` covers the decorator form. `parse_non_error_keyword_identifier` bumps unconditionally but is only reachable after `at(TokenKind::Async)`, so it's safe. No issue.

## What I cannot verify from these excerpts

The HIR-metadata side of the milestone — `RustInteropDeclaration` shape (spans/effects/ABI-requirements), the `rust_interop_plan` module, the lowering pass that produces `HirClass.rust_interop`/`HirFunction.rust_interop`, and the decorator value-grammar validator — is referenced (`crate::RustInteropDeclaration`, `crate::rust_interop_plan::interop_build_plan_for_module`) but not present in this diff. The wiring at `entrypoints.rs:195`, `lib_modules_and_codegen.rs:67/80/879`, `lib_project_codegen.rs:80`, and `hir_nodes.rs:68/103` is consistent with the milestone, but the substance of the milestone lives in the files not shown. I can't confirm scope coverage without those.

## Bottom line

Not review-satisfied. The cache-key Debug-format (#1) is a real correctness risk to resolve before merge. #2 and #3 are minor but worth tightening now since they shape the public diagnostic surface. The parser change is fine. Please share the `RustInteropDeclaration`, `rust_interop_plan`, and decorator-validation excerpts for the rest of the review.

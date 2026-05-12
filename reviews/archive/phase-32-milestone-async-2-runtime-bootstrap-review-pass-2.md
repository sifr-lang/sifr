

## Pass 2 Review: Phase 32 milestone_async_2 runtime-bootstrap

### Pass 1 Blocker Resolution

**`clippy::borrowed_box` in async_with.rs** — RESOLVED ✓

The `Option<&Box<Expr>>` signature was replaced with `Option<&Expr>` (line 20) and `item.optional_vars.as_deref()` (line 103) at the call site. Clippy passes on both sifr_hir and sifr_codegen with `-D warnings`.

### Scope Verification

| Requirement | Status |
|---|---|
| Async main runtime bootstrap with Tokio dependency only for async entrypoints | ✓ |
| Mechanical HIR guardrail splits (async_await.rs 57L, async_with.rs 155L, external_defs.rs 36L) | ✓ |
| Docs updated (32_async_ecosystem.md, roadmap.md) | ✓ |
| Tests adequate (unit, codegen, e2e fixture) | ✓ |

### Local Validation Results

| Check | Result |
|---|---|
| Unit tests (33 passed, 0 new failures) | ✓ |
| `cargo clippy -q -p sifr_hir -- -D warnings` | ✓ |
| `cargo clippy -q -p sifr_codegen -- -D warnings` | ✓ |
| HIR maintainability guardrails | ✓ |
| `async_runtime_bootstrap.sifr` type-checks | ✓ |
| Generated Rust has `#[tokio::main(flavor = "current_thread")]` for async main | ✓ |
| Generated Rust has no tokio annotation for sync main | ✓ |
| New codegen tests (3/3 pass) | ✓ |

### Pre-existing Failures (unrelated to this PR)

- 8 e2e pass fixtures failing on main: `iterator_basics`, `iterator_pipeline_cloning`, `iterator_sources`, `lambda_basic`, `lazy_builtins`, `list_slice_copy`, `nested_function_nonlocal_accumulator`, `async_runtime_bootstrap` (pre-tokio support)
- 22 codegen test failures on main
- **No regression introduced** — my changes add 1 new passing fixture and 3 new passing tests while net-zero on existing failures

### Key Implementation Detail

`annotate_async_main_entrypoint()` (lib.rs:753-780) scans `assembled_body_items` for the first async function named `main`, checks if it's already annotated (avoids double-annotation), and inserts `#[tokio::main(flavor = "current_thread")]`. Tokio dependency is added to the crate set only when this function returns `true`. This ensures:
- Zero overhead for sync programs
- Minimal overhead for async programs (`current_thread` flavor)
- No annotation conflicts in generated code

### Verdict

**SATISFIED**

No blockers remain. The pass 1 clippy blocker is resolved. The PR-sized scope is complete with adequate tests, docs, and guardrail-compliant HIR decomposition. Local validation confirms all checkpoints pass. Ready for PR merge.



Based on my review of the working tree, here is my assessment:

**Module structure and re-exports:**
- `lower_stmt.rs` replaced `pub use candidate_and_validation::*;` and `pub(crate) use simple_dispatch_and_bindings::*;` with explicit imports/exports. Good.
- `stmt_support_emitter.rs` similarly replaced `pub(crate) use expr_call_and_literal_helpers::*;` with explicit items. Good.
- No `include!` macros anywhere in the touched roots or children. Good.

**Naming:**
- Modules are named by responsibility: `async_cleanup`, `call_args_and_returns`, `comprehension_exprs`, `field_assignment`, `try_handlers`, `result_type_helpers`, `statement_output`. No `_1`/`_2` style. Good.
- Old monolithic files (`comprehension_and_nested_subscript.rs`, `print_calls_and_returns.rs`, `try_handlers_and_cleanup.rs`, `with_async_and_if.rs`) are gone, replaced by smaller named modules.

**Visibility:**
- Production items use `pub(super)` for sibling access, `pub(crate)` for crate-wide exports, `pub` only for genuinely public API (`try_lower_simple_stmt`, `try_lower_expr_stmt`, `SimpleStmtLoweringCtx`). Appropriate.
- Test items all private (no `pub fn` in any `*_tests.rs` file). Good.

**Test layout:**
- Test modules use `use super::*` to import from their parent, which is the idiomatic Rust pattern for sibling test access. This is acceptable and was the original design intent.
- `lib_codegen_tests.rs` exports `empty_module` and `generate_rust_from_source` as `pub(crate)` for use by sibling test modules.

**Line counts:**
- `stmt_expr_binop.rs`: 852 lines — contains heavy macro (`stmt_expr_binop!`). Single-responsibility (binary operation lowering). Acceptable as a named macro-only file.
- `stmt_block.rs`: 804 lines — contains heavy macro (`stmt_expr_block!`). Single-responsibility (block statement lowering). Acceptable as a named macro-only file.
- Both are well under the 900-line hard cap. They are not cap-driven splits; they are natural responsibility boundaries containing macro definitions.

**No behavioral regressions detected** — mechanical splitting only, no algorithmic changes.

**SATISFIED**

I have enough understanding to render the verdict. Let me produce the concrete findings.

## M3 Review — Typed Intrinsic Identity and Source-Declared Retained Callables

### Scope confirmation

I read the phase issue, the M3 plan, the working-tree diff (~70 files, +741/-249), the new fixture (`compiler_intrinsic_name_collisions.sifr`), the new demo, the new lowering module (`lower/compiler_intrinsics.rs`), the new source declarations (`stdlib/sifr/test.sifr`, `stdlib/sifr/task.sifr`), the private bytes bridge (`stdlib/_sifr/bytes.sifr`), the manifest, the guard script, the bootstrap/re-export changes, and every HIR-consumer touchpoint (traversal, effects, flow, snapshots, ranges, codegen dispatch).

### Verified against M3 acceptance criteria

| Criterion | Location | Verdict |
| --- | --- | --- |
| Ordinary calls cannot reach intrinsic codegen by spelling | `stmt_support_emitter/expr_call_and_literal_helpers.rs`, `stmt_support_emitter/print_calls.rs`, `intrinsic_method_emitters/recursive_exprs.rs` (raw-name path deleted) | ✓ |
| Test/task aliases preserve intrinsic identity | `lower/imports.rs:157-165`, `lower/imported_defaults.rs:21-30`, `compiler_intrinsic_name_collisions.sifr` uses `assert_eq as verify_equal`, `current_context as active_context` | ✓ |
| Re-exported compiler-intrinsic callables preserve identity | `stdlib/re_exports.rs:95-104` + `synthetic_sysroot_re_export_preserves_compiler_identity` test | ✓ |
| First-class value use rejected | `expressions/core_and_calls.rs:213-222` + `source_declared_intrinsic_is_not_a_first_class_value` test | ✓ |
| User/package `@compiler_intrinsic` rejected | `lower/compiler_intrinsics.rs:27-34` + `user_and_private_sysroot_declarations_are_rejected` test | ✓ |
| Test failures retain caller-local values and Sifr call/argument ranges | Macro expansion preserves values; `HirExpr::IntrinsicCall.call_range`/`arg_ranges` populated in every lowering site; verified for aliased call in `imported_alias_call_preserves_identity_and_source_ranges` | ✓ |
| `hashlib.sifr` no raw-name dispatch or fallback | `_sifr/bytes.sifr` declares `bytes_to_hex_strict` as `@rust(sifr_stdlib.bytes.bytes_to_hex_strict)`; codegen registry no longer contains that lowerer; `sifr_stdlib::bytes::bytes_to_hex_strict` now `pub` | ✓ |
| No lowering path constructs string-named `HirExpr::Call` for any of the 17 typed IDs | `builtin_calls/bytes_len_range.rs`, `builtin_calls/constructors.rs`, `expressions/methods_lambdas_and_comprehensions.rs`, `expressions/call_shadowable_builtins.rs`, `expressions/regular_calls.rs` — every synthesis site emits `IntrinsicCall` | ✓ |
| Codegen exhaustive over `CompilerIntrinsicId` | `intrinsics/registry.rs` — total match, no `_ =>` wildcard | ✓ |
| HIR consumers updated | traversal_impl, flow effects, hir/name snapshot, error_refs, string_char_cache_scan, print_calls, string_assignment, stmt_block_helpers, try_tuple_flow, candidate_and_validation, python_callback_bounds, expr_call_and_literal_helpers, nonlocal_support — all handle the new variant | ✓ |
| Sysroot-only policy is enforced | `LoweringSourceOrigin::SysrootPublicStdlib` gates registration; private declaration & user origins are both rejected in tests | ✓ |
| Counter typed IDs unreachable by raw name | No source declares them, no lowering site synthesizes them; only codegen unit tests manually construct them. Consistent with M3 plan's explicit allowance | ✓ (deferred to M4) |
| Manifest & guard updated to typed IDs | `stdlib_retained_compiler_intrinsics.toml` — new names (`open_binary`, `test_assert_equal`, `bytes_from_integers`, `bytes_decode_with_encoding`, `task_current_context`), `bytes_to_hex_strict` removed. `check_stdlib_native_intrinsic_allowlist.py` derives from `hir_nodes.rs::declaration_name()` via `TYPED_INTRINSIC_NAME_RE`. `sources.rs` drops `_sifr.task`; `_sifr.task` file deleted; `sifr_retained_intrinsics/src/lib.rs` drops `_sifr.task` fallback and `task.rs` module | ✓ |
| M4 remains correctly bounded | Counter enum variants + dispatch arms + registry files + manifest row still present exactly as expected for M4 deletion. JSON-string defaultdict helpers untouched. Bytes manifest row already renamed & `bytes_to_hex_strict` removed (M4 is a no-op for that item) | ✓ |

### Findings

Only minor, non-blocking observations. None materially defective.

1. **Doc drift** — `internal_docs/sifr_sysroot_and_stdlib_architecture.md:108` still lists `_sifr.task → task.sifr`. `_sifr.task` was deleted this milestone. The phase plan assigns doc reconciliation to M6, but this row is now factually wrong at HEAD.

2. **Ranges are captured but not consumed by codegen** — `call_range` / `arg_ranges` on `HirExpr::IntrinsicCall` are populated by every lowering site and preserved by every walker, but no downstream code reads them (assertion failure messages still come from Rust-side macros). This satisfies the plan's "carry callsite metadata through HIR" language but leaves the fields as future-facing scaffolding. Not a defect.

3. **Duplicate diagnostic on invalid decorator with ellipsis body** — When `@compiler_intrinsic(bad_id)` or a non-sysroot origin fails `register_declaration`, `annotations_and_function_lowering.rs` still routes the function through `compiler_intrinsics::classify_stub_body` with `intrinsic = None`. Because the body is `...` (which is intentional in the negative tests), `classify_stub_body` emits the extra `"must contain exactly one ellipsis statement"` message on top of the primary rejection. Tests still pass because they only assert-contains a needle. UX polish only.

4. **Prescan intrinsic-name path not extended for source-declared identities** — `module_prescan.rs::collect_import_metadata` still branches on `stdlib_intrinsic_names` (a fallback-derived set). Source-declared identities (`assert_eq`, `current_context`) rely on `apply_intrinsic_registry_side_effects` inserting `declaration_name()` into `intrinsic_functions` during emit. `emit_named_module` runs before the `intrinsic_functions.contains("task_current_context")` check on line 395, so ordering is correct — but the invariant is subtle. Not a defect.

### Blocker check

- **Bootstrap identity export**: `bootstrap_tests.rs::retained_public_declarations_export_typed_compiler_identity` confirms `sifr.test.assert_eq` and `sifr.task.current_context` are exported. ✓
- **Raw-name dispatch retirement**: verified `try_lower_registry_intrinsic_call_expr` is now only invoked from `HirExpr::IntrinsicCall` arms; every `HirExpr::Call { func, args, .. }` path has had that call site removed. ✓
- **Collision safety**: `compiler_intrinsic_name_collisions.sifr` exercises local shadowing, nested function, method, imported alias, task alias, and test alias — verified native run passes in the reported validation. ✓
- **Bytes-bridge migration**: `sifr_stdlib::bytes::bytes_to_hex_strict` is `pub`, declared in `_sifr/bytes.sifr` via `@rust`, dispatch arm removed from registry, live caller (`hashlib.sifr`) routes through the private declaration. ✓
- **`_sifr.task` placeholder removed**: file deleted, `sifr.task` no longer imports it, `PRIVATE_STDLIB_MODULES` no longer lists it, `sifr_retained_intrinsics` no longer routes to it. ✓

### Verdict

The four items above are cosmetic or forward-looking; none is a milestone-breaking defect and none should block PR. The plan's own text explicitly defers exhaustive guard cross-referencing to M6 and Counter deletion to M4, and both boundaries are respected.

SATISFIED

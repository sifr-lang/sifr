## M3 Review — Round 2

**Verdict: SATISFIED**

### Scope
Full working-tree diff (~80 files, +741/−250) against HEAD, the M3 plan, round-1 review, retained manifest and guard, HIR/lowering/codegen/bootstrap/re-export changes, stdlib source, native adapter, collision fixture, and demo.

### Verified against M3 acceptance
| Criterion | Evidence | Status |
| --- | --- | --- |
| Typed `CompilerIntrinsicId` in HIR carries id, args, ty, `call_range`, `arg_ranges` | `crates/sifr_ir/src/hir_nodes.rs:117-210, 541-548` | ✓ |
| `FunctionType` unchanged; identity is separate callable metadata | No `crates/sifr_type_system` changes in diff | ✓ |
| Identity flows source → `ExternalDefs.compiler_intrinsics` → `LowerCtx.compiler_intrinsics` → `HirExpr::IntrinsicCall` | `bootstrap.rs:126-137,436-439`; `imports.rs:156-162`; `mod_impl.rs:360-368, 590-597`; `regular_calls.rs:501-512` | ✓ |
| Re-exports preserve identity | `re_exports.rs:7,95-104,143-193` + `synthetic_sysroot_re_export_preserves_compiler_identity` | ✓ |
| Ordinary calls cannot reach intrinsic dispatch by spelling | `stmt_support_emitter/expr_call_and_literal_helpers.rs:573-578`, `stmt_support_emitter/print_calls.rs:9-14`, `intrinsic_method_emitters/recursive_exprs.rs:43-45` (raw-name paths removed) | ✓ |
| Codegen exhaustive over `CompilerIntrinsicId` (no wildcard) | `intrinsics/registry.rs:22-91` | ✓ |
| Sysroot-only `@compiler_intrinsic`; user/private/malformed/unknown/synthesized-id/non-ellipsis-body all rejected structurally | `compiler_intrinsics.rs:27-48,72-79,96-128`; `compiler_intrinsics_tests.rs:52-93` | ✓ |
| First-class value use rejected; import/alias direct calls preserved | `expressions/core_and_calls.rs:213-222`; `compiler_intrinsics_tests.rs:96-153,156-185` | ✓ |
| Source-declared bodies not emitted | `module_body.rs:27-30` skips `func.compiler_intrinsic.is_some()` | ✓ |
| Bytes bridge migration: `sifr_stdlib::bytes::bytes_to_hex_strict` published; `_sifr.bytes` declaration added; dispatch/lowerer removed | `crates/sifr_stdlib/src/bytes.rs:16-17`; `stdlib/_sifr/bytes.sifr:13-15`; `intrinsics/registry/bytes.rs` (helper deleted) | ✓ |
| Primitive open/bytes/encode/decode lowering sites emit typed IntrinsicCall | `builtin_calls/bytes_len_range.rs:47-55,84-92`, `builtin_calls/constructors.rs:757-765`, `expressions/methods_lambdas_and_comprehensions.rs:219-254`, `expressions/call_shadowable_builtins.rs:240-361` | ✓ |
| `_sifr.task` placeholder deleted; task current_context source-declared | `stdlib/_sifr/task.sifr` gone; `stdlib/sifr/task.sifr:27-29`; `PRIVATE_STDLIB_MODULES` no longer lists `_sifr.task`; `sifr_retained_intrinsics/src/task.rs` deleted | ✓ |
| Collision fixture exercises local/nested/method/test-alias/task-alias | `crates/sifr/tests/e2e/pass/compiler_intrinsic_name_collisions.sifr` | ✓ |
| Every exhaustive HIR consumer updated | traversal, flow effects, error_refs, snapshots, validate_shape, python_callback_bounds, string_char_cache_scan, try_tuple_flow, nonlocal, name_resolution_snapshot | ✓ |
| Retained manifest, guard, arch doc coherent | `stdlib_retained_compiler_intrinsics.toml` renames (`open_binary`, `test_*`, `bytes_from_integers`, `bytes_decode_with_encoding`, `task_current_context`), `bytes_to_hex_strict` dropped, `_sifr.task` fallback row gone; `check_stdlib_native_intrinsic_allowlist.py` derives from `hir_nodes.rs::declaration_name()` | ✓ |
| Round-1 doc-drift observation resolved | `internal_docs/sifr_sysroot_and_stdlib_architecture.md` no longer lists `_sifr.task → task.sifr` | ✓ |

### Bootstrap ordering
`compile_stdlib_sources_with_sysroot` (`bootstrap.rs:123-179,244-247,433-441`) exports `func.compiler_intrinsic` alongside `fn_exports`, preserves it through public/`python_core` re-export processing, retains it under `should_export_callable`, and populates `stdlib_defs.compiler_intrinsics` — reachable by both the public-stdlib and private-declaration lowering entrypoints (`mod_impl.rs:360-368, 590-597`; `private_stdlib_imports.rs:62-67`).

### Non-blocking observations
1. **Ranges captured, not yet consumed** — `call_range`/`arg_ranges` are populated at every synthesis site and forwarded by every walker, but no downstream code reads them yet. Satisfies "carry callsite metadata through HIR"; downstream consumption is scaffolding.
2. **Duplicate diagnostic on rejected `@compiler_intrinsic`** — user/private-origin declarations first fail with "reserved for canonical public sysroot"; then `classify_stub_body` fires "must contain exactly one ellipsis statement" on ellipsis bodies. UX polish; tests use `contains` matchers so they still pass.
3. **Shadowing edge case (advisory)** — an unaliased `from sifr.test import assert_eq` followed by a local `def assert_eq(...)` overwrites `ctx.functions` but leaves `ctx.compiler_intrinsics["assert_eq"]` populated; `lower_regular_call` would still emit `HirExpr::IntrinsicCall`. Not exercised by the collision fixture (aliases are used) and not required by the plan's collision list, but consider clearing the entry when a local def re-registers the same name.
4. **Task/test callables count** — `bytes_from_integers` and `bytes_decode_with_encoding` in the manifest are the sole "extra" spellings vs the prior 27; the target 17 is met.

### Validation gaps
- Aggregate `run_all_tests.sh --profile create-pr` did not complete due to a macOS `syspolicyd/fseventsd` environment saturation, as reported. That is environmental, not a waived gate. The focused suites (745 codegen + 327 driver + 612 lowering, focused policy/bootstrap/re-export tests, native collision fixture, demo, hashlib check/emit/build/run, fmt, allowlist guard + self-test, manifest schema, bootstrap ordering, HIR guard, 900-line guard) exercised every M3-owned surface. **M3 is ready to merge once the unchanged create-PR gate exits zero after cooldown.**

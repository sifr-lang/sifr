# Diagnostic Emission Inventory

This inventory is the `milestone_diag_3` handoff into diagnostic registry population. It does not migrate emissions. It records the current emission surfaces, the target diagnostic identity for each user-facing category, representative fixtures, and notes that the migration milestones must preserve.

Coverage snapshot from April 29, 2026:

- `rg "ctx\\.error\\(" crates/sifr_hir/src -g '*.rs'` finds 489 raw HIR lowering emissions across 22 files.
- `rg "CompileError \\{" crates/sifr_driver/src crates/sifr/src -g '*.rs'` has 54 raw textual matches, including `struct`, `impl`, and return-type lines. The actual legacy driver/CLI construction surface is 47 sites: 43 production `CompileError { ... }` literals plus 4 test-only diagnostic construction sites.
- `rg "TypeErrorKind::" crates/sifr_type_system/src crates/sifr_hir/src -g '*.rs'` finds 24 current type-system typed-error construction sites.
- `rg "# expect-error" crates/sifr/tests/e2e/fail crates/sifr/tests/e2e.rs -g '*.sifr' -g '*.rs'` finds 92 fail-fixture expectations plus 8 harness test samples.

## HIR Lowering Surface

Raw `ctx.error(...)` remains the largest source of user-facing semantic diagnostics. The migration target is domain-owned helpers that construct `SifrDiagnostic` directly with primary spans from the AST node being lowered.

| File | Raw calls | Primary categories | Target families |
| --- | ---: | --- | --- |
| `crates/sifr_hir/src/lower/expressions.rs` | 205 | undefined names/functions, unsupported operators/calls, builtin and stdlib call validation, collection/method validation, comprehension/generator restrictions, tuple slicing, decimal constructor/method forwarding, protocol-style callable checks, unsupported expression/operator features | `NAME`, `TYPE`, `CALL`, `STDLIB`, `PROTO`, `DECIMAL`, `FLOW` |
| `crates/sifr_hir/src/lower/statements.rs` | 61 | break/continue, raise/result rules, with/context-manager rules, match patterns and exhaustiveness, assignment shape, borrow/move assignment, for-loop target/iterable rules | `FLOW`, `RESULT`, `PROTO`, `MATCH`, `TYPE`, `OWN`, `CALL` |
| `crates/sifr_hir/src/lower/builtin_calls.rs` | 55 | builtin call arity, argument type checks, constructor restrictions, exact numeric conversion checks | `CALL`, `TYPE`, `DECIMAL`, `STDLIB` |
| `crates/sifr_hir/src/lower/typing_and_functions.rs` | 24 | unknown type names, invalid annotations, callable/result/dict/list annotation shape, TypeVar/generic arity and bounds, unsupported default argument expression | `NAME`, `TYPE`, `RESULT`, `PROTO`, `CALL` |
| `crates/sifr_hir/src/lower/mod.rs` | 20 | TypeVar declarations, source-level import resolution, stdlib/intrinsic module member lookup, module lookup; workspace graph failures move to driver | `TYPE`, `IMPORT`, `NAME`, `STDLIB`; wrong-layer `WORKSPACE` moves to driver |
| `crates/sifr_hir/src/lower/classes.rs` | 19 | class inheritance, constructor/field order, auto-init, enum duplicate values, field/method lookup, iterator/reversible protocol method shape | `CLASS`, `TYPE`, `NAME`, `PROTO` |
| `crates/sifr_hir/src/lower/decimal_methods.rs` | 18 | decimal/bigdecimal round and quantize argument/type/context restrictions, decimal method arity, unknown decimal method | `DECIMAL`, `CALL`, `NAME` |
| `crates/sifr_hir/src/lower/aug_assign_lowering.rs` | 17 | unsupported augmented assignment operators/targets, type-check forwarding, undefined assignment target, move/borrow forwarding | `TYPE`, `OWN`, `NAME` |
| `crates/sifr_hir/src/lower/bytes_methods.rs` | 16 | bytes/str encode/decode method arity, type, and supported-encoding restrictions | `STDLIB`, `CALL`, `TYPE` |
| `crates/sifr_hir/src/lower/method_call_args.rs` | 13 | method receiver/call arity, unexpected keywords, duplicate arguments, missing required arguments | `CALL`, `TYPE` |
| `crates/sifr_hir/src/lower/tuple_unpack.rs` | 13 | tuple/list unpack arity and target restrictions | `TYPE`, `FLOW` |
| `crates/sifr_hir/src/lower/container_literal_specialization.rs` | 11 | list/set/dict/tuple literal element type conflicts and type-check forwarding | `TYPE` |
| `crates/sifr_hir/src/lower/subscript_type.rs` | 3 | tuple index bounds and non-indexable types | `TYPE` |
| `crates/sifr_hir/src/lower/nonlocal_support.rs` | 3 | invalid nonlocal declarations and missing enclosing binding | `FLOW`, `NAME` |
| `crates/sifr_hir/src/lower/min_max_validation.rs` | 2 | min/max argument validation | `CALL`, `TYPE` |
| `crates/sifr_hir/src/lower/nested_function_inference.rs` | 2 | nested function inference conflict and unsupported recursive nonlocal behavior | `TYPE`, `FLOW` |
| `crates/sifr_hir/src/lower/type_aliases.rs` | 2 | invalid alias target and recursive alias detection | `TYPE`, `NAME` |
| `crates/sifr_hir/src/lower/binding_mutability.rs` | 1 | mutability rebinding conflict | `OWN` |
| `crates/sifr_hir/src/lower/control_flow_conditions.rs` | 1 | boolean condition type requirement | `FLOW`, `TYPE` |
| `crates/sifr_hir/src/lower/if_expression.rs` | 1 | if-expression branch type incompatibility | `TYPE` |
| `crates/sifr_hir/src/lower/module_function_registry.rs` | 1 | missing module-level callable declaration | `NAME` |
| `crates/sifr_hir/src/lower/mutating_methods.rs` | 1 | mutation of immutable receiver | `OWN` |

Wrong-layer notes:

- Import resolution in `lower/mod.rs` currently emits ordinary type-check errors for workspace/module discovery failures. File-system workspace failures should remain in `sifr_driver`/`WORKSPACE`; source-level import syntax/member failures should be `IMPORT` or `NAME`.
- Decimal pseudo-codes are embedded in `sifr_type_system` and HIR messages. They must become `DECIMAL-*` identity at construction, not message text.
- Builtin and stdlib surface checks are split between `expressions.rs`, `builtin_calls.rs`, and dedicated modules. Migration should move helpers near the semantic owner but keep shared argument-shape utilities to avoid divergent `CALL-*` templates.

## Parser Surface

Parser errors originate in the Ruff fork (`sifr_python_parser`, exported from `third_party/ruff/crates/ruff_python_parser/src/error.rs`) and are currently wrapped by `sifr_driver::frontend::api` as phase-derived `SIFR-PARSE-0001`. `SIFR-PARSE-0001` should be retired as a catch-all in `milestone_diag_2b`; parser migration (`milestone_diag_7`) should map the exposed error categories below.

| Proposed code | Ruff category / examples | Fixture plan |
| --- | --- | --- |
| `SIFR-PARSE-0001` | retired legacy phase bucket | registry-only retired entry, no active fixture |
| `SIFR-PARSE-0002` | expected token or generic recovery context (`ExpectedToken`, `OtherError("Expected ...")`, `ExpectedExpression`, `UnexpectedExpressionToken`) | fixture pending in `milestone_diag_7`: missing delimiter/expression; existing invalid source unit tests can seed it |
| `SIFR-PARSE-0003` | lexical and interpolated-string errors (`Lexical`, `FStringError`, `TStringError`) | fixture pending in `milestone_diag_7`: malformed string/f-string |
| `SIFR-PARSE-0004` | indentation and same-line statement layout (`UnexpectedIndentation`, `SimpleStatementsOnSameLine`, `SimpleAndCompoundStatementOnSameLine`) | fixture pending in `milestone_diag_7`: indentation/layout |
| `SIFR-PARSE-0005` | invalid assignment/delete/starred/named-expression targets (`InvalidAssignmentTarget`, `InvalidAnnotatedAssignmentTarget`, `InvalidNamedAssignmentTarget`, `InvalidAugmentedAssignmentTarget`, `InvalidDeleteTarget`, `InvalidStarredExpressionUsage`) | fixture pending in `milestone_diag_7`: invalid target |
| `SIFR-PARSE-0006` | invalid call argument order/unpacking (`PositionalAfterKeywordArgument`, `PositionalAfterKeywordUnpacking`, `InvalidArgumentUnpackingOrder`, `DuplicateKeywordArgumentError`) | fixture pending in `milestone_diag_7`: invalid call syntax |
| `SIFR-PARSE-0007` | empty or malformed declaration lists (`EmptyImportNames`, `EmptyGlobalNames`, `EmptyNonlocalNames`, `EmptyTypeParams`, parameter-order errors) | fixture pending in `milestone_diag_7`: malformed import/global/nonlocal/type-param |
| `SIFR-PARSE-0008` | invalid pattern/match syntax (`InvalidStarPatternUsage`, invalid mapping/class pattern errors, expected pattern recovery) | fixture pending in `milestone_diag_7`: invalid match-pattern |
| `SIFR-PARSE-0009` | explicitly unsupported parser syntax or interactive-only syntax (`UnsupportedSyntaxErrorKind`, `UnexpectedIpythonEscapeCommand`, unsupported async token contexts) | fixture pending in `milestone_diag_7`: unsupported syntax |

## Type System Surface

`sifr_type_system::TypeError` and `TypeErrorKind` are transitional only. They have typed payloads but no spans, no stable code, and no canonical renderer model. Migration should replace them with direct domain helper calls in HIR/type-checking code, then delete these symbols.

| Current variant / construction | Current message category | Target code | Representative fixture |
| --- | --- | --- | --- |
| `TypeErrorKind::TypeMismatch` | ordinary expected/actual mismatch | `SIFR-TYPE-0002` | `crates/sifr/tests/e2e/fail/type_mismatch.sifr` |
| `TypeErrorKind::TypeMismatch` | union/optional mismatch | `SIFR-TYPE-0002` with union args | `crates/sifr/tests/e2e/fail/union_type_mismatch.sifr` |
| `TypeErrorKind::TypeMismatch` | decimal/bigdecimal comparison | `SIFR-DECIMAL-0004` | `crates/sifr/tests/e2e/fail/decimal_bigdecimal_mixed_arithmetic.sifr` |
| `TypeErrorKind::TypeMismatch` | float compared with decimal family | `SIFR-DECIMAL-0003` | `crates/sifr/tests/e2e/fail/decimal_float_mixed_arithmetic.sifr` |
| `TypeErrorKind::UndefinedVariable` | unresolved local/name expression | `SIFR-NAME-0001` | `crates/sifr/tests/e2e/fail/undefined_var.sifr` |
| `TypeErrorKind::UndefinedFunction` | unresolved callable name | `SIFR-NAME-0002` | `crates/sifr/tests/e2e/fail/stdlib_invalid_module.sifr` |
| `TypeErrorKind::WrongArgumentCount` | positional arity mismatch | `SIFR-CALL-0001` | `crates/sifr/tests/e2e/fail/stdlib_wrong_arg_count.sifr` |
| `TypeErrorKind::UseAfterMove` | moved value use | `SIFR-OWN-0001` | `crates/sifr/tests/e2e/fail/use_after_move.sifr` |
| `TypeErrorKind::MissingTypeAnnotation` | annotation required for inference boundary | `SIFR-TYPE-0004` | fixture pending in `milestone_diag_7` |
| `TypeErrorKind::InvalidOperator` | unsupported arithmetic/comparison/unary/bool operator | `SIFR-TYPE-0005` | `crates/sifr/tests/e2e/fail/optional_arithmetic_without_narrowing.sifr` |
| `TypeErrorKind::InvalidOperator` | int/bigint arithmetic or comparison requires conversion | `SIFR-TYPE-0006` | `crates/sifr/tests/e2e/fail/bigint_int_mixed_arithmetic.sifr`, `crates/sifr/tests/e2e/fail/bigint_int_mixed_comparison.sifr` |
| `TypeErrorKind::InvalidOperator` | decimal-family mixed arithmetic | `SIFR-DECIMAL-0003`, `SIFR-DECIMAL-0004` | decimal mixed arithmetic fixtures |
| `TypeErrorKind::NotCallable` | non-callable receiver/expression used as callable | `SIFR-CALL-0005` | fixture pending in call migration |

## Driver And CLI Surface

Legacy `CompileError` is the current outer transport and phase-derived code source. The migration target is `DiagnosticSink` plus `ErrorEmitted`; `CompileError` and `CompilerDiagnostic` should disappear by residual cleanup.

| Surface | Current construction count | Current code source | Target handling |
| --- | ---: | --- | --- |
| `crates/sifr_driver/src/diagnostics.rs` | 1 | `CompilePhase` maps `Parse -> SIFR-PARSE-0001`, `TypeCheck -> SIFR-TYPE-0001`, `Codegen -> SIFR-CODEGEN-0001`, `Build -> SIFR-BUILD-0001`; workspace prefix classifier maps some build messages to `SIFR-WORKSPACE-*`; one actual construction is the codegen panic boundary | Delete phase-derived codes in `milestone_diag_4a`; route already-structured diagnostics through shared renderer. Workspace prefix classifier is replaced by typed `WORKSPACE-*` constructors. |
| `crates/sifr_driver/src/frontend/api.rs` | 2 | parser frontend errors become `CompilePhase::Parse`; HIR lowering errors become `CompilePhase::TypeCheck` | Parser adapter emits `PARSE-*`; HIR returns `LoweringOutcome` diagnostics. |
| `crates/sifr_driver/src/frontend/module_lowering.rs` | 1 | module lowering errors become `TypeCheck` | Preserve module/source span and direct HIR diagnostic identity. |
| `crates/sifr_driver/src/project/discovery.rs` | 6 | workspace discovery and reachable parse failures | Keep workspace discovery in `WORKSPACE-*`; reachable source parse failures are `PARSE-*`. |
| `crates/sifr_driver/src/project/compile_order.rs` | 1 | dependency cycle as `TypeCheck` | `SIFR-WORKSPACE-0104` or `SIFR-IMPORT-0004` depending on whether the cycle is workspace graph or source import graph. |
| `crates/sifr_driver/src/project/frontend.rs` | 1 | project frontend setup as `Build` | Use `WORKSPACE-*` for project assembly failures. |
| `crates/sifr_driver/src/build/entrypoint.rs` | 3 | build planning/materialization failures | `BUILD-*` for tool/build actions; `WORKSPACE-*` for project graph inputs. |
| `crates/sifr_driver/src/build/materialize.rs` | 1 | file materialization failure | `SIFR-BUILD-0002`. |
| `crates/sifr_driver/src/build/workspace.rs` | 7 | temporary dir, cargo manifest, rustc/cargo execution, binary artifact failures | `SIFR-BUILD-0002..0006` by operation. |
| `crates/sifr_driver/src/stdlib/bootstrap.rs` | 4 | embedded stdlib parse/typecheck/bootstrap failure | `SIFR-STDLIB-0001..0003` for embedded stdlib bootstrap defects; internal if invariant-only. |
| `crates/sifr_driver/src/stdlib/cache.rs` | 1 | stdlib cache build reuse failure | `SIFR-STDLIB-0004` or `SIFR-BUILD-*` depending on failing operation. |
| `crates/sifr_driver/src/workspace/mod.rs` | 2 | manifest parse/source-root validation | Existing `SIFR-WORKSPACE-0001..0004` reviewed and kept if templates remain precise. |
| `crates/sifr_driver/src/test_runner/execution.rs` | 8 | test-runner compile/run/build failures | `SIFR-BUILD-*` for generated Rust test harness build/run operations. |
| `crates/sifr_driver/src/test_runner/orchestrator.rs` | 2 | test orchestration failure and frontend error forwarding | `BUILD-*` for orchestration; forwarded frontend diagnostics retain original identity. |
| `crates/sifr/src/main.rs` | 3 | CLI command path build/typecheck/codegen failures | CLI should render diagnostics from driver; direct construction should disappear. |

The 4 test-only `CompileError` construction sites in `crates/sifr_driver/src/tests/diagnostics.rs` intentionally exercise parse/typecheck/codegen/build phase mapping and recovery-limit behavior. They should be rewritten or deleted with the legacy diagnostic abstraction rather than treated as runtime emission sites.

CLI and renderer tests also manually construct `CompilerDiagnostic` values. These are test-only surfaces but must be updated in `milestone_diag_5` when the harness and renderer contracts stop accepting phase-bucket and pseudo-code strings:

| File | Manual `CompilerDiagnostic` sites | Current hard-coded identities | Migration owner |
| --- | ---: | --- | --- |
| `crates/sifr/src/main.rs` | 9 | `SIFR-TYPE-0001`, `SIFR-PARSE-0001`, `[E2507]`-style message content in compact-renderer tests | `milestone_diag_5` test harness/renderer contract cleanup |
| `crates/sifr_driver/src/tests/diagnostics.rs` | 2 | `SIFR-TYPE-0001` recovery-limit fixtures | `milestone_diag_5` or residual legacy diagnostic cleanup |

Current public-code mechanisms to remove:

| Mechanism | Current owner | Current effect | Replacement |
| --- | --- | --- | --- |
| Phase-derived `CompilePhase` mapping | `crates/sifr_driver/src/diagnostics.rs` | assigns `SIFR-PARSE-0001`, `SIFR-TYPE-0001`, `SIFR-CODEGEN-0001`, or `SIFR-BUILD-0001` after the real rule has already been flattened | domain helpers construct `SifrDiagnostic` with the canonical code before driver rendering |
| Workspace prefix classifier | `CompileError::workspace_diagnostic_code` | infers some `SIFR-WORKSPACE-*` identities from rendered message prefixes | typed workspace/project discovery constructors with structured path/module args |
| Type-error string forwarding | HIR sites that call `ctx.error(e.message)` or `ctx.error(error.message)` | loses `TypeErrorKind`, source relation, expected/actual/operator args, and decimal identity | HIR call site emits the target `TYPE-*`, `DECIMAL-*`, `NAME-*`, `CALL-*`, or `OWN-*` diagnostic directly with the AST span |
| Message-embedded pseudo-code | decimal/type-system messages and fixture expectations | keeps `[E25xx]` as text inside a broader `SIFR-TYPE-0001` diagnostic | top-level `SIFR-DECIMAL-*` diagnostic code and no secondary message code |
| Test-only hard-coded diagnostics | CLI renderer and driver diagnostics tests | locks renderer behavior to legacy phase buckets and pseudo-code text | renderer/harness tests construct canonical diagnostics through `sifr_diagnostics` fixtures |

Workspace code review for `milestone_diag_2b`:

- Keep `SIFR-WORKSPACE-0001` for malformed `sifr.toml` parsing.
- Keep `SIFR-WORKSPACE-0002` for `[source].roots` path escaping workspace root.
- Keep `SIFR-WORKSPACE-0003` for `[source].roots` path not resolving to a directory.
- Keep `SIFR-WORKSPACE-0004` for invalid source-root entry shape/path.
- Keep `SIFR-WORKSPACE-0101` for unresolved import after workspace search path enumeration.
- Keep `SIFR-WORKSPACE-0102` for ambiguous module resolution across workspace roots.
- Keep `SIFR-WORKSPACE-0103` for namespace package directory collision.
- Add `SIFR-WORKSPACE-0104` if project import-cycle diagnostics remain driver-owned.

## E2E Expectation And Baseline Surface

Current harness behavior in `crates/sifr/tests/e2e.rs` treats `# expect-error:` as substring matching. Harness sample tests explicitly accept `SIFR-PARSE-0001`, `SIFR-TYPE-0001`, and `[E2507]`. That is intentional legacy state until `milestone_diag_6`; inventory confirms these are the surfaces to tighten.

Current fail-fixture and harness-sample code markers:

| Marker | Count | Migration action |
| --- | ---: | --- |
| `SIFR-TYPE-0001` | 95 | Retire catch-all. Replace with category-specific codes during family migration milestones. |
| `SIFR-PARSE-0001` | 2 harness samples | Replace parse harness samples with canonical parse codes once parser emits structured diagnostics. |
| `[E2501]` | 1 | `SIFR-DECIMAL-0001`. |
| `[E2502]` | 2 | `SIFR-DECIMAL-0002`. |
| `[E2503]` | 1 | `SIFR-DECIMAL-0003`. |
| `[E2504]` | 2 | `SIFR-DECIMAL-0004`. |
| `[E2505]` | 3 | `SIFR-DECIMAL-0005`. |
| `[E2506]` | 2 | `SIFR-DECIMAL-0006`. |
| `[E2507]` | 5 | `SIFR-DECIMAL-0007`. |
| `[E2508]` | 2 | `SIFR-DECIMAL-0008`. |

Unannotated fail fixtures are also part of the migration surface. There are 88 fail fixtures with no `# expect-error` today; they currently assert only "compilation must fail". They should gain code assertions in the milestone that migrates the owning family, using the target-family grouping below.

| Group | Files | Target family/code plan |
| --- | --- | --- |
| stdlib unsupported or constrained APIs | `argparse_*`, `async_popen_unsupported`, `bisect_key_unsupported`, `configparser_*`, `counter_*`, `csv_dynamic_registry_unsupported`, `datetime_*`, `difflib_*`, `functools_*`, `glob_*`, `graphlib_*`, `hashlib_*`, `html_*`, `io_*`, `ip_address_*`, `itertools_*`, `json_*`, `logging_*`, `math_isclose_*`, `operator_*`, `os_*`, `pathlib_*`, `pyio_*`, `random_*`, `re_*`, `secrets_*`, `sha3_*`, `shutil_*`, `spooled_*`, `statistics_*`, `subprocess_*`, `sys_*`, `system_random_*`, `timeit_*`, `timezone_*`, `uuid_*`, `zip_*`, `zipfile_*` | `SIFR-STDLIB-*` for unsupported stdlib surface; `SIFR-CALL-*` for arity/keyword shape; `SIFR-TYPE-*` for wrong argument type |
| bytes and binary/text I/O | `bytes_*`, `bytesio_*`, `stringio_*` | `SIFR-STDLIB-*` for unsupported surface/codec limitations; `SIFR-CALL-*` for arity; `SIFR-TYPE-*` for wrong data type; `SIFR-OWN-*` for immutable bytes assignment |
| ownership and mutability | `borrowed_mut_parameter_return_escape`, `nested_function_capture_mutates_immutable_param`, `own_parameter_*` | `SIFR-OWN-0003` for escape, `SIFR-OWN-*` mutability-specific codes for mutation without `mut` |
| collection/container shape | `deque_index_invalid_bound`, `dict_*`, `list_unexpected_keyword`, `set_update_non_iterable`, `str_replace_invalid_count`, `tuple_*` | `SIFR-CALL-*` for keyword/arity/default conflicts; `SIFR-TYPE-*` for element/index type and tuple shape |
| type alias / recursive typing | `recursive_*`, `type_alias_missing_dependency` | `SIFR-TYPE-*` for recursive alias boundaries/arity, `SIFR-NAME-*` for missing alias dependency |

The full unannotated set at this snapshot is:

```text
argparse_formatter_class_unsupported.sifr
argparse_parse_args_non_string_list.sifr
async_popen_unsupported.sifr
bisect_key_unsupported.sifr
borrowed_mut_parameter_return_escape.sifr
bytes_append_unsupported.sifr
bytes_buffer_protocol_unsupported.sifr
bytes_bytearray_unsupported.sifr
bytes_bytes_subclass_unsupported.sifr
bytes_constructor_non_int.sifr
bytes_decode_non_string_codec.sifr
bytes_encode_non_string_codec.sifr
bytes_from_hex_non_string.sifr
bytes_from_ints_non_int_list.sifr
bytes_implicit_str_bytes_coercion_unsupported.sifr
bytes_memoryview_unsupported.sifr
bytes_non_utf8_codec_unsupported.sifr
bytes_read_bytes_not_list.sifr
bytes_subscript_assignment_unsupported.sifr
bytes_write_bytes_rejects_int_list.sifr
bytesio_text_write_unsupported.sifr
configparser_converter_registration_unsupported.sifr
counter_iterable_constructor_unsupported.sifr
counter_kwargs_constructor_unsupported.sifr
csv_dynamic_registry_unsupported.sifr
datetime_from_timestamp_non_float.sifr
datetime_tzinfo_zoneinfo_unsupported.sifr
deque_index_invalid_bound.sifr
dict_get_duplicate_default.sifr
dict_setdefault_invalid_default.sifr
dict_update_invalid_pairs.sifr
difflib_sequence_matcher_isjunk_unsupported.sifr
functools_partial_unsupported.sifr
glob_non_string_pattern.sifr
graphlib_add_non_int_predecessor.sifr
hashlib_new_non_string_name.sifr
hashlib_pbkdf2_hmac_unsupported.sifr
hashlib_scrypt_unsupported.sifr
html_package_parser_unsupported.sifr
io_open_non_string_mode.sifr
ip_address_non_string.sifr
itertools_groupby_unsupported.sifr
itertools_materialization_required.sifr
itertools_starmap_non_binary_callable.sifr
itertools_tee_unsupported.sifr
json_dynamic_hooks_unsupported.sifr
list_unexpected_keyword.sifr
logging_dictconfig_unsupported.sifr
logging_loggeradapter_unsupported.sifr
math_isclose_non_float_tol.sifr
nested_function_capture_mutates_immutable_param.sifr
operator_attrgetter_unsupported.sifr
operator_methodcaller_unsupported.sifr
ordered_counter_kwargs_constructor_unsupported.sifr
os_mkdir_non_string_path.sifr
own_parameter_method_mutation_requires_mut.sifr
own_parameter_mutation_requires_mut.sifr
pathlib_iterator_materialization_required.sifr
pyio_inheritance_unsupported.sifr
random_choices_weights_unsupported.sifr
re_search_non_string_pattern.sifr
recursive_generic_type_alias_wrong_arity.sifr
recursive_mutual_type_alias_missing_boundary.sifr
recursive_tree_attribute_without_narrowing.sifr
recursive_type_alias_missing_boundary.sifr
reversed_runtime_iterator_not_reversible.sifr
secrets_token_urlsafe_unsupported.sifr
set_update_non_iterable.sifr
sha3_object_model_unsupported.sifr
shutil_copy_non_string_path.sifr
spooled_tempfile_unsupported.sifr
statistics_mean_non_float_list.sifr
statistics_normaldist_unsupported.sifr
str_replace_invalid_count.sifr
stringio_read_bytes_unsupported.sifr
subprocess_non_string_cmd.sifr
sys_exit_non_int_code.sifr
system_random_state_unsupported.sifr
timeit_non_callable_stmt.sifr
timeit_string_eval_unsupported.sifr
timezone_mutation_unsupported.sifr
tuple_heterogeneous_iteration_unsupported.sifr
tuple_index_invalid_bound.sifr
type_alias_missing_dependency.sifr
uuid_from_hex_non_string.sifr
zip_bzip2_constant_unsupported.sifr
zip_ext_file_unsupported.sifr
zipfile_write_non_string_content.sifr
```

## Verification Baseline Surface

Checked-in verification baselines under `crates/sifr/tests/verification` are a separate migration surface from fail fixtures. They should be regenerated in the milestone that changes the corresponding renderer/harness behavior.

| Verification case | Current baseline markers | Target / owner |
| --- | --- | --- |
| `diagnostics/decimal_invalid_literal` | `SIFR-TYPE-0001` plus message-embedded `[E2501]` in compact/json/human output | `SIFR-DECIMAL-0001`; regenerate in decimal migration and renderer integration |
| `project/missing_import_reports_error` | `SIFR-WORKSPACE-0101` in compact/json output | keep `SIFR-WORKSPACE-0101`; renderer integration regenerates schema shape only |
| `project/workspace_unresolved_import` | `SIFR-WORKSPACE-0101` in compact/json output | keep `SIFR-WORKSPACE-0101`; add related searched paths |
| `project/workspace_ambiguous_import` | `SIFR-WORKSPACE-0102` in compact/json output | keep `SIFR-WORKSPACE-0102`; add related candidate paths |
| `project/workspace_malformed_manifest` | `SIFR-WORKSPACE-0001` in compact/json output | keep `SIFR-WORKSPACE-0001`; add manifest span/path metadata where available |
| `project/multi_module_run`, `project/workspace_dotted_helper_run` | no diagnostic marker; pass baselines exercise project mode stability | no diagnostic migration, but rerun with renderer tests |
| `crashes/CR-0001_cfg_invariant_minimized`, `crashes/CR-0002_parser_invariant_minimized` | crash minimization inputs, no checked renderer baseline | panic-boundary/internal diagnostic validation, likely `SIFR-INTERNAL-0001` if surfaced through user path |

## Non-Error Emission Paths

Warnings and notes are part of the same diagnostic stream and cannot remain uncoded side channels.

| Surface | Current sites | Current behavior | Target code / owner |
| --- | ---: | --- | --- |
| `ctx.warn(...)` arithmetic overflow risk | 5 in `lower/arithmetic_warnings.rs` | warning strings for int exponentiation, multiplication, and shift overflow risk | `SIFR-TYPE-0901` warning, owner `sifr_hir::lower::arithmetic_warnings` |
| `ctx.warn(...)` unreachable statement | 1 in `lower/statements.rs` | warning string when a statement after guaranteed exit is ignored | `SIFR-FLOW-0901` warning |
| `ctx.warn(...)` exhaustive-return validation panic recovery | 1 in `lower/typing_and_functions.rs` | warning string after `catch_unwind` skips control-flow validation | wrong-layer internal boundary; route as `SIFR-INTERNAL-0001` or eliminate panic path rather than keeping a user warning |
| `ctx.reveal_types` | `reveal_type(...)` in `lower/builtin_calls.rs`; guarded-index reveal propagation in `lower/guarded_index.rs` | note-like developer output currently stored as strings | `SIFR-TYPE-0902` note with `revealed_type` arg and recovery-cap participation |
| HIR-internal `catch_unwind` | `lower/typing_and_functions.rs` exhaustive-return check | catches an internal CFG invariant failure and downgrades it to warning | wrong-layer internal failure; should not be categorized as user-fixable |

## Target Code And Fixture Plan

These entries are the proposed active registry population for `milestone_diag_2b`. The exact templates and declared args can be adjusted during population, but every migrated category must keep a specific code and representative fixture.

| Code | Category | Owner module | Representative fixture / proof |
| --- | --- | --- | --- |
| `SIFR-PARSE-0002` | expected token or generic parser recovery with source span | parser adapter / driver frontend | fixture pending in `milestone_diag_7`: missing delimiter/expression or existing invalid source tests |
| `SIFR-PARSE-0003` | lexical or interpolated-string parser error | parser adapter / driver frontend | fixture pending in `milestone_diag_7`: malformed string/f-string |
| `SIFR-PARSE-0004` | indentation or same-line statement layout error | parser adapter / driver frontend | fixture pending in `milestone_diag_7`: indentation/layout |
| `SIFR-PARSE-0005` | invalid assignment/delete/starred/named-expression target syntax | parser adapter / driver frontend | fixture pending in `milestone_diag_7`: invalid target |
| `SIFR-PARSE-0006` | invalid call argument order/unpacking syntax | parser adapter / driver frontend | fixture pending in `milestone_diag_7`: invalid call syntax |
| `SIFR-PARSE-0007` | empty or malformed declaration list syntax | parser adapter / driver frontend | fixture pending in `milestone_diag_7`: malformed import/global/nonlocal/type-param |
| `SIFR-PARSE-0008` | invalid match-pattern syntax | parser adapter / driver frontend | fixture pending in `milestone_diag_7`: invalid match-pattern |
| `SIFR-PARSE-0009` | unsupported parser syntax or interactive-only syntax | parser adapter / driver frontend | fixture pending in `milestone_diag_7`: unsupported syntax |
| `SIFR-NAME-0001` | undefined variable | `sifr_hir::lower` name lookup | `crates/sifr/tests/e2e/fail/undefined_var.sifr` |
| `SIFR-NAME-0002` | undefined function/callable | `sifr_hir::lower` call lowering | `crates/sifr/tests/e2e/fail/stdlib_invalid_module.sifr` |
| `SIFR-NAME-0003` | unknown type or generic type name | type annotation lowering | `crates/sifr/tests/e2e/fail/generic_class_missing_type_arg.sifr` |
| `SIFR-NAME-0004` | module/member does not exist | import/member lookup | `crates/sifr/tests/e2e/fail/stdlib_missing_function.sifr` |
| `SIFR-IMPORT-0001` | forbidden `_sifr.*` intrinsic import | import lowering | `crates/sifr/tests/e2e/fail/import_intrinsic.sifr` |
| `SIFR-IMPORT-0002` | unknown source module/import target | import lowering/project discovery | `crates/sifr/tests/e2e/fail/import_nonexistent_local.sifr` |
| `SIFR-TYPE-0001` | retired catch-all | registry only | no active fixture; document retired replacement policy |
| `SIFR-TYPE-0002` | expected/actual type mismatch | type checking / assignment/call helpers | `crates/sifr/tests/e2e/fail/type_mismatch.sifr` |
| `SIFR-TYPE-0003` | if/conditional branch type mismatch | `if_expression` lowering | `crates/sifr/tests/e2e/fail/ternary_type_mismatch.sifr` |
| `SIFR-TYPE-0004` | missing required type annotation/inference boundary | type annotation/inference | fixture pending in `milestone_diag_7` |
| `SIFR-TYPE-0005` | unsupported operator or operand types | `sifr_type_system` / HIR expression lowering | `crates/sifr/tests/e2e/fail/optional_arithmetic_without_narrowing.sifr` |
| `SIFR-TYPE-0006` | int/bigint mixed arithmetic/comparison without conversion | type system numeric checks | bigint mixed arithmetic/comparison fixtures |
| `SIFR-TYPE-0007` | invalid type annotation shape | type annotation lowering | fixture pending in `milestone_diag_7` from annotation tests |
| `SIFR-TYPE-0008` | container literal element/key/value type conflict | container literal specialization | fixture pending in `milestone_diag_7` |
| `SIFR-TYPE-0009` | tuple/list unpacking shape mismatch | tuple unpack lowering | `crates/sifr/tests/e2e/fail/tuple_dynamic_list_shape.sifr` |
| `SIFR-DECIMAL-0001` | `Decimal()` invalid exact literal | decimal constructor lowering | `crates/sifr/tests/e2e/fail/decimal_invalid_literal_string.sifr` |
| `SIFR-DECIMAL-0002` | `BigDecimal()` invalid or non-literal exact string | decimal constructor lowering | bigdecimal invalid/non-literal fixtures |
| `SIFR-DECIMAL-0003` | float mixed with decimal family | type system decimal checks | `crates/sifr/tests/e2e/fail/decimal_float_mixed_arithmetic.sifr` |
| `SIFR-DECIMAL-0004` | decimal mixed with bigdecimal | type system decimal checks | `crates/sifr/tests/e2e/fail/decimal_bigdecimal_mixed_arithmetic.sifr` |
| `SIFR-DECIMAL-0005` | decimal float construction/conversion forbidden | decimal constructor/conversion lowering | decimal float constructor/conversion fixtures |
| `SIFR-DECIMAL-0006` | bigdecimal float construction/conversion forbidden | decimal constructor/conversion lowering | bigdecimal float constructor/conversion fixtures |
| `SIFR-DECIMAL-0007` | decimal round/quantize scale invalid | `decimal_methods` | decimal round/quantize scale fixtures |
| `SIFR-DECIMAL-0008` | bigdecimal round/quantize scale/context invalid | `decimal_methods` | bigdecimal round/quantize fixtures |
| `SIFR-CALL-0001` | wrong positional argument count | call/method validation | `crates/sifr/tests/e2e/fail/stdlib_wrong_arg_count.sifr` |
| `SIFR-CALL-0002` | unexpected keyword argument | call/method validation | `crates/sifr/tests/e2e/fail/sorted_unexpected_keyword.sifr` |
| `SIFR-CALL-0003` | duplicate argument from positional/keyword overlap | call/method validation | `crates/sifr/tests/e2e/fail/keyword_after_positional_error.sifr` |
| `SIFR-CALL-0004` | missing required argument | call/method validation | `crates/sifr/tests/e2e/fail/missing_required_argument.sifr` |
| `SIFR-CALL-0005` | callable arity/non-callable mismatch | call/method validation | `crates/sifr/tests/e2e/fail/map_callable_arity_mismatch.sifr` |
| `SIFR-OWN-0001` | use after move | ownership tracking | use-after-move fixtures |
| `SIFR-OWN-0002` | double mutable borrow | ownership/borrow checker | `crates/sifr/tests/e2e/fail/double_mut_borrow.sifr` |
| `SIFR-OWN-0003` | borrowed parameter escapes by return/store | ownership tracking | borrow escape fixtures |
| `SIFR-OWN-0004` | moved value across loop iteration | ownership tracking | `crates/sifr/tests/e2e/fail/use_after_move_loop.sifr` |
| `SIFR-FLOW-0001` | `break` outside loop | statement lowering | `crates/sifr/tests/e2e/fail/break_outside_loop.sifr` |
| `SIFR-FLOW-0002` | `continue` outside loop | statement lowering | `crates/sifr/tests/e2e/fail/continue_outside_loop.sifr` |
| `SIFR-FLOW-0003` | unsupported or invalid nonlocal/nested function flow | nested function/nonlocal lowering | `crates/sifr/tests/e2e/fail/nested_function_recursive_nonlocal_unsupported.sifr` |
| `SIFR-MATCH-0001` | non-exhaustive match | match lowering | match non-exhaustive fixtures |
| `SIFR-MATCH-0002` | match guard must be bool | match lowering | `crates/sifr/tests/e2e/fail/match_type_mismatch_guard.sifr` |
| `SIFR-MATCH-0003` | invalid class pattern field | match lowering | `crates/sifr/tests/e2e/fail/match_invalid_field_name.sifr` |
| `SIFR-PROTO-0001` | protocol bound/conformance failure | generic/protocol checking | generic bounds fixtures |
| `SIFR-PROTO-0002` | invalid iterator/reversible protocol signature | protocol checking | invalid iterator/reversible fixtures |
| `SIFR-PROTO-0003` | context-manager protocol missing | with-statement lowering | `crates/sifr/tests/e2e/fail/with_non_context_manager.sifr` |
| `SIFR-PROTO-0004` | hashable/comparable protocol required | protocol checking | unhashable/comparable fixtures |
| `SIFR-CLASS-0001` | class has fields but no required initializer/super initializer | class lowering | `crates/sifr/tests/e2e/fail/auto_init_inheritance_missing_super.sifr` |
| `SIFR-CLASS-0002` | required field declared after default | class lowering | `crates/sifr/tests/e2e/fail/auto_init_required_after_default.sifr` |
| `SIFR-CLASS-0003` | duplicate enum/class value or invalid variant | class/enum lowering | enum duplicate/invalid variant fixtures |
| `SIFR-CLASS-0004` | missing class field | attribute lowering | `crates/sifr/tests/e2e/fail/missing_field.sifr` |
| `SIFR-RESULT-0001` | unused `Result` value | result/error-flow checking | `crates/sifr/tests/e2e/fail/unused_result.sifr` |
| `SIFR-RESULT-0002` | invalid `Result` error type | result/error type lowering | `crates/sifr/tests/e2e/fail/error_str_not_allowed.sifr` |
| `SIFR-RESULT-0003` | invalid `raise` expression | statement lowering | `crates/sifr/tests/e2e/fail/error_raise_str.sifr` |
| `SIFR-STDLIB-0001` | unsupported stdlib constructor/method surface | stdlib/builtin lowering | `crates/sifr/tests/e2e/fail/defaultdict_keyword_constructor_unsupported.sifr` |
| `SIFR-STDLIB-0002` | stdlib method/argument type mismatch | stdlib call validation | stdlib wrong type/count fixtures, unless better represented by `CALL`/`TYPE` helper |
| `SIFR-WORKSPACE-0001..0104` | workspace manifest/source-root/import graph diagnostics | `sifr_driver::workspace` and project discovery | existing driver workspace tests |
| `SIFR-CODEGEN-0002` | codegen panic boundary/internal backend failure | driver codegen boundary | panic boundary tests |
| `SIFR-BUILD-0002..0006` | build workspace/materialization/cargo/rustc/test harness failures | driver build/test runner | driver build tests |
| `SIFR-INTERNAL-0001` | unclassified compiler panic after panic boundary | panic boundary | panic boundary tests |

## Recovery Expectations By Category

These expectations are not implemented until `milestone_diag_10`, but the code assignments above should preserve enough structured args to make them possible.

| Category / codes | Recovery expectation | Dedupe key sketch |
| --- | --- | --- |
| `PARSE-*` | non-tainting at diagnostic level; parser recovery decides continuation; cap-summarize after canonical source order | code + primary span + parser category |
| `NAME-0001..0004` | taint expression as `Unknown` where possible so dependent type errors do not cascade; unresolved import/module errors should stop the affected module path | code + name/module/member + primary span |
| `IMPORT-*`, `WORKSPACE-*` | fail affected module/project resolution; do not generate downstream type diagnostics for unreachable modules | code + module id/path + primary span or workspace path |
| `TYPE-0002..0009` | taint expression or binding as `Unknown` after primary mismatch; repeated mismatches cap/dedupe by same expected/actual/operator args | code + expected + actual/operator + primary span |
| `DECIMAL-0001..0008` | taint expression as `Unknown` for invalid constructor/method/type operation; do not emit a secondary `TYPE-*` for the same expression | code + decimal operation + operand/scale args + primary span |
| `CALL-0001..0005` | non-tainting for callable declaration; taint call expression result as `Unknown`; missing/duplicate/unexpected arg diagnostics should not poison later arguments | code + callable + arg name/index + primary span |
| `OWN-0001..0004` | taint moved/borrowed binding state to suppress repeated use-after-move/borrow-escape cascades; keep distinct primary spans for independent ownership errors | code + binding + ownership state + primary span |
| `FLOW-*` | non-tainting except invalid nested/nonlocal binding state; unreachable-statement warnings remain warnings and do not affect exit status | code + flow construct + primary span |
| `MATCH-*` | non-tainting for subject expression; non-exhaustiveness emits once per match expression; invalid pattern field can taint only that pattern arm | code + match subject type + missing variants/field + primary span |
| `PROTO-*` | taint conformance check result for that type/protocol pair; avoid repeating the same missing method/bound for every downstream call | code + type + protocol + method/bound + primary span |
| `CLASS-*` | taint class declaration metadata so constructor/field follow-on diagnostics do not cascade from the same malformed class | code + class/field/variant + primary span |
| `RESULT-*` | taint error-flow expression only; unused-result diagnostics are non-tainting warnings/errors depending on policy and should dedupe by expression span/type | code + result/error type + primary span |
| `STDLIB-*` | taint the call expression or constructed value; unsupported-surface diagnostics should not also emit generic `CALL-*`/`TYPE-*` for the same call | code + stdlib symbol + operation + primary span |
| `CODEGEN-*`, `BUILD-*`, `INTERNAL-*` | no semantic recovery; emit once per failing backend/build/internal boundary and preserve source diagnostics already emitted | code + operation/context + path if available |
| `TYPE-0901`, `FLOW-0901`, `TYPE-0902` | warnings/notes do not affect exit status; reveal-type notes participate in the 50 top-level cap with explicit omission summaries | code + message template args + primary span/order |

## Span, Related-Span, And Recovery Notes

- Every HIR source diagnostic should use the AST node span that caused the semantic failure. Existing raw messages usually have the expression/statement node available at the emission site; the migration should not introduce spanless HIR diagnostics for those paths.
- `TypeError` forwarding sites (`ctx.error(e.message)` / `ctx.error(error.message)`) currently lose the source relation between type-system payload and AST node. The migrated helper should be called at the HIR site that knows the node span and should pass expected/actual/operator/name args explicitly.
- Import and workspace diagnostics need related spans/paths for "searched here", "ambiguous candidate", and "import requested here" notes. Do not encode those in message strings.
- Recovery deduplication remains `milestone_diag_10`. Until then, repeated type errors keep the existing recovery behavior but must share `message_template` and explicit dedupe args once migrated.

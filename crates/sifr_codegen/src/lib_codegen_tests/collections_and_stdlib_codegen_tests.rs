use super::*;

#[test]
fn method_calls_on_self_collection_fields_do_not_clone_for_read_only_receivers() {
    let generated = generate_rust_from_source(
        r#"
def head(values: list[int]) -> int:
    value: int | None = values[0]
    if value is not None:
        return value
    return 0

class Store:
    items: list[int]
    lookup: dict[int, int]

    def __init__(self):
        self.items = [1, 2, 3]
        self.lookup = {1: 10}

    def total(self, key: int) -> int:
        return len(self.items) + self.lookup.get(key, 0) + head(self.items)
"#,
    );

    assert!(generated.contains("self.items.len() as i64"));
    assert!(generated.contains("self.lookup.get(&key)"));
    assert!(generated.contains("head(&self.items)"));
    assert!(!generated.contains("self.items.clone().len()"));
    assert!(!generated.contains("self.lookup.clone().get"));
    assert!(!generated.contains("head(&self.items.clone())"));
}

#[test]
fn dict_get_empty_list_default_returns_owned_list_value() {
    let generated = generate_rust_from_source(
        r#"
class Store:
    values: dict[str, list[tuple[str, int]]]

    def __init__(self):
        self.values = {}

    def count(self, key: str) -> int:
        values = self.values.get(key, [])
        return len(values)
"#,
    );

    assert!(
        generated.contains("self.values.get((key).as_str()).cloned().unwrap_or(vec![])"),
        "{generated}"
    );
    assert!(
        !generated.contains("self.values.get((key).as_str()).map(Vec::as_slice).unwrap_or(&[])"),
        "{generated}"
    );
}

#[test]
fn string_loop_set_membership_uses_char_storage() {
    let generated = generate_rust_from_source(
        r#"
def count_groups(text: str) -> int:
    groups = 0
    seen = set()
    for ch in text:
        if ch in seen:
            groups = groups + 1
            seen = set()
        seen.add(ch)
    return groups + 1
"#,
    );

    assert!(generated.contains("for ch in text.chars()"), "{generated}");
    assert!(generated.contains("seen.contains(&ch)"), "{generated}");
    assert!(
        generated.contains("seen.insert((ch).clone())"),
        "{generated}"
    );
    assert!(
        !generated.contains("text.chars().map(|c| c.to_string())"),
        "{generated}"
    );
}

#[test]
fn test_structured_stmt_path_lowers_collection_truthiness_inside_boolop_condition() {
    let tuple_ty = Type::Tuple(vec![Type::Int, Type::Int]);
    let stmt = HirStmt::While {
        condition: HirExpr::BoolOp {
            op: "and".to_string(),
            values: vec![
                HirExpr::Name {
                    name: "stack".to_string(),
                    ty: Type::List(Box::new(tuple_ty.clone())),
                },
                HirExpr::Compare {
                    left: Box::new(HirExpr::Index {
                        object: Box::new(HirExpr::Index {
                            object: Box::new(HirExpr::Name {
                                name: "stack".to_string(),
                                ty: Type::List(Box::new(tuple_ty.clone())),
                            }),
                            index: Box::new(HirExpr::UnaryOp {
                                op: "-".to_string(),
                                operand: Box::new(HirExpr::IntLiteral(1)),
                                ty: Type::Int,
                            }),
                            ty: Type::Union(vec![tuple_ty.clone(), Type::None]),
                        }),
                        index: Box::new(HirExpr::IntLiteral(1)),
                        ty: Type::Int,
                    }),
                    ops: vec![">".to_string()],
                    comparators: vec![HirExpr::Name {
                        name: "h".to_string(),
                        ty: Type::Int,
                    }],
                    ty: Type::Bool,
                },
            ],
            ty: Type::Bool,
        },
        body: vec![HirStmt::Let {
            name: "x".to_string(),
            ty: Type::Int,
            value: HirExpr::IntLiteral(1),
            is_mutable: true,
        }],
        else_body: None,
    };

    let mut emitter = RustEmitter::new();
    let captured = emitter.capture_structured_stmts(|inner| inner.emit_stmt(&stmt));

    let Some(RustStmt::While { cond, .. }) = captured.first() else {
        panic!("expected while stmt");
    };
    let rendered = crate::render_expr(cond);
    assert!(rendered.contains("is_empty"));
    assert!(rendered.contains("&&"));
}

#[test]
fn test_structured_stmt_path_lowers_option_call_truthiness_to_bool_condition() {
    let stmt = HirStmt::If {
        condition: HirExpr::MethodCall {
            object: Box::new(HirExpr::Name {
                name: "nums".to_string(),
                ty: Type::Dict(Box::new(Type::Int), Box::new(Type::Int)),
            }),
            method: "get".to_string(),
            args: vec![HirExpr::Name {
                name: "i".to_string(),
                ty: Type::Int,
            }],
            ty: Type::Union(vec![Type::Int, Type::None]),
        },
        then_body: vec![HirStmt::Pass],
        elif_clauses: vec![],
        else_body: None,
    };

    let mut emitter = RustEmitter::new();
    let captured = emitter.capture_structured_stmts(|inner| inner.emit_stmt(&stmt));

    assert!(matches!(
        captured.first(),
        Some(RustStmt::If {
            cond: RustExpr::MethodCall { method, .. },
            ..
        }) if method == "is_some_and"
    ));
}

#[test]
fn test_structured_stmt_path_lowers_nested_string_augassign_to_push_str() {
    let stmt = HirStmt::While {
        condition: HirExpr::BoolLiteral(true),
        body: vec![HirStmt::If {
            condition: HirExpr::BoolLiteral(true),
            then_body: vec![HirStmt::AugAssign {
                name: "out".to_string(),
                op: "+=".to_string(),
                value: HirExpr::Name {
                    name: "part".to_string(),
                    ty: Type::Str,
                },
            }],
            elif_clauses: vec![],
            else_body: None,
        }],
        else_body: None,
    };

    let mut emitter = RustEmitter::new();
    let captured = emitter.capture_structured_stmts(|inner| inner.emit_stmt(&stmt));

    let Some(RustStmt::While { body, .. }) = captured.first() else {
        panic!("expected while stmt");
    };
    let Some(RustStmt::If { then_body, .. }) = body.first() else {
        panic!("expected nested if stmt");
    };
    assert!(matches!(
        then_body.first(),
        Some(RustStmt::Expr(RustExpr::MethodCall { method, .. })) if method == "push_str"
    ));
}

#[test]
fn test_structured_stmt_path_string_contains_avoids_double_borrow_pattern_arg() {
    let module = HirModule {
        functions: vec![HirFunction {
            name: "contains_self".to_string(),
            params: vec![
                HirParam {
                    name: "text".to_string(),
                    ty: Type::Str,
                    default: None,
                    keyword_only: false,
                    convention: ParamConvention::borrow(),
                },
                HirParam {
                    name: "s".to_string(),
                    ty: Type::Str,
                    default: None,
                    keyword_only: false,
                    convention: ParamConvention::borrow(),
                },
            ],
            return_type: Type::Bool,
            body: vec![HirStmt::Return {
                value: Some(HirExpr::ContainsOp {
                    element: Box::new(HirExpr::Name {
                        name: "s".to_string(),
                        ty: Type::Str,
                    }),
                    collection: Box::new(HirExpr::Name {
                        name: "text".to_string(),
                        ty: Type::Str,
                    }),
                    ty: Type::Bool,
                }),
            }],
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let generated = generate_rust_with_metadata(&module);
    assert!(!generated.rust_source.contains(".contains(&(s))"));
    assert!(generated.rust_source.contains(".contains("));
    assert!(!generated.rust_source.contains(".contains(&("));
}

#[test]
fn test_plain_call_canonicalizes_heapq_compat_symbol_name() {
    let module = HirModule {
        functions: vec![HirFunction {
            name: "touch".to_string(),
            params: vec![HirParam {
                name: "min_h".to_string(),
                ty: Type::List(Box::new(Type::Int)),
                default: None,
                keyword_only: false,
                convention: ParamConvention::mut_borrow(),
            }],
            return_type: Type::None,
            body: vec![HirStmt::Expr {
                expr: HirExpr::Call {
                    func: "__compat_sifr_heapq_heapify".to_string(),
                    args: vec![HirExpr::Name {
                        name: "min_h".to_string(),
                        ty: Type::List(Box::new(Type::Int)),
                    }],
                    ty: Type::None,
                },
            }],
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let generated = generate_rust_with_metadata(&module);
    assert!(generated.rust_source.contains("heapify("));
    assert!(!generated
        .rust_source
        .contains("__compat_sifr_heapq_heapify("));
}

#[test]
fn test_list_builtin_uses_owned_collection_for_unknown_set_with_list_hint() {
    let module = HirModule {
        functions: vec![HirFunction {
            name: "to_list".to_string(),
            params: vec![HirParam {
                name: "result".to_string(),
                ty: Type::Set(Box::new(Type::Any)),
                default: None,
                keyword_only: false,
                convention: ParamConvention::borrow(),
            }],
            return_type: Type::List(Box::new(Type::Str)),
            body: vec![HirStmt::Return {
                value: Some(HirExpr::Call {
                    func: "list".to_string(),
                    args: vec![HirExpr::Name {
                        name: "result".to_string(),
                        ty: Type::Set(Box::new(Type::Any)),
                    }],
                    ty: Type::List(Box::new(Type::Str)),
                }),
            }],
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let generated = generate_rust_with_metadata(&module);
    assert!(generated
        .rust_source
        .contains(".iter().cloned().collect::<Vec<_>>()"));
}

#[test]
fn test_set_builtin_with_generator_lowers_to_collect_not_plain_set_call() {
    let generator = HirExpr::GeneratorExpr {
        expr: Box::new(HirExpr::Call {
            func: "str".to_string(),
            args: vec![HirExpr::Name {
                name: "i".to_string(),
                ty: Type::Int,
            }],
            ty: Type::Str,
        }),
        var: "i".to_string(),
        iter: Box::new(HirExpr::RangeLiteral {
            start: Box::new(HirExpr::IntLiteral(0)),
            end: Box::new(HirExpr::IntLiteral(3)),
            step: None,
            ty: Type::Range,
        }),
        filter: None,
        ty: Type::Iterator(Box::new(Type::Str)),
    };
    let module = HirModule {
        functions: vec![HirFunction {
            name: "build_set".to_string(),
            params: vec![],
            return_type: Type::Set(Box::new(Type::Str)),
            body: vec![HirStmt::Return {
                value: Some(HirExpr::Call {
                    func: "set".to_string(),
                    args: vec![generator],
                    ty: Type::Set(Box::new(Type::Str)),
                }),
            }],
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let generated = generate_rust_with_metadata(&module);
    assert!(generated
        .rust_source
        .contains("collect::<std::collections::HashSet<_>>()"));
    assert!(!generated.rust_source.contains("return set("));
}

#[test]
fn test_list_repeat_lowers_without_vec_mul_shape() {
    let module = HirModule {
        functions: vec![HirFunction {
            name: "repeat_zero".to_string(),
            params: vec![HirParam {
                name: "n".to_string(),
                ty: Type::Int,
                default: None,
                keyword_only: false,
                convention: ParamConvention::own(),
            }],
            return_type: Type::List(Box::new(Type::Int)),
            body: vec![HirStmt::Return {
                value: Some(HirExpr::BinOp {
                    left: Box::new(HirExpr::ListLiteral {
                        elements: vec![HirExpr::IntLiteral(0)],
                        ty: Type::List(Box::new(Type::Int)),
                    }),
                    op: "*".to_string(),
                    right: Box::new(HirExpr::Name {
                        name: "n".to_string(),
                        ty: Type::Int,
                    }),
                    ty: Type::List(Box::new(Type::Int)),
                }),
            }],
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let generated = generate_rust_with_metadata(&module);
    assert!(!generated.rust_source.contains("vec![0 as i64] * n"));
    assert!(generated.rust_source.contains("std::iter::repeat(0_i64)"));
    assert!(!generated.rust_source.contains("__sifr_repeat_out.extend("));
}

#[test]
fn test_compare_lowers_int_float_mixed_operands_with_cast() {
    let module = HirModule {
        functions: vec![HirFunction {
            name: "cmp".to_string(),
            params: vec![
                HirParam {
                    name: "coins".to_string(),
                    ty: Type::Float,
                    default: None,
                    keyword_only: false,
                    convention: ParamConvention::own(),
                },
                HirParam {
                    name: "n".to_string(),
                    ty: Type::Int,
                    default: None,
                    keyword_only: false,
                    convention: ParamConvention::own(),
                },
            ],
            return_type: Type::Bool,
            body: vec![HirStmt::Return {
                value: Some(HirExpr::Compare {
                    left: Box::new(HirExpr::Name {
                        name: "coins".to_string(),
                        ty: Type::Float,
                    }),
                    ops: vec![">".to_string()],
                    comparators: vec![HirExpr::Name {
                        name: "n".to_string(),
                        ty: Type::Int,
                    }],
                    ty: Type::Bool,
                }),
            }],
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let generated = generate_rust_with_metadata(&module);
    assert!(generated.rust_source.contains("as f64"));
}

#[test]
fn test_bool_typed_boolop_coerces_optional_operand_to_condition_bool() {
    let optional_index = HirExpr::Index {
        object: Box::new(HirExpr::Name {
            name: "grid2".to_string(),
            ty: Type::List(Box::new(Type::Int)),
        }),
        index: Box::new(HirExpr::Name {
            name: "i".to_string(),
            ty: Type::Int,
        }),
        ty: Type::Union(vec![Type::Int, Type::None]),
    };
    let module = HirModule {
        functions: vec![HirFunction {
            name: "cond".to_string(),
            params: vec![
                HirParam {
                    name: "grid2".to_string(),
                    ty: Type::List(Box::new(Type::Int)),
                    default: None,
                    keyword_only: false,
                    convention: ParamConvention::borrow(),
                },
                HirParam {
                    name: "i".to_string(),
                    ty: Type::Int,
                    default: None,
                    keyword_only: false,
                    convention: ParamConvention::own(),
                },
            ],
            return_type: Type::Bool,
            body: vec![HirStmt::Return {
                value: Some(HirExpr::BoolOp {
                    op: "and".to_string(),
                    values: vec![optional_index, HirExpr::BoolLiteral(true)],
                    ty: Type::Bool,
                }),
            }],
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let generated = generate_rust_with_metadata(&module);
    assert!(generated.rust_source.contains("is_some_and"));
}

#[test]
fn test_string_slice_negative_stop_normalizes_against_length() {
    let rust_code = generate_rust_from_source(
        "def has_marker_inside_sentinel_window(text: str) -> bool:\n    framed = \"[\" + text + \"]\"\n    return \"!\" in framed[1:-1]\n",
    );
    assert!(
        rust_code.contains("_slice_len_i64"),
        "string slice lowering should materialize source length for negative-stop normalization"
    );
    assert!(
        rust_code.contains("_slice_stop_i64"),
        "string slice lowering should compute normalized stop bound"
    );
    assert!(
        !rust_code.contains("((-(1 as i64)).max(0) - (1 as i64).max(0)).max(0)"),
        "negative stop must not be clamped directly to zero"
    );
}

#[test]
fn test_union_display_impl_uses_structured_ir() {
    let union_src = include_str!("../union_type_helpers.rs");
    assert!(union_src.contains("RustType::Ref {"));
    assert!(union_src.contains("RustStmt::Match {"));
    assert!(union_src.contains("RustExpr::Literal(RustLiteral::Str(fmt_spec.to_string()))"));
}

#[test]
fn test_union_enum_definitions_emit_structured_items() {
    let union_src = include_str!("../union_type_helpers.rs");
    let lib_src = include_str!("../lib.rs");

    assert!(union_src.contains("self.enum_items.push(RustItem::Enum {"));
    assert!(!union_src.contains("enum_defs"));
    assert!(!lib_src.contains("enum_defs"));
}

#[test]
fn test_generate_rust_with_stdlib_assembles_single_rust_file() {
    let lib_src = include_str!("../lib_modules_and_codegen.rs");
    let start = lib_src
        .find("pub fn generate_rust_with_stdlib")
        .expect("generate_rust_with_stdlib should exist");
    let generate_block = &lib_src[start..];

    assert!(generate_block.contains("let file_issues = validate_items(&file_items);"));
    assert!(generate_block.contains("let rust_file = RustFile { items: file_items };"));
    assert!(generate_block.contains("Renderer::new().render_file(&rust_file)"));
    assert!(!generate_block.contains("assert_output_drained("));
    assert!(!generate_block.contains("emitter.output"));
}

#[test]
fn test_generate_rust_multi_assembles_single_rust_file() {
    let lib_src = include_str!("../lib_project_codegen.rs");
    let start = lib_src
        .find("pub fn generate_rust_multi_with_metadata")
        .expect("generate_rust_multi_with_metadata should exist");
    let end = lib_src
        .find("/// Generate a complete Rust project (Cargo.toml + main.rs content).")
        .expect("generate_project docs should exist");
    let generate_block = &lib_src[start..end];

    assert!(generate_block.contains("generate_rust_with_stdlib(module, &project_codegen_code)"));
    assert!(generate_block.contains("render_local_module_imports(module)"));
    assert!(generate_block.contains("publicize_generated_module_source(&rust_source)"));
    assert!(generate_block.contains("required_crates.extend(codegen_result.required_crates)"));
    assert!(!generate_block.contains("assert_output_drained("));
    assert!(!generate_block.contains("emitter.output"));
    assert!(!generate_block.contains("module_import_prelude"));
}

#[test]
fn test_generate_project_emits_sifr_runtime_path_dependency_when_required() {
    let module = empty_module();
    let required_crates = HashSet::from(["sifr_runtime".to_string()]);
    let (cargo_toml, _main_rs) = generate_project_with_deps_and_crates(
        &module,
        "sifr_output",
        &HashSet::new(),
        &required_crates,
    );

    assert!(cargo_toml.contains("sifr_runtime = { path = "));
}

#[test]
fn test_async_main_entrypoint_gets_tokio_bootstrap_dependency() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module("async def main() -> None:\n    await task.sleep(0.0)\n    return None\n")
                .expect("parse failed")
                .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result
        .rust_source
        .contains("#[tokio::main(flavor = \"current_thread\")]"));
    assert!(result.rust_source.contains("async fn main()"));
    assert!(result.required_crates.contains("tokio"));
}

#[test]
fn test_async_result_main_entrypoint_keeps_result_return() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def main() -> Result[None, ValueError]:\n    await task.sleep(0.0)\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result
        .rust_source
        .contains("#[tokio::main(flavor = \"current_thread\")]"));
    assert!(result
        .rust_source
        .contains("async fn main() -> Result<(), ValueError>"));
    assert!(result.rust_source.contains("Ok(())"));
    assert!(result.required_crates.contains("tokio"));
}

#[test]
fn test_task_sleep_lowers_to_tokio_sleep_and_requires_tokio() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module("async def main() -> None:\n    await task.sleep(0.0)\n    return None\n")
                .expect("parse failed")
                .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result.rust_source.contains("tokio::time::sleep"));
    assert!(result
        .rust_source
        .contains("std::time::Duration::from_secs_f64"));
    assert!(result.required_crates.contains("tokio"));
}

#[test]
fn test_task_sleep_requires_tokio_without_async_main() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def wait_once() -> None:\n    await task.sleep(0.0)\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(!result
        .rust_source
        .contains("#[tokio::main(flavor = \"current_thread\")]"));
    assert!(result.required_crates.contains("tokio"));
}

#[test]
fn test_task_scope_context_materializes_runtime_container() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def main() -> None:\n    async with task.scope() as scope:\n        await task.sleep(0.0)\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result.rust_source.contains("struct __SifrTaskScope"));
    assert!(result.rust_source.contains("impl __SifrTaskScope"));
    assert!(result
        .rust_source
        .contains("let mut scope = __SifrTaskScope::new();"));
    assert!(result
        .rust_source
        .contains("scope.__sifr_join_all().await;"));
    assert!(result.required_crates.contains("tokio"));
}

#[test]
fn test_scope_spawn_lowers_to_owned_task_handle_substrate() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def worker() -> int:\n    await task.sleep(0.0)\n    return 41\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        handle = scope.spawn(worker())\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result.rust_source.contains("struct __SifrTask<T, E>"));
    assert!(result.rust_source.contains("fn __sifr_spawn_infallible<"));
    assert!(result.rust_source.contains("struct __SifrScopeChild"));
    assert!(result.rust_source.contains("tokio::sync::oneshot::channel"));
    assert!(result
        .rust_source
        .contains("scope.__sifr_spawn_infallible(worker());"));
    assert!(result
        .rust_source
        .contains("if let Err(__sifr_scope_failure) = scope.__sifr_join_all().await"));
    assert!(result.required_crates.contains("tokio"));
}

#[test]
fn test_spawn_blocking_lowers_to_distinct_blocking_task_substrate() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "@cpu_heavy\ndef compute_value() -> int:\n    return 42\n\nasync def main() -> Result[None, ScopeFailure]:\n    handle = task.spawn_blocking(compute_value)\n    result = await handle\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result
        .rust_source
        .contains("struct __SifrBlockingTask<T, E>"));
    assert!(result
        .rust_source
        .contains("fn __sifr_spawn_blocking_infallible<"));
    assert!(result
        .rust_source
        .contains("tokio::task::spawn_blocking(move || __SifrTaskResult::Ok(work()))"));
    assert!(result
        .rust_source
        .contains("__sifr_spawn_blocking_infallible(compute_value);"));
    assert!(result.required_crates.contains("tokio"));
}

#[test]
fn test_thread_pool_executor_submit_reuses_blocking_task_substrate() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "class ThreadPoolExecutor:\n    pass\n\n\n@cpu_heavy\ndef compute_value() -> int:\n    return 42\n\nasync def main() -> Result[None, ScopeFailure]:\n    executor: ThreadPoolExecutor = ThreadPoolExecutor()\n    handle = executor.submit(compute_value)\n    result = await handle\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result
        .rust_source
        .contains("struct __SifrBlockingTask<T, E>"));
    assert!(result
        .rust_source
        .contains("fn __sifr_spawn_blocking_infallible<"));
    assert!(result
        .rust_source
        .contains("__sifr_spawn_blocking_infallible(compute_value);"));
    assert!(result.required_crates.contains("tokio"));
}

#[test]
fn test_scope_spawn_lowers_owned_coroutine_arguments() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def worker(value: int) -> int:\n    await task.sleep(0.0)\n    return value\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        value: int = 41\n        handle = scope.spawn(worker(value))\n        result = await handle\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result
        .rust_source
        .contains("scope.__sifr_spawn_infallible(worker(value));"));
}

#[test]
fn test_scope_spawn_lowers_owned_move_coroutine_arguments() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def worker(own items: list[int]) -> int:\n    await task.sleep(0.0)\n    return len(items)\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        handle = scope.spawn(worker([1, 2]))\n        result = await handle\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result
        .rust_source
        .contains("scope.__sifr_spawn_infallible(worker(vec![1_i64, 2_i64]));"));
}

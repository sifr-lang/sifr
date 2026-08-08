use super::*;

#[test]
fn test_nested_stdlib_constructor_uses_canonical_nominal_path() {
    let class = |name: &str, identity: &str| Type::Class {
        identity: Some(identity.to_string()),
        type_args: Vec::new(),
        name: name.to_string(),
        fields: Vec::new(),
        methods: Vec::new(),
        parent_class: None,
    };
    let locale_ty = class("LocaleId", "sifr.i18n.LocaleId");
    let formatter_ty = class("NumberFormatter", "sifr.i18n.NumberFormatter");
    let expr = HirExpr::ConstructorCall {
        class_name: "NumberFormatter".to_string(),
        args: vec![HirExpr::ConstructorCall {
            class_name: "LocaleId".to_string(),
            args: vec![HirExpr::StringLiteral("bn".to_string())],
            ty: locale_ty,
        }],
        ty: formatter_ty,
    };
    let mut emitter = RustEmitter::new();

    let lowered = emitter
        .try_lower_registry_expr_strict(&expr)
        .expect("nested constructor should lower through the structured registry path");
    let rendered = crate::render_expr(&lowered);
    let formatter_name = sifr_type_system::stdlib_class_rust_name("sifr.i18n", "NumberFormatter");
    let locale_name = sifr_type_system::stdlib_class_rust_name("sifr.i18n", "LocaleId");

    assert_eq!(
        rendered,
        format!("{formatter_name}::new({locale_name}::new(\"bn\".to_string()))")
    );
}

#[test]
fn test_structured_expr_path_handles_plain_signature_call_expression() {
    let module = HirModule {
        functions: vec![
            HirFunction {
                name: "helper".to_string(),
                params: vec![],
                return_type: Type::None,
                body: vec![HirStmt::Expr {
                    expr: HirExpr::Call {
                        mutable_arg_places: Vec::new(),
                        func: "print".to_string(),
                        args: vec![HirExpr::StringLiteral("inner".to_string())],
                        ty: Type::None,
                    },
                }],
                is_async: false,
                method_kind: MethodKind::Regular,
                receiver: None,
                decorators: vec![],
                rust_interop: Vec::new(),
                python_interop: Vec::new(),
                compiler_intrinsic: None,
                type_params: vec![],
            },
            HirFunction {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::None,
                body: vec![HirStmt::Expr {
                    expr: HirExpr::Call {
                        mutable_arg_places: Vec::new(),
                        func: "helper".to_string(),
                        args: vec![],
                        ty: Type::None,
                    },
                }],
                is_async: false,
                method_kind: MethodKind::Regular,
                receiver: None,
                decorators: vec![],
                rust_interop: Vec::new(),
                python_interop: Vec::new(),
                compiler_intrinsic: None,
                type_params: vec![],
            },
        ],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let generated = generate_rust_with_metadata(&module);
    assert!(generated.rust_source.contains("helper();"));
    assert!(
        generated.lowering_stats.expr_structured > 0,
        "plain calls with by-value signatures should use structured expr path"
    );
}

#[test]
fn test_structured_expr_path_handles_registry_method_call_expression() {
    let list_ty = Type::List(Box::new(Type::Int));
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![
                HirStmt::Let {
                    name: "items".to_string(),
                    ty: list_ty.clone(),
                    value: HirExpr::ListLiteral {
                        elements: vec![HirExpr::IntLiteral(1)],
                        ty: list_ty.clone(),
                    },
                    is_mutable: true,
                },
                HirStmt::Expr {
                    expr: HirExpr::MethodCall {
                        object: Box::new(HirExpr::Name {
                            name: "items".to_string(),
                            binding_id: Some(sifr_ir::BindingId(1)),
                            ty: list_ty,
                        }),
                        method: "clear".to_string(),
                        args: vec![],
                        receiver_convention: Some(
                            sifr_type_system::ReceiverConvention::MutableBorrow,
                        ),
                        receiver_target: Some(sifr_ir::MutableReceiverTarget::Place(
                            sifr_ir::Place {
                                root: sifr_ir::BindingId(1),
                                projections: Vec::new(),
                            },
                        )),
                        mutable_arg_places: Vec::new(),
                        source: None,
                        ty: Type::None,
                    },
                },
            ],
            is_async: false,
            method_kind: MethodKind::Regular,
            receiver: None,
            decorators: vec![],
            rust_interop: Vec::new(),
            python_interop: Vec::new(),
            compiler_intrinsic: None,
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let generated = generate_rust_with_metadata(&module);
    assert!(generated.rust_source.contains("items.clear();"));
    assert!(
        generated.lowering_stats.expr_structured > 0,
        "registry-backed method call should be emitted through structured expr path"
    );
}

#[test]
fn test_structured_with_context_manager_target_is_mutable_when_body_mutates_it() {
    let handle_ty = Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "TextFileHandle".to_string(),
        fields: vec![],
        methods: vec![],
        parent_class: None,
    };
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![HirStmt::With {
                items: vec![HirWithItem {
                    target: "out".to_string(),
                    context: HirExpr::Name {
                        name: "ctx".to_string(),
                        binding_id: None,
                        ty: handle_ty.clone(),
                    },
                    kind: HirWithItemKind::Native {
                        has_context_manager_protocol: true,
                    },
                }],
                body: vec![HirStmt::Expr {
                    expr: HirExpr::MethodCall {
                        object: Box::new(HirExpr::Name {
                            name: "out".to_string(),
                            binding_id: Some(sifr_ir::BindingId(2)),
                            ty: handle_ty,
                        }),
                        method: "write".to_string(),
                        args: vec![HirExpr::StringLiteral("x".to_string())],
                        receiver_convention: Some(
                            sifr_type_system::ReceiverConvention::MutableBorrow,
                        ),
                        receiver_target: Some(sifr_ir::MutableReceiverTarget::Place(
                            sifr_ir::Place {
                                root: sifr_ir::BindingId(2),
                                projections: Vec::new(),
                            },
                        )),
                        mutable_arg_places: vec![None],
                        source: None,
                        ty: Type::Int,
                    },
                }],
            }],
            is_async: false,
            method_kind: MethodKind::Regular,
            receiver: None,
            decorators: vec![],
            rust_interop: Vec::new(),
            python_interop: Vec::new(),
            compiler_intrinsic: None,
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let generated = generate_rust_with_metadata(&module);
    assert!(
        generated
            .rust_source
            .contains("let mut out = __guard_0.ctx.__enter__();"),
        "context-manager target should be mutable when body calls a mutating method"
    );
}

#[test]
fn test_registry_dict_update_with_typed_literal_arg_lowers_to_extend() {
    let dict_ty = Type::Dict(Box::new(Type::Str), Box::new(Type::Int));
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![
                HirStmt::Let {
                    name: "d2".to_string(),
                    ty: dict_ty.clone(),
                    value: HirExpr::DictLiteral {
                        keys: vec![HirExpr::StringLiteral("a".to_string())],
                        values: vec![HirExpr::IntLiteral(1)],
                        ty: dict_ty.clone(),
                    },
                    is_mutable: true,
                },
                HirStmt::Expr {
                    expr: HirExpr::MethodCall {
                        object: Box::new(HirExpr::Name {
                            name: "d2".to_string(),
                            binding_id: None,
                            ty: dict_ty.clone(),
                        }),
                        method: "update".to_string(),
                        args: vec![HirExpr::DictLiteral {
                            keys: vec![HirExpr::StringLiteral("c".to_string())],
                            values: vec![HirExpr::IntLiteral(3)],
                            ty: dict_ty.clone(),
                        }],
                        receiver_convention: Some(
                            sifr_type_system::ReceiverConvention::SharedBorrow,
                        ),
                        receiver_target: None,
                        mutable_arg_places: vec![None],
                        source: None,
                        ty: Type::None,
                    },
                },
            ],
            is_async: false,
            method_kind: MethodKind::Regular,
            receiver: None,
            decorators: vec![],
            rust_interop: Vec::new(),
            python_interop: Vec::new(),
            compiler_intrinsic: None,
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let generated = generate_rust_with_metadata(&module);
    assert!(
        generated.rust_source.contains("d2.extend("),
        "dict update should lower via registry to HashMap::extend"
    );
    assert!(
        !generated.rust_source.contains("d2.update("),
        "dict update method call should not be emitted"
    );
}

#[test]
fn test_structured_stmt_path_handles_copy_typed_assign_expr() {
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![
                HirStmt::Let {
                    name: "x".to_string(),
                    ty: Type::Int,
                    value: HirExpr::IntLiteral(0),
                    is_mutable: true,
                },
                HirStmt::Assign {
                    name: "x".to_string(),
                    value: HirExpr::IntLiteral(7),
                },
            ],
            is_async: false,
            method_kind: MethodKind::Regular,
            receiver: None,
            decorators: vec![],
            rust_interop: Vec::new(),
            python_interop: Vec::new(),
            compiler_intrinsic: None,
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let generated = generate_rust_with_metadata(&module);
    assert!(generated.rust_source.contains("x ="));
    assert!(generated.rust_source.contains("x = 7"));
    assert!(
        generated.lowering_stats.stmt_structured >= 2,
        "let + assign should be emitted through structured stmt path"
    );
}

#[test]
fn test_structured_stmt_path_handles_copy_typed_let_expr() {
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![HirStmt::Let {
                name: "x".to_string(),
                ty: Type::Int,
                value: HirExpr::IntLiteral(7),
                is_mutable: false,
            }],
            is_async: false,
            method_kind: MethodKind::Regular,
            receiver: None,
            decorators: vec![],
            rust_interop: Vec::new(),
            python_interop: Vec::new(),
            compiler_intrinsic: None,
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let generated = generate_rust_with_metadata(&module);
    assert!(generated.rust_source.contains("let x: i64 ="));
    assert!(generated.rust_source.contains("let x: i64 = 7"));
    assert!(
        generated.lowering_stats.stmt_structured >= 1,
        "copy-typed let should be emitted through structured stmt path"
    );
}

#[test]
fn test_structured_stmt_path_handles_copy_typed_return_expr() {
    let module = HirModule {
        functions: vec![HirFunction {
            name: "value".to_string(),
            params: vec![],
            return_type: Type::Int,
            body: vec![HirStmt::Return {
                value: Some(HirExpr::IntLiteral(7)),
            }],
            is_async: false,
            method_kind: MethodKind::Regular,
            receiver: None,
            decorators: vec![],
            rust_interop: Vec::new(),
            python_interop: Vec::new(),
            compiler_intrinsic: None,
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let generated = generate_rust_with_metadata(&module);
    assert!(generated.rust_source.contains("fn value() -> i64"));
    assert!(generated.rust_source.contains("7"));
    assert!(
        generated.lowering_stats.stmt_structured >= 1,
        "copy-typed return should be emitted through structured stmt path"
    );
}

#[test]
fn test_structured_stmt_path_wraps_non_optional_string_index_into_option_local() {
    let stmt = HirStmt::Let {
        name: "part".to_string(),
        ty: Type::Union(vec![Type::Str, Type::None]),
        value: HirExpr::Index {
            object: Box::new(HirExpr::Name {
                name: "text".to_string(),
                binding_id: None,
                ty: Type::Str,
            }),
            index: Box::new(HirExpr::Name {
                name: "j".to_string(),
                binding_id: None,
                ty: Type::Int,
            }),
            ty: Type::Str,
        },
        is_mutable: false,
    };
    let mut emitter = RustEmitter::new();

    let captured = emitter.capture_structured_stmts(|inner| inner.emit_stmt(&stmt));
    let Some(RustStmt::Let {
        name, ty, value, ..
    }) = captured.first()
    else {
        panic!("expected structured let for optional local");
    };
    assert_eq!(name, "part");
    assert!(
        matches!(
            ty,
            Some(RustType::Option(inner)) if matches!(inner.as_ref(), RustType::String_)
        ) || matches!(ty, Some(RustType::Named(named)) if named == "Option<String>")
    );
    assert!(matches!(
        value,
        RustExpr::FnCall { func, args }
            if matches!(func.as_ref(), RustExpr::Path(path) if path == &vec!["Some".to_string()])
                && matches!(args.as_slice(), [RustExpr::Block { .. }])
    ));
}

#[test]
fn test_structured_stmt_path_handles_non_optional_string_index_return_expr() {
    let module = HirModule {
        functions: vec![HirFunction {
            name: "char_at".to_string(),
            params: vec![
                HirParam {
                    name: "text".to_string(),
                    ty: Type::Str,
                    default: None,
                    keyword_only: false,
                    convention: ParamConvention::borrow(),
                },
                HirParam {
                    name: "j".to_string(),
                    ty: Type::Int,
                    default: None,
                    keyword_only: false,
                    convention: ParamConvention::own(),
                },
            ],
            return_type: Type::Str,
            body: vec![HirStmt::Return {
                value: Some(HirExpr::Index {
                    object: Box::new(HirExpr::Name {
                        name: "text".to_string(),
                        binding_id: None,
                        ty: Type::Str,
                    }),
                    index: Box::new(HirExpr::Name {
                        name: "j".to_string(),
                        binding_id: None,
                        ty: Type::Int,
                    }),
                    ty: Type::Str,
                }),
            }],
            is_async: false,
            method_kind: MethodKind::Regular,
            receiver: None,
            decorators: vec![],
            rust_interop: Vec::new(),
            python_interop: Vec::new(),
            compiler_intrinsic: None,
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
        .contains("let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();"));
    assert!(generated
        .rust_source
        .contains("let Some(__indexed_char) = __sifr_chars_text.get(j as usize).map(|c| c.to_string()) else {"));
    assert!(generated.rust_source.contains(";\n    __indexed_char\n}"));
    assert!(
        generated.lowering_stats.stmt_structured >= 1,
        "non-optional string index return should stay on the structured stmt path"
    );
}

#[test]
fn test_structured_tuple_index_field_assign_clones_non_copy_element() {
    let stmt = HirStmt::FieldAssign {
        object: "callback".to_string(),
        field: "kind".to_string(),
        field_ty: Type::Str,
        value: HirExpr::Index {
            object: Box::new(HirExpr::Name {
                name: "raw".to_string(),
                binding_id: None,
                ty: Type::Tuple(vec![Type::Int, Type::Str]),
            }),
            index: Box::new(HirExpr::IntLiteral(1)),
            ty: Type::Str,
        },
    };
    let mut emitter = RustEmitter::new();

    let captured = emitter.capture_structured_stmts(|inner| inner.emit_stmt(&stmt));
    let Some(RustStmt::Assign { target, value }) = captured.first() else {
        panic!("expected structured tuple-index field assignment");
    };
    assert!(matches!(
        target,
        RustExpr::Field { expr, field }
            if matches!(expr.as_ref(), RustExpr::Ident(object) if object == "callback")
                && field == "kind"
    ));
    assert!(
        matches!(
            value,
            RustExpr::Clone(inner)
                if matches!(
                    inner.as_ref(),
                    RustExpr::Field { expr, field }
                        if matches!(
                            expr.as_ref(),
                            RustExpr::Paren(inner)
                                if matches!(inner.as_ref(), RustExpr::Ident(object) if object == "raw")
                        ) && field == "1"
                )
        ),
        "expected cloned tuple field, got {value:?}"
    );
}

#[test]
fn test_structured_tuple_index_field_assign_moves_non_clone_element() {
    let resource_ty = Type::Class {
        identity: Some("_sifr.python.ResourceIdentity".to_string()),
        type_args: Vec::new(),
        name: "ResourceIdentity".to_string(),
        fields: Vec::new(),
        methods: Vec::new(),
        parent_class: Some("NonSend".to_string()),
    };
    let stmt = HirStmt::FieldAssign {
        object: "callback".to_string(),
        field: "identity".to_string(),
        field_ty: resource_ty.clone(),
        value: HirExpr::Index {
            object: Box::new(HirExpr::Name {
                name: "raw".to_string(),
                binding_id: None,
                ty: Type::Tuple(vec![resource_ty.clone(), Type::Str]),
            }),
            index: Box::new(HirExpr::IntLiteral(0)),
            ty: resource_ty,
        },
    };
    let mut emitter = RustEmitter::new();

    let captured = emitter.capture_structured_stmts(|inner| inner.emit_stmt(&stmt));
    let Some(RustStmt::Assign { value, .. }) = captured.first() else {
        panic!("expected structured tuple-index field assignment");
    };
    assert!(
        matches!(
            value,
            RustExpr::Field { expr, field }
                if matches!(
                    expr.as_ref(),
                    RustExpr::Paren(inner)
                        if matches!(inner.as_ref(), RustExpr::Ident(object) if object == "raw")
                ) && field == "0"
        ),
        "expected moved tuple field, got {value:?}"
    );
}

#[test]
fn test_emit_expr_prefers_structured_name_path() {
    let mut emitter = RustEmitter::new();
    emitter.intrinsic_functions.insert("clock".to_string());
    let expr = HirExpr::Name {
        name: "clock".to_string(),
        binding_id: None,
        ty: Type::Int,
    };

    let lowered = emitter
        .try_lower_registry_expr_strict(&expr)
        .expect("name expression should lower through structured registry path");

    assert_eq!(crate::render_expr(&lowered), "clock");
}

#[test]
fn test_emit_expr_borrowed_compare_is_structured() {
    let mut emitter = RustEmitter::new();
    emitter.borrowed_params.insert("lhs".to_string());
    let expr = HirExpr::Compare {
        left: Box::new(HirExpr::Name {
            name: "lhs".to_string(),
            binding_id: None,
            ty: Type::Str,
        }),
        ops: vec!["==".to_string()],
        comparators: vec![HirExpr::StringLiteral("ok".to_string())],
        ty: Type::Bool,
    };

    let lowered = emitter
        .try_lower_registry_expr_strict(&expr)
        .expect("borrowed compare should lower through structured registry path");
    let rendered = crate::render_expr(&lowered);

    assert!(rendered.contains("lhs"));
    assert!(rendered.contains("=="));
}

#[test]
fn test_lib_decomposition_guards_keep_stmt_expr_logic_out_of_lib_rs() {
    let lib_src = include_str!("../lib.rs");
    let emitter_state_src = include_str!("../lib_emitter_state.rs");

    assert!(!lib_src.contains("mod stmt_emitter;"));
    assert!(!lib_src.contains("mod expr_emitter;"));
    assert!(!lib_src.contains("CodegenLoweringMode"));
    assert!(!lib_src.contains("StructuredPreferred"));
    assert!(!lib_src.contains("should_force_stmt_string_path"));
    assert!(!lib_src.contains("should_force_expr_string_path"));
    assert!(!lib_src.contains("fn emit_expr(&mut self, expr: &HirExpr) {"));
    assert!(!lib_src.contains("fn try_lower_structured_expr("));
    assert!(!lib_src.contains("fn emit_stmt(&mut self, stmt: &HirStmt) {"));

    let emit_stmt_start = emitter_state_src
        .find("fn emit_stmt(&mut self, stmt: &HirStmt) {")
        .expect("emit_stmt wrapper should exist");
    let impl_end = emitter_state_src[emit_stmt_start..]
        .find("\n    }\n}")
        .map(|offset| emit_stmt_start + offset)
        .expect("emit_stmt wrapper should end before impl close");
    let emit_stmt_wrapper = &emitter_state_src[emit_stmt_start..impl_end];
    assert!(emit_stmt_wrapper.contains("structured statement emission missing for production path"));
    assert!(!emit_stmt_wrapper.contains("self.try_emit_stmt_string_"));
    assert!(
        !emit_stmt_wrapper.contains("match stmt"),
        "emit_stmt should stay orchestration-only"
    );

    let lib_lines = lib_src.lines().count();
    assert!(
        lib_lines <= 1450,
        "lib.rs should stay decomposed (current lines: {lib_lines})"
    );
}

#[test]
fn test_production_lowering_rules_uses_result_helpers_only() {
    let lib_src = include_str!("../lib.rs");
    let emitter_state_src = include_str!("../lib_emitter_state.rs");
    let lower_expr_src = include_str!("../lower_expr/leaves_and_plain_calls.rs");
    let module_constants_src = include_str!("../module_constants.rs");
    let field_rewrites_src = include_str!("../expr_render_helpers/field_and_stdlib_rewrites.rs");

    assert!(emitter_state_src.contains("try_lower_simple_stmt_with_scope_result_and_bindings("));
    assert!(lower_expr_src.contains("pub fn try_lower_leaf_expr_result("));
    assert!(module_constants_src.contains("try_lower_simple_module_constant_item_result("));
    assert!(field_rewrites_src.contains("try_lower_registry_expr_result("));

    assert!(!lib_src.contains("try_lower_simple_stmt_with_scope("));
    assert!(!lib_src.contains("try_lower_leaf_expr("));
    assert!(!module_constants_src.contains("try_lower_simple_module_constant_item("));
}

#[test]
fn test_capture_structured_stmts_collects_ir_without_output_writes() {
    let mut emitter = RustEmitter::new();
    let stmt = HirStmt::Let {
        name: "x".to_string(),
        ty: Type::Int,
        value: HirExpr::IntLiteral(1),
        is_mutable: false,
    };

    let captured = emitter.capture_structured_stmts(|inner| inner.emit_stmt(&stmt));

    assert_eq!(captured.len(), 1);
    assert!(matches!(
        captured.first(),
        Some(RustStmt::Let {
            name,
            mutable: false,
            ..
        }) if name == "x"
    ));
}

#[test]
fn test_structured_stmt_path_handles_nested_subscript_augassign_inside_loop_if() {
    let stmt = HirStmt::For {
        target: "i".to_string(),
        target_ty: Type::Int,
        iter: HirExpr::RangeLiteral {
            start: Box::new(HirExpr::IntLiteral(0)),
            end: Box::new(HirExpr::IntLiteral(3)),
            step: None,
            ty: Type::Range,
        },
        body: vec![HirStmt::If {
            condition: HirExpr::BoolLiteral(true),
            then_body: vec![HirStmt::SubscriptAugAssign {
                object: "values".to_string(),
                index: HirExpr::BinOp {
                    left: Box::new(HirExpr::Name {
                        name: "i".to_string(),
                        binding_id: None,
                        ty: Type::Int,
                    }),
                    op: "+".to_string(),
                    right: Box::new(HirExpr::IntLiteral(1)),
                    ty: Type::Int,
                },
                op: "*=".to_string(),
                value: HirExpr::IntLiteral(2),
                object_ty: Type::List(Box::new(Type::Int)),
                missing_key_error: None,
            }],
            elif_clauses: vec![],
            else_body: None,
        }],
        else_body: None,
    };

    let mut emitter = RustEmitter::new();
    let captured = emitter.capture_structured_stmts(|inner| inner.emit_stmt(&stmt));

    assert!(matches!(captured.first(), Some(RustStmt::For { .. })));
}

#[test]
fn test_structured_stmt_path_handles_attribute_list_subscript_assign_inside_if() {
    let stmt = HirStmt::If {
        condition: HirExpr::BoolLiteral(true),
        then_body: vec![HirStmt::AttributeSubscriptAssign {
            object: "self".to_string(),
            field: "history".to_string(),
            index: HirExpr::BinOp {
                left: Box::new(HirExpr::Name {
                    name: "i".to_string(),
                    binding_id: None,
                    ty: Type::Int,
                }),
                op: "+".to_string(),
                right: Box::new(HirExpr::IntLiteral(1)),
                ty: Type::Int,
            },
            value: HirExpr::Call {
                mutable_arg_places: Vec::new(),
                func: "str".to_string(),
                args: vec![HirExpr::Name {
                    name: "url".to_string(),
                    binding_id: None,
                    ty: Type::Str,
                }],
                ty: Type::Str,
            },
            field_ty: Type::List(Box::new(Type::Str)),
        }],
        elif_clauses: vec![],
        else_body: None,
    };

    let mut emitter = RustEmitter::new();
    let captured = emitter.capture_structured_stmts(|inner| inner.emit_stmt(&stmt));

    assert!(matches!(captured.first(), Some(RustStmt::If { .. })));
}

#[test]
fn test_structured_stmt_path_handles_top_level_attribute_list_subscript_assign() {
    let stmt = HirStmt::AttributeSubscriptAssign {
        object: "self".to_string(),
        field: "history".to_string(),
        index: HirExpr::Name {
            name: "i".to_string(),
            binding_id: None,
            ty: Type::Int,
        },
        value: HirExpr::StringLiteral("page".to_string()),
        field_ty: Type::List(Box::new(Type::Str)),
    };

    let mut emitter = RustEmitter::new();
    let captured = emitter.capture_structured_stmts(|inner| inner.emit_stmt(&stmt));

    assert!(matches!(captured.first(), Some(RustStmt::Block(_))));
}

#[test]
fn test_structured_stmt_path_handles_delete_with_name_key_inside_loop_if() {
    let stmt = HirStmt::For {
        target: "ch".to_string(),
        target_ty: Type::Str,
        iter: HirExpr::Name {
            name: "order".to_string(),
            binding_id: None,
            ty: Type::Str,
        },
        body: vec![HirStmt::If {
            condition: HirExpr::ContainsOp {
                element: Box::new(HirExpr::Name {
                    name: "ch".to_string(),
                    binding_id: None,
                    ty: Type::Str,
                }),
                collection: Box::new(HirExpr::Name {
                    name: "counts".to_string(),
                    binding_id: None,
                    ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
                }),
                ty: Type::Bool,
            },
            then_body: vec![HirStmt::Delete {
                object: HirExpr::Name {
                    name: "counts".to_string(),
                    binding_id: None,
                    ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
                },
                index: HirExpr::Name {
                    name: "ch".to_string(),
                    binding_id: None,
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

    assert!(matches!(captured.first(), Some(RustStmt::For { .. })));
}

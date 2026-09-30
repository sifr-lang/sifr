use crate::{HirExpr, RustEmitter, RustExpr, render_expr};
use sifr_ir::{BindingId, CallableIdentity, MethodAuthority, MutableReceiverTarget, Place};
use sifr_type_system::{ReceiverConvention, Type};

pub(crate) fn builtin_authority(method: &str) -> MethodAuthority {
    MethodAuthority::BuiltinIntrinsic {
        declaration: identity("sifr.builtin", method),
    }
}

pub(crate) fn contextual_authority(method: &str) -> MethodAuthority {
    MethodAuthority::RustAdapted {
        declaration: identity("fixture.bridge", method),
    }
}

fn identity(module: &str, method: &str) -> CallableIdentity {
    CallableIdentity {
        module: module.to_string(),
        owner: Some("Owner".to_string()),
        symbol: method.to_string(),
        generic_arguments: Vec::new(),
        signature: "() -> None".to_string(),
    }
}

fn call(
    receiver_ty: Type,
    method: &str,
    authority: MethodAuthority,
    args: Vec<HirExpr>,
) -> HirExpr {
    let mutable = method == "append";
    HirExpr::MethodCall {
        object: Box::new(HirExpr::Name {
            name: "value".to_string(),
            binding_id: Some(BindingId(1)),
            ty: receiver_ty,
        }),
        method: method.to_string(),
        mutable_arg_places: vec![None; args.len()],
        args,
        authority,
        receiver_convention: Some(if mutable {
            ReceiverConvention::MutableBorrow
        } else {
            ReceiverConvention::SharedBorrow
        }),
        receiver_target: mutable.then(|| {
            MutableReceiverTarget::Place(Place {
                root: BindingId(1),
                projections: Vec::new(),
            })
        }),
        source: None,
        ty: Type::None,
    }
}

fn builtin(receiver_ty: Type, method: &str, args: Vec<HirExpr>) -> HirExpr {
    call(
        receiver_ty,
        method,
        MethodAuthority::BuiltinIntrinsic {
            declaration: identity("sifr.builtin", method),
        },
        args,
    )
}

#[test]
fn typed_builtin_dispatch_and_strict_decline() {
    let list_ty = Type::List(Box::new(Type::Int));
    let accepted = builtin(list_ty.clone(), "append", vec![HirExpr::IntLiteral(1)]);
    assert_eq!(
        crate::method_call_emitter::source_method_path(&accepted).unwrap(),
        crate::method_call_emitter::SourceMethodPath::BuiltinIntrinsic
    );
    let emitted = RustEmitter::new()
        .lower_stmt_expr_for_ir(&accepted)
        .unwrap()
        .unwrap();
    assert_eq!(render_expr(&emitted), "value.push(SifrInt::from_i64(1))");

    let declined = builtin(list_ty.clone(), "invented", Vec::new());
    let error = RustEmitter::new()
        .lower_stmt_expr_for_ir(&declined)
        .unwrap_err();
    assert!(
        error
            .message
            .contains("builtin method 'invented' is unsupported")
    );
    assert!(
        crate::method_call_emitter::source_method_path(&call(
            list_ty.clone(),
            "append",
            MethodAuthority::Unclassified,
            vec![HirExpr::IntLiteral(1)]
        ))
        .unwrap_err()
        .message
        .contains("unclassified source method")
    );
    assert!(
        crate::method_call_emitter::source_method_path(&call(
            list_ty,
            "append",
            MethodAuthority::BuiltinIntrinsic {
                declaration: identity("other.module", "append"),
            },
            vec![HirExpr::IntLiteral(1)]
        ))
        .is_err()
    );
}

#[test]
fn user_protocol_and_contextual_paths() {
    let nominal_ty = Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "Owner".to_string(),
        fields: Default::default(),
        methods: Default::default(),
        parent_class: None,
    };
    for authority in [
        MethodAuthority::LocalNominal {
            declaration: identity("user", "append"),
        },
        MethodAuthority::Protocol {
            declaration: identity("user", "append"),
        },
        MethodAuthority::RustAdapted {
            declaration: identity("bridge", "append"),
        },
    ] {
        let expr = call(
            nominal_ty.clone(),
            "append",
            authority.clone(),
            vec![HirExpr::IntLiteral(1)],
        );
        let expected = if matches!(authority, MethodAuthority::RustAdapted { .. }) {
            crate::method_call_emitter::SourceMethodPath::ContextualRust
        } else {
            crate::method_call_emitter::SourceMethodPath::UserProtocol
        };
        assert_eq!(
            crate::method_call_emitter::source_method_path(&expr).unwrap(),
            expected
        );
        let emitted = RustEmitter::new()
            .lower_stmt_expr_for_ir(&expr)
            .unwrap()
            .unwrap();
        assert!(render_expr(&emitted).contains("value.append("));
    }

    let rust_ir_call = RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident("value".to_string())),
        method: "clone".to_string(),
        args: Vec::new(),
    };
    assert_eq!(render_expr(&rust_ir_call), "value.clone()");
}

#[test]
fn list_append_cloned_decline_regression() {
    let list_ty = Type::List(Box::new(Type::Int));
    let object = RustExpr::Ident("value".to_string());
    assert!(crate::methods::lower_method(&list_ty, "append", &object, &[]).is_none());
    assert!(
        crate::methods::lower_method(
            &list_ty,
            "cloned",
            &object,
            &[RustExpr::Ident("extra".to_string())],
        )
        .is_none()
    );

    let malformed_append = builtin(list_ty.clone(), "append", Vec::new());
    let malformed_cloned = builtin(list_ty.clone(), "cloned", vec![HirExpr::IntLiteral(1)]);
    for expr in [&malformed_append, &malformed_cloned] {
        let error = RustEmitter::new().lower_stmt_expr_for_ir(expr).unwrap_err();
        assert!(error.message.contains("builtin method"));
    }

    let cloned = builtin(list_ty, "cloned", Vec::new());
    let emitted = RustEmitter::new()
        .lower_stmt_expr_for_ir(&cloned)
        .unwrap()
        .unwrap();
    assert_eq!(render_expr(&emitted), "value.clone()");
}

#[test]
fn nested_builtin_declines_preserve_structural_errors() {
    let list_ty = Type::List(Box::new(Type::Int));
    let mut receiver_call = builtin(list_ty.clone(), "cloned", Vec::new());
    if let HirExpr::MethodCall { object, .. } = &mut receiver_call {
        *object = Box::new(builtin(list_ty.clone(), "invented_receiver", Vec::new()));
    }
    let argument_call = builtin(
        list_ty.clone(),
        "append",
        vec![builtin(list_ty.clone(), "invented_argument", Vec::new())],
    );
    let function_call = HirExpr::Call {
        func: "consume".to_string(),
        args: vec![builtin(
            list_ty.clone(),
            "invented_function_arg",
            Vec::new(),
        )],
        mutable_arg_places: vec![None],
        ty: Type::None,
    };
    for (expr, method) in [
        (receiver_call, "invented_receiver"),
        (argument_call, "invented_argument"),
        (function_call, "invented_function_arg"),
    ] {
        let error = RustEmitter::new()
            .lower_stmt_expr_for_ir(&expr)
            .unwrap_err();
        assert!(
            error
                .message
                .contains(&format!("builtin method '{method}' is unsupported")),
            "{}",
            error.message
        );
    }
    let mut unclassified_argument = builtin(list_ty.clone(), "cloned", Vec::new());
    if let HirExpr::MethodCall { authority, .. } = &mut unclassified_argument {
        *authority = MethodAuthority::Unclassified;
    }
    let expr = builtin(list_ty, "append", vec![unclassified_argument]);
    assert!(
        RustEmitter::new()
            .lower_stmt_expr_for_ir(&expr)
            .unwrap_err()
            .message
            .contains("unclassified source method")
    );
}

#[derive(Clone, Copy)]
enum StatementMethodConsumer {
    StatementOnly,
    StructuredStatement,
    NestedBlock,
    Class,
}

fn statement_method_output(
    consumer: StatementMethodConsumer,
    expr: &HirExpr,
) -> Result<String, String> {
    let mut emitter = RustEmitter::new();
    match consumer {
        StatementMethodConsumer::StatementOnly => emitter
            .try_lower_stmt_expr_statement_only(expr)
            .map_err(|error| error.message)
            .map(|lowered| render_expr(&lowered.expect("admitted statement method"))),
        StatementMethodConsumer::StructuredStatement => {
            let stmt = sifr_ir::HirStmt::Expr { expr: expr.clone() };
            let mut result = Ok(false);
            let captured = emitter.capture_structured_stmts(|inner| {
                result = inner.try_lower_structured_stmt_with_following(&stmt, None);
            });
            assert!(result.map_err(|error| error.message)?);
            let [crate::RustStmt::Expr(lowered)] = captured.as_slice() else {
                panic!("expected one statement expression: {captured:?}");
            };
            Ok(render_expr(lowered))
        }
        StatementMethodConsumer::NestedBlock => {
            let stmt = sifr_ir::HirStmt::If {
                condition: HirExpr::BoolLiteral(true),
                then_body: vec![sifr_ir::HirStmt::Expr { expr: expr.clone() }],
                elif_clauses: Vec::new(),
                else_body: None,
            };
            let lowered = emitter
                .try_lower_stmt_block_for_ir(&[stmt])
                .map_err(|error| error.message)?
                .expect("admitted nested method");
            let [crate::RustStmt::If { then_body, .. }] = lowered.as_slice() else {
                panic!("expected nested if: {lowered:?}");
            };
            let [crate::RustStmt::Expr(lowered)] = then_body.as_slice() else {
                panic!("expected one nested expression: {then_body:?}");
            };
            Ok(render_expr(lowered))
        }
        StatementMethodConsumer::Class => {
            // The existing strict class boundary surfaces structural errors as
            // compiler diagnostics via panic; it must not emit a builtin retry.
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                emitter.lower_class_expr_strict(expr, "method authority regression")
            }))
            .map(|lowered| render_expr(&lowered))
            .map_err(|payload| {
                if let Some(message) = payload.downcast_ref::<String>() {
                    message.clone()
                } else if let Some(message) = payload.downcast_ref::<&str>() {
                    (*message).to_string()
                } else {
                    panic!("class error must retain its diagnostic");
                }
            })
        }
    }
}

fn assert_statement_method_admission(consumer: StatementMethodConsumer) {
    let list_ty = Type::List(Box::new(Type::Int));
    let accepted = builtin(list_ty.clone(), "append", vec![HirExpr::IntLiteral(1)]);
    assert_eq!(
        statement_method_output(consumer, &accepted).unwrap(),
        "value.push(SifrInt::from_i64(1))"
    );
    assert_eq!(
        statement_method_output(consumer, &builtin(list_ty.clone(), "cloned", vec![])).unwrap(),
        "value.clone()"
    );
    for expr in [
        builtin(list_ty.clone(), "invented", vec![]),
        builtin(list_ty.clone(), "append", vec![]),
        builtin(list_ty.clone(), "cloned", vec![HirExpr::IntLiteral(1)]),
    ] {
        let HirExpr::MethodCall { method, .. } = &expr else {
            unreachable!()
        };
        assert!(
            statement_method_output(consumer, &expr)
                .unwrap_err()
                .contains(&format!("builtin method '{method}' is unsupported"))
        );
    }
    for (authority, diagnostic) in [
        (MethodAuthority::Unclassified, "unclassified source method"),
        (
            MethodAuthority::BuiltinIntrinsic {
                declaration: identity("invalid.builtin", "append"),
            },
            "invalid builtin method authority",
        ),
        (
            MethodAuthority::Imported {
                declaration: identity("user", "other"),
            },
            "invalid user or protocol method authority",
        ),
    ] {
        let expr = call(
            list_ty.clone(),
            "append",
            authority,
            vec![HirExpr::IntLiteral(1)],
        );
        assert!(
            statement_method_output(consumer, &expr)
                .unwrap_err()
                .contains(diagnostic)
        );
    }
    // A builtin-shaped receiver and spelling cannot override a proven
    // nonbuiltin carrier: all five authorities retain `.append`, never `.push`.
    for authority in [
        MethodAuthority::LocalNominal {
            declaration: identity("user", "append"),
        },
        MethodAuthority::InheritedNominal {
            declaration: identity("user", "append"),
        },
        MethodAuthority::Imported {
            declaration: identity("user", "append"),
        },
        MethodAuthority::Protocol {
            declaration: identity("user", "append"),
        },
        contextual_authority("append"),
    ] {
        let expr = call(
            list_ty.clone(),
            "append",
            authority,
            vec![HirExpr::IntLiteral(1)],
        );
        let emitted = statement_method_output(consumer, &expr).unwrap();
        assert_eq!(emitted, "value.append(SifrInt::from_i64(1))");
        assert!(!emitted.contains(".push("));
    }
}

#[test]
fn statement_only_method_admission_and_strict_decline() {
    assert_statement_method_admission(StatementMethodConsumer::StatementOnly);
    assert_statement_method_admission(StatementMethodConsumer::StructuredStatement);
}

#[test]
fn nested_statement_method_admission_and_strict_decline() {
    assert_statement_method_admission(StatementMethodConsumer::NestedBlock);
}

#[test]
fn class_statement_method_admission_and_strict_decline() {
    assert_statement_method_admission(StatementMethodConsumer::Class);
}

fn length_call(receiver_ty: Type, authority: MethodAuthority, result_ty: Type) -> HirExpr {
    let mut expr = call(receiver_ty, "len", authority, Vec::new());
    if let HirExpr::MethodCall { ty, .. } = &mut expr {
        *ty = result_ty;
    }
    expr
}

fn condition_variants(expr: &HirExpr) -> Vec<HirExpr> {
    vec![
        expr.clone(),
        HirExpr::UnaryOp {
            op: "not".to_string(),
            operand: Box::new(expr.clone()),
            ty: Type::Bool,
        },
        HirExpr::Compare {
            left: Box::new(expr.clone()),
            ops: vec![">".to_string()],
            comparators: vec![if matches!(expr.ty(), Type::Float) {
                HirExpr::FloatLiteral(0.0)
            } else {
                HirExpr::IntLiteral(0)
            }],
            ty: Type::Bool,
        },
    ]
}

fn condition_stmts(condition: &HirExpr) -> Vec<sifr_ir::HirStmt> {
    use sifr_ir::HirStmt;
    vec![
        HirStmt::If {
            condition: condition.clone(),
            then_body: vec![HirStmt::Pass],
            elif_clauses: Vec::new(),
            else_body: None,
        },
        HirStmt::While {
            condition: condition.clone(),
            body: vec![HirStmt::Break],
            else_body: None,
        },
        HirStmt::Assert {
            test: condition.clone(),
            msg: None,
        },
    ]
}

fn indexed_length(expr: &HirExpr, dictionary: bool) -> HirExpr {
    let mut expr = expr.clone();
    if let HirExpr::MethodCall { object, .. } = &mut expr {
        let ty = object.ty().clone();
        *object = Box::new(HirExpr::Index {
            object: Box::new(HirExpr::Name {
                name: "rows".to_string(),
                binding_id: Some(BindingId(2)),
                ty: if dictionary {
                    Type::Dict(Box::new(Type::Int), Box::new(ty.clone()))
                } else {
                    Type::List(Box::new(ty.clone()))
                },
            }),
            index: Box::new(HirExpr::IntLiteral(0)),
            ty,
        });
    }
    expr
}

fn simple_condition_stmt(stmt: &sifr_ir::HirStmt) -> Option<Vec<crate::RustStmt>> {
    crate::lower_stmt::try_lower_simple_stmt(stmt, false, &Default::default(), &Default::default())
}

#[test]
fn condition_specializations_decline_unclassified_without_builtin_retry() {
    let list_ty = Type::List(Box::new(Type::Int));
    let invalid = [
        (
            length_call(list_ty.clone(), MethodAuthority::Unclassified, Type::Int),
            "unclassified source method",
        ),
        (
            length_call(
                list_ty.clone(),
                MethodAuthority::BuiltinIntrinsic {
                    declaration: identity("other.module", "len"),
                },
                Type::Int,
            ),
            "invalid builtin method authority",
        ),
        (
            length_call(list_ty.clone(), builtin_authority("other"), Type::Int),
            "invalid builtin method authority",
        ),
        (
            length_call(Type::Int, builtin_authority("len"), Type::Int),
            "builtin method 'len' is unsupported",
        ),
    ];
    // The shared registry must also decline unsupported receivers, so that
    // declining a condition specialization reaches the same structural error.
    assert!(
        crate::methods::lower_method(
            &Type::Int,
            "len",
            &RustExpr::Ident("value".to_string()),
            &[],
        )
        .is_none()
    );
    assert!(
        crate::methods::lower_method(
            &Type::Union(vec![Type::Int, Type::None]),
            "len",
            &RustExpr::Ident("value".to_string()),
            &[],
        )
        .is_none()
    );
    for (expr, diagnostic) in invalid {
        let mut variants = condition_variants(&expr);
        for dictionary in [false, true] {
            variants.push(condition_variants(&indexed_length(&expr, dictionary)).remove(2));
        }
        for condition in variants {
            for stmt in condition_stmts(&condition) {
                assert!(simple_condition_stmt(&stmt).is_none(), "{stmt:?}");
                let error = RustEmitter::new()
                    .try_lower_stmt_block_for_ir(&[stmt])
                    .unwrap_err();
                assert!(error.message.contains(diagnostic), "{}", error.message);
            }
            let error = RustEmitter::new()
                .lower_condition_expr_for_ir(&condition)
                .unwrap_err();
            assert!(error.message.contains(diagnostic), "{}", error.message);
        }
    }
}

#[test]
fn condition_comparison_and_truthiness_preserve_authority() {
    use sifr_ir::HirStmt;
    let list_ty = Type::List(Box::new(Type::Int));
    let authorities = [
        builtin_authority("len"),
        MethodAuthority::LocalNominal {
            declaration: identity("user", "len"),
        },
        MethodAuthority::InheritedNominal {
            declaration: identity("user", "len"),
        },
        MethodAuthority::Imported {
            declaration: identity("user", "len"),
        },
        MethodAuthority::Protocol {
            declaration: identity("user", "len"),
        },
        contextual_authority("len"),
    ];
    let text_len = length_call(Type::Str, builtin_authority("len"), Type::Int);
    assert!(simple_condition_stmt(&condition_stmts(&text_len)[0]).is_none());
    let mut cached_emitter = RustEmitter::new();
    cached_emitter
        .string_char_cache_vars
        .insert("value".to_string(), "cached_chars".to_string());
    let ordinary_cached = render_expr(
        &cached_emitter
            .lower_stmt_expr_for_ir(&text_len)
            .unwrap()
            .unwrap(),
    );
    let cached_condition = render_expr(
        &cached_emitter
            .lower_condition_expr_for_ir(&text_len)
            .unwrap()
            .unwrap(),
    );
    assert!(
        cached_condition.contains(&ordinary_cached),
        "{cached_condition}"
    );
    assert!(
        cached_condition.contains("cached_chars.len()"),
        "{cached_condition}"
    );
    assert!(!cached_condition.contains("chars()"), "{cached_condition}");
    for authority in authorities {
        let builtin = matches!(authority, MethodAuthority::BuiltinIntrinsic { .. });
        let mut receivers = vec![list_ty.clone(), Type::Str];
        if !builtin {
            receivers.push(Type::Class {
                identity: None,
                type_args: Vec::new(),
                name: "Owner".to_string(),
                fields: Default::default(),
                methods: Default::default(),
                parent_class: Some("Base".to_string()),
            });
        }
        for receiver_ty in receivers {
            for result_ty in if builtin {
                vec![Type::Int]
            } else {
                vec![
                    Type::Int,
                    Type::Float,
                    Type::Union(vec![Type::Int, Type::None]),
                ]
            } {
                let expr = length_call(receiver_ty.clone(), authority.clone(), result_ty);
                let ordinary = render_expr(
                    &RustEmitter::new()
                        .lower_stmt_expr_for_ir(&expr)
                        .unwrap()
                        .unwrap(),
                );
                if builtin && matches!(receiver_ty, Type::Str) {
                    assert!(ordinary.contains("chars()"), "{ordinary}");
                } else if !builtin {
                    assert_eq!(ordinary, "value.len()");
                }
                for condition in condition_variants(&expr) {
                    let structured = render_expr(
                        &RustEmitter::new()
                            .lower_condition_expr_for_ir(&condition)
                            .unwrap()
                            .unwrap(),
                    );
                    assert!(structured.contains(&ordinary), "{structured} vs {ordinary}");
                    for stmt in condition_stmts(&condition) {
                        if let Some(simple) = simple_condition_stmt(&stmt) {
                            let output = crate::render_stmts(&simple);
                            assert!(builtin, "nonbuiltin escaped to simple: {output}");
                            assert!(output.contains(&ordinary), "{output} vs {ordinary}");
                        }
                        for candidate in [
                            stmt.clone(),
                            HirStmt::If {
                                condition: HirExpr::BoolLiteral(true),
                                then_body: vec![stmt],
                                elif_clauses: Vec::new(),
                                else_body: None,
                            },
                        ] {
                            let lowered = RustEmitter::new()
                                .try_lower_stmt_block_for_ir(&[candidate])
                                .unwrap()
                                .unwrap();
                            let output = crate::render_stmts(&lowered);
                            assert!(output.contains(&ordinary), "{output} vs {ordinary}");
                            if !builtin {
                                assert!(!output.contains("SifrInt::from(value.len())"), "{output}");
                            }
                        }
                    }
                }
                for dictionary in [false, true] {
                    let indexed = indexed_length(&expr, dictionary);
                    let comparison = condition_variants(&indexed).remove(2);
                    for stmt in condition_stmts(&comparison) {
                        if let Some(simple) = simple_condition_stmt(&stmt) {
                            let output = crate::render_stmts(&simple);
                            assert!(builtin, "{output}");
                            assert!(output.contains(".map("), "{output}");
                            if matches!(receiver_ty, Type::Str) {
                                assert!(output.contains("chars()"), "{output}");
                            }
                        }
                        let output = crate::render_stmts(
                            &RustEmitter::new()
                                .try_lower_stmt_block_for_ir(&[stmt])
                                .unwrap()
                                .unwrap(),
                        );
                        assert!(output.contains("rows"), "{output}");
                        assert!(output.contains("get("), "{output}");
                        assert!(!output.contains("unwrap("), "{output}");
                        if !builtin {
                            assert!(!output.contains("SifrInt::from("), "{output}");
                            assert!(!output.contains("chars()"), "{output}");
                        }
                    }
                }
            }
        }
        if !builtin {
            for convention in [
                ReceiverConvention::SharedBorrow,
                ReceiverConvention::MutableBorrow,
            ] {
                let mut expr = length_call(list_ty.clone(), authority.clone(), Type::Int);
                if let HirExpr::MethodCall {
                    object,
                    receiver_convention,
                    receiver_target,
                    ..
                } = &mut expr
                {
                    *object = Box::new(HirExpr::FieldAccess {
                        object: Box::new(HirExpr::Name {
                            name: "value".to_string(),
                            binding_id: Some(BindingId(1)),
                            ty: Type::Class {
                                identity: None,
                                type_args: Vec::new(),
                                name: "Container".to_string(),
                                fields: Default::default(),
                                methods: Default::default(),
                                parent_class: None,
                            },
                        }),
                        field: "items".to_string(),
                        ty: list_ty.clone(),
                    });
                    *receiver_convention = Some(convention);
                    *receiver_target = Some(MutableReceiverTarget::Place(Place {
                        root: BindingId(1),
                        projections: vec![sifr_ir::PlaceProjection::Field(
                            sifr_ir::FieldIdentity {
                                declaring_class: "Container".to_string(),
                                field: "items".to_string(),
                            },
                        )],
                    }));
                }
                let ordinary = render_expr(
                    &RustEmitter::new()
                        .lower_stmt_expr_for_ir(&expr)
                        .unwrap()
                        .unwrap(),
                );
                assert!(ordinary.contains("value.items"), "{ordinary}");
                for condition in condition_variants(&expr) {
                    let lowered = render_expr(
                        &RustEmitter::new()
                            .lower_condition_expr_for_ir(&condition)
                            .unwrap()
                            .unwrap(),
                    );
                    assert!(lowered.contains(&ordinary), "{lowered} vs {ordinary}");
                    assert!(!lowered.contains("clone()"), "{lowered}");
                }
            }
        }
    }
}

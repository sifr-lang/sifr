// Included in the authority test namespace so the reserved exact names remain
// stable; these helpers and cases own checked-dictionary guard admission only.
fn keys_guard_parts(authority: Option<MethodAuthority>, negated: bool) -> (HirExpr, HirExpr) {
    let dictionary_ty = Type::Dict(Box::new(Type::Int), Box::new(Type::Str));
    let mut collection = call(
        dictionary_ty,
        "keys",
        authority
            .clone()
            .unwrap_or_else(|| builtin_authority("keys")),
        vec![],
    );
    if let HirExpr::MethodCall { ty, .. } = &mut collection {
        *ty = Type::List(Box::new(Type::Int));
    }
    let HirExpr::MethodCall { object, .. } = &collection else {
        unreachable!()
    };
    let dictionary = object.as_ref().clone();
    let element = HirExpr::BinOp {
        left: Box::new(HirExpr::Name {
            name: "key".into(),
            binding_id: Some(BindingId(2)),
            ty: Type::Int,
        }),
        op: "+".into(),
        right: Box::new(HirExpr::IntLiteral(1)),
        ty: Type::Int,
    };
    let read = HirExpr::Index {
        object: Box::new(dictionary.clone()),
        index: Box::new(element.clone()),
        ty: Type::Str,
    };
    let contains = HirExpr::ContainsOp {
        element: Box::new(element),
        collection: Box::new(if authority.is_some() {
            collection
        } else {
            dictionary
        }),
        ty: Type::Bool,
    };
    let condition = if negated {
        HirExpr::UnaryOp {
            op: "not".into(),
            operand: Box::new(contains),
            ty: Type::Bool,
        }
    } else {
        contains
    };
    (condition, read)
}

fn keys_guard_observe(expr: HirExpr) -> crate::HirStmt {
    crate::HirStmt::Expr {
        expr: HirExpr::Call {
            func: "observe".into(),
            args: vec![expr],
            mutable_arg_places: vec![None],
            ty: Type::None,
        },
    }
}

fn keys_guard_stmt(condition: HirExpr, read: HirExpr, negated: bool, exit: bool) -> crate::HirStmt {
    let present = vec![keys_guard_observe(read)];
    let absent = vec![crate::HirStmt::Expr {
        expr: HirExpr::Call {
            func: "absent".into(),
            args: vec![],
            mutable_arg_places: vec![],
            ty: Type::None,
        },
    }];
    crate::HirStmt::If {
        condition,
        then_body: if exit {
            vec![crate::HirStmt::Return { value: None }]
        } else if negated {
            absent.clone()
        } else {
            present.clone()
        },
        elif_clauses: vec![],
        else_body: if exit {
            None
        } else {
            Some(if negated { present } else { absent })
        },
    }
}

#[derive(Clone, Copy, Debug)]
enum KeysGuardEntry {
    Helper,
    If,
    Exit,
    Block,
    NestedBlock,
    Capture,
    Structured,
}

fn keys_guard_function(body: Vec<crate::HirStmt>) -> crate::HirFunction {
    crate::HirFunction {
        name: "guard_test".into(),
        params: vec![],
        return_type: Type::None,
        body,
        is_async: false,
        method_kind: sifr_ir::MethodKind::Regular,
        receiver: None,
        decorators: vec![],
        rust_interop: vec![],
        python_interop: vec![],
        compiler_intrinsic: None,
        type_params: vec![],
    }
}

fn keys_guard_prepare(emitter: &mut RustEmitter, function: &crate::HirFunction) {
    emitter.body_analysis =
        crate::body_analysis::BodyAnalysis::build(function, &Default::default()).0;
}

fn keys_guard_output(
    emitter: &mut RustEmitter,
    entry: KeysGuardEntry,
    stmt: &crate::HirStmt,
) -> Result<Option<String>, crate::CodegenError> {
    let body = if matches!(entry, KeysGuardEntry::NestedBlock) {
        vec![crate::HirStmt::If {
            condition: HirExpr::BoolLiteral(true),
            then_body: vec![stmt.clone()],
            elif_clauses: vec![],
            else_body: None,
        }]
    } else {
        vec![stmt.clone()]
    };
    let function = keys_guard_function(body);
    keys_guard_prepare(emitter, &function);
    let stmt = if matches!(entry, KeysGuardEntry::NestedBlock) {
        let crate::HirStmt::If { then_body, .. } = &function.body[0] else {
            unreachable!()
        };
        &then_body[0]
    } else {
        &function.body[0]
    };
    let crate::HirStmt::If {
        condition,
        then_body,
        elif_clauses,
        else_body,
    } = stmt
    else {
        unreachable!()
    };
    match entry {
        KeysGuardEntry::Helper => emitter
            .checked_dict_read_guard_for_ir(condition)
            .map(|guard| guard.map(|guard| render_expr(&guard.option))),
        KeysGuardEntry::If => emitter
            .try_lower_checked_dict_if_for_ir(
                condition,
                then_body,
                elif_clauses,
                else_body.as_deref(),
            )
            .map(|stmt| stmt.map(|stmt| crate::render_stmts(&[stmt]))),
        KeysGuardEntry::Exit => emitter
            .try_lower_checked_dict_exit_guard_for_ir(stmt)
            .map(|stmt| stmt.map(|stmt| crate::render_stmts(&[stmt]))),
        KeysGuardEntry::Block | KeysGuardEntry::NestedBlock => emitter
            .try_lower_stmt_block_for_ir(&function.body)
            .map(|stmts| stmts.map(|stmts| crate::render_stmts(&stmts))),
        KeysGuardEntry::Capture | KeysGuardEntry::Structured => {
            let mut result = Ok(false);
            let captured = emitter.capture_structured_stmts(|inner| {
                result = if matches!(entry, KeysGuardEntry::Capture) {
                    inner.try_capture_checked_place_control_stmt(stmt, None)
                } else {
                    inner.try_lower_structured_stmt_with_following(stmt, None)
                };
            });
            match result {
                Ok(accepted) => Ok(accepted.then(|| crate::render_stmts(&captured))),
                Err(error) => {
                    assert!(captured.is_empty());
                    Err(error)
                }
            }
        }
    }
}

#[test]
fn checked_dict_keys_guards_reject_unclassified_and_mismatched_authority() {
    let invalid = [
        (MethodAuthority::Unclassified, "unclassified source method"),
        (
            MethodAuthority::BuiltinIntrinsic {
                declaration: identity("wrong.module", "keys"),
            },
            "invalid builtin method authority",
        ),
        (
            builtin_authority("wrong_symbol"),
            "invalid builtin method authority",
        ),
    ];
    for (authority, diagnostic) in invalid {
        for (negated, exit) in [(false, false), (true, false), (true, true)] {
            for entry in [
                KeysGuardEntry::Helper,
                KeysGuardEntry::If,
                KeysGuardEntry::Exit,
                KeysGuardEntry::Block,
                KeysGuardEntry::NestedBlock,
                KeysGuardEntry::Capture,
                KeysGuardEntry::Structured,
            ] {
                if matches!(entry, KeysGuardEntry::Exit) && !exit
                    || matches!(entry, KeysGuardEntry::If) && exit
                {
                    continue;
                }
                // The same HIR with builtin authority must actually optimize;
                // this prevents an unrelated shape decline from passing rejection.
                let (condition, read) = keys_guard_parts(Some(builtin_authority("keys")), negated);
                let control = keys_guard_stmt(condition, read, negated, exit);
                let admitted = keys_guard_output(&mut RustEmitter::new(), entry, &control)
                    .unwrap()
                    .unwrap_or_else(|| {
                        panic!("successful guard shape: {entry:?}, negated={negated}, exit={exit}")
                    });
                assert!(admitted.contains("value.get("), "{entry:?}: {admitted}");
                let (condition, read) = keys_guard_parts(Some(authority.clone()), negated);
                let stmt = keys_guard_stmt(condition, read, negated, exit);
                let mut emitter = RustEmitter::new();
                let error = keys_guard_output(&mut emitter, entry, &stmt).unwrap_err();
                assert!(
                    error.message.contains(diagnostic),
                    "{entry:?}: {}",
                    error.message
                );
                assert!(emitter.checked_place_read_witnesses.is_empty(), "{entry:?}");
            }
        }
    }
}

#[test]
fn checked_dict_keys_guards_preserve_nonbuiltin_authority_without_retry() {
    for authority in [
        MethodAuthority::Protocol {
            declaration: identity("user", "keys"),
        },
        MethodAuthority::LocalNominal {
            declaration: identity("user", "keys"),
        },
        MethodAuthority::InheritedNominal {
            declaration: identity("user", "keys"),
        },
        MethodAuthority::Imported {
            declaration: identity("user", "keys"),
        },
        contextual_authority("keys"),
    ] {
        for (negated, exit) in [(false, false), (true, false), (true, true)] {
            let (condition, read) = keys_guard_parts(Some(authority.clone()), negated);
            let stmt = keys_guard_stmt(condition.clone(), read, negated, exit);
            for entry in [
                KeysGuardEntry::Helper,
                KeysGuardEntry::If,
                KeysGuardEntry::Exit,
                KeysGuardEntry::Capture,
            ] {
                let mut emitter = RustEmitter::new();
                assert!(
                    keys_guard_output(&mut emitter, entry, &stmt)
                        .unwrap()
                        .is_none(),
                    "{entry:?}"
                );
                assert!(emitter.checked_place_read_witnesses.is_empty());
            }
            let contains = if let HirExpr::UnaryOp { operand, .. } = &condition {
                operand.as_ref()
            } else {
                &condition
            };
            let HirExpr::ContainsOp { collection, .. } = contains else {
                unreachable!()
            };
            let ordinary = render_expr(
                &RustEmitter::new()
                    .lower_stmt_expr_for_ir(collection)
                    .unwrap()
                    .unwrap(),
            );
            assert_eq!(ordinary, "value.keys()");
            // Ordinary source behavior has no builtin presence guarantee. Use
            // independent branch bodies when testing its production emission.
            let mut ordinary_stmt = stmt.clone();
            if let crate::HirStmt::If {
                then_body,
                else_body,
                ..
            } = &mut ordinary_stmt
            {
                if !exit {
                    *then_body = vec![crate::HirStmt::Pass];
                    *else_body = Some(vec![crate::HirStmt::Pass]);
                }
            }
            for entry in [
                KeysGuardEntry::Block,
                KeysGuardEntry::NestedBlock,
                KeysGuardEntry::Structured,
            ] {
                let mut emitter = RustEmitter::new();
                let output = keys_guard_output(&mut emitter, entry, &ordinary_stmt)
                    .unwrap()
                    .expect("ordinary admitted emission");
                assert!(output.contains(&ordinary), "{entry:?}: {output}");
                assert_eq!(output.matches("value.keys()").count(), 1, "{output}");
                assert!(!output.contains("value.get("), "{output}");
                assert!(!output.contains("cloned().collect"), "{output}");
                assert!(!output.contains("__sifr_checked_value"), "{output}");
                assert!(emitter.checked_place_read_witnesses.is_empty());
            }
        }
    }
}

#[test]
fn checked_dict_keys_guards_preserve_admitted_if_and_exit_behavior() {
    for authority in [None, Some(builtin_authority("keys"))] {
        for (negated, exit) in [(false, false), (true, false), (true, true)] {
            let (condition, read) = keys_guard_parts(authority.clone(), negated);
            let stmt = keys_guard_stmt(condition, read.clone(), negated, exit);
            let mut emitter = RustEmitter::new();
            let function = keys_guard_function(if exit {
                vec![stmt, keys_guard_observe(read)]
            } else {
                vec![stmt]
            });
            keys_guard_prepare(&mut emitter, &function);
            let lowered = emitter
                .try_lower_stmt_block_for_ir(&function.body)
                .unwrap()
                .unwrap();
            let output = crate::render_stmts(&lowered);
            assert_eq!(output.matches("value.get(").count(), 1, "{output}");
            assert_eq!(output.matches("&key +").count(), 1, "{output}");
            assert!(output.contains("observe("), "{output}");
            assert!(output.contains("__sifr_checked_value"), "{output}");
            assert!(output.contains("clone()"), "{output}");
            assert!(!output.contains("value.keys()"), "{output}");
            assert!(!output.contains("unwrap("), "{output}");
            assert!(!output.contains("expect("), "{output}");
            if exit {
                assert!(output.contains("else {"));
                assert!(output.contains("return;"));
            } else {
                assert!(output.contains("if let Some("));
                assert!(output.contains("absent()"));
            }
        }
    }
}

// These cases own the admitted nonbuiltin narrowing boundary and its result
// representation. Analysis is prepared on real lowered bodies before authority
// mutation; source ranges and receiver conventions must survive that mutation.
#[derive(Clone, Copy, Debug)]
enum PopEntry {
    Value,
    Discard,
    Block,
    Scoped,
    Nested,
    Capture,
    Structured,
}

fn pop_function(indexed: bool, field: bool, binding: bool) -> crate::HirFunction {
    let method = if indexed { "pop(0)" } else { "pop()" };
    let receiver = if field { "self._data" } else { "values" };
    let action = if binding {
        format!("value: int = {receiver}.{method}\n        return value")
    } else {
        format!("return {receiver}.{method}")
    };
    let source = if field {
        format!(
            "class deque:\n    _data: list[int]\n    def take(mut self) -> int:\n        while {receiver}:\n            {}\n        return 0\n",
            action.replace('\n', "\n    ")
        )
    } else {
        format!(
            "def take(mut values: list[int]) -> int:\n    while {receiver}:\n        {action}\n    return 0\n"
        )
    };
    let parsed = sifr_python_parser::parse_module(&source).unwrap();
    let mut module = sifr_lowering::lower_module(parsed.suite())
        .unwrap_or_else(|errors| panic!("{source}: {errors:?}"))
        .module;
    if field {
        module.classes.remove(0).methods.remove(0)
    } else {
        module.functions.remove(0)
    }
}

fn pop_body(function: &crate::HirFunction) -> &[crate::HirStmt] {
    let crate::HirStmt::While { body, .. } = &function.body[0] else {
        panic!("expected real narrowed while: {:?}", function.body)
    };
    body
}

fn pop_body_mut(function: &mut crate::HirFunction) -> &mut Vec<crate::HirStmt> {
    let crate::HirStmt::While { body, .. } = &mut function.body[0] else {
        unreachable!()
    };
    body
}

fn pop_value(stmt: &crate::HirStmt) -> &HirExpr {
    match stmt {
        crate::HirStmt::Return { value: Some(value) }
        | crate::HirStmt::Let { value, .. }
        | crate::HirStmt::Expr { expr: value } => value,
        _ => panic!("expected method-bearing statement: {stmt:?}"),
    }
}

fn pop_value_mut(stmt: &mut crate::HirStmt) -> &mut HirExpr {
    match stmt {
        crate::HirStmt::Return { value: Some(value) }
        | crate::HirStmt::Let { value, .. }
        | crate::HirStmt::Expr { expr: value } => value,
        _ => unreachable!(),
    }
}

fn pop_authorities(method: &str) -> Vec<MethodAuthority> {
    vec![
        MethodAuthority::Protocol {
            declaration: identity("user", method),
        },
        MethodAuthority::LocalNominal {
            declaration: identity("user", method),
        },
        MethodAuthority::InheritedNominal {
            declaration: identity("user", method),
        },
        MethodAuthority::Imported {
            declaration: identity("user", method),
        },
        contextual_authority(method),
    ]
}

fn pop_set_authority(value: &mut HirExpr, authority: &MethodAuthority) {
    let before = sequence_exit_method_provenance(value);
    let HirExpr::MethodCall {
        authority: stored, ..
    } = value
    else {
        unreachable!()
    };
    *stored = authority.clone();
    assert_eq!(sequence_exit_method_provenance(value), before);
}

fn pop_prepare(function: &crate::HirFunction, field: bool) -> RustEmitter {
    let mut emitter = RustEmitter::new();
    if field {
        emitter.current_class_name = Some("deque".into());
    }
    keys_guard_prepare(&mut emitter, function);
    emitter
}

fn pop_output(
    emitter: &mut RustEmitter,
    entry: PopEntry,
    function: &crate::HirFunction,
) -> Result<Option<String>, crate::CodegenError> {
    let body = pop_body(function);
    let value = pop_value(&body[0]);
    match entry {
        PopEntry::Value => emitter
            .lower_return_value_expr_for_ir(value, Some(value.ty()))
            .map(|value| value.map(|value| render_expr(&value))),
        PopEntry::Discard => emitter
            .try_lower_stmt_expr_statement_only(value)
            .map(|value| value.map(|value| render_expr(&value))),
        PopEntry::Block => emitter
            .try_lower_stmt_block_for_ir(body)
            .map(|stmts| stmts.map(|stmts| crate::render_stmts(&stmts))),
        PopEntry::Scoped => emitter
            .try_lower_scoped_stmt_block_for_ir(body)
            .map(|stmts| stmts.map(|stmts| crate::render_stmts(&stmts))),
        PopEntry::Nested => emitter
            .try_lower_stmt_block_for_ir(&function.body)
            .map(|stmts| stmts.map(|stmts| crate::render_stmts(&stmts))),
        PopEntry::Capture | PopEntry::Structured => {
            let mut result = Ok(false);
            let captured = emitter.capture_structured_stmts(|inner| {
                result = if matches!(entry, PopEntry::Capture) {
                    inner
                        .try_lower_stmt_block_for_ir(&function.body)
                        .map(|stmts| {
                            if let Some(stmts) = stmts {
                                for stmt in stmts {
                                    inner.push_captured_stmt(&stmt);
                                }
                                true
                            } else {
                                false
                            }
                        })
                } else {
                    inner.try_lower_structured_stmt_with_following(&function.body[0], None)
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

fn pop_entries() -> [PopEntry; 7] {
    [
        PopEntry::Value,
        PopEntry::Discard,
        PopEntry::Block,
        PopEntry::Scoped,
        PopEntry::Nested,
        PopEntry::Capture,
        PopEntry::Structured,
    ]
}

#[test]
fn narrowing_specializations_decline_unclassified_without_builtin_retry() {
    let mut invalid = vec![
        (MethodAuthority::Unclassified, "unclassified source method"),
        (
            MethodAuthority::BuiltinIntrinsic {
                declaration: identity("wrong", "pop"),
            },
            "invalid builtin method authority",
        ),
        (
            builtin_authority("wrong"),
            "invalid builtin method authority",
        ),
        (
            MethodAuthority::Protocol {
                declaration: identity("user", "wrong"),
            },
            "invalid user or protocol method authority",
        ),
        (
            MethodAuthority::Imported {
                declaration: identity("user", "wrong"),
            },
            "invalid user or protocol method authority",
        ),
    ];
    invalid.extend(
        pop_authorities("wrong")
            .into_iter()
            .filter(|authority| !matches!(authority, MethodAuthority::RustAdapted { .. }))
            .map(|authority| (authority, "invalid user or protocol method authority")),
    );
    for indexed in [false, true] {
        for field in [false, true] {
            for (authority, diagnostic) in &invalid {
                for entry in pop_entries() {
                    let mut function = pop_function(indexed, field, false);
                    assert_eq!(pop_value(&pop_body(&function)[0]).ty(), &Type::Int);
                    let mut emitter = pop_prepare(&function, field);
                    pop_set_authority(
                        pop_value_mut(&mut pop_body_mut(&mut function)[0]),
                        authority,
                    );
                    let error = pop_output(&mut emitter, entry, &function).unwrap_err();
                    assert!(
                        error.message.contains(diagnostic),
                        "{entry:?}: {}",
                        error.message
                    );
                }
            }
        }
    }
    // Nested receiver and argument calls are genuine narrowed carriers; an
    // admitted ordinary outer call must propagate their structural errors.
    for argument in [false, true] {
        for (authority, diagnostic) in &invalid {
            let mut function = pop_function(false, false, false);
            let mut emitter = pop_prepare(&function, false);
            let value = pop_value_mut(&mut pop_body_mut(&mut function)[0]);
            pop_set_authority(value, authority);
            let outer = if argument {
                call(
                    Type::Int,
                    "observe",
                    contextual_authority("observe"),
                    vec![value.clone()],
                )
            } else {
                let mut outer = call(
                    Type::Int,
                    "observe",
                    contextual_authority("observe"),
                    vec![],
                );
                let HirExpr::MethodCall { object, .. } = &mut outer else {
                    unreachable!()
                };
                *object = Box::new(value.clone());
                outer
            };
            for discarded in [false, true] {
                let error = emitter
                    .lower_source_method_expr_for_ir(&outer, discarded)
                    .unwrap_err();
                assert!(error.message.contains(diagnostic), "{}", error.message);
            }
        }
    }
}

fn pop_assert_ordinary(output: &str, ordinary: &str) {
    assert!(output.contains(ordinary), "{output} vs {ordinary}");
    for substituted in [".remove(", ".drain(", ".len(", "pop_front(", "pop_back("] {
        assert!(!output.contains(substituted), "{output}");
    }
}

#[test]
fn nonempty_pop_and_mapping_adaptation_preserve_authority() {
    for indexed in [false, true] {
        for field in [false, true] {
            for authority in pop_authorities("pop") {
                for entry in pop_entries() {
                    let mut function = pop_function(indexed, field, false);
                    let mut emitter = pop_prepare(&function, field);
                    let value = pop_value_mut(&mut pop_body_mut(&mut function)[0]);
                    pop_set_authority(value, &authority);
                    let receiver = if field { "self._data" } else { "values" };
                    let ordinary = if indexed {
                        format!("{receiver}.pop(SifrInt::from_i64(0))")
                    } else {
                        format!("{receiver}.pop()")
                    };
                    let output = pop_output(&mut emitter, entry, &function)
                        .unwrap()
                        .unwrap_or_else(|| panic!("{entry:?} declined"));
                    pop_assert_ordinary(&output, &ordinary);
                    if matches!(entry, PopEntry::Value) {
                        let value = pop_value(&pop_body(&function)[0]);
                        assert!(!crate::stmt_support_emitter::compiler_verified_pop_lowers_as_option_for_ir(value));
                        for operator in ["==", "!="] {
                            let comparison = HirExpr::Compare {
                                left: Box::new(value.clone()),
                                ops: vec![operator.into()],
                                comparators: vec![HirExpr::IntLiteral(1)],
                                ty: Type::Bool,
                            };
                            for condition_entry in [false, true] {
                                let lowered = if condition_entry {
                                    emitter.lower_condition_expr_for_ir(&comparison)
                                } else {
                                    emitter.lower_stmt_expr_for_ir(&comparison)
                                }
                                .unwrap()
                                .unwrap();
                                let output = render_expr(&lowered);
                                pop_assert_ordinary(&output, &ordinary);
                                assert!(!output.contains("Some("), "{output}");
                            }
                        }
                        let truthy = render_expr(
                            &emitter.lower_condition_expr_for_ir(value).unwrap().unwrap(),
                        );
                        pop_assert_ordinary(&truthy, &ordinary);
                        assert!(!truthy.contains("Some("), "{truthy}");
                    }
                }
            }
            // The source-admitted builtin path still selects its replacement.
            let function = pop_function(indexed, field, false);
            let mut emitter = pop_prepare(&function, field);
            let lowered = pop_output(&mut emitter, PopEntry::Value, &function);
            if field && indexed {
                // The deque registry supports only zero-argument pop. Its
                // existing structural decline is preserved; this is not a
                // newly admitted builtin shape.
                assert!(
                    lowered
                        .unwrap_err()
                        .message
                        .contains("builtin method 'pop' is unsupported")
                );
                continue;
            }
            let output = lowered.unwrap().unwrap();
            assert!(output.contains(".remove("), "{output}");
            assert_eq!(output.contains(".drain("), field, "{output}");
            assert_eq!(output.contains(".len("), !indexed, "{output}");
        }
    }
    // popleft has the same narrowed carrier shape as pop(0). List source has
    // no popleft declaration; author that shape before analysis, then change
    // only dispatch authority. The deque-storage builtin is a positive control.
    for field in [false, true] {
        let mut function = pop_function(false, field, false);
        let value = pop_value_mut(&mut pop_body_mut(&mut function)[0]);
        let HirExpr::MethodCall {
            method, authority, ..
        } = value
        else {
            unreachable!()
        };
        *method = "popleft".into();
        *authority = builtin_authority("popleft");
        for authority in pop_authorities("popleft") {
            for entry in pop_entries() {
                let mut function = function.clone();
                let mut emitter = pop_prepare(&function, field);
                pop_set_authority(
                    pop_value_mut(&mut pop_body_mut(&mut function)[0]),
                    &authority,
                );
                let ordinary = if field {
                    "self._data.popleft()"
                } else {
                    "values.popleft()"
                };
                let output = pop_output(&mut emitter, entry, &function).unwrap().unwrap();
                pop_assert_ordinary(&output, ordinary);
            }
        }
        if field {
            let mut emitter = pop_prepare(&function, field);
            let output = pop_output(&mut emitter, PopEntry::Value, &function)
                .unwrap()
                .unwrap();
            assert!(output.contains(".drain("), "{output}");
            assert!(output.contains(".remove("), "{output}");
            assert!(!output.contains(".len("), "{output}");
        }
    }
    // Plain list popleft has no builtin registry declaration. Preserve the
    // existing compiler-internal index-zero replacement itself without claiming
    // that this unsupported source shape is admitted by the registry.
    let replacement =
        crate::stmt_support_emitter::unwrap_compiler_verified_nonempty_pop_result_for_ir(
            &Type::List(Box::new(Type::Int)),
            "popleft",
            &[],
            &Type::Int,
            RustExpr::Ident("values".into()),
            false,
            RustExpr::from_source_method(
                RustExpr::Ident("values".into()),
                "popleft".into(),
                vec![],
            ),
        );
    let replacement = render_expr(&replacement);
    assert!(replacement.contains("values.remove("), "{replacement}");
    assert!(!replacement.contains("len("), "{replacement}");
    assert!(!replacement.contains("drain("), "{replacement}");
    pop_preserve_optional_and_tail();
    pop_preserve_deque_comparison();
    pop_preserve_mapping_default();
}

fn pop_preserve_optional_and_tail() {
    for field in [false, true] {
        for authority in pop_authorities("pop")
            .into_iter()
            .chain([builtin_authority("pop")])
        {
            let mut function = pop_function(false, field, true);
            let mut emitter = pop_prepare(&function, field);
            let value = pop_value_mut(&mut pop_body_mut(&mut function)[0]);
            pop_set_authority(value, &authority);
            let stmt = &pop_body(&function)[0];
            let output = emitter
                .try_lower_nonempty_pop_tail_for_ir(stmt, &pop_body(&function)[1..])
                .unwrap()
                .unwrap();
            let output = crate::render_stmts(&[output]);
            assert!(output.contains("if let Some(value)"), "{output}");
            if !matches!(authority, MethodAuthority::BuiltinIntrinsic { .. }) {
                pop_assert_ordinary(
                    &output,
                    if field {
                        "self._data.pop()"
                    } else {
                        "values.pop()"
                    },
                );
            } else {
                assert!(
                    output.contains(if field { "pop_back()" } else { "values.pop()" }),
                    "{output}"
                );
            }
            let mut optional = pop_value(stmt).clone();
            let HirExpr::MethodCall { ty, .. } = &mut optional else {
                unreachable!()
            };
            *ty = sifr_type_system::make_union(vec![Type::Int, Type::None]);
            let output = render_expr(&emitter.lower_stmt_expr_for_ir(&optional).unwrap().unwrap());
            assert!(!output.contains(".remove("), "{output}");
            assert!(!output.contains(".drain("), "{output}");
        }
    }
}

fn pop_preserve_deque_comparison() {
    // A real ordinary deque declaration returns Option in Rust despite scalar
    // HIR narrowing at the caller. Preserve that representation contract.
    let source = "class deque:\n    _data: list[int]\n    def __bool__(self) -> bool:\n        return bool(self._data)\n    def pop(mut self) -> int | None:\n        return self._data.pop()\n\ndef take(mut values: deque) -> int:\n    while values:\n        return values.pop()\n    return 0\n";
    let parsed = sifr_python_parser::parse_module(source).unwrap();
    let lowered = sifr_lowering::lower_module(parsed.suite())
        .unwrap_or_else(|errors| panic!("{source}: {errors:?}"));
    let original = lowered.module.functions.into_iter().next().unwrap();
    assert_eq!(pop_value(&pop_body(&original)[0]).ty(), &Type::Int);
    let HirExpr::MethodCall { object, .. } = pop_value(&pop_body(&original)[0]) else {
        unreachable!()
    };
    let Type::Class { methods, .. } = object.ty() else {
        unreachable!()
    };
    let signature = methods.iter().find(|(name, _)| name == "pop").unwrap();
    assert!(signature.1.return_type.optional_member_type().is_some());
    for authority in pop_authorities("pop") {
        let mut function = original.clone();
        let mut emitter = pop_prepare(&function, false);
        let value = pop_value_mut(&mut pop_body_mut(&mut function)[0]);
        pop_set_authority(value, &authority);
        assert!(crate::stmt_support_emitter::compiler_verified_pop_lowers_as_option_for_ir(value));
        let output = render_expr(&emitter.lower_stmt_expr_for_ir(value).unwrap().unwrap());
        assert_eq!(output, "values.pop()");
        let unchanged =
            crate::stmt_support_emitter::unwrap_compiler_verified_nonempty_pop_result_for_ir(
                object.ty(),
                "pop",
                &[],
                &Type::Int,
                RustExpr::Ident("values".into()),
                false,
                RustExpr::from_source_method(
                    RustExpr::Ident("values".into()),
                    "pop".into(),
                    vec![],
                ),
            );
        assert_eq!(render_expr(&unchanged), "values.pop()");
        for operator in ["==", "!="] {
            for reversed in [false, true] {
                let scalar = HirExpr::IntLiteral(1);
                let (left, right) = if reversed {
                    (scalar, value.clone())
                } else {
                    (value.clone(), scalar)
                };
                let comparison = HirExpr::Compare {
                    left: Box::new(left),
                    ops: vec![operator.into()],
                    comparators: vec![right],
                    ty: Type::Bool,
                };
                for condition_entry in [false, true] {
                    let output = if condition_entry {
                        emitter.lower_condition_expr_for_ir(&comparison)
                    } else {
                        emitter.lower_stmt_expr_for_ir(&comparison)
                    }
                    .unwrap()
                    .unwrap();
                    let output = render_expr(&output);
                    pop_assert_ordinary(&output, "values.pop()");
                    assert!(output.contains("Some("), "{output}");
                    assert!(!output.contains("unwrap("), "{output}");
                }
            }
        }
    }
}

fn pop_preserve_mapping_default() {
    let source = "def lookup(values: dict[str, str], key: str, fallback: str) -> str:\n    return values.get(key, fallback)\n";
    let parsed = sifr_python_parser::parse_module(source).unwrap();
    let mut function = sifr_lowering::lower_module(parsed.suite())
        .unwrap()
        .module
        .functions
        .remove(0);
    let original = pop_value(&function.body[0]).clone();
    for authority in pop_authorities("get")
        .into_iter()
        .chain([builtin_authority("get")])
    {
        let builtin = matches!(authority, MethodAuthority::BuiltinIntrinsic { .. });
        let mut emitter = RustEmitter::new();
        keys_guard_prepare(&mut emitter, &function);
        emitter
            .borrowed_params
            .extend(["values".into(), "key".into(), "fallback".into()]);
        let value = pop_value_mut(&mut function.body[0]);
        *value = original.clone();
        pop_set_authority(value, &authority);
        let output = render_expr(&emitter.lower_stmt_expr_for_ir(value).unwrap().unwrap());
        if builtin {
            assert!(output.contains("fallback.to_owned()"), "{output}");
            assert!(output.contains("unwrap_or("), "{output}");
        } else {
            assert!(output.contains("values.get("), "{output}");
            assert!(!output.contains("unwrap_or("), "{output}");
        }
        let HirExpr::MethodCall {
            object,
            method,
            args,
            ..
        } = value
        else {
            unreachable!()
        };
        let mut lowered = vec![
            RustExpr::Ident("key".into()),
            RustExpr::Ident("fallback".into()),
        ];
        emitter.adapt_owned_mapping_default(object.ty(), method, args, &mut lowered);
        assert_eq!(render_expr(&lowered[1]), "fallback.to_owned()");
    }
}

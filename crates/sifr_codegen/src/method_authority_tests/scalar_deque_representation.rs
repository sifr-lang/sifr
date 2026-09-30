// Resolved source declarations own the result representation independently of
// dispatch authority and a narrowed call's contextual HIR type.
#[test]
fn scalar_deque_comparisons_preserve_declared_result_representation() {
    for call in ["pop()", "pop(0)", "popleft()"] {
        let declaration = match call {
            "pop(0)" => "def pop(self, index: int) -> int:",
            "popleft()" => "def popleft(self) -> int:",
            _ => "def pop(self) -> int:",
        };
        for operator in ["==", "!="] {
            for reversed in [false, true] {
                let comparison = if reversed {
                    format!("1 {operator} values.{call}")
                } else {
                    format!("values.{call} {operator} 1")
                };
                let source = format!(
                    "class deque:\n    {declaration}\n        return 1\n\ndef take(values: deque) -> bool:\n    return {comparison}\n\ndef condition(values: deque) -> int:\n    if {comparison}:\n        return 1\n    return 0\n"
                );
                let parsed = sifr_python_parser::parse_module(&source).unwrap();
                let lowered = sifr_lowering::lower_module(parsed.suite())
                    .unwrap_or_else(|errors| panic!("{source}: {errors:?}"));
                let ordinary = match call {
                    "pop(0)" => "values.pop(SifrInt::from_i64(0))",
                    "popleft()" => "values.popleft()",
                    _ => "values.pop()",
                };
                for function in &lowered.module.functions {
                    let comparison = match &function.body[0] {
                        crate::HirStmt::Return { value: Some(value) } => value,
                        crate::HirStmt::If { condition, .. } => condition,
                        other => panic!("expected source return/If: {other:?}"),
                    };
                    let HirExpr::Compare {
                        left, comparators, ..
                    } = comparison
                    else {
                        panic!("expected source comparison: {comparison:?}")
                    };
                    let value = if reversed {
                        &comparators[0]
                    } else {
                        left.as_ref()
                    };
                    let HirExpr::MethodCall {
                        object,
                        method,
                        authority,
                        ..
                    } = value
                    else {
                        panic!("expected resolved source call: {value:?}")
                    };
                    assert!(matches!(authority, MethodAuthority::LocalNominal { .. }));
                    assert_eq!(value.ty(), &Type::Int);
                    let Type::Class { methods, .. } = object.ty() else {
                        unreachable!()
                    };
                    let signature = &methods.iter().find(|(name, _)| name == method).unwrap().1;
                    assert_eq!(signature.return_type.as_ref(), &Type::Int);
                    assert!(
                        !crate::stmt_support_emitter::compiler_verified_pop_lowers_as_option_for_ir(
                            value
                        )
                    );
                    let mut emitter = RustEmitter::new();
                    keys_guard_prepare(&mut emitter, function);
                    for condition_entry in [false, true] {
                        let output = if condition_entry {
                            emitter.lower_condition_expr_for_ir(comparison)
                        } else {
                            emitter.lower_stmt_expr_for_ir(comparison)
                        }
                        .unwrap()
                        .unwrap();
                        scalar_deque_assert_representation(&render_expr(&output), ordinary);
                    }
                }
                let output = crate::lib_codegen_tests::generate_rust_from_source(&source);
                scalar_deque_assert_representation(&output, ordinary);
                assert!(
                    output.contains("fn pop(") || output.contains("fn popleft("),
                    "{output}"
                );
                assert!(output.contains("-> SifrInt"), "{output}");
                assert!(output.contains("fn take("), "{output}");
                assert!(output.contains("fn condition("), "{output}");
                assert!(output.contains("if "), "{output}");
            }
        }
    }
}

fn scalar_deque_assert_representation(output: &str, ordinary: &str) {
    assert!(output.contains(ordinary), "{output} vs {ordinary}");
    for injected in [
        "Some(",
        "is_some(",
        "is_none(",
        ".remove(",
        ".drain(",
        "pop_front(",
        "pop_back(",
        ".unwrap(",
        ".expect(",
    ] {
        assert!(!output.contains(injected), "{output}");
    }
}

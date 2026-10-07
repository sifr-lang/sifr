// Real lowered function bodies keep BodyAnalysis statement identities and source
// method ownership/ranges intact; only the authority under test is changed.
#[derive(Clone, Copy, Debug)]
enum SequenceExitEntry {
    Direct,
    Block,
    Scoped,
    Nested,
    Capture,
    Structured,
}

#[derive(Clone, Copy, Debug)]
enum SequenceExitShape {
    Full,
    Partial,
    Or,
    Prefix,
}

fn sequence_exit_function(shape: SequenceExitShape, receiver: &str) -> crate::HirFunction {
    let result = if receiver == "str" { "str" } else { "int" };
    let empty = if receiver == "str" { "\"\"" } else { "0" };
    let body = match shape {
        SequenceExitShape::Full => {
            format!("    if len(values) < 3:\n        return {empty}\n    return values[0]\n")
        }
        SequenceExitShape::Partial => format!(
            "    if len(values) < 3 or stop:\n        return {empty}\n    return values[0]\n"
        ),
        SequenceExitShape::Or => format!(
            "    if len(values) < 3 or len(values) == 0:\n        return {empty}\n    return values[0]\n"
        ),
        SequenceExitShape::Prefix => format!(
            "    if len(values) == 0:\n        return {empty}\n    first = values[0]\n    if len(values) == 1:\n        return first\n    return values[1]\n"
        ),
    };
    let body = if receiver == "bytes" {
        body.replace("values[0]", "int(values[0])")
            .replace("values[1]", "int(values[1])")
    } else {
        body
    };
    let source = format!("def guard(values: {receiver}, stop: bool) -> {result}:\n{body}");
    let parsed = sifr_python_parser::parse_module(&source).unwrap();
    let module = sifr_lowering::lower_module(parsed.suite())
        .unwrap_or_else(|errors| panic!("{source}: {errors:?}"));
    module.module.functions.into_iter().next().unwrap()
}

fn sequence_exit_target(shape: SequenceExitShape) -> usize {
    if matches!(shape, SequenceExitShape::Prefix) {
        2
    } else {
        0
    }
}

fn sequence_exit_method_provenance(expr: &HirExpr) -> Vec<String> {
    let mut provenance = Vec::new();
    crate::hir_analysis::traversal::walk_expr(expr, &mut |candidate| {
        if let HirExpr::MethodCall {
            receiver_convention,
            receiver_target,
            source,
            ..
        } = candidate
        {
            assert!(source.is_some(), "real source method range");
            provenance.push(format!(
                "{receiver_convention:?}/{receiver_target:?}/{source:?}"
            ));
        }
    });
    provenance
}

fn sequence_exit_replace_authority(expr: &mut HirExpr, authority: &MethodAuthority) {
    match expr {
        HirExpr::MethodCall {
            method,
            authority: stored,
            ..
        } if method == "len" => *stored = authority.clone(),
        HirExpr::Compare {
            left, comparators, ..
        } => {
            sequence_exit_replace_authority(left, authority);
            for value in comparators {
                sequence_exit_replace_authority(value, authority);
            }
        }
        HirExpr::BoolOp { values, .. } => {
            for value in values {
                sequence_exit_replace_authority(value, authority);
            }
        }
        _ => {}
    }
}

fn sequence_exit_output(
    emitter: &mut RustEmitter,
    entry: SequenceExitEntry,
    function: &crate::HirFunction,
    shape: SequenceExitShape,
) -> Result<Option<String>, crate::CodegenError> {
    let offset = sequence_exit_target(shape);
    let stmt = &function.body[offset];
    let following = Some(&function.body[offset + 1..]);
    if matches!(shape, SequenceExitShape::Prefix) {
        let prior = emitter
            .try_lower_checked_sequence_exit_guards_for_ir(
                &function.body[0],
                Some(&function.body[1..]),
            )?
            .unwrap();
        assert!(crate::render_stmts(&prior).contains("let Some("));
    }
    match entry {
        SequenceExitEntry::Direct => emitter
            .try_lower_checked_sequence_exit_guards_for_ir(stmt, following)
            .map(|stmts| stmts.map(|stmts| crate::render_stmts(&stmts))),
        SequenceExitEntry::Block => emitter
            .try_lower_stmt_block_for_ir(&function.body[offset..])
            .map(|stmts| stmts.map(|stmts| crate::render_stmts(&stmts))),
        SequenceExitEntry::Scoped => emitter
            .try_lower_scoped_stmt_block_for_ir(&function.body[offset..])
            .map(|stmts| stmts.map(|stmts| crate::render_stmts(&stmts))),
        SequenceExitEntry::Nested => {
            // Lower a real nested block whose analysis was prepared before entry.
            emitter
                .try_lower_stmt_block_for_ir(&function.body)
                .map(|stmts| stmts.map(|stmts| crate::render_stmts(&stmts)))
        }
        SequenceExitEntry::Capture | SequenceExitEntry::Structured => {
            let mut result = Ok(false);
            let captured = emitter.capture_structured_stmts(|inner| {
                result = if matches!(entry, SequenceExitEntry::Capture) {
                    inner.try_capture_checked_place_control_stmt(stmt, following)
                } else {
                    inner.try_lower_structured_stmt_with_following(stmt, following)
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

fn sequence_exit_run(
    emitter: &mut RustEmitter,
    entry: SequenceExitEntry,
    shape: SequenceExitShape,
    receiver: &str,
    authority: MethodAuthority,
) -> Result<Option<String>, crate::CodegenError> {
    let mut function = sequence_exit_function(shape, receiver);
    if matches!(entry, SequenceExitEntry::Nested) {
        function.body = vec![crate::HirStmt::If {
            condition: HirExpr::BoolLiteral(true),
            then_body: function.body,
            elif_clauses: vec![],
            else_body: None,
        }];
    }
    keys_guard_prepare(emitter, &function);
    let body = if matches!(entry, SequenceExitEntry::Nested) {
        let crate::HirStmt::If { then_body, .. } = &mut function.body[0] else {
            unreachable!()
        };
        then_body
    } else {
        &mut function.body
    };
    let crate::HirStmt::If { condition, .. } = &mut body[sequence_exit_target(shape)] else {
        unreachable!()
    };
    let provenance = sequence_exit_method_provenance(condition);
    sequence_exit_replace_authority(condition, &authority);
    assert_eq!(sequence_exit_method_provenance(condition), provenance);
    if matches!(entry, SequenceExitEntry::Nested) {
        emitter
            .try_lower_stmt_block_for_ir(&function.body)
            .map(|stmts| stmts.map(|stmts| crate::render_stmts(&stmts)))
    } else {
        sequence_exit_output(emitter, entry, &function, shape)
    }
}

const SEQUENCE_EXIT_ENTRIES: [SequenceExitEntry; 6] = [
    SequenceExitEntry::Direct,
    SequenceExitEntry::Block,
    SequenceExitEntry::Scoped,
    SequenceExitEntry::Nested,
    SequenceExitEntry::Capture,
    SequenceExitEntry::Structured,
];
const SEQUENCE_EXIT_SHAPES: [SequenceExitShape; 4] = [
    SequenceExitShape::Full,
    SequenceExitShape::Partial,
    SequenceExitShape::Or,
    SequenceExitShape::Prefix,
];

#[test]
fn checked_sequence_exit_guards_reject_unclassified_and_mismatched_authority() {
    for (authority, diagnostic) in [
        (MethodAuthority::Unclassified, "unclassified source method"),
        (
            MethodAuthority::BuiltinIntrinsic {
                declaration: identity("wrong.module", "len"),
            },
            "invalid builtin method authority",
        ),
        (
            builtin_authority("wrong_symbol"),
            "invalid builtin method authority",
        ),
        (
            MethodAuthority::Protocol {
                declaration: identity("user", "wrong_symbol"),
            },
            "invalid user or protocol method authority",
        ),
    ] {
        for shape in SEQUENCE_EXIT_SHAPES {
            for entry in SEQUENCE_EXIT_ENTRIES {
                let output = sequence_exit_run(
                    &mut RustEmitter::new(),
                    entry,
                    shape,
                    "list[int]",
                    builtin_authority("len"),
                )
                .unwrap()
                .unwrap();
                assert!(
                    output.contains("let Some("),
                    "positive control {entry:?}/{shape:?}: {output}"
                );
                let mut emitter = RustEmitter::new();
                let error =
                    sequence_exit_run(&mut emitter, entry, shape, "list[int]", authority.clone())
                        .unwrap_err();
                assert!(
                    error.message.contains(diagnostic),
                    "{entry:?}/{shape:?}: {}",
                    error.message
                );
                assert!(
                    emitter
                        .checked_place_read_witnesses
                        .keys()
                        .all(|key| matches!(shape, SequenceExitShape::Prefix)
                            && key.ends_with("[int:0]")),
                    "{entry:?}/{shape:?}"
                );
            }
        }
    }
    // A valid nonbuiltin carrier cannot hide a malformed carrier later in
    // the same replacement condition.
    let mut function = sequence_exit_function(SequenceExitShape::Or, "list[int]");
    let mut emitter = RustEmitter::new();
    keys_guard_prepare(&mut emitter, &function);
    let crate::HirStmt::If {
        condition: HirExpr::BoolOp { values, .. },
        ..
    } = &mut function.body[0]
    else {
        unreachable!()
    };
    sequence_exit_replace_authority(&mut values[0], &contextual_authority("len"));
    sequence_exit_replace_authority(&mut values[1], &MethodAuthority::Unclassified);
    let error = emitter
        .try_lower_checked_sequence_exit_guards_for_ir(&function.body[0], Some(&function.body[1..]))
        .unwrap_err();
    assert!(error.message.contains("unclassified source method"));
    assert!(emitter.checked_place_read_witnesses.is_empty());
}

#[test]
fn checked_sequence_exit_guards_preserve_nonbuiltin_authority_without_retry() {
    for authority in [
        MethodAuthority::Protocol {
            declaration: identity("user", "len"),
        },
        MethodAuthority::LocalNominal {
            declaration: identity("user", "len"),
        },
        MethodAuthority::InheritedNominal {
            declaration: identity("user", "len"),
        },
        MethodAuthority::Imported {
            declaration: identity("user", "len"),
        },
        contextual_authority("len"),
    ] {
        for shape in SEQUENCE_EXIT_SHAPES {
            for entry in SEQUENCE_EXIT_ENTRIES {
                let mut emitter = RustEmitter::new();
                let output =
                    sequence_exit_run(&mut emitter, entry, shape, "list[int]", authority.clone())
                        .unwrap();
                if matches!(
                    entry,
                    SequenceExitEntry::Direct | SequenceExitEntry::Capture
                ) {
                    assert!(output.is_none(), "{entry:?}/{shape:?}");
                } else if let Some(output) = output {
                    assert!(
                        output.contains("values.len()"),
                        "{entry:?}/{shape:?}: {output}"
                    );
                    let count = if matches!(shape, SequenceExitShape::Or) {
                        2
                    } else {
                        1
                    };
                    assert_eq!(output.matches("values.len()").count(), count, "{output}");
                    let prior_guards = usize::from(
                        matches!(shape, SequenceExitShape::Prefix)
                            && matches!(entry, SequenceExitEntry::Nested),
                    );
                    assert_eq!(
                        output.matches("let Some(").count(),
                        prior_guards,
                        "{entry:?}/{shape:?}: {output}"
                    );
                    assert!(
                        !output.contains("SifrInt::from_usize(values.len())"),
                        "{output}"
                    );
                } else {
                    assert!(
                        !matches!(entry, SequenceExitEntry::Structured),
                        "structured statement must retain ordinary dispatch"
                    );
                }
                // Blocks can decline when the scalar read loses its builtin
                // presence proof. The retained condition must still use ordinary
                // source dispatch, without a second builtin registry attempt.
                let mut function = sequence_exit_function(shape, "list[int]");
                let crate::HirStmt::If { condition, .. } =
                    &mut function.body[sequence_exit_target(shape)]
                else {
                    unreachable!()
                };
                sequence_exit_replace_authority(condition, &authority);
                let ordinary = render_expr(
                    &RustEmitter::new()
                        .lower_condition_expr_for_ir(condition)
                        .unwrap()
                        .unwrap(),
                );
                assert!(ordinary.contains("values.len()"), "{ordinary}");
                assert!(!ordinary.contains("SifrInt::from_usize"), "{ordinary}");
                if matches!(
                    entry,
                    SequenceExitEntry::Block
                        | SequenceExitEntry::Scoped
                        | SequenceExitEntry::Nested
                ) {
                    // Ordinary calls do not prove scalar presence. Retain the
                    // exact condition with an independent subsequent return to
                    // exercise each successful public block entry as well.
                    let offset = sequence_exit_target(shape);
                    function.body.truncate(offset + 1);
                    function.body.push(crate::HirStmt::Return {
                        value: Some(HirExpr::IntLiteral(0)),
                    });
                    if matches!(entry, SequenceExitEntry::Nested) {
                        function.body = vec![crate::HirStmt::If {
                            condition: HirExpr::BoolLiteral(true),
                            then_body: function.body,
                            elif_clauses: vec![],
                            else_body: None,
                        }];
                    }
                    let mut ordinary_emitter = RustEmitter::new();
                    keys_guard_prepare(&mut ordinary_emitter, &function);
                    let lowered = if matches!(entry, SequenceExitEntry::Scoped) {
                        ordinary_emitter
                            .try_lower_scoped_stmt_block_for_ir(&function.body[offset..])
                    } else if matches!(entry, SequenceExitEntry::Nested) {
                        ordinary_emitter.try_lower_stmt_block_for_ir(&function.body)
                    } else {
                        ordinary_emitter.try_lower_stmt_block_for_ir(&function.body[offset..])
                    };
                    let output =
                        crate::render_stmts(&lowered.unwrap().expect("independent ordinary block"));
                    assert!(
                        output.contains("values.len()"),
                        "{entry:?}/{shape:?}: {output}"
                    );
                    assert!(
                        !output.contains("SifrInt::from_usize(values.len())"),
                        "{output}"
                    );
                    if !matches!(entry, SequenceExitEntry::Nested) {
                        assert!(!output.contains("let Some("), "{output}");
                    }
                }

                assert!(
                    emitter
                        .checked_place_read_witnesses
                        .keys()
                        .all(|key| matches!(shape, SequenceExitShape::Prefix)
                            && key.ends_with("[int:0]"))
                );
            }
        }
    }
}

#[test]
fn checked_sequence_exit_guards_preserve_admitted_read_and_prefix_behavior() {
    for receiver in ["list[int]", "bytes", "str"] {
        for shape in SEQUENCE_EXIT_SHAPES {
            for entry in SEQUENCE_EXIT_ENTRIES {
                let output = sequence_exit_run(
                    &mut RustEmitter::new(),
                    entry,
                    shape,
                    receiver,
                    builtin_authority("len"),
                )
                .unwrap()
                .unwrap();
                assert!(
                    output.contains("let Some("),
                    "{receiver}/{entry:?}/{shape:?}: {output}"
                );
                assert!(output.contains("return"), "{output}");
                if matches!(shape, SequenceExitShape::Partial) {
                    assert!(output.contains("stop"), "{output}");
                } else {
                    assert!(!output.contains("values.len()"), "{output}");
                }
                assert!(!output.contains("unwrap("));
                assert!(!output.contains("expect("));
            }
        }
    }
    // Preserve source paths whose declarations or conditions still lower.
    for source in [
        "def truth(values: list[int]) -> int:\n    if not values:\n        return 0\n    return values[0]\n",
        "def alias(values: list[int]) -> int:\n    size = len(values)\n    if size == 0:\n        return 0\n    first = values[0]\n    return first\n",
        "def endpoints(values: list[int]) -> int:\n    if not values:\n        return 0\n    left, right = 0, len(values) - 1\n    first, last = values[left], values[right]\n    return first + last\n",
    ] {
        let output = crate::lib_codegen_tests::generate_rust_from_source(source);
        assert!(output.contains("let Some("), "{output}");
        assert!(!output.contains("compile_error!"), "{output}");
        assert!(!output.contains("unwrap("));
        assert!(!output.contains("expect("));
    }
    let mut emitter = RustEmitter::new();
    emitter.current_return_type = Some(Type::Result(
        Box::new(Type::Int),
        Box::new(Type::Class {
            identity: None,
            type_args: vec![],
            name: "IndexError".into(),
            fields: Default::default(),
            methods: Default::default(),
            parent_class: Some("Error".into()),
        }),
    ));
    let function = sequence_exit_function(SequenceExitShape::Full, "list[int]");
    keys_guard_prepare(&mut emitter, &function);
    assert!(
        emitter
            .try_lower_checked_sequence_exit_guards_for_ir(
                &function.body[0],
                Some(&function.body[1..])
            )
            .unwrap()
            .is_none()
    );
    assert!(emitter.checked_place_read_witnesses.is_empty());
}

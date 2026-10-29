use super::*;

fn valid_cfg(stmts: &[HirStmt]) -> ControlFlowGraph {
    build_control_flow_graph(stmts).expect("test HIR must produce a valid CFG")
}

fn valid_flow_facts(stmts: &[HirStmt]) -> FlowFacts {
    let facts = flow_facts(stmts).expect("test HIR must produce valid flow facts");
    assert_eq!(block_always_exits(stmts), Ok(facts.always_exits()));
    facts
}

#[test]
fn flow_facts_reports_always_raises_for_raise_only_branch() {
    let stmts = vec![HirStmt::If {
        condition: HirExpr::BoolLiteral(true),
        then_body: vec![HirStmt::Raise {
            value: HirExpr::Call {
                mutable_arg_places: Vec::new(),
                func: "ValueError".to_string(),
                args: vec![HirExpr::StringLiteral("bad".to_string())],
                ty: Type::Unknown,
            },
        }],
        elif_clauses: vec![],
        else_body: Some(vec![HirStmt::Raise {
            value: HirExpr::Call {
                mutable_arg_places: Vec::new(),
                func: "ValueError".to_string(),
                args: vec![HirExpr::StringLiteral("also bad".to_string())],
                ty: Type::Unknown,
            },
        }]),
    }];

    let facts = valid_flow_facts(&stmts);
    assert_eq!(facts.exit_effect(), FlowExitEffect::AlwaysRaises);
    assert!(facts.always_exits());
    assert!(!facts.has_reachable_return());
}

#[test]
fn flow_facts_marks_trailing_stmt_unreachable_after_return() {
    let stmts = vec![
        HirStmt::Return {
            value: Some(HirExpr::IntLiteral(1)),
        },
        HirStmt::Expr {
            expr: HirExpr::IntLiteral(2),
        },
    ];

    let facts = valid_flow_facts(&stmts);
    assert_eq!(facts.reachable_top_level_stmt_indices(), &[0]);
    assert_eq!(facts.unreachable_top_level_stmt_indices(), &[1]);
}

#[test]
fn flow_facts_collects_reachable_return_types_only() {
    let stmts = vec![
        HirStmt::Return {
            value: Some(HirExpr::IntLiteral(1)),
        },
        HirStmt::Return {
            value: Some(HirExpr::StringLiteral("never".to_string())),
        },
    ];

    let facts = valid_flow_facts(&stmts);
    assert_eq!(facts.reachable_return_types(), &[Type::Int]);
}

#[test]
fn suppressible_python_context_return_body_does_not_close_cfg_fallthrough() {
    let statements = vec![HirStmt::With {
        items: vec![sifr_ir::HirWithItem {
            target: "value".to_string(),
            context: HirExpr::Name {
                name: "manager".to_string(),
                binding_id: None,
                ty: Type::Unknown,
            },
            kind: HirWithItemKind::Python {
                entered_type: Type::Unknown,
                enter_error_type: Type::Unknown,
                exit_error_type: Type::Unknown,
                entered_is_opaque_borrow: false,
                body_may_raise: true,
            },
        }],
        body: vec![HirStmt::Return {
            value: Some(HirExpr::IntLiteral(1)),
        }],
    }];

    let facts = valid_flow_facts(&statements);
    assert_eq!(facts.exit_effect(), FlowExitEffect::FallsThrough);
    assert!(facts.has_reachable_return());
}

#[test]
fn suppressible_python_async_context_return_body_keeps_following_code_reachable() {
    let statements = vec![
        HirStmt::AsyncWith {
            kind: HirAsyncWithKind::Python {
                context: HirExpr::Name {
                    name: "manager".to_string(),
                    binding_id: None,
                    ty: Type::Unknown,
                },
                manager_class: "Manager".to_string(),
                entered_type: Type::Unknown,
                enter_error_type: Type::Unknown,
                exit_error_type: Type::Unknown,
                entered_is_opaque_borrow: false,
                active_error_type: Type::Unknown,
                body_may_raise: true,
            },
            target: None,
            body: vec![HirStmt::Return {
                value: Some(HirExpr::IntLiteral(1)),
            }],
        },
        HirStmt::Return {
            value: Some(HirExpr::IntLiteral(2)),
        },
    ];

    let facts = valid_flow_facts(&statements);
    assert_eq!(facts.reachable_top_level_stmt_indices(), &[0, 1]);
    assert_eq!(facts.exit_effect(), FlowExitEffect::AlwaysReturns);
}

#[test]
fn control_flow_graph_validate_accepts_valid_graph() {
    let stmts = vec![HirStmt::If {
        condition: HirExpr::BoolLiteral(true),
        then_body: vec![HirStmt::Return {
            value: Some(HirExpr::IntLiteral(1)),
        }],
        elif_clauses: vec![],
        else_body: Some(vec![HirStmt::Return {
            value: Some(HirExpr::IntLiteral(2)),
        }]),
    }];
    let cfg = valid_cfg(&stmts);
    assert!(cfg.validate().is_ok());
}

#[test]
fn single_arm_match_uses_goto_instead_of_incomplete_branch() {
    let stmts = vec![HirStmt::Match {
        subject: HirExpr::Name {
            name: "value".to_string(),
            binding_id: None,
            ty: Type::Union(vec![Type::Int, Type::None]),
        },
        subject_ty: Type::Union(vec![Type::Int, Type::None]),
        arms: vec![crate::HirMatchArm {
            pattern: crate::HirPattern::Class {
                class_name: "int".to_string(),
                class_type: Type::Int,
                fields: Vec::new(),
            },
            guard: None,
            body: vec![HirStmt::Return {
                value: Some(HirExpr::IntLiteral(1)),
            }],
        }],
    }];

    let cfg = valid_cfg(&stmts);
    let match_block = cfg
        .blocks()
        .iter()
        .find(|block| block.label == CfgBlockLabel::Statement("match"))
        .expect("match block");
    assert!(matches!(match_block.terminator, CfgTerminator::Goto(_)));
}

#[test]
fn cfg_builder_returns_invariant_error_instead_of_panicking() {
    let mut builder = CfgBuilder::new(0);
    let invalid = builder.new_block(CfgBlockLabel::Synthetic, None);
    builder.set_terminator(invalid, CfgTerminator::Branch(vec![builder.exit]));

    let error = builder
        .finish(invalid)
        .expect_err("one-target branch must return an invariant error");
    assert!(error.to_string().contains("incomplete (1 target(s))"));
}

#[test]
fn control_flow_graph_validate_rejects_invalid_edge() {
    let cfg = ControlFlowGraph::new(
        vec![CfgBlock {
            id: 0,
            label: CfgBlockLabel::Entry,
            top_level_stmt_index: None,
            terminator: CfgTerminator::Goto(usize::MAX),
        }],
        0,
        0,
        vec![],
    );
    let err = cfg
        .validate()
        .expect_err("invalid edge should fail validation");
    assert!(err.to_string().contains("invalid successor"));
}

#[test]
fn control_flow_graph_shape_is_deterministic_across_rebuilds() {
    let stmts = vec![
        HirStmt::While {
            condition: HirExpr::BoolLiteral(true),
            body: vec![HirStmt::If {
                condition: HirExpr::BoolLiteral(true),
                then_body: vec![HirStmt::Continue],
                elif_clauses: vec![],
                else_body: Some(vec![HirStmt::Break]),
            }],
            else_body: Some(vec![HirStmt::Return {
                value: Some(HirExpr::IntLiteral(7)),
            }]),
        },
        HirStmt::TryExcept {
            body: vec![HirStmt::Return {
                value: Some(HirExpr::IntLiteral(9)),
            }],
            handlers: vec![crate::HirExceptHandler {
                error_type: Some("Error".to_string()),
                error_resolved_type: None,
                name: Some("e".to_string()),
                body: vec![HirStmt::Raise {
                    value: HirExpr::Call {
                        mutable_arg_places: Vec::new(),
                        func: "ValueError".to_string(),
                        args: vec![HirExpr::StringLiteral("bad".to_string())],
                        ty: Type::Unknown,
                    },
                }],
            }],
            body_error_types: vec![Type::Any],
        },
    ];

    let cfg_one = valid_cfg(&stmts);
    let cfg_two = valid_cfg(&stmts);
    let facts_one = valid_flow_facts(&stmts);
    let facts_two = valid_flow_facts(&stmts);
    assert_eq!(cfg_one.shape_fingerprint(), cfg_two.shape_fingerprint());
    assert_eq!(facts_one, facts_two);
}

#[test]
fn cfg_repeat_run_matrix_is_deterministic() {
    let corpus: Vec<Vec<HirStmt>> = vec![
        vec![HirStmt::If {
            condition: HirExpr::BoolLiteral(true),
            then_body: vec![HirStmt::Return {
                value: Some(HirExpr::IntLiteral(1)),
            }],
            elif_clauses: vec![(
                HirExpr::BoolLiteral(false),
                vec![HirStmt::Raise {
                    value: HirExpr::Call {
                        mutable_arg_places: Vec::new(),
                        func: "ValueError".to_string(),
                        args: vec![HirExpr::StringLiteral("bad".to_string())],
                        ty: Type::Unknown,
                    },
                }],
            )],
            else_body: Some(vec![HirStmt::Return {
                value: Some(HirExpr::IntLiteral(2)),
            }]),
        }],
        vec![HirStmt::For {
            target: "n".to_string(),
            target_ty: Type::Int,
            iter: HirExpr::RangeLiteral {
                start: Box::new(HirExpr::IntLiteral(0)),
                end: Box::new(HirExpr::IntLiteral(5)),
                step: None,
                ty: Type::List(Box::new(Type::Int)),
            },
            body: vec![
                HirStmt::If {
                    condition: HirExpr::BoolLiteral(true),
                    then_body: vec![HirStmt::Continue],
                    elif_clauses: vec![],
                    else_body: Some(vec![HirStmt::Break]),
                },
                HirStmt::Expr {
                    expr: HirExpr::IntLiteral(9),
                },
            ],
            else_body: Some(vec![HirStmt::Return {
                value: Some(HirExpr::IntLiteral(7)),
            }]),
        }],
        vec![
            HirStmt::Raise {
                value: HirExpr::Call {
                    mutable_arg_places: Vec::new(),
                    func: "ValueError".to_string(),
                    args: vec![HirExpr::StringLiteral("x".to_string())],
                    ty: Type::Unknown,
                },
            },
            HirStmt::Return {
                value: Some(HirExpr::IntLiteral(99)),
            },
        ],
    ];

    for stmts in corpus {
        let cfg_first = valid_cfg(&stmts);
        let cfg_second = valid_cfg(&stmts);
        let facts_first = valid_flow_facts(&stmts);
        let facts_second = valid_flow_facts(&stmts);
        assert_eq!(
            cfg_first.shape_fingerprint(),
            cfg_second.shape_fingerprint()
        );
        assert_eq!(facts_first, facts_second);
    }
}

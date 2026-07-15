use super::*;
use ruff_text_size::TextRange;
use sifr_ir::{
    PythonCleanupPolicy, PythonInteropDeclaration, PythonInteropDecoratorKind, PythonInteropEffect,
    PythonTargetPath,
};

fn class_type(name: &str) -> Type {
    Type::Class {
        name: name.to_string(),
        fields: vec![],
        methods: vec![],
        parent_class: None,
    }
}

fn python_item(target: &str, manager: &str) -> HirWithItem {
    HirWithItem {
        target: target.to_string(),
        context: sifr_ir::HirExpr::Name {
            name: manager.to_string(),
            ty: class_type("Manager"),
        },
        kind: HirWithItemKind::Python {
            entered_type: class_type("Entered"),
            enter_error_type: class_type("PythonError"),
            exit_error_type: class_type("PythonError"),
            entered_is_opaque_borrow: true,
        },
    }
}

fn emitter() -> RustEmitter {
    let mut emitter = RustEmitter::new();
    emitter
        .try_closure_error_type
        .push("PythonError".to_string());
    emitter.python_opaque_classes.insert(
        "Entered".to_string(),
        PythonInteropDeclaration {
            kind: PythonInteropDecoratorKind::Opaque,
            target: Some(PythonTargetPath {
                segments: vec!["fixture".to_string(), "Entered".to_string()],
                span: TextRange::default(),
            }),
            span: TextRange::default(),
            effect: PythonInteropEffect::BlockingIo,
            cleanup: Some(PythonCleanupPolicy::Drop),
            consumes_receiver: false,
            parameters: vec![],
            required_import_root: Some("fixture".to_string()),
            callbacks: Vec::new(),
            buffer: None,
        },
    );
    emitter
}

#[test]
fn python_context_emits_enter_outcome_and_replay_aware_exit() {
    let mut emitter = emitter();
    let lowered = emitter
        .try_lower_python_context_with_for_ir(
            &[python_item("entered", "manager")],
            &[HirStmt::Expr {
                expr: sifr_ir::HirExpr::Name {
                    name: "entered".to_string(),
                    ty: class_type("Entered"),
                },
            }],
        )
        .expect("lowering should succeed")
        .expect("Python context should lower");
    let rendered = crate::render_stmts(&[lowered]);

    assert!(rendered.contains("context_enter_with_callbacks("));
    assert!(rendered.contains("&__sifr_python_context_manager_0.__sifr_python_callbacks"));
    assert!(rendered.contains(
        "context_exit_normal_with_callbacks(__sifr_python_context_manager_0.__sifr_python_object"
    ));
    assert!(rendered.contains("context_exit_python_error_with_callbacks("));
    assert!(rendered.contains("context_exit_sifr_cause_with_callbacks("));
    assert!(rendered.contains("Ok(Ok(Some(_)))"));
    assert!(!rendered.contains("return __sifr_context_return"));
    syn::parse_file(&format!("fn generated() {{ {rendered} }}"))
        .expect("rendered context lowering should be valid Rust syntax");
}

#[test]
fn normal_context_exit_observes_typed_retained_callback_failure() {
    let mut emitter = emitter();
    emitter
        .python_retained_callback_errors
        .insert("Manager".to_string(), vec![class_type("HandlerError")]);
    let lowered = emitter
        .try_lower_python_context_with_for_ir(&[python_item("entered", "manager")], &[])
        .expect("lowering should succeed")
        .expect("Python context should lower");
    let rendered = crate::render_stmts(&[lowered]);
    assert!(
        rendered.contains("__sifr_python_context_manager_0.__sifr_python_callbacks.owner()"),
        "{rendered}"
    );
    assert!(
        rendered.contains("__sifr_python_context_manager_0.__sifr_python_callback_failure_0"),
        "{rendered}"
    );
    assert!(rendered.contains("take_if_owner_first"), "{rendered}");
    syn::parse_file(&format!("fn generated() {{ {rendered} }}"))
        .expect("typed context callback cleanup should be valid Rust syntax");
}

#[test]
fn multiple_python_contexts_nest_for_reverse_exit_order() {
    let mut emitter = emitter();
    let lowered = emitter
        .try_lower_python_context_with_for_ir(
            &[
                python_item("outer_value", "outer"),
                python_item("inner_value", "inner"),
            ],
            &[],
        )
        .expect("lowering should succeed")
        .expect("Python contexts should lower");
    let rendered = crate::render_stmts(&[lowered]);

    let outer_enter = rendered
        .find("&__sifr_python_context_manager_1.__sifr_python_object")
        .expect("outer enter");
    let inner_enter = rendered
        .find("&__sifr_python_context_manager_0.__sifr_python_object")
        .expect("inner enter");
    let inner_exit = rendered
        .find("context_exit_normal_with_callbacks(__sifr_python_context_manager_0.")
        .expect("inner exit");
    let outer_exit = rendered
        .find("context_exit_normal_with_callbacks(__sifr_python_context_manager_1.")
        .expect("outer exit");
    assert!(outer_enter < inner_enter && inner_enter < inner_exit && inner_exit < outer_exit);
}

#[test]
fn top_level_break_and_continue_become_cleanup_outcomes() {
    let rewritten = rewrite_context_control_flow(
        vec![
            RustStmt::Break,
            RustStmt::Loop {
                body: vec![RustStmt::Continue],
            },
        ],
        0,
    );
    assert!(matches!(rewritten[0], RustStmt::Return(Some(_))));
    assert!(matches!(
        &rewritten[1],
        RustStmt::Loop { body } if matches!(body.as_slice(), [RustStmt::Continue])
    ));
}

#[test]
fn let_else_control_flow_is_rewritten_for_context_cleanup() {
    let rewritten = rewrite_context_control_flow(
        vec![RustStmt::LetElse {
            pattern: "Some(value)".to_string(),
            value: RustExpr::Ident("maybe".to_string()),
            else_body: vec![RustStmt::Break],
        }],
        0,
    );
    assert!(matches!(
        &rewritten[0],
        RustStmt::LetElse { else_body, .. }
            if matches!(else_body.as_slice(), [RustStmt::Return(Some(_))])
    ));
}

#[test]
fn entered_binding_preserves_mutability_and_mixed_item_nesting() {
    let mut emitter = emitter();
    emitter.mutated_vars.insert("entered".to_string());
    let native = HirWithItem {
        target: "native".to_string(),
        context: sifr_ir::HirExpr::IntLiteral(1),
        kind: HirWithItemKind::Native {
            has_context_manager_protocol: false,
        },
    };
    let lowered = emitter
        .try_lower_python_context_with_for_ir(&[native, python_item("entered", "manager")], &[])
        .expect("mixed lowering should succeed")
        .expect("mixed context should lower");
    let rendered = crate::render_stmts(&[lowered]);
    assert!(rendered.contains("let Some(mut entered)"));
    assert!(
        rendered.find("let _native = 1_i64").expect("native item")
            < rendered
                .find("context_enter_with_callbacks(")
                .expect("Python enter")
    );
}

#[test]
fn sync_python_context_uses_async_closure_when_nested_body_awaits() {
    let mut emitter = emitter();
    let lowered = emitter
        .try_lower_python_context_with_for_ir(
            &[python_item("entered", "manager")],
            &[HirStmt::Expr {
                expr: sifr_ir::HirExpr::Await {
                    value: Box::new(sifr_ir::HirExpr::Name {
                        name: "pending".to_string(),
                        ty: Type::Unknown,
                    }),
                    ty: Type::Unknown,
                },
            }],
        )
        .expect("lowering should succeed")
        .expect("Python context should lower");
    let rendered = crate::render_stmts(&[lowered]);

    assert!(rendered.contains("(async || {"), "{rendered}");
    assert!(rendered.contains("})().await"), "{rendered}");
}

#[test]
fn cause_classification_uses_canonical_resolved_types() {
    assert_eq!(
        classify_cause_kind(Some(&class_type("TimeoutError")), "Alias"),
        "Timeout"
    );
    assert_eq!(
        classify_cause_kind(Some(&class_type("CancelableTask")), "CancelableTask"),
        "OrdinaryError"
    );
    assert_eq!(
        classify_cause_kind(Some(&class_type("CancellationError")), "Alias"),
        "Cancellation"
    );
}

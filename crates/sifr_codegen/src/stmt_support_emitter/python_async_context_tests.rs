use super::*;
use crate::HirExpr;
use ruff_text_size::TextRange;
use sifr_ir::{
    PythonCleanupPolicy, PythonInteropDeclaration, PythonInteropDecoratorKind, PythonInteropEffect,
    PythonTargetPath,
};

fn class_type(name: &str) -> Type {
    Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: name.to_string(),
        fields: vec![],
        methods: vec![],
        parent_class: None,
    }
}

fn emitter() -> RustEmitter {
    let mut emitter = RustEmitter::new();
    emitter
        .try_closure_error_type
        .push("PythonError".to_string());
    for (name, cleanup) in [
        ("Manager", PythonCleanupPolicy::AsyncContext),
        ("Entered", PythonCleanupPolicy::Drop),
    ] {
        emitter.python_opaque_classes.insert(
            name.to_string(),
            PythonInteropDeclaration {
                kind: PythonInteropDecoratorKind::Opaque,
                target: Some(PythonTargetPath {
                    segments: vec!["fixture".to_string(), name.to_string()],
                    span: TextRange::default(),
                }),
                span: TextRange::default(),
                effect: PythonInteropEffect::BlockingIo,
                cleanup: Some(cleanup),
                consumes_receiver: false,
                parameters: vec![],
                required_import_root: Some("fixture".to_string()),
                callbacks: Vec::new(),
                buffer: None,
            },
        );
    }
    emitter
}

fn kind(active_error_type: Type) -> sifr_ir::HirAsyncWithKind {
    sifr_ir::HirAsyncWithKind::Python {
        context: HirExpr::Name {
            name: "manager".to_string(),
            ty: class_type("Manager"),
        },
        manager_class: "Manager".to_string(),
        entered_type: class_type("Entered"),
        enter_error_type: class_type("PythonError"),
        exit_error_type: class_type("PythonError"),
        entered_is_opaque_borrow: false,
        active_error_type,
    }
}

#[test]
fn async_python_context_emits_biased_cancellation_and_masked_exit() {
    let mut emitter = emitter();
    let lowered = emitter
        .try_lower_async_with_stmt_for_ir(
            &kind(class_type("PythonError")),
            Some("entered"),
            &[HirStmt::Pass],
        )
        .expect("lowering should succeed")
        .expect("Python async context should lower");
    let rendered = crate::render_stmts(&[lowered]);

    assert_eq!(rendered.matches("tokio::select!").count(), 1);
    assert_eq!(rendered.matches("biased;").count(), 1);
    assert!(rendered.contains("CancellationScopeLease::claim"));
    assert!(rendered.contains("__SIFR_TASK_CANCELLATION.scope"));
    assert!(rendered.contains("submit_async_context_enter"));
    assert!(rendered.contains("abandon_callback_owner_after_error_async"));
    assert!(rendered.contains("submit_async_context_exit"));
    assert!(rendered.contains("PythonAsyncExitCause::Python(replay.clone())"));
    assert!(rendered.contains("release_and_resume_parent"));
    assert!(rendered.contains("CancellationResume::Invoked"));
    assert!(rendered.contains("tokio::task::yield_now().await"));
    assert!(rendered.contains("record_context_cleanup_evidence"));
    assert!(rendered.contains(
        "record_context_ignored_suppression(\n                    \"cancellation:CancellationError\""
    ));
    assert!(!rendered.contains("enter_cancel"));
    syn::parse_file(&format!(
        "async fn generated() -> Result<(), PythonError> {{ {rendered} Ok(()) }}"
    ))
    .expect("rendered async context lowering should be valid Rust syntax");
}

#[test]
fn async_python_context_emits_all_concrete_body_outcomes() {
    let mut emitter = emitter();
    emitter.try_closure_depth = 1;
    emitter.loop_else_stack.push(false);
    let lowered = emitter
        .try_lower_async_with_stmt_for_ir(
            &kind(class_type("DomainError")),
            Some("entered"),
            &[
                HirStmt::Return {
                    value: Some(HirExpr::IntLiteral(7)),
                },
                HirStmt::Break,
                HirStmt::Continue,
            ],
        )
        .expect("lowering should succeed")
        .expect("Python async context should lower");
    let rendered = crate::render_stmts(&[lowered]);

    for outcome in [
        "Some(Ok(Ok(None)))",
        "Some(Ok(Ok(Some(__sifr_context_return))))",
        "Some(Ok(Err(false)))",
        "Some(Ok(Err(true)))",
        "Some(Err(mut __sifr_python_async_context_body_error_0))",
        "None =>",
    ] {
        assert!(
            rendered.contains(outcome),
            "missing outcome {outcome}\n{rendered}"
        );
    }
    assert!(rendered.contains("PythonAsyncExitCause::Sifr"));
    assert!(rendered.contains("record_context_ignored_suppression"));
}

#[test]
fn normal_async_context_exit_observes_typed_retained_callback_failure() {
    let mut emitter = emitter();
    emitter
        .python_retained_callback_errors
        .insert("Manager".to_string(), vec![class_type("HandlerError")]);
    let lowered = emitter
        .try_lower_async_with_stmt_for_ir(
            &kind(class_type("PythonError")),
            Some("entered"),
            &[HirStmt::Pass],
        )
        .expect("lowering should succeed")
        .expect("Python async context should lower");
    let rendered = crate::render_stmts(&[lowered]);

    assert!(
        rendered.contains("__sifr_python_async_context_manager_0.__sifr_python_callbacks.owner()"),
        "{rendered}"
    );
    assert!(
        rendered.contains(
            "__sifr_python_async_context_manager_0.__sifr_python_callback_failure_0.clone()"
        ),
        "{rendered}"
    );
    assert!(rendered.contains("take_if_owner_first"), "{rendered}");
}

#[test]
fn nested_async_python_context_preserves_outer_context_outcome_envelope() {
    let mut emitter = emitter();
    emitter.try_closure_depth = 1;
    emitter.python_context_envelope_depth = 1;
    let lowered = emitter
        .try_lower_async_with_stmt_for_ir(
            &kind(class_type("PythonError")),
            Some("entered"),
            &[HirStmt::Return {
                value: Some(HirExpr::IntLiteral(7)),
            }],
        )
        .expect("lowering should succeed")
        .expect("Python async context should lower");
    let rendered = crate::render_stmts(&[lowered]);

    assert!(
        rendered.contains("return Ok(Ok(Some(__sifr_context_return)))"),
        "{rendered}"
    );
}

#[test]
fn cancellation_arm_precedes_body_when_both_are_ready() {
    let mut emitter = emitter();
    let lowered = emitter
        .try_lower_async_with_stmt_for_ir(&kind(class_type("DomainError")), None, &[])
        .expect("lowering should succeed")
        .expect("Python async context should lower");
    let rendered = crate::render_stmts(&[lowered]);
    let body_select = rendered.rfind("tokio::select!").expect("body select");
    let body_select = &rendered[body_select..];
    let cancellation = body_select.find("_ = &mut").expect("cancellation arm");
    let body = body_select.find("result = &mut").expect("body arm");
    assert!(
        cancellation < body,
        "biased cancellation must be polled first"
    );
}

#[test]
fn async_python_context_converts_enter_failures_to_the_active_error_type() {
    let mut emitter = emitter();
    let lowered = emitter
        .try_lower_async_with_stmt_for_ir(&kind(class_type("Error")), None, &[])
        .expect("lowering should succeed")
        .expect("Python async context should lower");
    let rendered = crate::render_stmts(&[lowered]);

    assert!(
        rendered.contains("return Err((error).into());"),
        "{rendered}"
    );
    assert!(rendered.contains("conversion_error_0.into())"));
    assert!(
        rendered.contains("Error::new(") && rendered.contains(".to_string()"),
        "{rendered}"
    );
}

#[test]
fn async_python_context_resumes_parent_cancellation_after_enter_failure() {
    let mut emitter = emitter();
    let lowered = emitter
        .try_lower_async_with_stmt_for_ir(&kind(class_type("PythonError")), None, &[])
        .expect("lowering should succeed")
        .expect("Python async context should lower");
    let rendered = crate::render_stmts(&[lowered]);
    let enter_failure = rendered
        .split("Err(error) =>")
        .nth(1)
        .and_then(|tail| {
            tail.split("let __sifr_python_async_context_conversion_0")
                .next()
        })
        .expect("generated enter-failure arm");

    assert!(!enter_failure.contains("notification().is_notified()"));
    assert!(enter_failure.contains("release_and_resume_parent()"));
    assert!(enter_failure.contains("tokio::task::yield_now().await"));
    assert!(enter_failure.contains("SifrPythonAsyncContextError"));
    assert!(enter_failure.contains("return Err((error).into());"));
}

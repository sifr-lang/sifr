use super::*;

#[test]
fn assembled_ir_validation_reports_the_owning_codegen_boundary() {
    let items = vec![crate::RustItem::Struct {
        name: "Broken".to_string(),
        visibility: crate::Visibility::Private,
        derives: vec![],
        fields: vec![
            ("value".to_string(), RustType::I64),
            ("value".to_string(), RustType::I64),
        ],
    }];

    let error = crate::lib_modules_and_codegen::validate_assembled_file_items(&items)
        .expect_err("duplicate assembled fields must fail");

    assert!(error.message.contains("assembled file"));
    assert!(error.message.contains("duplicate field"));
}

#[test]
fn stdlib_preamble_reparse_reports_the_owning_codegen_boundary() {
    let error = crate::lib_modules_and_codegen::validate_stdlib_preamble_source("fn broken(")
        .expect_err("invalid stdlib preamble must fail");

    assert!(error.message.contains("stdlib preamble boundary"));
}

#[test]
fn sysroot_planning_failure_reports_the_project_codegen_boundary() {
    let error = crate::lib_project_codegen::map_sysroot_dependency_plan(
        Result::<Vec<String>, _>::Err("missing manifest"),
    )
    .expect_err("failed sysroot plan must stop project codegen");

    assert!(error.message.contains("failed to resolve Sifr sysroot"));
    assert!(error.message.contains("missing manifest"));
}

#[test]
fn unsupported_statement_shape_reaches_the_structured_codegen_error_boundary() {
    let mut emitter = RustEmitter::new();
    let stmt = HirStmt::Expr {
        expr: HirExpr::Compare {
            left: Box::new(HirExpr::IntLiteral(1)),
            ops: vec!["<".to_string(), "<".to_string()],
            comparators: vec![HirExpr::IntLiteral(2)],
            ty: Type::Bool,
        },
    };

    emitter.emit_stmt(&stmt);
    let error = emitter
        .take_codegen_error()
        .expect("invalid statement shape must reach the codegen accumulator");

    assert!(
        error
            .message
            .contains("structured statement lowering failed")
    );
    assert!(error.message.contains("ops/comparators length mismatch"));
}

#[test]
fn test_lib_decomposition_guards_keep_stmt_expr_logic_out_of_lib_rs() {
    let lib_src = include_str!("../lib.rs");
    let stmt_entrypoints_src = include_str!("../structured_stmt_entrypoints.rs");

    assert!(!lib_src.contains("mod stmt_emitter;"));
    assert!(!lib_src.contains("mod expr_emitter;"));
    assert!(!lib_src.contains("CodegenLoweringMode"));
    assert!(!lib_src.contains("StructuredPreferred"));
    assert!(!lib_src.contains("should_force_stmt_string_path"));
    assert!(!lib_src.contains("should_force_expr_string_path"));
    assert!(!lib_src.contains("fn emit_expr(&mut self, expr: &HirExpr) {"));
    assert!(!lib_src.contains("fn try_lower_structured_expr("));
    assert!(!lib_src.contains("fn emit_stmt(&mut self, stmt: &HirStmt) {"));

    let emit_stmt_start = stmt_entrypoints_src
        .find("fn emit_stmt(&mut self, stmt: &HirStmt) {")
        .expect("emit_stmt wrapper should exist");
    let impl_end = stmt_entrypoints_src[emit_stmt_start..]
        .find("\n    }\n}")
        .map(|offset| emit_stmt_start + offset)
        .expect("emit_stmt wrapper should end before impl close");
    let emit_stmt_wrapper = &stmt_entrypoints_src[emit_stmt_start..impl_end];
    assert!(
        emit_stmt_wrapper.contains("structured statement emission missing for production path")
    );
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

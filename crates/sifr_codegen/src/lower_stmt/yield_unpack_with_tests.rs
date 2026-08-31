use super::*;
use sifr_ir::{HirWithItem, HirWithItemKind};

#[test]
fn lowers_simple_yield_inside_generator_closure() {
    let stmt = HirStmt::Yield {
        value: HirExpr::IntLiteral(7),
    };
    let lowered = try_lower_simple_stmt_with_ctx(
        &stmt,
        false,
        &HashSet::new(),
        &HashSet::new(),
        SimpleStmtLoweringCtx {
            return_type: None,
            in_display_impl: false,
            in_class_scope: false,
            in_generator_closure: true,
        },
    )
    .expect("yield lowered");

    assert!(matches!(lowered[0], RustStmt::Expr(RustExpr::Await(_))));
}

#[test]
fn lowers_simple_yield_outside_generator_closure() {
    let stmt = HirStmt::Yield {
        value: HirExpr::IntLiteral(7),
    };
    let lowered = try_lower_simple_stmt_with_ctx(
        &stmt,
        false,
        &HashSet::new(),
        &HashSet::new(),
        SimpleStmtLoweringCtx::default(),
    );

    assert!(lowered.is_none());
}

#[test]
fn simple_star_unpack_requires_structured_cardinality_handling() {
    let stmt = HirStmt::StarUnpack {
        before: vec![sifr_ir::HirTupleTarget {
            binding: sifr_ir::HirTupleTargetBinding::Name("head".to_string()),
            ty: Type::Int,
            rebind_existing: false,
        }],
        star: sifr_ir::HirTupleTarget {
            binding: sifr_ir::HirTupleTargetBinding::Name("mid".to_string()),
            ty: Type::List(Box::new(Type::Int)),
            rebind_existing: false,
        },
        after: vec![sifr_ir::HirTupleTarget {
            binding: sifr_ir::HirTupleTargetBinding::Name("tail".to_string()),
            ty: Type::Int,
            rebind_existing: false,
        }],
        value: HirExpr::Name {
            name: "xs".to_string(),
            binding_id: None,
            ty: Type::List(Box::new(Type::Int)),
        },
        failure: Some(Type::Class {
            identity: None,
            name: "ValueError".to_string(),
            type_args: vec![],
            fields: vec![],
            methods: vec![],
            parent_class: None,
        }),
    };
    assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new()).is_none());
}

#[test]
fn lowers_simple_with_without_context_manager_protocol() {
    let stmt = HirStmt::With {
        items: vec![HirWithItem {
            target: "x".to_string(),
            context: HirExpr::IntLiteral(1),
            kind: HirWithItemKind::Native {
                has_context_manager_protocol: false,
            },
        }],
        body: vec![HirStmt::Expr {
            expr: HirExpr::Name {
                name: "x".to_string(),
                binding_id: None,
                ty: Type::Int,
            },
        }],
    };
    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("with lowered");
    assert!(matches!(lowered[0], RustStmt::Block(_)));
}

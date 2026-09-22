use super::*;
use crate::{HirExpr, RustExpr};
use sifr_type_system::ReceiverConvention;

fn list_method_stmt(method: &str) -> crate::HirStmt {
    crate::HirStmt::Expr {
        expr: HirExpr::MethodCall {
            object: Box::new(HirExpr::Name {
                name: "values".to_string(),
                binding_id: None,
                ty: Type::List(Box::new(Type::Int)),
            }),
            method: method.to_string(),
            args: Vec::new(),
            receiver_convention: Some(ReceiverConvention::MutableBorrow),
            receiver_target: None,
            mutable_arg_places: Vec::new(),
            source: None,
            ty: Type::None,
        },
    }
}

#[test]
fn refresh_fallback_rejects_presence_removing_mutations() {
    let witness = super::super::CheckedPlaceReadWitness {
        exclusive_owner: None,
        binding: "checked".to_string(),
        borrowed: false,
        copy_value: false,
        option: RustExpr::Ident("option".to_string()),
        dependencies: vec!["values".to_string()],
        order: 0,
    };

    assert!(RustEmitter::checked_place_refresh_precondition_holds(
        "name:values[int:0]",
        &witness,
        &list_method_stmt("append")
    ));
    assert!(!RustEmitter::checked_place_refresh_precondition_holds(
        "name:values[int:0]",
        &witness,
        &list_method_stmt("clear")
    ));
    assert!(RustEmitter::checked_place_refresh_precondition_holds(
        "name:values[int:0]",
        &witness,
        &list_method_stmt("reverse")
    ));
}

#[test]
fn tuple_rebinding_invalidates_index_alias_witnesses() {
    let witness = super::super::CheckedPlaceReadWitness {
        exclusive_owner: None,
        binding: "checked".to_string(),
        borrowed: false,
        copy_value: false,
        option: RustExpr::Ident("option".to_string()),
        dependencies: vec!["index".to_string()],
        order: 0,
    };
    for rebind_existing in [false, true] {
        let stmt = crate::HirStmt::TupleUnpack {
            targets: vec![sifr_ir::HirTupleTarget {
                binding: sifr_ir::HirTupleTargetBinding::Name("index".to_string()),
                ty: Type::Int,
                rebind_existing,
            }],
            value: HirExpr::TupleLiteral {
                elements: vec![HirExpr::IntLiteral(1)],
                ty: Type::Tuple(vec![Type::Int]),
            },
        };
        assert_eq!(
            RustEmitter::checked_place_witness_is_invalidated_by_stmt(&witness, &stmt),
            rebind_existing,
        );
    }
}

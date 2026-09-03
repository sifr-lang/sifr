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
        binding: "checked".to_string(),
        borrowed: false,
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

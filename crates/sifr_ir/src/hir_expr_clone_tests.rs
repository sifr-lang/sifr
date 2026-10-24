use crate::{BindingId, CompilerIntrinsicId, HirExpr};
use ruff_text_size::{TextRange, TextSize};
use sifr_type_system::Type;

#[test]
fn cloned_recursive_expression_keeps_independent_values_and_source_identity() {
    let range = TextRange::new(TextSize::from(7), TextSize::from(19));
    let mut original = HirExpr::IntrinsicCall {
        intrinsic: CompilerIntrinsicId::TestAssertEqual,
        args: vec![
            HirExpr::Name {
                name: "original".to_owned(),
                binding_id: Some(BindingId(42)),
                ty: Type::Int,
            },
            HirExpr::ListLiteral {
                elements: vec![HirExpr::StringLiteral("retained".to_owned())],
                ty: Type::List(Box::new(Type::Str)),
            },
        ],
        ty: Type::None,
        call_range: range,
        arg_ranges: vec![range, range],
    };
    let expected = format!("{original:?}");
    let cloned = original.clone();
    assert_eq!(format!("{cloned:?}"), expected);

    let HirExpr::IntrinsicCall {
        args, arg_ranges, ..
    } = &mut original
    else {
        panic!("intrinsic fixture");
    };
    let HirExpr::Name {
        name, binding_id, ..
    } = &mut args[0]
    else {
        panic!("name fixture");
    };
    name.clear();
    *binding_id = None;
    let HirExpr::ListLiteral { elements, .. } = &mut args[1] else {
        panic!("list fixture");
    };
    elements.clear();
    arg_ranges.clear();

    assert_eq!(format!("{cloned:?}"), expected);
    assert_ne!(format!("{original:?}"), expected);
}

#[test]
fn cloned_generator_keeps_boxed_body_and_optional_filter_independent() {
    let mut original = HirExpr::GeneratorExpr {
        expr: Box::new(HirExpr::StringLiteral("body".to_owned())),
        var: "item".to_owned(),
        iter: Box::new(HirExpr::ListLiteral {
            elements: vec![HirExpr::IntLiteral(7)],
            ty: Type::List(Box::new(Type::Int)),
        }),
        filter: Some(Box::new(HirExpr::BoolLiteral(true))),
        ty: Type::Str,
    };
    let expected = format!("{original:?}");
    let cloned = original.clone();
    let HirExpr::GeneratorExpr {
        expr,
        iter,
        filter,
        var,
        ..
    } = &mut original
    else {
        panic!("generator fixture");
    };
    *expr = Box::new(HirExpr::NoneLiteral);
    *iter = Box::new(HirExpr::NoneLiteral);
    *filter = None;
    var.clear();
    assert_eq!(format!("{cloned:?}"), expected);
    assert_ne!(format!("{original:?}"), expected);
}

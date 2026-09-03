use super::*;
use crate::{RustExpr, RustItem, RustStmt, RustType, Visibility};
use sifr_ir::HirExpr;
use sifr_type_system::{FixedIntType, Type};

#[test]
fn dispatcher_lowers_simple_module_constant_item() {
    let (item, rust_name) = try_lower_simple_module_constant_item(
        "answer",
        &Type::FixedInt(FixedIntType::I64),
        &HirExpr::IntLiteral(42),
    )
    .expect("dispatcher should lower simple constant");
    assert_eq!(rust_name, "ANSWER");
    assert!(matches!(item, RustItem::Const { .. }));
}

#[test]
fn dispatcher_result_lowers_simple_module_constant_item() {
    let lowered = try_lower_simple_module_constant_item_result(
        "answer",
        &Type::FixedInt(FixedIntType::I64),
        &HirExpr::IntLiteral(42),
    )
    .expect("result dispatcher should validate and lower")
    .expect("dispatcher should lower simple constant");
    assert_eq!(lowered.1, "ANSWER");
    assert!(matches!(lowered.0, RustItem::Const { .. }));
}

#[test]
fn module_non_finite_float_constants_render_canonically() {
    for (name, value, expected) in [
        ("nan", f64::NAN, "const NAN: f64 = f64::NAN;"),
        ("inf", f64::INFINITY, "const INF: f64 = f64::INFINITY;"),
        (
            "neg_inf",
            f64::NEG_INFINITY,
            "const NEG_INF: f64 = f64::NEG_INFINITY;",
        ),
    ] {
        let (item, _) = try_lower_simple_module_constant_item_result(
            name,
            &Type::Float,
            &HirExpr::FloatLiteral(value),
        )
        .expect("non-finite float module constant should lower")
        .expect("non-finite float module constant should be a Rust item");

        assert_eq!(crate::render_items(&[item]), format!("{expected}\n"));
    }
}

#[test]
fn dispatcher_result_reports_invalid_module_constant_name() {
    let err =
        try_lower_simple_module_constant_item_result("9bad", &Type::Int, &HirExpr::IntLiteral(42))
            .expect_err("invalid constant name should return error");
    assert!(
        err.message
            .contains("name must start with ASCII letter or underscore")
    );
}

#[test]
fn dispatcher_result_propagates_leaf_lowering_errors() {
    let err = try_lower_simple_module_constant_item_result(
        "answer",
        &Type::Bool,
        &HirExpr::Compare {
            left: Box::new(HirExpr::IntLiteral(1)),
            ops: vec!["==".to_string()],
            comparators: vec![],
            ty: Type::Bool,
        },
    )
    .expect_err("invalid compare shape should propagate as codegen error");
    assert!(err.message.contains("ops/comparators length mismatch"));
}

#[test]
fn exact_module_int_const_uses_stateful_helper_lowering() {
    assert!(
        try_lower_simple_module_const_item("answer", &Type::Int, &HirExpr::IntLiteral(42))
            .is_none()
    );
}

#[test]
fn module_constant_helper_names_are_injective_and_warning_clean() {
    let upper = module_constant_helper_name("BASE");
    let lower = module_constant_helper_name("base");
    let separated = module_constant_helper_name("B_ASE");

    assert_eq!(upper, "__sifr_const_42415345");
    assert_eq!(lower, "__sifr_const_62617365");
    assert_eq!(separated, "__sifr_const_425f415345");
    assert_ne!(upper, lower);
    assert_ne!(upper, separated);
    for helper in [&upper, &lower, &separated] {
        assert!(
            helper
                .bytes()
                .all(|byte| byte == b'_' || byte.is_ascii_lowercase() || byte.is_ascii_digit()),
            "helper must be a Rust snake_case identifier: {helper}"
        );
        assert!(
            helper
                .bytes()
                .next()
                .is_some_and(|byte| byte == b'_' || byte.is_ascii_lowercase()),
            "helper must not begin with a digit: {helper}"
        );
    }
}

#[test]
fn dispatcher_result_lowers_large_module_int_const_as_sifr_int_helper() {
    let (item, rust_name_call) = try_lower_simple_module_constant_item_result(
        "limit",
        &Type::Int,
        &HirExpr::LargeIntLiteral("100000000000000000000".to_string()),
    )
    .expect("large exact int module const should validate")
    .expect("large exact int module const should lower");

    assert_eq!(rust_name_call, "__sifr_const_6c696d6974()");
    assert!(matches!(
        item,
        RustItem::Fn {
            ref name,
            visibility: Visibility::Private,
            ret: Some(RustType::Named(ref ret)),
            ..
        } if name == "__sifr_const_6c696d6974" && ret == "SifrInt"
    ));

    let rendered = crate::render_items(&[item]);
    assert!(rendered.contains("fn __sifr_const_6c696d6974() -> SifrInt"));
    assert!(rendered.contains("SifrInt::from_signed_bytes_be"));
    assert!(!rendered.contains("parse_decimal"));
    assert!(!rendered.contains(".unwrap("));
    assert!(!rendered.contains(".expect("));
}

#[test]
fn exact_module_name_const_uses_stateful_helper_lowering() {
    assert!(
        try_lower_simple_module_const_item(
            "answer",
            &Type::Int,
            &HirExpr::Name {
                name: "x".to_string(),
                binding_id: None,
                ty: Type::Int,
            },
        )
        .is_none()
    );
}

#[test]
fn does_not_lower_non_primitive_module_const_item() {
    assert!(
        try_lower_simple_module_const_item(
            "name",
            &Type::Str,
            &HirExpr::StringLiteral("x".to_string()),
        )
        .is_none()
    );
}

#[test]
fn does_not_lower_non_leaf_module_const_item() {
    assert!(
        try_lower_simple_module_const_item(
            "answer",
            &Type::Int,
            &HirExpr::Call {
                mutable_arg_places: Vec::new(),
                func: "compute_answer".to_string(),
                args: vec![],
                ty: Type::Int,
            },
        )
        .is_none()
    );
}

#[test]
fn exact_literal_int_const_uses_stateful_helper_lowering() {
    assert!(
        try_lower_simple_module_const_item(
            "answer",
            &Type::LiteralInt(42),
            &HirExpr::IntLiteral(42),
        )
        .is_none()
    );
}

#[test]
fn lowers_simple_module_literal_bool_const_item() {
    let (item, rust_name) = try_lower_simple_module_const_item(
        "enabled",
        &Type::LiteralBool(true),
        &HirExpr::BoolLiteral(true),
    )
    .expect("literal bool const should lower");
    assert_eq!(rust_name, "ENABLED");
    assert!(matches!(
        item,
        RustItem::Const {
            name,
            visibility: Visibility::Private,
            ty: crate::RustType::Bool,
            ..
        } if name == "ENABLED"
    ));
}

#[test]
fn exact_alias_int_const_uses_stateful_helper_lowering() {
    let alias_int = Type::alias("Meters", Type::Int);
    assert!(
        try_lower_simple_module_const_item("answer", &alias_int, &HirExpr::IntLiteral(42))
            .is_none()
    );
}

#[test]
fn dispatcher_lowers_alias_primitive_module_const_as_const_item() {
    let alias_bool = Type::alias("Flag", Type::Bool);
    let (item, rust_name) = try_lower_simple_module_constant_item(
        "enabled",
        &alias_bool,
        &HirExpr::Name {
            name: "flag".to_string(),
            binding_id: None,
            ty: alias_bool.clone(),
        },
    )
    .expect("dispatcher should lower alias primitive constant");
    assert_eq!(rust_name, "ENABLED");
    assert!(matches!(
        item,
        RustItem::Const {
            name,
            visibility: Visibility::Private,
            value: RustExpr::Ident(ref ident),
            ..
        } if name == "ENABLED" && ident == "flag"
    ));
}

#[test]
fn does_not_lower_alias_primitive_module_helper_const_item() {
    let alias_int = Type::alias("Meters", Type::Int);
    assert!(
        try_lower_simple_module_helper_const_item("answer", &alias_int, &HirExpr::IntLiteral(42),)
            .is_none()
    );
}

#[test]
fn lowers_simple_module_string_const_item() {
    let (item, rust_name_call) = try_lower_simple_module_string_const_item(
        "greeting",
        &Type::Str,
        &HirExpr::StringLiteral("hi".to_string()),
    )
    .expect("simple string const should lower");
    assert_eq!(rust_name_call, "__sifr_const_6772656574696e67()");
    assert!(matches!(
        item,
        RustItem::Fn {
            name,
            visibility: Visibility::Private,
            ret: Some(RustType::String_),
            ..
        } if name == "__sifr_const_6772656574696e67"
    ));
}

#[test]
fn lowers_simple_module_alias_string_const_item() {
    let alias_str = Type::alias("Message", Type::Str);
    let (item, rust_name_call) = try_lower_simple_module_string_const_item(
        "greeting",
        &alias_str,
        &HirExpr::StringLiteral("hi".to_string()),
    )
    .expect("alias string const should lower");
    assert_eq!(rust_name_call, "__sifr_const_6772656574696e67()");
    assert!(matches!(
        item,
        RustItem::Fn {
            name,
            visibility: Visibility::Private,
            ret: Some(RustType::String_),
            ..
        } if name == "__sifr_const_6772656574696e67"
    ));
}

#[test]
fn dispatcher_lowers_alias_string_module_const_as_string_item() {
    let alias_str = Type::alias("Message", Type::Str);
    let (item, rust_name_call) = try_lower_simple_module_constant_item(
        "greeting",
        &alias_str,
        &HirExpr::Name {
            name: "msg".to_string(),
            binding_id: None,
            ty: alias_str.clone(),
        },
    )
    .expect("dispatcher should lower alias string constant");
    assert_eq!(rust_name_call, "__sifr_const_6772656574696e67()");
    assert!(matches!(
        item,
        RustItem::Fn {
            name,
            visibility: Visibility::Private,
            ret: Some(RustType::String_),
            body,
            ..
        } if name == "__sifr_const_6772656574696e67"
            && matches!(
                body.first(),
                Some(RustStmt::Return(Some(RustExpr::MethodCall { receiver, method, .. })))
                    if matches!(receiver.as_ref(), RustExpr::Ident(n) if n == "msg") && method == "to_string"
            )
    ));
}

#[test]
fn does_not_lower_non_string_module_string_const_item() {
    assert!(
        try_lower_simple_module_string_const_item(
            "greeting",
            &Type::Int,
            &HirExpr::StringLiteral("hi".to_string()),
        )
        .is_none()
    );
}

#[test]
fn lowers_simple_module_string_name_const_item() {
    let (item, rust_name_call) = try_lower_simple_module_string_const_item(
        "greeting",
        &Type::Str,
        &HirExpr::Name {
            name: "msg".to_string(),
            binding_id: None,
            ty: Type::Str,
        },
    )
    .expect("string name const should lower");
    assert_eq!(rust_name_call, "__sifr_const_6772656574696e67()");
    assert!(matches!(
        item,
        RustItem::Fn {
            name,
            visibility: Visibility::Private,
            ret: Some(RustType::String_),
            body,
            ..
        } if name == "__sifr_const_6772656574696e67"
            && matches!(
                body.first(),
                Some(RustStmt::Return(Some(RustExpr::MethodCall { receiver, method, .. })))
                    if matches!(receiver.as_ref(), RustExpr::Ident(n) if n == "msg") && method == "to_string"
            )
    ));
}

#[test]
fn lowers_simple_module_literal_string_const_item() {
    let (item, rust_name_call) = try_lower_simple_module_string_const_item(
        "greeting",
        &Type::LiteralStr("hi".to_string()),
        &HirExpr::StringLiteral("hi".to_string()),
    )
    .expect("literal string const should lower");
    assert_eq!(rust_name_call, "__sifr_const_6772656574696e67()");
    assert!(matches!(
        item,
        RustItem::Fn {
            name,
            visibility: Visibility::Private,
            ret: Some(RustType::String_),
            ..
        } if name == "__sifr_const_6772656574696e67"
    ));
}

#[test]
fn does_not_lower_non_leaf_module_string_const_item() {
    assert!(
        try_lower_simple_module_string_const_item(
            "greeting",
            &Type::Str,
            &HirExpr::BinOp {
                left: Box::new(HirExpr::StringLiteral("a".to_string())),
                op: "+".to_string(),
                right: Box::new(HirExpr::StringLiteral("b".to_string())),
                ty: Type::Str,
            },
        )
        .is_none()
    );
}

#[test]
fn lowers_simple_module_none_const_item() {
    let (item, rust_name_call) =
        try_lower_simple_module_none_const_item("nothing", &Type::None, &HirExpr::NoneLiteral)
            .expect("none const should lower");
    assert_eq!(rust_name_call, "__sifr_const_6e6f7468696e67()");
    assert!(matches!(
        item,
        RustItem::Fn {
            name,
            visibility: Visibility::Private,
            ret: Some(RustType::Unit),
            ..
        } if name == "__sifr_const_6e6f7468696e67"
    ));
}

#[test]
fn lowers_simple_module_alias_none_const_item() {
    let alias_none = Type::alias("Nothing", Type::None);
    let (item, rust_name_call) =
        try_lower_simple_module_none_const_item("nothing", &alias_none, &HirExpr::NoneLiteral)
            .expect("alias none const should lower");
    assert_eq!(rust_name_call, "__sifr_const_6e6f7468696e67()");
    assert!(matches!(
        item,
        RustItem::Fn {
            name,
            visibility: Visibility::Private,
            ret: Some(RustType::Unit),
            ..
        } if name == "__sifr_const_6e6f7468696e67"
    ));
}

#[test]
fn lowers_simple_module_none_name_const_item() {
    let (item, rust_name_call) = try_lower_simple_module_none_const_item(
        "nothing",
        &Type::None,
        &HirExpr::Name {
            name: "n".to_string(),
            binding_id: None,
            ty: Type::None,
        },
    )
    .expect("none name const should lower");
    assert_eq!(rust_name_call, "__sifr_const_6e6f7468696e67()");
    assert!(matches!(
        item,
        RustItem::Fn {
            name,
            visibility: Visibility::Private,
            ret: Some(RustType::Unit),
            body,
            ..
        } if name == "__sifr_const_6e6f7468696e67"
            && matches!(body.first(), Some(RustStmt::Return(Some(RustExpr::Ident(n)))) if n == "n")
    ));
}

#[test]
fn lowers_simple_module_alias_none_name_const_item() {
    let alias_none = Type::alias("Nothing", Type::None);
    let (item, rust_name_call) = try_lower_simple_module_none_const_item(
        "nothing",
        &alias_none,
        &HirExpr::Name {
            name: "n".to_string(),
            binding_id: None,
            ty: alias_none.clone(),
        },
    )
    .expect("alias none name const should lower");
    assert_eq!(rust_name_call, "__sifr_const_6e6f7468696e67()");
    assert!(matches!(
        item,
        RustItem::Fn {
            name,
            visibility: Visibility::Private,
            ret: Some(RustType::Unit),
            body,
            ..
        } if name == "__sifr_const_6e6f7468696e67"
            && matches!(body.first(), Some(RustStmt::Return(Some(RustExpr::Ident(n)))) if n == "n")
    ));
}

#[test]
fn dispatcher_lowers_alias_none_module_const_as_none_item() {
    let alias_none = Type::alias("Nothing", Type::None);
    let (item, rust_name_call) =
        try_lower_simple_module_constant_item("nothing", &alias_none, &HirExpr::NoneLiteral)
            .expect("dispatcher should lower alias none constant");
    assert_eq!(rust_name_call, "__sifr_const_6e6f7468696e67()");
    assert!(matches!(
        item,
        RustItem::Fn {
            name,
            visibility: Visibility::Private,
            ret: Some(RustType::Unit),
            ..
        } if name == "__sifr_const_6e6f7468696e67"
    ));
}

#[test]
fn dispatcher_lowers_alias_none_name_module_const_as_none_item() {
    let alias_none = Type::alias("Nothing", Type::None);
    let (item, rust_name_call) = try_lower_simple_module_constant_item(
        "nothing",
        &alias_none,
        &HirExpr::Name {
            name: "n".to_string(),
            binding_id: None,
            ty: alias_none.clone(),
        },
    )
    .expect("dispatcher should lower alias none name constant");
    assert_eq!(rust_name_call, "__sifr_const_6e6f7468696e67()");
    assert!(matches!(
        item,
        RustItem::Fn {
            name,
            visibility: Visibility::Private,
            ret: Some(RustType::Unit),
            body,
            ..
        } if name == "__sifr_const_6e6f7468696e67"
            && matches!(body.first(), Some(RustStmt::Return(Some(RustExpr::Ident(n)))) if n == "n")
    ));
}

#[test]
fn does_not_lower_non_none_module_none_const_item() {
    assert!(
        try_lower_simple_module_none_const_item("nothing", &Type::None, &HirExpr::IntLiteral(0),)
            .is_none()
    );
}

#[test]
fn does_not_lower_non_none_name_module_none_const_item() {
    assert!(
        try_lower_simple_module_none_const_item(
            "nothing",
            &Type::None,
            &HirExpr::Name {
                name: "x".to_string(),
                binding_id: None,
                ty: Type::Int,
            },
        )
        .is_none()
    );
}

#[test]
fn does_not_lower_alias_none_module_helper_const_item() {
    let alias_none = Type::alias("Nothing", Type::None);
    assert!(
        try_lower_simple_module_helper_const_item("nothing", &alias_none, &HirExpr::NoneLiteral)
            .is_none()
    );
}

#[test]
fn lowers_simple_module_helper_const_item_for_list_literal() {
    let ty = Type::List(Box::new(Type::Int));
    let value = HirExpr::ListLiteral {
        elements: vec![HirExpr::IntLiteral(1), HirExpr::IntLiteral(2)],
        ty: ty.clone(),
    };
    let (item, rust_name_call) = try_lower_simple_module_helper_const_item("nums", &ty, &value)
        .expect("simple non-primitive helper const should lower");
    assert_eq!(rust_name_call, "__sifr_const_6e756d73()");
    assert!(matches!(
        item,
        RustItem::Fn {
            name,
            visibility: Visibility::Private,
            ret: Some(RustType::Vec(_)),
            ..
        } if name == "__sifr_const_6e756d73"
    ));
}

#[test]
fn lowers_simple_module_helper_name_const_item() {
    let ty = Type::List(Box::new(Type::Int));
    let value = HirExpr::Name {
        name: "nums".to_string(),
        binding_id: None,
        ty: ty.clone(),
    };
    let (item, rust_name_call) = try_lower_simple_module_helper_const_item("data", &ty, &value)
        .expect("simple helper name const should lower");
    assert_eq!(rust_name_call, "__sifr_const_64617461()");
    assert!(matches!(
        item,
        RustItem::Fn {
            name,
            visibility: Visibility::Private,
            ret: Some(RustType::Vec(_)),
            body,
            ..
        } if name == "__sifr_const_64617461"
            && matches!(body.first(), Some(RustStmt::Return(Some(RustExpr::Ident(n)))) if n == "nums")
    ));
}

#[test]
fn does_not_lower_primitive_module_helper_const_item() {
    assert!(
        try_lower_simple_module_helper_const_item("answer", &Type::Int, &HirExpr::IntLiteral(42),)
            .is_none()
    );
}

#[test]
fn does_not_lower_alias_string_module_helper_const_item() {
    let alias_str = Type::alias("Message", Type::Str);
    assert!(
        try_lower_simple_module_helper_const_item(
            "greeting",
            &alias_str,
            &HirExpr::StringLiteral("hi".to_string()),
        )
        .is_none()
    );
}

#[test]
fn does_not_lower_non_leaf_module_helper_const_item() {
    let ty = Type::List(Box::new(Type::Int));
    let value = HirExpr::ListLiteral {
        elements: vec![HirExpr::Call {
            mutable_arg_places: Vec::new(),
            func: "build".to_string(),
            args: vec![],
            ty: Type::Int,
        }],
        ty: ty.clone(),
    };
    assert!(try_lower_simple_module_helper_const_item("nums", &ty, &value).is_none());
}

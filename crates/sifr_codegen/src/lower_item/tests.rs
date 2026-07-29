use super::*;
use crate::{RustExpr, RustItem, RustStmt, RustType, Visibility};
use sifr_ir::HirExpr;
use sifr_type_system::Type;

#[test]
fn dispatcher_lowers_simple_module_constant_item() {
    let (item, rust_name) =
        try_lower_simple_module_constant_item("answer", &Type::Int, &HirExpr::IntLiteral(42))
            .expect("dispatcher should lower simple constant");
    assert_eq!(rust_name, "ANSWER");
    assert!(matches!(item, RustItem::Const { .. }));
}

#[test]
fn dispatcher_result_lowers_simple_module_constant_item() {
    let lowered = try_lower_simple_module_constant_item_result(
        "answer",
        &Type::Int,
        &HirExpr::IntLiteral(42),
    )
    .expect("result dispatcher should validate and lower")
    .expect("dispatcher should lower simple constant");
    assert_eq!(lowered.1, "ANSWER");
    assert!(matches!(lowered.0, RustItem::Const { .. }));
}

#[test]
fn dispatcher_result_reports_invalid_module_constant_name() {
    let err =
        try_lower_simple_module_constant_item_result("9bad", &Type::Int, &HirExpr::IntLiteral(42))
            .expect_err("invalid constant name should return error");
    assert!(err
        .message
        .contains("name must start with ASCII letter or underscore"));
}

#[test]
fn dispatcher_result_propagates_leaf_lowering_errors() {
    let err = try_lower_simple_module_constant_item_result(
        "answer",
        &Type::Int,
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
fn lowers_simple_module_int_const_item() {
    let (item, rust_name) =
        try_lower_simple_module_const_item("answer", &Type::Int, &HirExpr::IntLiteral(42))
            .expect("simple const should lower");
    assert_eq!(rust_name, "ANSWER");
    assert!(matches!(
        item,
        RustItem::Const {
            name,
            visibility: Visibility::Private,
            ty: crate::RustType::I64,
            ..
        } if name == "ANSWER"
    ));
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

    assert_eq!(rust_name_call, "__const_limit()");
    assert!(matches!(
        item,
        RustItem::Fn {
            ref name,
            visibility: Visibility::Private,
            ret: Some(RustType::Named(ref ret)),
            ..
        } if name == "__const_limit" && ret == "SifrInt"
    ));

    let rendered = crate::render_items(&[item]);
    assert!(rendered.contains("fn __const_limit() -> SifrInt"));
    assert!(rendered.contains(
            "SifrInt::parse_decimal(\"100000000000000000000\", ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS)"
        ));
    assert!(!rendered.contains(".unwrap("));
    assert!(!rendered.contains(".expect("));
}

#[test]
fn lowers_simple_module_name_const_item() {
    let (item, rust_name) = try_lower_simple_module_const_item(
        "answer",
        &Type::Int,
        &HirExpr::Name {
            name: "x".to_string(),
            binding_id: None,
            ty: Type::Int,
        },
    )
    .expect("simple name const should lower");
    assert_eq!(rust_name, "ANSWER");
    assert!(matches!(
        item,
        RustItem::Const {
            name,
            visibility: Visibility::Private,
            ty: crate::RustType::I64,
            value: RustExpr::Ident(ident),
        } if name == "ANSWER" && ident == "x"
    ));
}

#[test]
fn does_not_lower_non_primitive_module_const_item() {
    assert!(try_lower_simple_module_const_item(
        "name",
        &Type::Str,
        &HirExpr::StringLiteral("x".to_string()),
    )
    .is_none());
}

#[test]
fn does_not_lower_non_leaf_module_const_item() {
    assert!(try_lower_simple_module_const_item(
        "answer",
        &Type::Int,
        &HirExpr::Call {
            func: "compute_answer".to_string(),
            args: vec![],
            ty: Type::Int,
        },
    )
    .is_none());
}

#[test]
fn lowers_simple_module_literal_int_const_item() {
    let (item, rust_name) = try_lower_simple_module_const_item(
        "answer",
        &Type::LiteralInt(42),
        &HirExpr::IntLiteral(42),
    )
    .expect("literal int const should lower");
    assert_eq!(rust_name, "ANSWER");
    assert!(matches!(
        item,
        RustItem::Const {
            name,
            visibility: Visibility::Private,
            ty: crate::RustType::I64,
            ..
        } if name == "ANSWER"
    ));
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
fn lowers_simple_module_alias_int_const_item() {
    let alias_int = Type::alias("Meters", Type::Int);
    let (item, rust_name) =
        try_lower_simple_module_const_item("answer", &alias_int, &HirExpr::IntLiteral(42))
            .expect("alias int const should lower");
    assert_eq!(rust_name, "ANSWER");
    assert!(matches!(
        item,
        RustItem::Const {
            name,
            visibility: Visibility::Private,
            ..
        } if name == "ANSWER"
    ));
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
    assert!(try_lower_simple_module_helper_const_item(
        "answer",
        &alias_int,
        &HirExpr::IntLiteral(42),
    )
    .is_none());
}

#[test]
fn lowers_simple_module_string_const_item() {
    let (item, rust_name_call) = try_lower_simple_module_string_const_item(
        "greeting",
        &Type::Str,
        &HirExpr::StringLiteral("hi".to_string()),
    )
    .expect("simple string const should lower");
    assert_eq!(rust_name_call, "__const_greeting()");
    assert!(matches!(
        item,
        RustItem::Fn {
            name,
            visibility: Visibility::Private,
            ret: Some(RustType::String_),
            ..
        } if name == "__const_greeting"
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
    assert_eq!(rust_name_call, "__const_greeting()");
    assert!(matches!(
        item,
        RustItem::Fn {
            name,
            visibility: Visibility::Private,
            ret: Some(RustType::String_),
            ..
        } if name == "__const_greeting"
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
    assert_eq!(rust_name_call, "__const_greeting()");
    assert!(matches!(
        item,
        RustItem::Fn {
            name,
            visibility: Visibility::Private,
            ret: Some(RustType::String_),
            body,
            ..
        } if name == "__const_greeting"
            && matches!(
                body.first(),
                Some(RustStmt::Return(Some(RustExpr::MethodCall { receiver, method, .. })))
                    if matches!(receiver.as_ref(), RustExpr::Ident(n) if n == "msg") && method == "to_string"
            )
    ));
}

#[test]
fn does_not_lower_non_string_module_string_const_item() {
    assert!(try_lower_simple_module_string_const_item(
        "greeting",
        &Type::Int,
        &HirExpr::StringLiteral("hi".to_string()),
    )
    .is_none());
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
    assert_eq!(rust_name_call, "__const_greeting()");
    assert!(matches!(
        item,
        RustItem::Fn {
            name,
            visibility: Visibility::Private,
            ret: Some(RustType::String_),
            body,
            ..
        } if name == "__const_greeting"
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
    assert_eq!(rust_name_call, "__const_greeting()");
    assert!(matches!(
        item,
        RustItem::Fn {
            name,
            visibility: Visibility::Private,
            ret: Some(RustType::String_),
            ..
        } if name == "__const_greeting"
    ));
}

#[test]
fn does_not_lower_non_leaf_module_string_const_item() {
    assert!(try_lower_simple_module_string_const_item(
        "greeting",
        &Type::Str,
        &HirExpr::BinOp {
            left: Box::new(HirExpr::StringLiteral("a".to_string())),
            op: "+".to_string(),
            right: Box::new(HirExpr::StringLiteral("b".to_string())),
            ty: Type::Str,
        },
    )
    .is_none());
}

#[test]
fn lowers_simple_module_none_const_item() {
    let (item, rust_name_call) =
        try_lower_simple_module_none_const_item("nothing", &Type::None, &HirExpr::NoneLiteral)
            .expect("none const should lower");
    assert_eq!(rust_name_call, "__const_nothing()");
    assert!(matches!(
        item,
        RustItem::Fn {
            name,
            visibility: Visibility::Private,
            ret: Some(RustType::Unit),
            ..
        } if name == "__const_nothing"
    ));
}

#[test]
fn lowers_simple_module_alias_none_const_item() {
    let alias_none = Type::alias("Nothing", Type::None);
    let (item, rust_name_call) =
        try_lower_simple_module_none_const_item("nothing", &alias_none, &HirExpr::NoneLiteral)
            .expect("alias none const should lower");
    assert_eq!(rust_name_call, "__const_nothing()");
    assert!(matches!(
        item,
        RustItem::Fn {
            name,
            visibility: Visibility::Private,
            ret: Some(RustType::Unit),
            ..
        } if name == "__const_nothing"
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
    assert_eq!(rust_name_call, "__const_nothing()");
    assert!(matches!(
        item,
        RustItem::Fn {
            name,
            visibility: Visibility::Private,
            ret: Some(RustType::Unit),
            body,
            ..
        } if name == "__const_nothing"
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
    assert_eq!(rust_name_call, "__const_nothing()");
    assert!(matches!(
        item,
        RustItem::Fn {
            name,
            visibility: Visibility::Private,
            ret: Some(RustType::Unit),
            body,
            ..
        } if name == "__const_nothing"
            && matches!(body.first(), Some(RustStmt::Return(Some(RustExpr::Ident(n)))) if n == "n")
    ));
}

#[test]
fn dispatcher_lowers_alias_none_module_const_as_none_item() {
    let alias_none = Type::alias("Nothing", Type::None);
    let (item, rust_name_call) =
        try_lower_simple_module_constant_item("nothing", &alias_none, &HirExpr::NoneLiteral)
            .expect("dispatcher should lower alias none constant");
    assert_eq!(rust_name_call, "__const_nothing()");
    assert!(matches!(
        item,
        RustItem::Fn {
            name,
            visibility: Visibility::Private,
            ret: Some(RustType::Unit),
            ..
        } if name == "__const_nothing"
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
    assert_eq!(rust_name_call, "__const_nothing()");
    assert!(matches!(
        item,
        RustItem::Fn {
            name,
            visibility: Visibility::Private,
            ret: Some(RustType::Unit),
            body,
            ..
        } if name == "__const_nothing"
            && matches!(body.first(), Some(RustStmt::Return(Some(RustExpr::Ident(n)))) if n == "n")
    ));
}

#[test]
fn does_not_lower_non_none_module_none_const_item() {
    assert!(try_lower_simple_module_none_const_item(
        "nothing",
        &Type::None,
        &HirExpr::IntLiteral(0),
    )
    .is_none());
}

#[test]
fn does_not_lower_non_none_name_module_none_const_item() {
    assert!(try_lower_simple_module_none_const_item(
        "nothing",
        &Type::None,
        &HirExpr::Name {
            name: "x".to_string(),
            binding_id: None,
            ty: Type::Int,
        },
    )
    .is_none());
}

#[test]
fn does_not_lower_alias_none_module_helper_const_item() {
    let alias_none = Type::alias("Nothing", Type::None);
    assert!(try_lower_simple_module_helper_const_item(
        "nothing",
        &alias_none,
        &HirExpr::NoneLiteral
    )
    .is_none());
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
    assert_eq!(rust_name_call, "__const_nums()");
    assert!(matches!(
        item,
        RustItem::Fn {
            name,
            visibility: Visibility::Private,
            ret: Some(RustType::Vec(_)),
            ..
        } if name == "__const_nums"
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
    assert_eq!(rust_name_call, "__const_data()");
    assert!(matches!(
        item,
        RustItem::Fn {
            name,
            visibility: Visibility::Private,
            ret: Some(RustType::Vec(_)),
            body,
            ..
        } if name == "__const_data"
            && matches!(body.first(), Some(RustStmt::Return(Some(RustExpr::Ident(n)))) if n == "nums")
    ));
}

#[test]
fn does_not_lower_primitive_module_helper_const_item() {
    assert!(try_lower_simple_module_helper_const_item(
        "answer",
        &Type::Int,
        &HirExpr::IntLiteral(42),
    )
    .is_none());
}

#[test]
fn does_not_lower_alias_string_module_helper_const_item() {
    let alias_str = Type::alias("Message", Type::Str);
    assert!(try_lower_simple_module_helper_const_item(
        "greeting",
        &alias_str,
        &HirExpr::StringLiteral("hi".to_string()),
    )
    .is_none());
}

#[test]
fn does_not_lower_non_leaf_module_helper_const_item() {
    let ty = Type::List(Box::new(Type::Int));
    let value = HirExpr::ListLiteral {
        elements: vec![HirExpr::Call {
            func: "build".to_string(),
            args: vec![],
            ty: Type::Int,
        }],
        ty: ty.clone(),
    };
    assert!(try_lower_simple_module_helper_const_item("nums", &ty, &value).is_none());
}

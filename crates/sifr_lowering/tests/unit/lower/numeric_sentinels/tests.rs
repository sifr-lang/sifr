use crate::{HirDiagnostic, HirExpr, HirModule, HirStmt, lower_module};
use sifr_python_parser::parse_module;
use sifr_type_system::Type;

fn lower_source(source: &str) -> Result<HirModule, Vec<HirDiagnostic>> {
    let parsed = parse_module(source).expect("parse failed");
    lower_module(parsed.suite()).map(|result| result.module)
}

#[test]
fn test_regular_float_string_parse_remains_fallible() {
    let module =
        lower_source("def main():\n    parsed = float(\"3.14\")\n").expect("lowering failed");

    let HirStmt::Let { ty, value, .. } = &module.functions[0].body[0] else {
        panic!("expected parsed let");
    };
    let Type::Result(ok_ty, _) = ty else {
        panic!("expected fallible float parse type, got {ty:?}");
    };
    assert_eq!(ok_ty.as_ref(), &Type::Float);
    assert_eq!(value.ty(), ty);
}

#[test]
fn test_min_call_resolves_unannotated_infinity_sentinel_to_int() {
    let module = lower_source(
        "def main() -> int:\n    best = float(\"inf\")\n    best = min(best, 1)\n    return best\n",
    )
    .expect("min-based sentinel should lower");

    let HirStmt::Let { ty, value, .. } = &module.functions[0].body[0] else {
        panic!("expected sentinel initializer");
    };
    assert_eq!(ty, &Type::Int);
    assert_eq!(value.ty(), &Type::Int);
    assert!(matches!(value, HirExpr::IntLiteral(i64::MAX)));

    let HirStmt::Assign { value, .. } = &module.functions[0].body[1] else {
        panic!("expected min-based reassignment");
    };
    assert_eq!(value.ty(), &Type::Int);
}

#[test]
fn test_sentinel_comparison_branch_returns_int_after_resolution() {
    let module = lower_source(
            "def main() -> int:\n    best = float(\"inf\")\n    best = min(best, 4)\n    return best if best != float(\"inf\") else 0\n",
        )
        .expect("sentinel branch should lower");

    let HirStmt::Return {
        value: Some(HirExpr::IfExpr { ty, .. }),
    } = &module.functions[0].body[2]
    else {
        panic!("expected int-typed sentinel if-expression return");
    };
    assert_eq!(ty, &Type::Int);
}

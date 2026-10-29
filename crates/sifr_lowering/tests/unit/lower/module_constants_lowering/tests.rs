
use super::*;
use crate::{ExternalDefs, lower_module_sysroot_private_declaration_with_externals};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_parser::parse_module;

fn lower_private_declaration_constants(source: &str) -> Vec<(String, Type, HirExpr)> {
    let parsed = parse_module(source).expect("source should parse");
    lower_module_sysroot_private_declaration_with_externals(
        parsed.suite(),
        &ExternalDefs::default(),
    )
    .expect("private declaration should lower")
    .module
    .constants
}

#[test]
fn private_declarations_collect_annotated_scalar_module_constants() {
    let constants = lower_private_declaration_constants(
        "pi: float = 3.141592653589793\ninf: float = 1.0 / 0.0\nnan: float = 0.0 / 0.0\nneg_inf: float = -1.0 / 0.0\nfinite: float = 1.0 / 2.0\nflag: bool = True\n",
    );

    assert!(constants.iter().any(|(name, ty, value)| name == "pi"
        && ty == &Type::Float
        && matches!(value, HirExpr::FloatLiteral(_))));
    assert!(constants.iter().any(|(name, ty, value)| name == "inf"
            && ty == &Type::Float
            && matches!(value, HirExpr::FloatLiteral(value) if value.is_infinite() && value.is_sign_positive())));
    assert!(constants.iter().any(|(name, ty, value)| name == "nan"
            && ty == &Type::Float
            && matches!(value, HirExpr::FloatLiteral(value) if value.to_bits() == 0x7ff8_0000_0000_0000)));
    assert!(constants.iter().any(|(name, ty, value)| name == "neg_inf"
            && ty == &Type::Float
            && matches!(value, HirExpr::FloatLiteral(value) if value.is_infinite() && value.is_sign_negative())));
    assert!(constants.iter().any(|(name, ty, value)| name == "finite"
        && ty == &Type::Float
        && matches!(value, HirExpr::BinOp { .. })));
    assert!(constants.iter().any(|(name, ty, value)| name == "flag"
        && ty == &Type::Bool
        && matches!(value, HirExpr::BoolLiteral(true))));
}

#[test]
fn dx8_non_finite_nan_constants_have_portable_ieee_bits() {
    let constants = lower_private_declaration_constants(
        "positive: float = 0.0 / 0.0\nnegative: float = -0.0 / 0.0\nnegated: float = -(0.0 / 0.0)\n",
    );
    assert_eq!(constants.len(), 3);
    for (_, _, value) in constants {
        let HirExpr::FloatLiteral(value) = value else {
            panic!("non-finite constant should be folded");
        };
        assert_eq!(value.to_bits(), 0x7ff8_0000_0000_0000);
    }
}

#[test]
fn annotated_scalar_module_constant_type_mismatch_is_diagnostic() {
    let parsed = parse_module("bad: float = \"not float\"\n").expect("source should parse");
    let errors = match lower_module_sysroot_private_declaration_with_externals(
        parsed.suite(),
        &ExternalDefs::default(),
    ) {
        Ok(_) => panic!("mismatched constant should fail"),
        Err(errors) => errors,
    };

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.message == "type mismatch: expected 'float', got 'str'"
    }));
}

#[test]
fn private_declaration_scalar_module_constant_alias_is_diagnostic() {
    let parsed = parse_module("pi: float = 3.0\nalias: float = pi\n").expect("source should parse");
    let errors = match lower_module_sysroot_private_declaration_with_externals(
        parsed.suite(),
        &ExternalDefs::default(),
    ) {
        Ok(_) => panic!("unsupported private declaration constant should fail"),
        Err(errors) => errors,
    };

    assert!(errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::TYPE_UNSUPPORTED_EXPRESSION_FORM)
                && error.message
                    == "unsupported private declaration constant 'alias': initializer must be a literal, supported constant expression, or constructor call"
        }));
}

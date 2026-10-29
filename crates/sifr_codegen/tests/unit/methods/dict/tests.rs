use super::*;

#[test]
#[should_panic(expected = "affine dict.setdefault must be rejected during typed lowering")]
fn setdefault_affine_types_are_an_internal_invariant_violation() {
    let object = RustExpr::Ident("values".to_string());
    let key = RustExpr::Ident("key".to_string());
    let value = RustExpr::Ident("value".to_string());
    let affine = Type::PythonBuffer(Box::new(Type::FixedInt(sifr_type_system::FixedIntType::U8)));
    let _ = lower_setdefault(&object, &Type::Str, &affine, &[key, value], false);
}

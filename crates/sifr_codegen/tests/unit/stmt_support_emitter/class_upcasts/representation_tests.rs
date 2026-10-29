use super::*;

fn render_conversion(target: &Type, source: &Type) -> String {
    let emitter = RustEmitter::new();
    crate::render_expr(&emitter.consuming_value_conversion_for_ir(
        target,
        source,
        crate::RustExpr::Ident("value".to_string()),
    ))
}

#[test]
fn representation_conversion_flattens_nested_option_once() {
    let target = sifr_type_system::make_union(vec![Type::Str, Type::None]);
    let source = Type::Union(vec![target.clone(), Type::None]);
    let rendered = render_conversion(&target, &source);

    assert_eq!(rendered.matches(".flatten(").count(), 1, "{rendered}");
    assert_eq!(rendered.matches(".map(").count(), 0, "{rendered}");
    assert_eq!(rendered.matches(".unwrap_or(").count(), 0, "{rendered}");
}

#[test]
fn representation_conversion_maps_option_union_once() {
    let payload = sifr_type_system::make_union(vec![Type::Int, Type::Str]);
    let source = sifr_type_system::safe_optional_result(payload);
    let target =
        sifr_type_system::make_union(vec![Type::Bool, Type::Int, Type::Str, Type::None]);
    let rendered = render_conversion(&target, &source);

    assert_eq!(rendered.matches(".map(").count(), 1, "{rendered}");
    assert_eq!(rendered.matches(".unwrap_or(").count(), 1, "{rendered}");
}

#[test]
fn representation_conversion_materializes_nullable_union_once() {
    let target = sifr_type_system::make_union(vec![Type::Int, Type::Str, Type::None]);
    let source = sifr_type_system::safe_optional_result(target.clone());
    let rendered = render_conversion(&target, &source);

    assert_eq!(rendered.matches(".map(").count(), 0, "{rendered}");
    assert_eq!(rendered.matches(".unwrap_or(").count(), 1, "{rendered}");
}

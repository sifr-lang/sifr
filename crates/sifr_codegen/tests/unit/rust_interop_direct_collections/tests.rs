use super::*;
use crate::render_expr;

#[test]
fn borrowed_nested_list_values_are_cloned_into_bridge_maps() {
    let expr =
        hash_map_to_bridge_index_map_expr("values", &Type::List(Box::new(Type::Str)), true);
    let rendered = render_expr(&expr);

    assert!(rendered.starts_with("&values.iter()"));
    assert!(rendered.contains("__sifr_bridge_key_0.clone()"));
    assert!(rendered.contains("__sifr_bridge_value_0.iter()"));
    assert!(rendered.contains("__sifr_bridge_item_1.clone()"));
    assert!(rendered.contains("IndexMap<_, _>"));
}

#[test]
fn owned_integer_dict_values_convert_to_exact_bridge_ints() {
    let expr = hash_map_to_bridge_index_map_expr("values", &Type::Int, false);
    let rendered = render_expr(&expr);

    assert!(rendered.contains("values.into_iter()"));
    assert!(rendered.contains("SifrIntBridge::from(__sifr_bridge_value_0)"));
    assert!(rendered.contains("IndexMap<_, _>"));
}

#[test]
fn nested_integer_lists_convert_recursively_in_both_directions() {
    let item = Type::List(Box::new(Type::Int));
    let argument = render_expr(&hash_map_to_bridge_index_map_expr("values", &item, true));
    let returned = render_expr(&bridge_index_map_to_hash_map_expr(
        &RustExpr::Ident("values".to_string()),
        &item,
    ));

    assert!(argument.contains("__sifr_bridge_item_1"));
    assert!(argument.contains("SifrIntBridge::from(__sifr_bridge_item_1)"));
    assert!(returned.contains("__sifr_value_1.into_sifr_int()"));
}

#[test]
fn nested_dicts_convert_between_hash_map_and_index_map_recursively() {
    let item = Type::Dict(Box::new(Type::Str), Box::new(Type::Int));
    let argument = render_expr(&hash_map_to_bridge_index_map_expr("values", &item, false));
    let returned = render_expr(&bridge_index_map_to_hash_map_expr(
        &RustExpr::Ident("values".to_string()),
        &item,
    ));

    assert_eq!(argument.matches("IndexMap<_, _>").count(), 2);
    assert_eq!(returned.matches("HashMap<_, _>").count(), 2);
    assert!(returned.contains("into_sifr_int()"));
}

#[test]
fn top_level_lists_convert_nested_collections_and_exact_ints() {
    let dict_list = Type::List(Box::new(Type::Dict(
        Box::new(Type::Str),
        Box::new(Type::Str),
    )));
    let int_lists = Type::List(Box::new(Type::List(Box::new(Type::Int))));
    let values = RustExpr::Ident("values".to_string());

    let borrowed_dicts = render_expr(&sifr_composite_to_bridge_expr(&values, &dict_list, true));
    let borrowed_ints = render_expr(&sifr_composite_to_bridge_expr(&values, &int_lists, true));
    let returned_dicts = render_expr(&bridge_composite_to_sifr_expr(
        &RustExpr::Ident("values".to_string()),
        &dict_list,
    ));
    let returned_ints = render_expr(&bridge_composite_to_sifr_expr(
        &RustExpr::Ident("values".to_string()),
        &int_lists,
    ));

    assert!(borrowed_dicts.starts_with("&values.iter()"));
    assert!(borrowed_dicts.contains("IndexMap<_, _>"));
    assert!(borrowed_ints.contains("SifrIntBridge::from(__sifr_bridge_item_1)"));
    assert!(returned_dicts.contains("HashMap<_, _>"));
    assert!(returned_ints.contains("__sifr_value_1.into_sifr_int()"));
}

#[test]
fn options_recursively_convert_collection_payloads_without_outer_references() {
    let optional_list = Type::Union(vec![Type::List(Box::new(Type::Int)), Type::None]);
    let values = RustExpr::Ident("values".to_string());

    let argument = render_expr(&sifr_composite_to_bridge_expr(
        &values,
        &optional_list,
        true,
    ));
    let returned = render_expr(&bridge_composite_to_sifr_expr(
        &RustExpr::Ident("values".to_string()),
        &optional_list,
    ));

    assert!(argument.starts_with("values.as_ref().map("));
    assert!(!argument.starts_with('&'));
    assert!(argument.contains("SifrIntBridge::from(__sifr_bridge_item_1)"));
    assert!(returned.contains("__sifr_value_1.into_sifr_int()"));
}

#[test]
fn scalar_options_do_not_emit_identity_maps() {
    let optional_float = Type::Union(vec![Type::Float, Type::None]);
    let value = RustExpr::Ident("value".to_string());

    let owned_argument = render_expr(&sifr_composite_to_bridge_expr(
        &value,
        &optional_float,
        false,
    ));
    let borrowed_argument = render_expr(&sifr_composite_to_bridge_expr(
        &value,
        &optional_float,
        true,
    ));
    let returned = render_expr(&bridge_composite_to_sifr_expr(&value, &optional_float));

    assert_eq!(owned_argument, "value");
    assert_eq!(borrowed_argument, "value.as_ref()");
    assert_eq!(returned, "(value)");
}

#[test]
fn composite_root_identifiers_escape_rust_keywords() {
    let keyword = RustExpr::Ident("type".to_string());
    let dict = Type::Dict(Box::new(Type::Str), Box::new(Type::Str));

    let argument = render_expr(&sifr_composite_to_bridge_expr(&keyword, &dict, true));

    assert!(argument.starts_with("&r#type.iter()"));
}

use crate::lower::container_literal_specialization::type_contains_unknown_or_any;
use sifr_python_ast::Expr;
use sifr_type_system::Type;

pub(in crate::lower) fn empty_collection_literal_kind(expr: &Expr) -> Option<&'static str> {
    match expr {
        Expr::List(list) if list.elts.is_empty() => Some("list"),
        Expr::Dict(dict) if dict.items.is_empty() => Some("dict"),
        Expr::Set(set) if set.elts.is_empty() => Some("set"),
        Expr::Call(call)
            if call.arguments.args.is_empty() && call.arguments.keywords.is_empty() =>
        {
            let Expr::Name(name) = call.func.as_ref() else {
                return None;
            };
            (name.id.as_str() == "set").then_some("set")
        }
        Expr::Call(call)
            if call.arguments.args.is_empty() && call.arguments.keywords.is_empty() =>
        {
            let Expr::Attribute(attr) = call.func.as_ref() else {
                return None;
            };
            match (attr.value.as_ref(), attr.attr.as_str()) {
                (Expr::Name(module), "deque") if module.id.as_str() == "collections" => {
                    Some("deque")
                }
                _ => None,
            }
        }
        _ => None,
    }
}

pub(super) fn hint_matches_empty_collection_shape(value_expr: &Expr, hint: &Type) -> bool {
    let Some(kind) = empty_collection_literal_kind(value_expr) else {
        return false;
    };
    match (kind, hint.resolve_alias()) {
        ("list", Type::List(_)) => true,
        ("dict", Type::Dict(_, _)) => true,
        ("set", Type::Set(_)) => true,
        ("deque", Type::Class { name, .. }) => name == "deque",
        _ => false,
    }
}

pub(super) fn should_adopt_inferred_binding_hint(
    value_expr: &Expr,
    value_ty: &Type,
    hint: &Type,
    allow_empty_collection_hint: bool,
    allow_empty_plain_dict_hint: bool,
) -> bool {
    if !type_contains_unknown_or_any(value_ty) {
        return false;
    }
    let empty_collection_kind = empty_collection_literal_kind(value_expr);
    if empty_collection_kind.is_some() {
        return (allow_empty_collection_hint
            || (allow_empty_plain_dict_hint && empty_collection_kind == Some("dict")))
            && !type_contains_unknown_or_any(hint)
            && hint_matches_empty_collection_shape(value_expr, hint);
    }
    if value_ty.is_assignable_to(hint) {
        return true;
    }
    if type_contains_unknown_or_any(hint) {
        return false;
    }
    hint_matches_empty_collection_shape(value_expr, hint)
}

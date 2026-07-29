use crate::hir_nodes::HirExpr;
use sifr_type_system::Type;

fn contains_unresolved_list_literal(expr: &HirExpr) -> bool {
    let HirExpr::ListLiteral { elements, ty } = expr else {
        return false;
    };
    ty.contains_unknown_or_any() || elements.iter().any(contains_unresolved_list_literal)
}

fn has_exact_resolved_type(expr: &HirExpr, expected: &Type) -> bool {
    !expr.ty().contains_unknown_or_any() && expr.ty().resolve_alias() == expected.resolve_alias()
}

fn specialize_list_element(element: &HirExpr, expected: &Type) -> Option<HirExpr> {
    if let Some(specialized) = try_specialize_empty_list_literal(element, expected) {
        return Some(specialized);
    }
    has_exact_resolved_type(element, expected).then_some(element.clone())
}

fn try_specialize_empty_list_literal(expr: &HirExpr, expected: &Type) -> Option<HirExpr> {
    let HirExpr::ListLiteral { elements, ty } = expr else {
        return None;
    };
    let Type::List(expected_element) = expected.resolve_alias() else {
        return None;
    };
    if expected_element.contains_unknown_or_any() || !contains_unresolved_list_literal(expr) {
        return None;
    }

    if elements.is_empty() {
        let Type::List(actual_element) = ty.resolve_alias() else {
            return None;
        };
        if !actual_element.contains_unknown_or_any() {
            return None;
        }
        return Some(HirExpr::ListLiteral {
            elements: Vec::new(),
            ty: Type::List(expected_element.clone()),
        });
    }

    let specialized_elements = elements
        .iter()
        .map(|element| specialize_list_element(element, expected_element))
        .collect::<Option<Vec<_>>>()?;

    Some(HirExpr::ListLiteral {
        elements: specialized_elements,
        ty: Type::List(expected_element.clone()),
    })
}

pub(in crate::lower) fn specialize_empty_list_literal(expr: HirExpr, expected: &Type) -> HirExpr {
    try_specialize_empty_list_literal(&expr, expected).unwrap_or(expr)
}

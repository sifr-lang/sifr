use super::LowerCtx;
use super::class_type_collection::class_method_signature;
use super::class_type_helpers::option_member_type;
use super::protocol_diagnostics;
use sifr_type_system::{FunctionType, Type};
use std::collections::HashMap;

fn class_next_element_type(class_name: &str, methods: &[(String, FunctionType)]) -> Option<Type> {
    let next_ft = class_method_signature(methods, "__next__")?;
    if !next_ft.params.is_empty() {
        return None;
    }
    let elem = option_member_type(next_ft.return_type.as_ref())?;
    if matches!(elem.resolve_alias(), Type::Class { name, .. } if name == class_name) {
        return None;
    }
    Some(elem)
}

fn class_iter_element_type(class_name: &str, methods: &[(String, FunctionType)]) -> Option<Type> {
    let iter_ft = class_method_signature(methods, "__iter__")?;
    if !iter_ft.params.is_empty() {
        return None;
    }
    match iter_ft.return_type.resolve_alias() {
        Type::Iterator(elem) | Type::Iterable(elem) => Some(*elem.clone()),
        Type::Class { name, .. } if name == class_name => {
            class_next_element_type(class_name, methods)
        }
        _ => None,
    }
}

fn class_reversed_element_type(
    class_name: &str,
    methods: &[(String, FunctionType)],
) -> Option<Type> {
    let reversed_ft = class_method_signature(methods, "__reversed__")?;
    if !reversed_ft.params.is_empty() {
        return None;
    }
    match reversed_ft.return_type.resolve_alias() {
        Type::Iterator(elem) | Type::Iterable(elem) => Some(*elem.clone()),
        Type::Class { name, .. } if name == class_name => {
            class_next_element_type(class_name, methods)
        }
        _ => None,
    }
}

pub(super) fn validate_iteration_protocol_methods(
    class_name: &str,
    methods: &[(String, FunctionType)],
    method_ranges: &HashMap<String, ruff_text_size::TextRange>,
    class_range: ruff_text_size::TextRange,
    ctx: &mut LowerCtx,
) {
    if let Some(iter_ft) = class_method_signature(methods, "__iter__") {
        let range = method_ranges["__iter__"];
        if !iter_ft.params.is_empty() {
            protocol_diagnostics::iterator_invalid_parameter_signature(
                ctx,
                &format!("{class_name}.__iter__"),
                range,
            );
        } else if class_iter_element_type(class_name, methods).is_none() {
            protocol_diagnostics::iterator_invalid_return_signature(
                ctx,
                &format!("{class_name}.__iter__"),
                "'Iterator[T]' or 'Iterable[T]'",
                range,
            );
        }
    }

    if let Some(next_ft) = class_method_signature(methods, "__next__") {
        let range = method_ranges["__next__"];
        if !next_ft.params.is_empty() {
            protocol_diagnostics::iterator_invalid_parameter_signature(
                ctx,
                &format!("{class_name}.__next__"),
                range,
            );
        } else if class_next_element_type(class_name, methods).is_none() {
            protocol_diagnostics::iterator_invalid_return_signature(
                ctx,
                &format!("{class_name}.__next__"),
                "'T | None'",
                range,
            );
        }
    }

    if let Some(reversed_ft) = class_method_signature(methods, "__reversed__") {
        let range = method_ranges["__reversed__"];
        if !reversed_ft.params.is_empty() {
            protocol_diagnostics::iterator_invalid_parameter_signature(
                ctx,
                &format!("{class_name}.__reversed__"),
                range,
            );
        } else if class_reversed_element_type(class_name, methods).is_none() {
            protocol_diagnostics::iterator_invalid_return_signature(
                ctx,
                &format!("{class_name}.__reversed__"),
                "'Iterator[T]' or 'Iterable[T]'",
                range,
            );
        }
    }

    if let (Some(iter_elem), Some(next_elem)) = (
        class_iter_element_type(class_name, methods),
        class_next_element_type(class_name, methods),
    ) {
        if !next_elem.is_assignable_to(&iter_elem) || !iter_elem.is_assignable_to(&next_elem) {
            protocol_diagnostics::iterator_element_mismatch(
                ctx,
                class_name,
                "__iter__",
                iter_elem.display_name().as_str(),
                "__next__",
                next_elem.display_name().as_str(),
                class_range,
            );
        }
    }

    if let (Some(iter_elem), Some(reversed_elem)) = (
        class_iter_element_type(class_name, methods),
        class_reversed_element_type(class_name, methods),
    ) {
        if !reversed_elem.is_assignable_to(&iter_elem)
            || !iter_elem.is_assignable_to(&reversed_elem)
        {
            protocol_diagnostics::iterator_element_mismatch(
                ctx,
                class_name,
                "__iter__",
                iter_elem.display_name().as_str(),
                "__reversed__",
                reversed_elem.display_name().as_str(),
                class_range,
            );
        }
    }
}

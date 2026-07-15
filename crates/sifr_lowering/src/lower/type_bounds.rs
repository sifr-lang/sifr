use ruff_text_size::TextRange;
use sifr_diagnostics::DiagnosticCode;
use sifr_type_system::Type;

use super::{decode_typevar_constraint, encode_typevar_constraint, LowerCtx};

fn lookup_named_type(name: &str, ctx: &LowerCtx) -> Option<Type> {
    match name {
        "int" => Some(Type::Int),
        "float" => Some(Type::Float),
        "bool" => Some(Type::Bool),
        "str" => Some(Type::Str),
        "None" => Some(Type::None),
        "bigint" => Some(Type::BigInt),
        _ => ctx
            .scope
            .lookup_type_alias(name)
            .cloned()
            .or_else(|| ctx.class_types.get(name).cloned()),
    }
}

fn resolve_named_bound_type(name: &str, ctx: &LowerCtx) -> Option<Type> {
    lookup_named_type(name, ctx).filter(|ty| !matches!(ty, Type::Unknown))
}

fn is_builtin_bound(name: &str) -> bool {
    matches!(name, "Comparable" | "Addable" | "Hashable")
}

fn is_known_bound_name(name: &str, ctx: &LowerCtx) -> bool {
    is_builtin_bound(name) || resolve_named_bound_type(name, ctx).is_some()
}

fn current_owner_typevar_specs<'a>(ctx: &'a LowerCtx, tv_name: &str) -> Option<&'a [String]> {
    let owner = ctx.current_owner.as_ref()?;
    ctx.type_param_bounds
        .get(owner)?
        .get(tv_name)
        .map(Vec::as_slice)
}

fn typevar_satisfies_spec(tv_name: &str, target_spec: &str, ctx: &LowerCtx) -> bool {
    let Some(specs) = current_owner_typevar_specs(ctx, tv_name) else {
        return false;
    };

    // Constraint checks are exact membership checks, but only for resolvable types.
    if let Some(target_constraint) = decode_typevar_constraint(target_spec) {
        if resolve_named_bound_type(target_constraint, ctx).is_none() {
            return false;
        }
        return specs.iter().any(|spec| spec == target_spec);
    }

    // Bound checks must target a known built-in bound or a resolvable named type/protocol.
    if !is_known_bound_name(target_spec, ctx) {
        return false;
    }

    let mut constraint_names: Vec<&str> = Vec::new();
    for spec in specs {
        if let Some(constraint_name) = decode_typevar_constraint(spec) {
            constraint_names.push(constraint_name);
            continue;
        }

        if !is_known_bound_name(spec, ctx) {
            continue;
        }
        if spec == target_spec {
            return true;
        }

        // Built-in bounds only imply themselves, which is handled by exact match above.
        if is_builtin_bound(spec) {
            continue;
        }

        if let Some(source_bound_ty) = resolve_named_bound_type(spec, ctx) {
            if !matches!(source_bound_ty, Type::TypeVar(_))
                && type_satisfies_bound(&source_bound_ty, target_spec, ctx)
            {
                return true;
            }
        }
    }

    // Constrained TypeVars satisfy a bound only if every possible constrained type does.
    !constraint_names.is_empty()
        && constraint_names.iter().all(|constraint_name| {
            resolve_named_bound_type(constraint_name, ctx)
                .is_some_and(|constraint_ty| type_satisfies_bound(&constraint_ty, target_spec, ctx))
        })
}

fn type_satisfies_comparable_bound(ty: &Type, ctx: &LowerCtx) -> bool {
    match ty.resolve_alias() {
        Type::Int | Type::Float | Type::Str | Type::Bool | Type::BigInt => true,
        Type::Tuple(elements) => elements
            .iter()
            .all(|element| type_satisfies_bound(element, "Comparable", ctx)),
        _ => false,
    }
}

fn contains_declared_generic_class(
    ty: &Type,
    ctx: &LowerCtx,
    visiting: &mut std::collections::HashSet<String>,
) -> bool {
    match ty.resolve_alias() {
        Type::Class { name, fields, .. } => {
            if ctx
                .class_declared_type_params
                .get(name)
                .is_some_and(|params| !params.is_empty())
            {
                return true;
            }
            if !visiting.insert(name.clone()) {
                return false;
            }
            let contains = fields
                .iter()
                .any(|(_, field)| contains_declared_generic_class(field, ctx, visiting));
            visiting.remove(name);
            contains
        }
        Type::List(element)
        | Type::Set(element)
        | Type::Iterable(element)
        | Type::Iterator(element)
        | Type::Newtype { inner: element, .. } => {
            contains_declared_generic_class(element, ctx, visiting)
        }
        Type::Dict(key, value) | Type::Result(key, value) => {
            contains_declared_generic_class(key, ctx, visiting)
                || contains_declared_generic_class(value, ctx, visiting)
        }
        Type::Tuple(elements) | Type::Union(elements) => elements
            .iter()
            .any(|element| contains_declared_generic_class(element, ctx, visiting)),
        _ => false,
    }
}

pub(in crate::lower) fn supports_hash_key_in_context(ty: &Type, ctx: &LowerCtx) -> bool {
    if let Type::TypeVar(tv_name) = ty.resolve_alias() {
        return typevar_satisfies_spec(tv_name, "Hashable", ctx);
    }
    ty.supports_hash_key()
        && !contains_declared_generic_class(ty, ctx, &mut std::collections::HashSet::new())
}

pub(in crate::lower) fn reject_unavailable_hash_key(
    key_ty: &Type,
    operation: &str,
    range: TextRange,
    ctx: &mut LowerCtx,
) -> bool {
    if matches!(key_ty.resolve_alias(), Type::Any | Type::Unknown)
        || supports_hash_key_in_context(key_ty, ctx)
    {
        return false;
    }
    ctx.error_with_code_at(
        DiagnosticCode::TYPE_MISMATCH,
        format!(
            "{operation} requires a key type with generated Rust Eq + Hash traits, unavailable for '{}'",
            key_ty.display_name()
        ),
        range,
    );
    true
}

pub(in crate::lower) fn reject_unavailable_dict_hash_key(
    dict_ty: &Type,
    index_ty: &Type,
    operation: &str,
    range: TextRange,
    ctx: &mut LowerCtx,
) -> bool {
    let Type::Dict(key_ty, _) = dict_ty.resolve_alias() else {
        return false;
    };
    reject_unavailable_hash_key(key_ty, operation, range, ctx)
        || reject_unavailable_hash_key(index_ty, operation, range, ctx)
}

pub(in crate::lower) fn supports_structural_equality_in_context(ty: &Type, ctx: &LowerCtx) -> bool {
    supports_structural_equality_in_context_inner(ty, ctx, &mut std::collections::HashSet::new())
}

fn supports_structural_equality_in_context_inner(
    ty: &Type,
    ctx: &LowerCtx,
    visiting: &mut std::collections::HashSet<String>,
) -> bool {
    match ty.resolve_alias() {
        Type::List(element) | Type::Iterable(element) => {
            supports_structural_equality_in_context_inner(element, ctx, visiting)
        }
        Type::Set(element) => supports_hash_key_in_context(element, ctx),
        Type::Dict(key, value) => {
            supports_hash_key_in_context(key, ctx)
                && supports_structural_equality_in_context_inner(value, ctx, visiting)
        }
        Type::Result(ok, error) => {
            supports_structural_equality_in_context_inner(ok, ctx, visiting)
                && supports_structural_equality_in_context_inner(error, ctx, visiting)
        }
        Type::Tuple(elements) | Type::Union(elements) => elements
            .iter()
            .all(|element| supports_structural_equality_in_context_inner(element, ctx, visiting)),
        Type::Class {
            name,
            fields,
            methods,
            ..
        } => {
            if methods.iter().any(|(method, _)| method == "__eq__") {
                return true;
            }
            if !ty.supports_structural_equality() {
                return false;
            }
            if ctx
                .class_declared_type_params
                .get(name)
                .is_some_and(|params| !params.is_empty())
            {
                return true;
            }
            if !visiting.insert(name.clone()) {
                return true;
            }
            let supports = fields.iter().all(|(_, field)| {
                supports_structural_equality_in_context_inner(field, ctx, visiting)
            });
            visiting.remove(name);
            supports
        }
        Type::Newtype { inner, .. } => {
            supports_structural_equality_in_context_inner(inner, ctx, visiting)
        }
        _ => ty.supports_structural_equality(),
    }
}

/// Check if a type satisfies a named bound (hard requirement).
pub(in crate::lower) fn type_satisfies_bound(ty: &Type, bound: &str, ctx: &LowerCtx) -> bool {
    if let Type::TypeVar(tv_name) = ty {
        return typevar_satisfies_spec(tv_name, bound, ctx);
    }
    match bound {
        "Comparable" => type_satisfies_comparable_bound(ty, ctx),
        "Addable" => matches!(ty, Type::Int | Type::Float | Type::Str | Type::BigInt),
        "Hashable" => supports_hash_key_in_context(ty, ctx),
        _ => resolve_named_bound_type(bound, ctx)
            .is_some_and(|bound_ty| ty.is_assignable_to(&bound_ty)),
    }
}

/// Check if a type satisfies a `TypeVar` constraints entry (`TypeVar("T", A, B)` / `T: (A, B)`).
pub(in crate::lower) fn type_satisfies_constraint(
    ty: &Type,
    constraint_name: &str,
    ctx: &LowerCtx,
) -> bool {
    let encoded = encode_typevar_constraint(constraint_name);
    if let Type::TypeVar(tv_name) = ty {
        return typevar_satisfies_spec(tv_name, &encoded, ctx);
    }
    resolve_named_bound_type(constraint_name, ctx)
        .is_some_and(|target_ty| ty.is_assignable_to(&target_ty))
}

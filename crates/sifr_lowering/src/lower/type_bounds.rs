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
    matches!(
        name,
        "Comparable" | "Addable" | "Hashable" | "Structural" | "StaticProgram"
    )
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

fn supports_structural_bridge_type(ty: &Type, ctx: &LowerCtx) -> bool {
    supports_structural_bridge_type_inner(ty, ctx, &mut std::collections::HashSet::new(), false)
}

fn structural_identity_inputs_supported(class_name: &str, ctx: &LowerCtx) -> bool {
    if let Some(supported) = ctx.imported_structural_identity_inputs.get(class_name) {
        return *supported;
    }
    ctx.class_field_defaults
        .get(class_name)
        .into_iter()
        .flatten()
        .all(|(_, value)| sifr_ir::canonical_structural_identity_value(value).is_some())
        && ctx
            .declaration_metadata
            .iter()
            .filter(|metadata| metadata.owner == class_name)
            .all(|metadata| sifr_ir::canonical_structural_identity_value(&metadata.value).is_some())
}

fn supports_structural_bridge_type_inner(
    ty: &Type,
    ctx: &LowerCtx,
    visiting: &mut std::collections::HashSet<(String, Vec<Type>)>,
    direct_record_field: bool,
) -> bool {
    match ty.resolve_alias() {
        Type::Int | Type::Float | Type::Bool | Type::Str | Type::None => true,
        Type::FixedInt(value) => !matches!(
            value,
            sifr_type_system::FixedIntType::ISize | sifr_type_system::FixedIntType::USize
        ),
        Type::Bytes => direct_record_field,
        Type::TypeVar(name) => typevar_satisfies_spec(name, "Structural", ctx),
        Type::List(value) => supports_structural_bridge_type_inner(value, ctx, visiting, false),
        Type::Set(value) => {
            supports_hash_key_in_context(value, ctx)
                && supports_structural_bridge_type_inner(value, ctx, visiting, false)
        }
        Type::Dict(key, value) => {
            supports_hash_key_in_context(key, ctx)
                && supports_structural_bridge_type_inner(key, ctx, visiting, false)
                && supports_structural_bridge_type_inner(value, ctx, visiting, false)
        }
        Type::Tuple(values) => {
            values.len() <= 4
                && values
                    .iter()
                    .all(|value| supports_structural_bridge_type_inner(value, ctx, visiting, false))
        }
        Type::Union(values) => values
            .iter()
            .all(|value| supports_structural_bridge_type_inner(value, ctx, visiting, false)),
        Type::Enum { name, variants, .. } => {
            sifr_ir::structural_identity_enum_variants_supported(variants)
                && structural_identity_inputs_supported(name, ctx)
                && !ctx.error_types.contains(name)
                && !ctx.python_opaque_classes.contains_key(name)
                && !ctx.rust_opaque_classes.contains(name)
        }
        Type::Class {
            identity,
            type_args,
            name,
            fields,
            parent_class,
            ..
        } => {
            if parent_class.is_some()
                || ctx.error_types.contains(name)
                || ctx.python_opaque_classes.contains_key(name)
                || ctx.rust_opaque_classes.contains(name)
                || !structural_identity_inputs_supported(name, ctx)
            {
                return false;
            }
            let key = (
                identity.clone().unwrap_or_else(|| name.clone()),
                type_args.clone(),
            );
            if !visiting.insert(key.clone()) {
                return true;
            }
            let supported = type_args
                .iter()
                .all(|value| supports_structural_bridge_type_inner(value, ctx, visiting, false))
                && fields.iter().all(|(_, field)| {
                    supports_structural_bridge_type_inner(field, ctx, visiting, true)
                });
            visiting.remove(&key);
            supported
        }
        _ => false,
    }
}

fn supports_static_program_type(ty: &Type, ctx: &LowerCtx) -> bool {
    let Type::Class { name, .. } = ty.resolve_alias() else {
        return false;
    };
    ctx.specialization_requests
        .iter()
        .any(|request| request.owner == *name)
        && supports_structural_bridge_type(ty, ctx)
}

fn contains_declared_generic_class(
    ty: &Type,
    ctx: &LowerCtx,
    visiting: &mut std::collections::HashSet<(String, Vec<Type>)>,
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
            let Some(key) = ty.class_recursion_key() else {
                return false;
            };
            if !visiting.insert(key.clone()) {
                return false;
            }
            let contains = fields
                .iter()
                .any(|(_, field)| contains_declared_generic_class(field, ctx, visiting));
            visiting.remove(&key);
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

/// Whether the generated Rust representation implements the total `Ord` trait
/// required by `slice::sort`. This is intentionally narrower than Sifr's
/// `Comparable` bound, which also admits partial orders such as `float`.
pub(in crate::lower) fn supports_total_order_in_context(ty: &Type, _ctx: &LowerCtx) -> bool {
    supports_total_order(ty)
}

fn supports_total_order(ty: &Type) -> bool {
    match ty.resolve_alias() {
        Type::Int
        | Type::FixedInt(_)
        | Type::Bool
        | Type::Str
        | Type::Bytes
        | Type::None
        | Type::LiteralInt(_)
        | Type::LiteralStr(_)
        | Type::LiteralBool(_)
        | Type::BigInt
        | Type::Decimal
        | Type::BigDecimal => true,
        Type::List(element) => supports_total_order(element),
        Type::Tuple(elements) => elements.iter().all(supports_total_order),
        Type::TypeVar(_) => false,
        _ => false,
    }
}

/// Whether `print` can use the exact Display/Debug strategy selected by codegen.
pub(in crate::lower) fn supports_print_formatting(ty: &Type) -> bool {
    let resolved = ty.resolve_alias();
    if let Type::Union(members) = resolved {
        if members.len() == 2 && members.iter().any(|member| matches!(member, Type::None)) {
            return members
                .iter()
                .find(|member| !matches!(member, Type::None))
                .is_some_and(supports_print_formatting);
        }
    }
    match resolved {
        Type::List(_)
        | Type::Bytes
        | Type::Dict(_, _)
        | Type::Set(_)
        | Type::Tuple(_)
        | Type::Iterable(_)
        | Type::Iterator(_)
        | Type::Function(_)
        | Type::AsyncFunction(_)
        | Type::Coroutine(_, _)
        | Type::Task(_, _)
        | Type::TaskResult(_, _)
        | Type::Failure(_)
        | Type::TimeoutResult(_)
        | Type::Select2(_, _)
        | Type::BlockingTask(_, _)
        | Type::JoinSet(_, _)
        | Type::Awaitable(_)
        | Type::AsyncIterator(_, _)
        | Type::AsyncGenerator(_, _)
        | Type::PythonBuffer(_)
        | Type::PythonArrow(_)
        | Type::PythonDlpackTensor(_)
        | Type::PythonDlpackStream
        | Type::Callable(..)
        | Type::AsyncCallable(..)
        | Type::Result(_, _)
        | Type::Protocol { .. }
        | Type::Any
        | Type::Unknown
        | Type::Intersection(_)
        | Type::Never => ty.supports_debug_formatting(),
        Type::None => true,
        _ => ty.supports_display_formatting(),
    }
}

fn supports_structural_equality_in_context_inner(
    ty: &Type,
    ctx: &LowerCtx,
    visiting: &mut std::collections::HashSet<(String, Vec<Type>)>,
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
            let Some(key) = ty.class_recursion_key() else {
                return false;
            };
            if !visiting.insert(key.clone()) {
                return true;
            }
            let supports = fields.iter().all(|(_, field)| {
                supports_structural_equality_in_context_inner(field, ctx, visiting)
            });
            visiting.remove(&key);
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
        "Structural" => supports_structural_bridge_type(ty, ctx),
        "StaticProgram" => supports_static_program_type(ty, ctx),
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

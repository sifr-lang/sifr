use super::{
    LowerCtx,
    classes::{body_contains_receiver_mutation, fixed_trait_receiver_convention},
    method_receiver_diagnostics,
};
use crate::hir_nodes::{HirClass, HirClassKind, HirExpr, HirFunction, HirIteratorOp, MethodKind};
use crate::scope::BindingKind;
use ruff_text_size::TextRange;
use sifr_ir::visit_hir_function_exprs_mut;
use sifr_type_system::{FunctionType, ReceiverConvention, Type};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MethodKey {
    class: String,
    method: String,
}

struct FixedReceiverViolation {
    class_name: String,
    method: String,
    trait_name: &'static str,
    range: TextRange,
}

struct ProtocolReceiverMismatch {
    class_name: String,
    method: String,
    protocol: String,
    range: TextRange,
}

pub(super) fn validate_and_annotate_class_receivers(classes: &mut [HirClass], ctx: &mut LowerCtx) {
    let mut mutable = HashSet::new();
    let mut dependencies: HashMap<MethodKey, HashSet<MethodKey>> = HashMap::new();
    let mut fixed_receivers = HashMap::new();

    for class in classes.iter_mut() {
        for method in &mut class.methods {
            collect_method_facts(
                &class.name,
                method,
                ctx,
                &mut mutable,
                &mut dependencies,
                &mut fixed_receivers,
            );
        }
        for (_, method) in &mut class.operator_impls {
            collect_method_facts(
                &class.name,
                method,
                ctx,
                &mut mutable,
                &mut dependencies,
                &mut fixed_receivers,
            );
        }
    }

    loop {
        let newly_mutable: Vec<_> = dependencies
            .iter()
            .filter(|(caller, callees)| {
                !mutable.contains(*caller) && callees.iter().any(|callee| mutable.contains(callee))
            })
            .map(|(caller, _)| caller.clone())
            .collect();
        if newly_mutable.is_empty() {
            break;
        }
        mutable.extend(newly_mutable);
    }
    validate_fixed_receiver_conventions(&fixed_receivers, &mutable, ctx);

    for class in classes.iter_mut() {
        for method in &mut class.methods {
            let key = MethodKey {
                class: class.name.clone(),
                method: method.name.clone(),
            };
            if mutable.contains(&key)
                && method.method_kind == MethodKind::Regular
                && method
                    .receiver
                    .is_some_and(|receiver| !receiver.is_mutable())
                && fixed_trait_receiver_convention(&method.name).is_none()
            {
                let range = ctx
                    .method_source_ranges
                    .get(&format!("{}.{}", class.name, method.name))
                    .copied()
                    .unwrap_or_default();
                super::ownership_diagnostics::immutable_parameter_mutation(ctx, "self", range);
                if let Some(binding_id) = ctx
                    .method_receiver_bindings
                    .get(&format!("{}.{}", class.name, method.name))
                    .copied()
                {
                    ctx.scope.patch_receiver_mutability_for_recovery(binding_id);
                }
            }
            persist_method_receiver(&class.name, method, ctx);
        }
        for (_, method) in &class.operator_impls {
            persist_method_receiver(&class.name, method, ctx);
        }
    }
    propagate_inherited_receiver_metadata(ctx);
    validate_protocol_receiver_conventions(classes, ctx);
    refresh_protocol_implementations(classes, ctx);

    let final_types = ctx.class_types.clone();
    let final_functions = ctx.functions.clone();
    for class in classes {
        for method in class
            .methods
            .iter_mut()
            .chain(class.operator_impls.iter_mut().map(|(_, method)| method))
        {
            annotate_function_calls(method, &final_types, &final_functions);
        }
    }
}

fn refresh_protocol_implementations(classes: &mut [HirClass], ctx: &LowerCtx) {
    let protocols = ctx
        .class_types
        .iter()
        .filter(|(_, ty)| matches!(ty.resolve_alias(), Type::Protocol { .. }))
        .map(|(name, ty)| (name.clone(), ty.clone()))
        .collect::<Vec<_>>();

    for class in classes {
        if matches!(class.kind, HirClassKind::Protocol) {
            class.implements_protocols.clear();
            continue;
        }
        let Some(class_ty) = ctx.class_types.get(&class.name) else {
            continue;
        };
        class.implements_protocols = protocols
            .iter()
            .filter(|(_, protocol)| class_ty.is_assignable_to(protocol))
            .map(|(name, _)| name.clone())
            .collect();
    }
}

fn collect_method_facts(
    class_name: &str,
    method: &mut HirFunction,
    ctx: &mut LowerCtx,
    mutable: &mut HashSet<MethodKey>,
    dependencies: &mut HashMap<MethodKey, HashSet<MethodKey>>,
    fixed_receivers: &mut HashMap<MethodKey, (ReceiverConvention, Option<ReceiverConvention>)>,
) {
    let mut calls = HashSet::new();
    let mut call_mutates_receiver = false;
    visit_hir_function_exprs_mut(method, &mut |expr| match expr {
        HirExpr::MethodCall {
            object,
            method,
            args,
            receiver_convention,
            ..
        } => {
            if receiver_rooted(object, ctx) {
                if *receiver_convention == Some(ReceiverConvention::MutableBorrow) {
                    call_mutates_receiver = true;
                }
                if let Some(target) = method_key_for_type(object.ty(), method, ctx) {
                    calls.insert(target);
                }
            }
            if method_signature(object.ty(), method, &ctx.class_types, &ctx.functions).is_some_and(
                |signature| {
                    args.iter()
                        .zip(&signature.params)
                        .any(|(arg, (_, _, convention))| {
                            convention.is_mut_borrow() && receiver_rooted(arg, ctx)
                        })
                },
            ) {
                call_mutates_receiver = true;
            }
        }
        HirExpr::Call {
            func,
            args,
            mutable_arg_places,
            ..
        } => {
            if func == "anext" && args.first().is_some_and(|arg| receiver_rooted(arg, ctx)) {
                call_mutates_receiver = true;
            }
            if args
                .iter()
                .zip(mutable_arg_places)
                .any(|(arg, target)| target.is_some() && receiver_rooted(arg, ctx))
            {
                call_mutates_receiver = true;
            }
        }
        HirExpr::IteratorCall {
            op: HirIteratorOp::Next,
            args,
            ..
        } if args.first().is_some_and(|arg| receiver_rooted(arg, ctx)) => {
            call_mutates_receiver = true;
        }
        _ => {}
    });
    let directly_mutates_receiver = body_contains_receiver_mutation(&method.body);
    let key = MethodKey {
        class: class_name.to_string(),
        method: method.name.clone(),
    };
    if let Some(required) = fixed_trait_receiver_convention(&method.name) {
        fixed_receivers.insert(key.clone(), (required, method.receiver));
    } else if method.method_kind != MethodKind::Regular {
        return;
    }
    if method.receiver.is_some_and(ReceiverConvention::is_mutable)
        || directly_mutates_receiver
        || call_mutates_receiver
    {
        mutable.insert(key.clone());
    }
    dependencies.insert(key, calls);
}

fn validate_fixed_receiver_conventions(
    fixed_receivers: &HashMap<MethodKey, (ReceiverConvention, Option<ReceiverConvention>)>,
    mutable: &HashSet<MethodKey>,
    ctx: &mut LowerCtx,
) {
    let mut violations = fixed_receivers
        .iter()
        .filter(|(key, (required, declared))| {
            mutable.contains(*key) || *declared != Some(*required)
        })
        .map(|(key, _)| FixedReceiverViolation {
            class_name: key.class.clone(),
            method: key.method.clone(),
            trait_name: fixed_trait_name(&key.method),
            range: ctx
                .method_source_ranges
                .get(&format!("{}.{}", key.class, key.method))
                .copied()
                .unwrap_or_default(),
        })
        .collect::<Vec<_>>();
    violations.sort_by(|left, right| {
        left.range
            .start()
            .cmp(&right.range.start())
            .then_with(|| left.class_name.cmp(&right.class_name))
            .then_with(|| left.method.cmp(&right.method))
    });

    for violation in violations {
        method_receiver_diagnostics::fixed_receiver_violation(
            ctx,
            &violation.class_name,
            &violation.method,
            violation.trait_name,
            violation.range,
        );
    }
}

fn fixed_trait_name(method: &str) -> &'static str {
    match method {
        "__eq__" => "PartialEq",
        "__lt__" => "PartialOrd",
        "__add__" | "__sub__" | "__mul__" | "__truediv__" | "__mod__" | "__neg__" => {
            "operator trait"
        }
        "__str__" | "__repr__" => "Display",
        "__getitem__" => "Index",
        _ => "Rust trait",
    }
}

fn validate_protocol_receiver_conventions(classes: &[HirClass], ctx: &mut LowerCtx) {
    let mut mismatches = Vec::new();
    for class in classes {
        if matches!(class.kind, HirClassKind::Protocol) {
            continue;
        }
        for protocol_name in &class.implements_protocols {
            let Some(Type::Protocol {
                methods: protocol_methods,
                ..
            }) = ctx.class_types.get(protocol_name).map(Type::resolve_alias)
            else {
                continue;
            };
            for method in class
                .methods
                .iter()
                .chain(class.operator_impls.iter().map(|(_, method)| method))
            {
                let Some((_, protocol_signature)) = protocol_methods
                    .iter()
                    .find(|(candidate, _)| candidate == &method.name)
                else {
                    continue;
                };
                if method.receiver == Some(ReceiverConvention::MutableBorrow)
                    && protocol_signature.receiver == Some(ReceiverConvention::SharedBorrow)
                {
                    let range = ctx
                        .method_source_ranges
                        .get(&format!("{}.{}", class.name, method.name))
                        .copied()
                        .unwrap_or_default();
                    mismatches.push(ProtocolReceiverMismatch {
                        class_name: class.name.clone(),
                        method: method.name.clone(),
                        protocol: protocol_name.clone(),
                        range,
                    });
                }
            }
        }
    }
    for mismatch in mismatches {
        method_receiver_diagnostics::protocol_receiver_convention_mismatch(
            ctx,
            &mismatch.class_name,
            &mismatch.method,
            &mismatch.protocol,
            mismatch.range,
        );
    }
}

fn method_key_for_type(ty: &Type, method: &str, ctx: &LowerCtx) -> Option<MethodKey> {
    let class = nominal_class_name(ty)?;
    let qualified = format!("{class}.{method}");
    let origin = ctx
        .class_method_origins
        .get(&qualified)
        .map_or(class, String::as_str);
    Some(MethodKey {
        class: origin.to_string(),
        method: method.to_string(),
    })
}

fn persist_method_receiver(class_name: &str, method: &HirFunction, ctx: &mut LowerCtx) {
    let Some(receiver) = method.receiver else {
        return;
    };
    let qualified = format!("{class_name}.{}", method.name);
    if let Some(signature) = ctx.functions.get_mut(&qualified) {
        signature.receiver = Some(receiver);
    }
    if let Some(Type::Class { methods, .. } | Type::Protocol { methods, .. }) =
        ctx.class_types.get_mut(class_name)
    {
        if let Some((_, signature)) = methods
            .iter_mut()
            .find(|(candidate, _)| candidate == &method.name)
        {
            signature.receiver = Some(receiver);
        }
    }
    if let Some(binding_id) = ctx.method_receiver_bindings.get(&qualified).copied() {
        ctx.scope.patch_receiver_convention(binding_id, receiver);
    }
}

fn propagate_inherited_receiver_metadata(ctx: &mut LowerCtx) {
    let updates: Vec<_> = ctx
        .class_types
        .iter()
        .flat_map(|(class_name, class_ty)| {
            let Type::Class { methods, .. } = class_ty else {
                return Vec::new();
            };
            methods
                .iter()
                .filter_map(|(method, _)| {
                    let qualified = format!("{class_name}.{method}");
                    let origin = ctx
                        .class_method_origins
                        .get(&qualified)
                        .map_or(class_name.as_str(), String::as_str);
                    ctx.functions
                        .get(&format!("{origin}.{method}"))
                        .and_then(|signature| signature.receiver)
                        .map(|receiver| (class_name.clone(), method.clone(), receiver))
                })
                .collect::<Vec<_>>()
        })
        .collect();

    for (class_name, method, receiver) in updates {
        if let Some(Type::Class { methods, .. }) = ctx.class_types.get_mut(&class_name) {
            if let Some((_, signature)) = methods
                .iter_mut()
                .find(|(candidate, _)| candidate == &method)
            {
                signature.receiver = Some(receiver);
            }
        }
        if let Some(signature) = ctx.functions.get_mut(&format!("{class_name}.{method}")) {
            signature.receiver = Some(receiver);
        }
    }
}

fn annotate_function_calls(
    function: &mut HirFunction,
    class_types: &HashMap<String, Type>,
    functions: &HashMap<String, FunctionType>,
) {
    visit_hir_function_exprs_mut(function, &mut |expr| {
        let HirExpr::MethodCall {
            object,
            method,
            receiver_convention,
            ..
        } = expr
        else {
            return;
        };
        if let Some(signature) = method_signature(object.ty(), method, class_types, functions) {
            if let Some(resolved) = signature.receiver {
                *receiver_convention = Some(resolved);
            }
        }
    });
}

pub(super) fn method_signature(
    ty: &Type,
    method: &str,
    class_types: &HashMap<String, Type>,
    functions: &HashMap<String, FunctionType>,
) -> Option<FunctionType> {
    match ty.resolve_alias() {
        Type::Class {
            identity,
            name,
            methods,
            ..
        } => resolved_nominal_method_signature(
            identity.as_deref(),
            name,
            methods,
            method,
            class_types,
        ),
        Type::Protocol {
            identity,
            name,
            methods,
        } => resolved_nominal_method_signature(
            identity.as_deref(),
            name,
            methods,
            method,
            class_types,
        ),
        Type::Enum { name, .. } => functions.get(&format!("{name}.{method}")).cloned(),
        Type::Alias { body, .. } => method_signature(body, method, class_types, functions),
        Type::TypeVar(name) => class_types
            .get(name)
            .and_then(|bound| method_signature(bound, method, class_types, functions)),
        _ => None,
    }
}

fn resolved_nominal_method_signature(
    identity: Option<&str>,
    name: &str,
    methods: &[(String, FunctionType)],
    method: &str,
    class_types: &HashMap<String, Type>,
) -> Option<FunctionType> {
    let mut signature = methods
        .iter()
        .find_map(|(candidate, signature)| (candidate == method).then(|| signature.clone()))?;
    let canonical = class_types.get(name).or_else(|| {
        let identity = identity?;
        class_types.values().find(|candidate| {
            matches!(
                candidate.resolve_alias(),
                Type::Class {
                    identity: Some(candidate_identity),
                    ..
                } if candidate_identity == identity
            )
        })
    });
    if let Some(
        Type::Class {
            methods: canonical_methods,
            ..
        }
        | Type::Protocol {
            methods: canonical_methods,
            ..
        },
    ) = canonical.map(Type::resolve_alias)
    {
        if let Some(receiver) = canonical_methods.iter().find_map(|(candidate, canonical)| {
            (candidate == method)
                .then_some(canonical.receiver)
                .flatten()
        }) {
            signature.receiver = Some(receiver);
        }
    }
    Some(signature)
}

fn nominal_class_name(ty: &Type) -> Option<&str> {
    match ty.resolve_alias() {
        Type::Class { name, .. } | Type::Protocol { name, .. } | Type::Enum { name, .. } => {
            Some(name)
        }
        _ => None,
    }
}

fn receiver_rooted(expr: &HirExpr, ctx: &LowerCtx) -> bool {
    match expr {
        HirExpr::Name {
            binding_id: Some(id),
            ..
        } => ctx
            .scope
            .retained_binding(*id)
            .is_some_and(|fact| fact.binding_kind == BindingKind::Receiver),
        HirExpr::FieldAccess { object, .. }
        | HirExpr::Index { object, .. }
        | HirExpr::Slice { object, .. } => receiver_rooted(object, ctx),
        _ => false,
    }
}

pub(super) fn annotate_and_verify_function_calls(function: &mut HirFunction, ctx: &LowerCtx) {
    annotate_function_calls(function, &ctx.class_types, &ctx.functions);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_trait_receiver_registry_is_explicit() {
        assert_eq!(
            fixed_trait_receiver_convention("__eq__"),
            Some(ReceiverConvention::SharedBorrow)
        );
        assert_eq!(
            fixed_trait_receiver_convention("__add__"),
            Some(ReceiverConvention::Owned)
        );
        assert_eq!(fixed_trait_receiver_convention("bump"), None);
    }
}

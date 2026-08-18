//! Canonical package declarations for typed class-adapter descriptors.

use super::{str, Expr, HirModule, LowerCtx, Ranged, Stmt, Type};
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::{
    ClassAdapterMarkerDeclaration, ClassAdapterProviderDeclaration, ClassAdapterSelection,
    DeclarationDescriptorFunction, DeclarationDescriptorKind,
};
use sifr_python_ast::{Decorator, StmtClassDef, StmtFunctionDef};
use std::collections::HashSet;

pub(in crate::lower) fn prepare(stmts: &[Stmt], ctx: &mut LowerCtx) {
    import_bindings(stmts, ctx);
    for stmt in stmts {
        match stmt {
            Stmt::FunctionDef(function) => collect_local_declarations(function, ctx),
            Stmt::ClassDef(class) => collect_local_marker(class, ctx),
            _ => {}
        }
    }
    collect_class_bases(stmts, ctx);
}

pub(in crate::lower) fn data_parent_name(class_name: &str, ctx: &LowerCtx) -> Option<String> {
    ctx.class_data_parents.get(class_name).cloned().flatten()
}

pub(in crate::lower) fn erase_markers(module: &mut HirModule, ctx: &LowerCtx) {
    let erased = ctx
        .class_adapter_markers
        .iter()
        .map(|marker| marker.symbol.as_str())
        .collect::<HashSet<_>>();
    module
        .classes
        .retain(|class| !erased.contains(class.name.as_str()));
}

pub(in crate::lower) fn descriptor_kind_for_call(
    expression: &Expr,
    ctx: &LowerCtx,
) -> Option<DeclarationDescriptorKind> {
    let Expr::Call(call) = expression else {
        return None;
    };
    let Expr::Name(name) = call.func.as_ref() else {
        return None;
    };
    ctx.descriptor_bindings
        .get(name.id.as_str())
        .map(|declaration| declaration.kind)
}

pub(in crate::lower) fn finalize(ctx: &mut LowerCtx) {
    let module = ctx.current_module_name.clone().unwrap_or_default();
    for index in 0..ctx.class_adapter_providers.len() {
        let declaration = ctx.class_adapter_providers[index].clone();
        let Some(descriptor_type) = resolve_declared_type(
            &declaration.descriptor_module,
            &declaration.descriptor_symbol,
            ctx,
        ) else {
            malformed(
                ctx,
                "adapter_provider_type",
                "class-adapter provider descriptor type does not resolve to a class",
                declaration.range,
            );
            continue;
        };
        if !closed_structural_type(&descriptor_type, &mut HashSet::new()) {
            malformed(
                ctx,
                "adapter_provider_type",
                "class-adapter provider descriptor type must be closed and structural",
                declaration.range,
            );
            continue;
        }
        let Some(function_type) = ctx.functions.get(&declaration.function).cloned() else {
            malformed(
                ctx,
                "adapter_provider_signature",
                "class-adapter provider function signature is unavailable",
                declaration.range,
            );
            continue;
        };
        if !provider_signature_matches(&function_type, &descriptor_type) {
            malformed(
                ctx,
                "adapter_provider_signature",
                "@class_adapter_provider requires (DeclarationInput[D]) -> DeclarationPlan[D] for its declared descriptor type D",
                declaration.range,
            );
            continue;
        }
        ctx.class_adapter_providers[index].descriptor_type = descriptor_type;
    }

    for index in 0..ctx.descriptor_functions.len() {
        let mut declaration = ctx.descriptor_functions[index].clone();
        let provider = local_provider(ctx, &module, &declaration)
            .or_else(|| external_provider(ctx, &declaration))
            .cloned();
        let Some(provider) = provider else {
            malformed(
                ctx,
                "descriptor_provider",
                "descriptor function references an unknown canonical class-adapter provider",
                declaration.range,
            );
            continue;
        };
        let Some(function_type) = ctx.functions.get(&declaration.function) else {
            malformed(
                ctx,
                "descriptor_signature",
                "descriptor function signature is unavailable",
                declaration.range,
            );
            continue;
        };
        if function_type
            .params
            .iter()
            .any(|(_, ty, _)| !descriptor_parameter_type(ty))
        {
            malformed(
                ctx,
                "descriptor_parameter_type",
                "descriptor function parameters must use closed const types or CallableIdentity",
                declaration.range,
            );
            continue;
        }
        if !descriptor_return_assignable(&function_type.return_type, &provider.descriptor_type) {
            malformed(
                ctx,
                "descriptor_return_type",
                "descriptor function return type is not assignable to its provider descriptor type",
                declaration.range,
            );
            continue;
        }
        declaration.descriptor_type = provider.descriptor_type;
        declaration.return_type = (*function_type.return_type).clone();
        ctx.descriptor_functions[index] = declaration.clone();
        ctx.descriptor_bindings
            .insert(declaration.function.clone(), declaration);
    }

    finalize_markers_and_selections(ctx, &module);
}

fn descriptor_return_assignable(source: &Type, target: &Type) -> bool {
    match source.resolve_alias() {
        Type::Any | Type::Unknown | Type::Never | Type::TypeVar(_) => false,
        Type::Union(members) => {
            !members.is_empty()
                && members
                    .iter()
                    .all(|member| descriptor_return_assignable(member, target))
        }
        source => source.is_assignable_to(target),
    }
}

fn descriptor_parameter_type(ty: &Type) -> bool {
    match ty.resolve_alias() {
        Type::Callable(params, _, result) => {
            params.iter().all(descriptor_parameter_type) && descriptor_parameter_type(result)
        }
        Type::Union(members) => {
            !members.is_empty() && members.iter().all(descriptor_parameter_type)
        }
        ty => closed_structural_type(ty, &mut HashSet::new()),
    }
}

fn import_bindings(stmts: &[Stmt], ctx: &mut LowerCtx) {
    for stmt in stmts {
        let Stmt::ImportFrom(import) = stmt else {
            continue;
        };
        let Some(module) = &import.module else {
            continue;
        };
        let module =
            ctx.effective_import_module_name(module.as_ref(), import.level, &ctx.externals);
        for alias in &import.names {
            let original = alias.name.to_string();
            let local = alias
                .asname
                .as_ref()
                .map_or_else(|| original.clone(), ToString::to_string);
            if let Some(declaration) = ctx
                .externals
                .descriptor_functions
                .get(&module)
                .and_then(|exports| exports.get(&original))
            {
                ctx.descriptor_bindings
                    .insert(local.clone(), declaration.clone());
            }
            if let Some(marker) = ctx
                .externals
                .class_adapter_markers
                .get(&module)
                .and_then(|exports| exports.get(&original))
            {
                ctx.adapter_marker_bindings
                    .insert(local.clone(), marker.clone());
            }
            if let Some(selection) = ctx
                .externals
                .class_adapter_selections
                .get(&module)
                .and_then(|exports| exports.get(&original))
            {
                ctx.adapted_class_bindings
                    .insert(local.clone(), selection.clone());
            }
        }
    }
}

fn collect_local_marker(class: &StmtClassDef, ctx: &mut LowerCtx) {
    let Some(decorator) = class.decorator_list.iter().find_map(|decorator| {
        declaration_decorator(decorator, ctx)
            .filter(|(name, _, _)| name == "class_adapter_marker")
            .map(|(_, module, function)| (decorator.expression.range(), module, function))
    }) else {
        return;
    };
    let valid_body = class
        .body
        .iter()
        .all(|statement| matches!(statement, Stmt::Pass(_)));
    if !class.bases().is_empty() || !valid_body {
        malformed(
            ctx,
            "adapter_marker_shape",
            "@class_adapter_marker requires a field-less class containing only pass",
            class.range(),
        );
    }
    let declaration = ClassAdapterMarkerDeclaration {
        module: ctx.current_module_name.clone().unwrap_or_default(),
        symbol: class.name.to_string(),
        provider_module: decorator.1,
        provider_function: decorator.2,
        descriptor_type: Type::Any,
        range: decorator.0,
    };
    ctx.adapter_marker_bindings
        .insert(class.name.to_string(), declaration.clone());
    ctx.class_adapter_markers.push(declaration);
}

fn collect_class_bases(stmts: &[Stmt], ctx: &mut LowerCtx) {
    for statement in stmts {
        let Stmt::ClassDef(class) = statement else {
            continue;
        };
        if ctx
            .adapter_marker_bindings
            .contains_key(class.name.as_str())
        {
            ctx.class_data_parents.insert(class.name.to_string(), None);
            continue;
        }
        let mut markers = Vec::new();
        let mut data_parents = Vec::new();
        for base in class.bases() {
            let Expr::Name(name) = base else {
                continue;
            };
            if let Some(marker) = ctx.adapter_marker_bindings.get(name.id.as_str()) {
                markers.push((name.range(), marker.clone()));
                continue;
            }
            if special_base(name.id.as_str()) {
                continue;
            }
            data_parents.push((name.range(), name.id.to_string()));
        }
        // Preserve the language's existing first-parent behavior for ordinary
        // classes. The one-data-parent restriction belongs to adapted classes.
        let data_parent = data_parents.first().map(|(_, name)| name.clone());
        ctx.class_data_parents
            .insert(class.name.to_string(), data_parent.clone());
        let inherited = data_parent
            .as_ref()
            .and_then(|parent| ctx.adapted_class_bindings.get(parent))
            .cloned();
        if markers.is_empty() && inherited.is_none() {
            continue;
        }
        for (range, _) in data_parents.iter().skip(1) {
            malformed(
                ctx,
                "adapter_data_parent",
                "an adapted class permits at most one ordinary data parent",
                *range,
            );
        }
        let selected_provider = markers
            .first()
            .map(|(_, marker)| {
                (
                    marker.provider_module.clone(),
                    marker.provider_function.clone(),
                    marker.descriptor_type.clone(),
                )
            })
            .or_else(|| {
                inherited.as_ref().map(|selection| {
                    (
                        selection.provider_module.clone(),
                        selection.provider_function.clone(),
                        selection.descriptor_type.clone(),
                    )
                })
            });
        let Some(selected_provider) = selected_provider else {
            continue;
        };
        for (range, marker) in markers.iter().skip(1) {
            if (
                marker.provider_module.as_str(),
                marker.provider_function.as_str(),
            ) != (selected_provider.0.as_str(), selected_provider.1.as_str())
            {
                malformed(
                    ctx,
                    "adapter_provider_conflict",
                    "class bases select conflicting canonical adapter providers",
                    *range,
                );
            }
        }
        if let Some(parent) = &inherited {
            if (
                parent.provider_module.as_str(),
                parent.provider_function.as_str(),
            ) != (selected_provider.0.as_str(), selected_provider.1.as_str())
            {
                malformed(
                    ctx,
                    "adapter_parent_provider",
                    "adapted data parent and marker must select the same canonical provider",
                    class.range(),
                );
            }
        }
        ctx.class_adapter_selections.push(ClassAdapterSelection {
            owner: class.name.to_string(),
            provider_module: selected_provider.0,
            provider_function: selected_provider.1,
            descriptor_type: selected_provider.2,
            marker_identities: markers
                .iter()
                .map(|(_, marker)| format!("{}.{}", marker.module, marker.symbol))
                .chain(
                    inherited
                        .iter()
                        .flat_map(|selection| selection.marker_identities.iter().cloned()),
                )
                .collect(),
            data_parent,
            range: markers
                .first()
                .map_or_else(|| class.range(), |(range, _)| *range),
        });
        if let Some(selection) = ctx.class_adapter_selections.last().cloned() {
            ctx.adapted_class_bindings
                .insert(class.name.to_string(), selection);
        }
    }
}

fn special_base(name: &str) -> bool {
    matches!(
        name,
        "Error" | "Protocol" | "int" | "float" | "str" | "bool" | "Enum"
    )
}

fn collect_local_declarations(function: &StmtFunctionDef, ctx: &mut LowerCtx) {
    let module = ctx.current_module_name.clone().unwrap_or_default();
    for decorator in &function.decorator_list {
        let Some((name, first, second)) = declaration_decorator(decorator, ctx) else {
            continue;
        };
        if name == "class_adapter_provider" {
            if !has_bare_decorator(function, "const_eval") {
                malformed(
                    ctx,
                    "adapter_provider_const",
                    "@class_adapter_provider is valid only on an @const_eval function",
                    decorator.expression.range(),
                );
            }
            ctx.class_adapter_providers
                .push(ClassAdapterProviderDeclaration {
                    module: module.clone(),
                    function: function.name.to_string(),
                    descriptor_module: first,
                    descriptor_symbol: second.clone(),
                    descriptor_type: Type::Class {
                        identity: Some(format!("{module}.{second}")),
                        type_args: Vec::new(),
                        name: second,
                        fields: Vec::new(),
                        methods: Vec::new(),
                        parent_class: None,
                    },
                    range: decorator.expression.range(),
                });
            continue;
        }
        let Some(kind) = descriptor_kind(&name) else {
            continue;
        };
        let declaration = DeclarationDescriptorFunction {
            module: module.clone(),
            function: function.name.to_string(),
            provider_module: first,
            provider_function: second,
            descriptor_type: Type::Any,
            return_type: Type::Any,
            kind,
            range: decorator.expression.range(),
        };
        if ctx
            .descriptor_bindings
            .insert(function.name.to_string(), declaration.clone())
            .is_some()
        {
            malformed(
                ctx,
                "descriptor_declaration",
                "a function may declare exactly one descriptor kind",
                decorator.expression.range(),
            );
        }
        ctx.descriptor_functions.push(declaration);
    }
}

fn declaration_decorator(
    decorator: &Decorator,
    ctx: &mut LowerCtx,
) -> Option<(String, String, String)> {
    let Expr::Call(call) = &decorator.expression else {
        return None;
    };
    let Expr::Name(name) = call.func.as_ref() else {
        return None;
    };
    let declaration_name = name.id.as_str();
    if !matches!(
        declaration_name,
        "class_adapter_provider" | "class_adapter_marker"
    ) && descriptor_kind(declaration_name).is_none()
    {
        return None;
    }
    if call.arguments.args.len() != 2 || !call.arguments.keywords.is_empty() {
        malformed(
            ctx,
            "descriptor_declaration",
            "descriptor declarations require canonical module and symbol string literals",
            call.range(),
        );
        return None;
    }
    let Some(first) = string_literal(&call.arguments.args[0]) else {
        malformed(
            ctx,
            "descriptor_declaration",
            "descriptor declaration module must be a string literal",
            call.arguments.args[0].range(),
        );
        return None;
    };
    let Some(second) = string_literal(&call.arguments.args[1]) else {
        malformed(
            ctx,
            "descriptor_declaration",
            "descriptor declaration symbol must be a string literal",
            call.arguments.args[1].range(),
        );
        return None;
    };
    Some((declaration_name.to_string(), first, second))
}

fn descriptor_kind(name: &str) -> Option<DeclarationDescriptorKind> {
    match name {
        "field_descriptor" => Some(DeclarationDescriptorKind::Field),
        "class_descriptor" => Some(DeclarationDescriptorKind::Class),
        "method_descriptor" => Some(DeclarationDescriptorKind::Method),
        "type_descriptor" => Some(DeclarationDescriptorKind::Type),
        _ => None,
    }
}

fn has_bare_decorator(function: &StmtFunctionDef, expected: &str) -> bool {
    function.decorator_list.iter().any(
        |decorator| matches!(&decorator.expression, Expr::Name(name) if name.id.as_str() == expected),
    )
}

fn string_literal(expression: &Expr) -> Option<String> {
    match expression {
        Expr::StringLiteral(value) => Some(value.value.to_str().to_string()),
        _ => None,
    }
}

fn resolve_declared_type(module: &str, name: &str, ctx: &LowerCtx) -> Option<Type> {
    if module == ctx.current_module_name.as_deref().unwrap_or_default() {
        return ctx
            .scope
            .lookup_type_alias(name)
            .or_else(|| ctx.class_types.get(name))
            .cloned();
    }
    ctx.externals.classes.get(module)?.get(name).cloned()
}

fn local_provider<'a>(
    ctx: &'a LowerCtx,
    module: &str,
    descriptor: &DeclarationDescriptorFunction,
) -> Option<&'a ClassAdapterProviderDeclaration> {
    ctx.class_adapter_providers.iter().find(|provider| {
        module == descriptor.provider_module && provider.function == descriptor.provider_function
    })
}

fn external_provider<'a>(
    ctx: &'a LowerCtx,
    descriptor: &DeclarationDescriptorFunction,
) -> Option<&'a ClassAdapterProviderDeclaration> {
    ctx.externals
        .class_adapter_providers
        .get(&descriptor.provider_module)?
        .get(&descriptor.provider_function)
}

fn provider_signature_matches(
    function: &sifr_type_system::FunctionType,
    descriptor: &Type,
) -> bool {
    function.params.len() == 1
        && generic_contract(&function.params[0].1, "DeclarationInput", descriptor)
        && generic_contract(&function.return_type, "DeclarationPlan", descriptor)
}

fn generic_contract(candidate: &Type, expected_name: &str, descriptor: &Type) -> bool {
    let expected_identity = format!("sifr.meta.{expected_name}");
    matches!(
        candidate.resolve_alias(),
        Type::Class { identity, name, type_args, .. }
            if name == expected_name
                && identity.as_deref() == Some(expected_identity.as_str())
                && type_args.len() == 1
                && descriptor.is_assignable_to(&type_args[0])
                && type_args[0].is_assignable_to(descriptor)
    )
}

fn finalize_markers_and_selections(ctx: &mut LowerCtx, module: &str) {
    for index in 0..ctx.class_adapter_markers.len() {
        let marker = ctx.class_adapter_markers[index].clone();
        let provider = if marker.provider_module == module {
            ctx.class_adapter_providers
                .iter()
                .find(|provider| provider.function == marker.provider_function)
        } else {
            ctx.externals
                .class_adapter_providers
                .get(&marker.provider_module)
                .and_then(|providers| providers.get(&marker.provider_function))
        }
        .cloned();
        let Some(provider) = provider else {
            malformed(
                ctx,
                "adapter_marker_provider",
                "@class_adapter_marker references an unknown canonical provider",
                marker.range,
            );
            continue;
        };
        ctx.class_adapter_markers[index].descriptor_type = provider.descriptor_type.clone();
        if let Some(binding) = ctx.adapter_marker_bindings.get_mut(&marker.symbol) {
            binding.descriptor_type = provider.descriptor_type;
        }
    }

    for index in 0..ctx.class_adapter_selections.len() {
        let selection = ctx.class_adapter_selections[index].clone();
        let provider = if selection.provider_module == module {
            ctx.class_adapter_providers
                .iter()
                .find(|provider| provider.function == selection.provider_function)
        } else {
            ctx.externals
                .class_adapter_providers
                .get(&selection.provider_module)
                .and_then(|providers| providers.get(&selection.provider_function))
        }
        .cloned();
        let Some(provider) = provider else {
            malformed(
                ctx,
                "adapter_selection_provider",
                "adapter marker selected an unavailable canonical provider",
                selection.range,
            );
            continue;
        };
        ctx.class_adapter_selections[index].descriptor_type = provider.descriptor_type;
    }
}

fn closed_structural_type(ty: &Type, visiting: &mut HashSet<String>) -> bool {
    match ty.resolve_alias() {
        Type::None
        | Type::Bool
        | Type::Int
        | Type::FixedInt(_)
        | Type::Float
        | Type::Str
        | Type::Bytes => true,
        Type::List(item) | Type::Set(item) => closed_structural_type(item, visiting),
        Type::Dict(key, value) => {
            closed_structural_type(key, visiting) && closed_structural_type(value, visiting)
        }
        Type::Tuple(items) | Type::Union(items) => {
            !items.is_empty()
                && items
                    .iter()
                    .all(|item| closed_structural_type(item, visiting))
        }
        Type::Class {
            identity,
            name,
            fields,
            parent_class,
            ..
        } => {
            if parent_class.as_deref() == Some("NonSend") {
                return false;
            }
            let identity = identity.as_ref().unwrap_or(name);
            if !visiting.insert(identity.clone()) {
                return true;
            }
            let valid = fields
                .iter()
                .all(|(_, field)| closed_structural_type(field, visiting));
            visiting.remove(identity);
            valid
        }
        Type::LiteralInt(_) | Type::LiteralStr(_) | Type::LiteralBool(_) => true,
        _ => false,
    }
}

fn malformed(ctx: &mut LowerCtx, reason: &str, detail: &str, range: ruff_text_size::TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::META_MALFORMED_DECLARATION,
        format!("malformed typed descriptor declaration {reason}: {detail}"),
        range,
    );
}

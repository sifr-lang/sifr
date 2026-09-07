use super::str;
use crate::hir_nodes::HirExpr;
use ruff_text_size::Ranged;
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::{Expr, Stmt, StmtClassDef};
use sifr_type_system::{FunctionType, ParamConvention, Type};
use std::collections::HashMap;

use super::super::declared_class_identity;
use super::async_await::coroutine_result_type;
use super::class_field_inference::collect_constructor_self_field_assignments;
use super::diagnostics::{
    collect_enum_variants, get_newtype_inner, has_decorator, is_enum_class,
    is_error_class_with_ctx, is_protocol_class,
};
use super::parameter_conventions::{
    class_declared_method_param_convention, class_method_param_convention,
    declared_receiver_convention, inherit_class_methods, record_declared_class_method_metadata,
    replace_method_signature,
};
use super::simple_expr::lower_expr_simple;
use super::typing_and_functions::resolve_annotation_expr;
use super::validate_iteration_protocol_methods;
use super::{LowerCtx, parse_typevar_bound_expr};
use crate::lower::workload_annotations;

pub(super) fn class_method_signature<'a>(
    methods: &'a [(String, FunctionType)],
    method_name: &str,
) -> Option<&'a FunctionType> {
    methods.iter().find_map(
        |(name, ft)| {
            if name == method_name { Some(ft) } else { None }
        },
    )
}

pub(super) fn method_signature_return_type(
    func: &sifr_python_ast::StmtFunctionDef,
    return_ty: Type,
) -> Type {
    if func.is_async {
        coroutine_result_type(&return_ty)
    } else {
        return_ty
    }
}

use super::class_declaration_diagnostics::{
    invalid_class_base, missing_method_param_annotation, parent_class_range,
    unsupported_class_declaration,
};

pub(in crate::lower) fn collect_class_type(
    class_def: &StmtClassDef,
    ctx: &mut LowerCtx,
    validate_iteration_protocols: bool,
) {
    let class_name = class_def.name.to_string();
    let mut fields: Vec<(String, Type)> = Vec::new();
    let mut methods: Vec<(String, FunctionType)> = Vec::new();
    let mut method_ranges: HashMap<String, ruff_text_size::TextRange> = HashMap::new();
    let is_error = is_error_class_with_ctx(class_def, ctx);
    let is_protocol = is_protocol_class(class_def);
    let newtype_inner = get_newtype_inner(class_def);

    // PEP 695: register inline type params (class C[T]) as type variables.
    // Class collection runs twice; bounds are source-shape declarations and should only emit
    // diagnostics/register specs once.
    if !validate_iteration_protocols {
        if let Some(ref type_params) = class_def.type_params {
            let mut declared_params = Vec::new();
            for tp in type_params.iter() {
                if let sifr_python_ast::TypeParam::TypeVar(tv) = tp {
                    let tp_name = tv.name.to_string();
                    ctx.type_vars.insert(tp_name.clone());
                    declared_params.push(tp_name.clone());
                    if let Some(ref bound) = tv.bound {
                        let specs = parse_typevar_bound_expr(bound, ctx);
                        if !specs.is_empty() {
                            ctx.type_param_bounds
                                .entry(class_name.clone())
                                .or_default()
                                .entry(tp_name)
                                .or_default()
                                .extend(specs);
                        }
                    }
                }
            }
            if !declared_params.is_empty() {
                ctx.class_declared_type_params
                    .insert(class_name.clone(), declared_params);
            }
        }
    }

    // For newtype declarations, register as a Newtype type
    if let Some(ref inner) = newtype_inner {
        let newtype_ty = Type::Newtype {
            identity: declared_class_identity(ctx.current_module_name.as_deref(), &class_name),
            name: class_name.clone(),
            inner: Box::new(inner.clone()),
        };
        ctx.class_types
            .insert(class_name.clone(), newtype_ty.clone());

        // Register constructor: ClassName(value) -> ClassName
        let ft = FunctionType::new(vec![("value".to_string(), inner.clone())], newtype_ty);
        ctx.functions.insert(class_name.clone(), ft);
        return;
    }

    // For enum declarations, register as an Enum type
    if is_enum_class(class_def) {
        let variants = collect_enum_variants(class_def);
        // Check for duplicate variant values
        {
            let mut seen_values: std::collections::HashMap<i64, String> =
                std::collections::HashMap::new();
            for variant in &variants {
                let val = variant.value.unwrap_or(0);
                if let Some(existing) = seen_values.get(&val) {
                    if variant.value.is_some() {
                        let enum_name = class_name.as_str();
                        let value = val;
                        let existing_variant = existing;
                        let duplicate_variant = variant.name.as_str();
                        ctx.error_with_code_at(
                            DiagnosticCode::CLASS_DUPLICATE_OR_INVALID_VALUE,
                            format!(
                                "enum '{enum_name}' has duplicate value {value}: variants '{existing_variant}' and '{duplicate_variant}'"
                            ),
                            variant.name_range,
                        );
                    }
                } else if variant.value.is_some() {
                    seen_values.insert(val, variant.name.clone());
                }
            }
        }
        let enum_ty = Type::Enum {
            identity: declared_class_identity(ctx.current_module_name.as_deref(), &class_name),
            name: class_name.clone(),
            variants: variants
                .iter()
                .map(|variant| (variant.name.clone(), variant.value))
                .collect(),
        };
        ctx.class_types.insert(class_name.clone(), enum_ty.clone());
        // Register each variant as a constant of the enum type
        for variant in &variants {
            ctx.functions.insert(
                format!("{}.{}", class_name, variant.name),
                FunctionType::new(vec![], enum_ty.clone()),
            );
        }
        // Collect method signatures from enum body and register them
        for stmt in &class_def.body {
            if let Stmt::FunctionDef(func) = stmt {
                let method_name = func.name.to_string();
                if method_name == "__init__" {
                    continue;
                }
                let mut params = Vec::new();
                for param in func.parameters.args.iter().skip(1) {
                    let param_name = param.parameter.name.to_string();
                    let param_ty = if let Some(ref ann) = param.parameter.annotation {
                        resolve_annotation_expr(ann, ctx)
                    } else {
                        missing_method_param_annotation(
                            ctx,
                            &class_name,
                            &method_name,
                            &param_name,
                            param.parameter.name.range(),
                        );
                        Type::Any
                    };
                    let convention =
                        class_method_param_convention(param.parameter.convention, &param_ty, ctx);
                    params.push((param_name, param_ty, convention));
                }
                let return_ty = if let Some(ref ret_ann) = func.returns {
                    resolve_annotation_expr(ret_ann, ctx)
                } else {
                    Type::None
                };
                let ft = FunctionType {
                    receiver: Some(declared_receiver_convention(&func.parameters)),
                    params,
                    return_type: Box::new(method_signature_return_type(func, return_ty)),
                };
                // Register method as ClassName.method_name for lookup
                ctx.functions
                    .insert(format!("{class_name}.{method_name}"), ft.clone());
                methods.push((method_name, ft));
            }
        }
        return;
    }

    // For protocol definitions, register as a Protocol type
    if is_protocol {
        // Collect method signatures for the protocol
        for stmt in &class_def.body {
            if let Stmt::FunctionDef(func) = stmt {
                let method_name = func.name.to_string();
                if method_name == "__init__" {
                    continue;
                }
                let mut params = Vec::new();
                for param in func.parameters.args.iter().skip(1) {
                    let param_name = param.parameter.name.to_string();
                    let param_ty = if let Some(ref ann) = param.parameter.annotation {
                        resolve_annotation_expr(ann, ctx)
                    } else {
                        missing_method_param_annotation(
                            ctx,
                            &class_name,
                            &method_name,
                            &param_name,
                            param.parameter.name.range(),
                        );
                        Type::Any
                    };
                    let convention =
                        class_method_param_convention(param.parameter.convention, &param_ty, ctx);
                    params.push((param_name, param_ty, convention));
                }
                let return_ty = if let Some(ref ret_ann) = func.returns {
                    resolve_annotation_expr(ret_ann, ctx)
                } else {
                    Type::None
                };
                let receiver = declared_receiver_convention(&func.parameters);
                methods.push((
                    method_name,
                    FunctionType {
                        receiver: Some(receiver),
                        params,
                        return_type: Box::new(method_signature_return_type(func, return_ty)),
                    },
                ));
            }
        }
        let proto_ty = Type::Protocol {
            identity: declared_class_identity(ctx.current_module_name.as_deref(), &class_name),
            name: class_name.clone(),
            methods: methods.clone(),
        };
        ctx.class_types.insert(class_name, proto_ty);
        return;
    }

    // Inherit parent fields and methods for single inheritance
    let parent_class_name =
        crate::lower::descriptor_declarations::data_parent_name(&class_name, ctx);
    let mut parent_class_chain = is_error.then(|| "Error".to_string());
    let mut inherited_field_defaults = Vec::new();
    if let Some(ref parent_name) = parent_class_name {
        if let Some(parent_ty) =
            crate::lower::descriptor_declarations::data_parent_type(class_def, ctx)
                .or_else(|| ctx.class_types.get(parent_name).cloned())
        {
            ctx.class_data_parent_types
                .insert(class_name.clone(), parent_ty.clone());
            inherited_field_defaults = ctx
                .class_field_defaults
                .get(parent_name)
                .cloned()
                .unwrap_or_default();
            if !super::super::generic_parent_representation::preserves_union_structure(
                &ctx.class_types,
                &ctx.class_declared_type_params,
                parent_name,
                &parent_ty,
            ) {
                invalid_class_base(
                    ctx,
                    &class_name,
                    "generic parent arguments change a declared union's member topology",
                    parent_class_range(class_def, parent_name),
                );
            }
            if let Type::Class {
                identity: parent_identity,
                name: parent_type_name,
                fields: parent_fields,
                methods: parent_methods,
                parent_class: parent_parent_chain,
                ..
            } = parent_ty
            {
                // Inherit parent fields
                for (fname, fty) in &parent_fields {
                    fields.push((fname.clone(), fty.clone()));
                }
                inherit_class_methods(&mut methods, &parent_methods, ctx, parent_name, &class_name);
                let parent_identity = parent_identity.unwrap_or(parent_type_name);
                parent_class_chain = Some(if let Some(chain) = parent_parent_chain {
                    format!("{parent_identity}|{chain}")
                } else {
                    parent_identity
                });
            } else {
                let reason = format!("parent type '{parent_name}' is not a class");
                invalid_class_base(
                    ctx,
                    &class_name,
                    reason.as_str(),
                    parent_class_range(class_def, parent_name),
                );
            }
        } else {
            let reason = format!("parent class '{parent_name}' not defined");
            invalid_class_base(
                ctx,
                &class_name,
                reason.as_str(),
                parent_class_range(class_def, parent_name),
            );
        }
    }
    let inherited_field_count = fields.len();
    if is_error {
        super::error_message_contract::seed(class_def, &mut fields);
    }

    // Register a preliminary class type so self-referential annotations work
    // (e.g., `def distance(self, other: Point)` inside class Point)
    ctx.class_types.insert(
        class_name.clone(),
        Type::Class {
            identity: declared_class_identity(ctx.current_module_name.as_deref(), &class_name),
            type_args: ctx
                .class_declared_type_params
                .get(&class_name)
                .into_iter()
                .flatten()
                .cloned()
                .map(Type::TypeVar)
                .collect(),
            name: class_name.clone(),
            fields: vec![],
            methods: vec![],
            parent_class: parent_class_chain.clone(),
        },
    );

    let mut field_defaults: Vec<(usize, HirExpr)> = inherited_field_defaults;
    let mut own_fields: Vec<(String, ruff_text_size::TextRange)> = Vec::new();
    let mut own_field_index_by_field_index = std::collections::HashMap::new();
    let mut own_field_default_indices = std::collections::HashSet::new();
    for stmt in &class_def.body {
        match stmt {
            // Field annotations: `x: float` or `x: float = 0.0`
            Stmt::AnnAssign(ann) => {
                if let Expr::Name(name) = ann.target.as_ref() {
                    let ty = resolve_annotation_expr(&ann.annotation, ctx);
                    let inherited_override = fields
                        .iter()
                        .take(inherited_field_count)
                        .position(|(field, _)| field == name.id.as_str());
                    if let Some(index) = inherited_override {
                        if !fields[index].1.is_assignable_to(&ty)
                            || !ty.is_assignable_to(&fields[index].1)
                        {
                            ctx.error_with_code_at(
                                DiagnosticCode::TYPE_MISMATCH,
                                format!(
                                    "inherited field '{}' cannot be re-annotated from '{}' to '{}'",
                                    name.id,
                                    fields[index].1.display_name(),
                                    ty.display_name()
                                ),
                                ann.annotation.range(),
                            );
                            continue;
                        }
                        fields[index].1 = ty.clone();
                    }
                    let field_idx = inherited_override.unwrap_or(fields.len());
                    let own_field_idx = own_fields.len();
                    if inherited_override.is_none() {
                        fields.push((name.id.to_string(), ty));
                    }
                    own_fields.push((name.id.to_string(), name.range()));
                    own_field_index_by_field_index.insert(field_idx, own_field_idx);
                    // Collect default value if present (for auto-init default params)
                    if let Some(ref default_expr) = ann.value {
                        field_defaults.retain(|(index, _)| *index != field_idx);
                        if let Some(kind) =
                            crate::lower::descriptor_declarations::descriptor_kind_for_call(
                                default_expr,
                                ctx,
                            )
                        {
                            if kind != sifr_ir::DeclarationDescriptorKind::Field {
                                ctx.error_with_code_at(
                                    DiagnosticCode::META_MALFORMED_DECLARATION,
                                    "descriptor function is not valid on an annotated field"
                                        .to_string(),
                                    default_expr.range(),
                                );
                            } else if ctx
                                .class_adapter_selections
                                .iter()
                                .any(|selection| selection.owner == class_name)
                            {
                                // The provisional pass must permit calls that omit a
                                // descriptor-owned default. The adapter replaces this
                                // typed sentinel before any finalized HIR is retained.
                                field_defaults.push((
                                    field_idx,
                                    HirExpr::Name {
                                        name: "__sifr_adapter_provisional_default".to_string(),
                                        binding_id: None,
                                        ty: fields[field_idx].1.clone(),
                                    },
                                ));
                                own_field_default_indices.insert(own_field_idx);
                            }
                            continue;
                        }
                        if let Some(hir_default) = lower_expr_simple(default_expr) {
                            field_defaults.push((field_idx, hir_default));
                            own_field_default_indices.insert(own_field_idx);
                        } else {
                            let detail =
                                format!("unsupported default expression for field '{}'", name.id);
                            unsupported_class_declaration(
                                ctx,
                                &class_name,
                                detail.as_str(),
                                default_expr.range(),
                            );
                        }
                    }
                }
            }
            Stmt::Assign(assign) => {
                let Some(kind) = crate::lower::descriptor_declarations::descriptor_kind_for_call(
                    &assign.value,
                    ctx,
                ) else {
                    unsupported_class_declaration(
                        ctx,
                        &class_name,
                        "unsupported statement in class body",
                        stmt.range(),
                    );
                    continue;
                };
                if kind != sifr_ir::DeclarationDescriptorKind::Class {
                    ctx.error_with_code_at(
                        DiagnosticCode::META_MALFORMED_DECLARATION,
                        "descriptor function is not valid on a consumed class assignment"
                            .to_string(),
                        assign.value.range(),
                    );
                } else if assign.targets.len() != 1
                    || !matches!(assign.targets.first(), Some(Expr::Name(_)))
                {
                    ctx.error_with_code_at(
                        DiagnosticCode::META_MALFORMED_DECLARATION,
                        "a consumed class descriptor requires one simple assignment target"
                            .to_string(),
                        stmt.range(),
                    );
                }
            }
            // Method definitions
            Stmt::FunctionDef(func) => {
                let method_name = func.name.to_string();
                let is_static = has_decorator(func, "staticmethod");
                let is_class = has_decorator(func, "classmethod");
                let previous_current_class = ctx.current_class.replace(class_name.clone());
                let previous_self_annotation_available = std::mem::replace(
                    &mut ctx.self_annotation_available,
                    method_name == "__init__" || (!is_static && !is_class),
                );
                method_ranges.insert(method_name.clone(), func.name.range());
                if method_name == "__init__" {
                    // Constructor: extract params (skip `self`)
                    let mut params = Vec::new();
                    let mut constructor_locals: HashMap<String, Type> = HashMap::new();
                    for param in func.parameters.args.iter().skip(1) {
                        let param_name = param.parameter.name.to_string();
                        let param_ty = if let Some(ref ann) = param.parameter.annotation {
                            resolve_annotation_expr(ann, ctx)
                        } else {
                            missing_method_param_annotation(
                                ctx,
                                &class_name,
                                "__init__",
                                &param_name,
                                param.parameter.name.range(),
                            );
                            Type::Any
                        };
                        constructor_locals.insert(param_name.clone(), param_ty.clone());
                        let convention = class_method_param_convention(
                            param.parameter.convention,
                            &param_ty,
                            ctx,
                        );
                        params.push((param_name, param_ty, convention));
                    }
                    // Constructor return type is registered after field collection.
                    let constructor_ft = FunctionType {
                        receiver: None,
                        params,
                        return_type: Box::new(Type::None),
                    };
                    ctx.functions.insert(class_name.clone(), constructor_ft);

                    collect_constructor_self_field_assignments(
                        &func.body,
                        &mut constructor_locals,
                        &mut fields,
                        ctx,
                    );

                    // Collect defaults for constructor
                    let mut defaults = Vec::new();
                    for (i, param) in func.parameters.args.iter().skip(1).enumerate() {
                        if let Some(ref default_expr) = param.default {
                            if let Some(hir_default) = lower_expr_simple(default_expr) {
                                defaults.push((i, hir_default));
                            } else {
                                ctx.error_with_code_at(
                                    DiagnosticCode::TYPE_UNSUPPORTED_DEFAULT_ARGUMENT,
                                    format!(
                                        "class '{class_name}.__init__': unsupported default argument expression for parameter '{}'",
                                        param.parameter.name
                                    ),
                                    default_expr.range(),
                                );
                            }
                        }
                    }
                    if !defaults.is_empty() {
                        ctx.function_defaults.insert(class_name.clone(), defaults);
                    }
                } else {
                    record_declared_class_method_metadata(
                        ctx,
                        &class_name,
                        &method_name,
                        !is_static && !is_class,
                    );
                    let skip_count = usize::from(!is_static);
                    let callback_policies = crate::lower::python_interop::callback_call_policies(
                        &func.decorator_list,
                        &func.parameters,
                        !is_static,
                    );
                    if !callback_policies.is_empty() {
                        ctx.python_callback_call_policies
                            .insert(format!("{class_name}.{method_name}"), callback_policies);
                    }
                    let mut params = Vec::new();
                    let mut method_locals: HashMap<String, Type> = HashMap::new();
                    for param in func.parameters.args.iter().skip(skip_count) {
                        let param_name = param.parameter.name.to_string();
                        let param_ty = if let Some(ref ann) = param.parameter.annotation {
                            resolve_annotation_expr(ann, ctx)
                        } else {
                            missing_method_param_annotation(
                                ctx,
                                &class_name,
                                &method_name,
                                &param_name,
                                param.parameter.name.range(),
                            );
                            Type::Any
                        };
                        method_locals.insert(param_name.clone(), param_ty.clone());
                        let convention = class_declared_method_param_convention(
                            param.parameter.convention,
                            &param_ty,
                            ctx,
                            (&class_name, &method_name),
                            (&param_name, param.parameter.name.range()),
                        );
                        params.push((param_name, param_ty, convention));
                    }
                    for param in &func.parameters.kwonlyargs {
                        let param_name = param.parameter.name.to_string();
                        let param_ty = if let Some(ref ann) = param.parameter.annotation {
                            resolve_annotation_expr(ann, ctx)
                        } else {
                            missing_method_param_annotation(
                                ctx,
                                &class_name,
                                &method_name,
                                &param_name,
                                param.parameter.name.range(),
                            );
                            Type::Any
                        };
                        method_locals.insert(param_name.clone(), param_ty.clone());
                        let convention = class_declared_method_param_convention(
                            param.parameter.convention,
                            &param_ty,
                            ctx,
                            (&class_name, &method_name),
                            (&param_name, param.parameter.name.range()),
                        );
                        params.push((param_name, param_ty, convention));
                    }
                    crate::lower::rust_callback_callsite::record_threadsafe_callback_target(
                        format!("{class_name}.{method_name}"),
                        &params,
                        &func.decorator_list,
                        ctx,
                    );
                    let return_ty = if let Some(ref ret_ann) = func.returns {
                        resolve_annotation_expr(ret_ann, ctx)
                    } else {
                        Type::None
                    };
                    let mut defaults = Vec::new();
                    for (i, param) in func.parameters.args.iter().skip(skip_count).enumerate() {
                        if let Some(ref default_expr) = param.default {
                            if let Some(hir_default) = lower_expr_simple(default_expr) {
                                defaults.push((i, hir_default));
                            } else {
                                ctx.error_with_code_at(
                                    DiagnosticCode::TYPE_UNSUPPORTED_DEFAULT_ARGUMENT,
                                    format!(
                                        "class '{class_name}.{method_name}': unsupported default argument expression for parameter '{}'",
                                        param.parameter.name
                                    ),
                                    default_expr.range(),
                                );
                            }
                        }
                    }
                    let regular_count = func.parameters.args.len().saturating_sub(skip_count)
                        + usize::from(func.parameters.vararg.is_some());
                    for (i, param) in func.parameters.kwonlyargs.iter().enumerate() {
                        if let Some(ref default_expr) = param.default {
                            if let Some(hir_default) = lower_expr_simple(default_expr) {
                                defaults.push((regular_count + i, hir_default));
                            } else {
                                ctx.error_with_code_at(
                                    DiagnosticCode::TYPE_UNSUPPORTED_DEFAULT_ARGUMENT,
                                    format!(
                                        "class '{class_name}.{method_name}': unsupported default argument expression for parameter '{}'",
                                        param.parameter.name
                                    ),
                                    default_expr.range(),
                                );
                            }
                        }
                    }
                    if !defaults.is_empty() {
                        ctx.function_defaults
                            .insert(format!("{class_name}.{method_name}"), defaults);
                    }
                    if let Some(workload) =
                        workload_annotations::annotation_for_decorators(func.decorator_list.iter())
                    {
                        ctx.function_workload_annotations
                            .insert(format!("{class_name}.{method_name}"), workload);
                    }
                    let receiver = (!is_static && !is_class)
                        .then(|| declared_receiver_convention(&func.parameters));
                    replace_method_signature(
                        &mut methods,
                        method_name,
                        FunctionType {
                            receiver,
                            params,
                            return_type: Box::new(method_signature_return_type(func, return_ty)),
                        },
                    );

                    collect_constructor_self_field_assignments(
                        &func.body,
                        &mut method_locals,
                        &mut fields,
                        ctx,
                    );
                }
                ctx.self_annotation_available = previous_self_annotation_available;
                ctx.current_class = previous_current_class;
            }
            Stmt::Pass(_) => {} // Allow pass in class body
            _ => {
                unsupported_class_declaration(
                    ctx,
                    &class_name,
                    "unsupported statement in class body",
                    stmt.range(),
                );
            }
        }
    }

    if validate_iteration_protocols {
        validate_iteration_protocol_methods(
            &class_name,
            &methods,
            &method_ranges,
            class_def.name.range(),
            ctx,
        );
    }

    if ctx.adapter_field_plans.contains_key(&class_name) {
        field_defaults = crate::lower::adapter_field_plans::defaults_for_class(
            &class_name,
            &fields,
            field_defaults,
            ctx,
        );
        own_field_default_indices = field_defaults
            .iter()
            .filter_map(|(index, _)| own_field_index_by_field_index.get(index).copied())
            .collect();
    }

    let is_python_opaque = ctx.python_opaque_classes.contains_key(&class_name);
    if is_error {
        super::error_message_contract::collect(
            class_def,
            &mut fields,
            &mut field_defaults,
            &mut parent_class_chain,
            ctx,
        );
    }
    let generic_type_args = ctx
        .class_declared_type_params
        .get(&class_name)
        .into_iter()
        .flatten()
        .cloned()
        .map(Type::TypeVar)
        .collect();
    let class_ty = Type::Class {
        identity: declared_class_identity(ctx.current_module_name.as_deref(), &class_name),
        type_args: generic_type_args,
        name: class_name.clone(),
        fields: fields.clone(),
        methods: methods.clone(),
        parent_class: if is_python_opaque {
            Some("NonSend".to_string())
        } else {
            parent_class_chain.clone()
        },
    };

    super::error_message_contract::inherit_constructor(class_def, &fields, is_error, ctx);
    // Update the constructor function to return the class type
    if is_python_opaque {
        ctx.functions.remove(&class_name);
    } else if let Some(ft) = ctx.functions.get_mut(&class_name) {
        *ft.return_type = class_ty.clone();
    } else {
        // No __init__ defined -- create a default constructor from fields

        // Descriptor calls are provisional defaults until the adapter plan is
        // available. Validate ordering only on ordinary classes or the
        // finalized adapted-class pass.
        let provisional_adapter = ctx
            .class_adapter_selections
            .iter()
            .any(|selection| selection.owner == class_name)
            && !ctx.adapter_field_plans.contains_key(&class_name);
        if !provisional_adapter {
            let mut seen_default = false;
            for (i, (fname, range)) in own_fields.iter().enumerate() {
                if own_field_default_indices.contains(&i) {
                    seen_default = true;
                } else if seen_default {
                    let field = fname.as_str();
                    ctx.error_with_code_at(
                        DiagnosticCode::CLASS_REQUIRED_FIELD_AFTER_DEFAULT,
                        format!(
                            "class '{class_name}': required field '{field}' declared after field with default value"
                        ),
                        *range,
                    );
                }
            }
        }

        // Inheritance diagnostic: warn when child has own fields but no __init__ and extends a parent
        if !ctx
            .class_adapter_selections
            .iter()
            .any(|selection| selection.owner == class_name)
            && !ctx.adapter_field_plans.contains_key(&class_name)
            && parent_class_name
                .as_deref()
                .is_some_and(|parent| parent != "NonSend")
        {
            let parent_field_count = if let Some(ref pname) = parent_class_name {
                ctx.class_types.get(pname).map_or(0, |ty| {
                    if let Type::Class { fields: pf, .. } = ty {
                        pf.len()
                    } else {
                        0
                    }
                })
            } else {
                0
            };
            let has_own_fields = fields.len() > parent_field_count;
            if has_own_fields {
                ctx.error_with_code_at(
                    DiagnosticCode::CLASS_MISSING_INITIALIZER,
                    format!(
                        "class '{class_name}' has fields but no __init__; parent fields will not be initialized. \
                         Define an explicit __init__ with super().__init__(...)"
                    ),
                    class_def.name.range(),
                );
            }
        }

        let params: Vec<(String, Type)> = fields.clone();
        let mut ft = FunctionType::new(params, class_ty.clone());
        for (_, ty, convention) in &mut ft.params {
            if ty.contains_affine_resource() {
                *convention = ParamConvention::own();
            }
        }
        ctx.functions.insert(class_name.clone(), ft);
        // Store field defaults for the auto-generated constructor
        if !field_defaults.is_empty() {
            ctx.function_defaults
                .insert(class_name.clone(), field_defaults.clone());
        }
    }

    // Generic class constructors are generic callables keyed by class name.
    if let Some(type_params) = ctx.class_declared_type_params.get(&class_name).cloned() {
        if !type_params.is_empty() {
            ctx.generic_functions
                .insert(class_name.clone(), type_params);
        }
    }

    if is_error {
        ctx.error_types.insert(class_name.clone());
    } else {
        // A declaration can shadow a builtin error name without inheriting it.
        ctx.error_types.remove(&class_name);
    }

    if !field_defaults.is_empty() {
        ctx.class_field_defaults
            .insert(class_name.clone(), field_defaults);
    }

    ctx.class_types.insert(class_name, class_ty);
}

use super::class_shape_metadata::{declaration_metadata, field_default_identities, field_defaults};
use super::is_hashable_type;
use super::parameter_conventions::{
    class_method_param_convention, class_method_param_default, declared_receiver_convention,
    prepare_method_param_ownership,
};
use super::rust_opaque_validation as opaque;
use super::{
    Expr, FunctionType, HirClass, HirClassKind, HirFunction, HirParam, LowerCtx, MethodKind,
    ParamConvention, Ranged, Stmt, StmtClassDef, Type, collect_enum_variants,
    constructor_uninitialized_storage_at_first_self_use, function_body_contains_yield,
    get_newtype_inner, has_decorator, is_enum_class, is_operator_dunder, is_protocol_class,
    lower_function_stmts, missing_method_param_annotation, resolve_annotation_expr,
    unsupported_class_declaration,
};
use crate::lower::ownership_diagnostics;
use crate::lower::python_interop::{
    classify_python_interop_stub_body, collect_python_method_declarations,
    has_python_interop_decorator_syntax, receiver_is_owned, validate_python_interop_signature,
};
use crate::lower::rust_interop::{
    RustInteropOwner, classify_rust_interop_stub_body, collect_rust_interop_declarations,
    has_rust_interop_decorator_syntax,
};
use sifr_type_system::ReceiverConvention;

pub(in crate::lower) fn lower_class(
    class_def: &StmtClassDef,
    ctx: &mut LowerCtx,
) -> Option<HirClass> {
    let class_name = class_def.name.to_string();
    let class_ty = ctx.class_types.get(&class_name)?.clone();
    let is_python_opaque = ctx.python_opaque_classes.contains_key(&class_name);
    let is_protocol = is_protocol_class(class_def);
    let newtype_inner = get_newtype_inner(class_def);
    opaque::validate_structurally_mapped_opaque_class(class_def, &class_ty, ctx);
    if is_protocol {
        let methods_sigs = match &class_ty {
            Type::Protocol { methods, .. } => methods.clone(),
            _ => return None,
        };
        let hir_methods: Vec<HirFunction> = methods_sigs
            .iter()
            .map(|(name, ft)| {
                HirFunction {
                    name: name.clone(),
                    params: ft
                        .params
                        .iter()
                        .map(|(pn, pt, _)| HirParam {
                            name: pn.clone(),
                            ty: pt.clone(),
                            default: None,
                            keyword_only: false,
                            convention: ParamConvention::default(),
                        })
                        .collect(),
                    return_type: *ft.return_type.clone(),
                    body: vec![], // Protocol methods have no body
                    is_async: false,
                    method_kind: MethodKind::Regular,
                    receiver: ft.receiver,
                    decorators: vec![],
                    rust_interop: Vec::new(),
                    python_interop: Vec::new(),
                    compiler_intrinsic: None,
                    type_params: Vec::new(),
                }
            })
            .collect();

        return Some(HirClass {
            name: class_name.clone(),
            identity: None,
            fields: vec![],
            field_defaults: Vec::new(),
            field_default_identities: Vec::new(),
            declaration_metadata: Vec::new(),
            methods: hir_methods,
            is_hashable: false,
            is_error_type: false,
            kind: HirClassKind::Protocol,
            operator_impls: Vec::new(),
            newtype_inner: None,
            implements_protocols: Vec::new(),
            parent_class: None,
            parent_type: None,
            type_params: Vec::new(),
            enum_variants: Vec::new(),
            rust_interop: collect_rust_interop_declarations(
                &class_def.decorator_list,
                RustInteropOwner::Class,
                ctx,
                false,
                false,
                false,
            ),
        });
    }

    if is_enum_class(class_def) {
        let variants = collect_enum_variants(class_def);
        let mut hir_methods = Vec::new();
        ctx.current_class = Some(class_name.clone());
        ctx.self_annotation_available = true;
        for stmt in &class_def.body {
            if let Stmt::FunctionDef(func) = stmt {
                let method_name = func.name.to_string();
                ctx.scope.push();
                let receiver = declared_receiver_convention(&func.parameters);
                let receiver_id =
                    ctx.scope
                        .define_receiver("self".to_string(), class_ty.clone(), receiver);
                ctx.method_receiver_bindings
                    .insert(format!("{class_name}.{method_name}"), receiver_id);

                // Define method parameters (skip `self`)
                let mut params = Vec::new();
                for param in func.parameters.args.iter().skip(1) {
                    let param_name = param.parameter.name.to_string();
                    let param_ty = if let Some(ref ann) = param.parameter.annotation {
                        resolve_annotation_expr(ann, ctx)
                    } else {
                        Type::Any
                    };
                    let convention =
                        class_method_param_convention(param.parameter.convention, &param_ty, ctx);
                    ctx.scope
                        .define_parameter(param_name.clone(), param_ty.clone(), convention);
                    params.push(HirParam {
                        name: param_name,
                        ty: param_ty,
                        default: None,
                        keyword_only: false,
                        convention,
                    });
                }

                let return_ty = if let Some(ref ret_ann) = func.returns {
                    resolve_annotation_expr(ret_ann, ctx)
                } else {
                    Type::None
                };

                let mut method_ft = FunctionType::new(
                    params
                        .iter()
                        .map(|p| (p.name.clone(), p.ty.clone()))
                        .collect(),
                    return_ty.clone(),
                );
                method_ft.receiver = Some(receiver);

                let rust_interop = collect_rust_interop_declarations(
                    &func.decorator_list,
                    RustInteropOwner::Method,
                    ctx,
                    has_decorator(func, "blocking_io"),
                    has_decorator(func, "cpu_heavy"),
                    func.is_async,
                );
                let stub_body = classify_rust_interop_stub_body(
                    &func.body,
                    has_rust_interop_decorator_syntax(&func.decorator_list),
                    ctx,
                );

                let previous_owner = ctx.current_owner.replace(class_name.clone());
                let previous_method = ctx.current_method.replace(method_name.clone());
                let previous_dynamic_python = ctx.current_function_trusts_dynamic_python;
                let previous_async = ctx.current_function_is_async;
                let previous_generator = ctx.current_function_is_generator;
                let previous_async_generator = ctx.current_function_is_async_generator;
                ctx.current_function_trusts_dynamic_python =
                    has_decorator(func, "trust_python_dynamic");
                ctx.current_function_is_async = func.is_async;
                ctx.current_function_is_generator = function_body_contains_yield(&func.body);
                ctx.current_function_is_async_generator =
                    func.is_async && function_body_contains_yield(&func.body);
                let body = if stub_body.skips_normal_body_lowering() {
                    Vec::new()
                } else {
                    lower_function_stmts(&func.body, &method_ft, ctx)
                };
                ctx.current_function_is_async = previous_async;
                ctx.current_function_is_generator = previous_generator;
                ctx.current_function_is_async_generator = previous_async_generator;
                ctx.current_function_trusts_dynamic_python = previous_dynamic_python;
                ctx.current_method = previous_method;
                ctx.current_owner = previous_owner;
                ctx.scope.pop();

                hir_methods.push(HirFunction {
                    name: method_name,
                    params,
                    return_type: return_ty,
                    body,
                    is_async: func.is_async,
                    method_kind: MethodKind::Regular,
                    receiver: method_ft.receiver,
                    decorators: vec![],
                    rust_interop,
                    python_interop: Vec::new(),
                    compiler_intrinsic: None,
                    type_params: Vec::new(),
                });
            }
        }
        ctx.self_annotation_available = false;
        ctx.current_class = None;
        return Some(HirClass {
            name: class_name.clone(),
            identity: None,
            fields: vec![],
            field_defaults: Vec::new(),
            field_default_identities: Vec::new(),
            declaration_metadata: declaration_metadata(ctx, &class_name),
            methods: hir_methods,
            is_hashable: true,
            is_error_type: false,
            kind: HirClassKind::Enum,
            operator_impls: Vec::new(),
            newtype_inner: None,
            implements_protocols: Vec::new(),
            parent_class: None,
            parent_type: None,
            type_params: Vec::new(),
            enum_variants: variants
                .iter()
                .map(|variant| (variant.name.clone(), variant.value))
                .collect(),
            rust_interop: collect_rust_interop_declarations(
                &class_def.decorator_list,
                RustInteropOwner::Class,
                ctx,
                false,
                false,
                false,
            ),
        });
    }

    // For newtype declarations, emit a minimal HirClass
    if let Some(ref inner) = newtype_inner {
        // Lower any methods defined in the newtype body
        let mut hir_methods = Vec::new();
        for stmt in &class_def.body {
            if let Stmt::FunctionDef(func) = stmt {
                let method_name = func.name.to_string();
                if method_name == "__init__" {
                    continue;
                } // Skip __init__ for newtypes
                ctx.current_class = Some(class_name.clone());
                ctx.self_annotation_available = true;
                ctx.scope.push();
                let receiver = declared_receiver_convention(&func.parameters);
                let receiver_id =
                    ctx.scope
                        .define_receiver("self".to_string(), class_ty.clone(), receiver);
                ctx.method_receiver_bindings
                    .insert(format!("{class_name}.{method_name}"), receiver_id);
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
                    ctx.scope
                        .define_parameter(param_name.clone(), param_ty.clone(), convention);
                    params.push(HirParam {
                        name: param_name,
                        ty: param_ty,
                        default: None,
                        keyword_only: false,
                        convention,
                    });
                }
                let return_ty = if let Some(ref ret_ann) = func.returns {
                    resolve_annotation_expr(ret_ann, ctx)
                } else {
                    Type::None
                };
                let mut method_ft = FunctionType::new(
                    params
                        .iter()
                        .map(|p| (p.name.clone(), p.ty.clone()))
                        .collect(),
                    return_ty.clone(),
                );
                method_ft.receiver = Some(receiver);
                let rust_interop = collect_rust_interop_declarations(
                    &func.decorator_list,
                    RustInteropOwner::Method,
                    ctx,
                    has_decorator(func, "blocking_io"),
                    has_decorator(func, "cpu_heavy"),
                    func.is_async,
                );
                let stub_body = classify_rust_interop_stub_body(
                    &func.body,
                    has_rust_interop_decorator_syntax(&func.decorator_list),
                    ctx,
                );

                let previous_owner = ctx.current_owner.replace(class_name.clone());
                let previous_method = ctx.current_method.replace(method_name.clone());
                let previous_dynamic_python = ctx.current_function_trusts_dynamic_python;
                let previous_async = ctx.current_function_is_async;
                let previous_generator = ctx.current_function_is_generator;
                let previous_async_generator = ctx.current_function_is_async_generator;
                ctx.current_function_trusts_dynamic_python =
                    has_decorator(func, "trust_python_dynamic");
                ctx.current_function_is_async = func.is_async;
                ctx.current_function_is_generator = function_body_contains_yield(&func.body);
                ctx.current_function_is_async_generator =
                    func.is_async && function_body_contains_yield(&func.body);
                let body = if stub_body.skips_normal_body_lowering() {
                    Vec::new()
                } else {
                    lower_function_stmts(&func.body, &method_ft, ctx)
                };
                ctx.current_function_is_async = previous_async;
                ctx.current_function_is_generator = previous_generator;
                ctx.current_function_is_async_generator = previous_async_generator;
                ctx.current_function_trusts_dynamic_python = previous_dynamic_python;
                ctx.current_method = previous_method;
                ctx.current_owner = previous_owner;
                ctx.scope.pop();
                ctx.self_annotation_available = false;
                ctx.current_class = None;
                hir_methods.push(HirFunction {
                    name: method_name,
                    params,
                    return_type: return_ty,
                    body,
                    is_async: func.is_async,
                    method_kind: MethodKind::Regular,
                    receiver: method_ft.receiver,
                    decorators: vec![],
                    rust_interop,
                    python_interop: Vec::new(),
                    compiler_intrinsic: None,
                    type_params: Vec::new(),
                });
            }
        }

        return Some(HirClass {
            name: class_name.clone(),
            identity: None,
            fields: vec![("0".to_string(), inner.clone())], // Single wrapped field
            field_defaults: Vec::new(),
            field_default_identities: Vec::new(),
            declaration_metadata: declaration_metadata(ctx, &class_name),
            methods: hir_methods,
            is_hashable: is_hashable_type(inner),
            is_error_type: false,
            kind: HirClassKind::Regular,
            operator_impls: Vec::new(),
            newtype_inner: Some(inner.clone()),
            parent_class: None,
            parent_type: None,
            implements_protocols: Vec::new(),
            type_params: Vec::new(),
            enum_variants: Vec::new(),
            rust_interop: collect_rust_interop_declarations(
                &class_def.decorator_list,
                RustInteropOwner::Class,
                ctx,
                false,
                false,
                false,
            ),
        });
    }

    let (all_fields, method_types) = match &class_ty {
        Type::Class {
            fields, methods, ..
        } => (fields.clone(), methods.clone()),
        _ => return None,
    };

    let parent_class_name =
        crate::lower::descriptor_declarations::data_parent_name(&class_name, ctx);
    let parent_type = crate::lower::descriptor_declarations::data_parent_type(class_def, ctx)
        .or_else(|| {
            parent_class_name
                .as_ref()
                .and_then(|parent_name| ctx.class_types.get(parent_name))
                .cloned()
        });

    // Separate own fields from inherited fields
    // For struct codegen, we only want the child's own fields (parent is embedded)
    let parent_field_names: Vec<String> =
        if let Some(Type::Class { fields: pf, .. }) = parent_type.as_ref() {
            pf.iter().map(|(n, _)| n.clone()).collect()
        } else {
            vec![]
        };

    let own_fields: Vec<(String, Type)> = all_fields
        .iter()
        .filter(|(name, _)| !parent_field_names.contains(name))
        .cloned()
        .collect();

    // Determine if all fields are hashable (primitives: int, float, bool, str)
    let is_hashable = all_fields.iter().all(|(_, ty)| ty.supports_derived_hash());

    let mut hir_methods = Vec::new();
    let mut operator_impls = Vec::new();

    for stmt in &class_def.body {
        if let Stmt::FunctionDef(func) = stmt {
            let method_name = func.name.to_string();

            // Detect @classmethod and @staticmethod decorators
            let is_classmethod = has_decorator(func, "classmethod");
            let is_staticmethod = has_decorator(func, "staticmethod");
            let method_kind = if is_classmethod {
                MethodKind::ClassMethod
            } else if is_staticmethod {
                MethodKind::StaticMethod
            } else {
                MethodKind::Regular
            };
            let declared_receiver = (method_kind == MethodKind::Regular)
                .then(|| declared_receiver_convention(&func.parameters));
            ctx.method_source_ranges
                .insert(format!("{class_name}.{method_name}"), func.name.range());

            // Set current class context for `self` resolution
            ctx.current_class = Some(class_name.clone());
            ctx.self_annotation_available = method_kind == MethodKind::Regular;
            ctx.current_parent_class.clone_from(&parent_class_name);
            ctx.current_parent_type.clone_from(&parent_type);

            // Push a new scope for the method
            ctx.scope.push();

            // For static methods, don't skip any parameter (no self/cls)
            // For class methods, skip `cls` parameter
            // For regular methods, skip `self` parameter
            let skip_count = usize::from(!is_staticmethod); // classmethod has cls, regular has self

            // Define `self` in scope (for regular methods)
            if !is_staticmethod && !is_classmethod {
                let Some(mut receiver) = declared_receiver else {
                    unreachable!("regular method must have a receiver convention");
                };
                // Constructors are emitted as static Rust `new` functions, so
                // their HIR function has no Rust receiver. Their local `self`
                // is nevertheless fresh mutable storage and must remain a
                // valid root for mutating calls during initialization.
                if method_name == "__init__" {
                    receiver = ReceiverConvention::MutableBorrow;
                }
                let receiver_id =
                    ctx.scope
                        .define_receiver("self".to_string(), class_ty.clone(), receiver);
                ctx.method_receiver_bindings
                    .insert(format!("{class_name}.{method_name}"), receiver_id);
            }

            // Define method parameters (skip `self`/`cls`)
            let has_python_interop = has_python_interop_decorator_syntax(&func.decorator_list);
            let mut params = Vec::new();
            for param in func.parameters.args.iter().skip(skip_count) {
                let param_name = param.parameter.name.to_string();
                let param_ty = if let Some(ref ann) = param.parameter.annotation {
                    resolve_annotation_expr(ann, ctx)
                } else {
                    Type::Any
                };
                let convention =
                    class_method_param_convention(param.parameter.convention, &param_ty, ctx);
                ctx.scope
                    .define_parameter(param_name.clone(), param_ty.clone(), convention);
                params.push(HirParam {
                    name: param_name,
                    ty: param_ty,
                    default: class_method_param_default(
                        param.default.as_deref(),
                        has_python_interop,
                        ctx,
                    ),
                    keyword_only: false,
                    convention,
                });
            }
            for param in &func.parameters.kwonlyargs {
                let param_name = param.parameter.name.to_string();
                let param_ty = if let Some(ref ann) = param.parameter.annotation {
                    resolve_annotation_expr(ann, ctx)
                } else {
                    Type::Any
                };
                let convention =
                    class_method_param_convention(param.parameter.convention, &param_ty, ctx);
                ctx.scope
                    .define_parameter(param_name.clone(), param_ty.clone(), convention);
                params.push(HirParam {
                    name: param_name,
                    ty: param_ty,
                    default: class_method_param_default(
                        param.default.as_deref(),
                        has_python_interop,
                        ctx,
                    ),
                    keyword_only: true,
                    convention,
                });
            }

            let return_ty = if method_name == "__init__" {
                Type::None
            } else if let Some(ref ret_ann) = func.returns {
                resolve_annotation_expr(ret_ann, ctx)
            } else {
                Type::None
            };

            // Create a dummy function type for lower_stmts
            let mut method_ft = FunctionType::new(
                params
                    .iter()
                    .map(|p| (p.name.clone(), p.ty.clone()))
                    .collect(),
                return_ty.clone(),
            );
            method_ft.receiver = declared_receiver;

            let mut rust_interop = collect_rust_interop_declarations(
                &func.decorator_list,
                RustInteropOwner::Method,
                ctx,
                has_decorator(func, "blocking_io"),
                has_decorator(func, "cpu_heavy"),
                func.is_async,
            );
            let consumes_rust_receiver = !rust_interop.is_empty()
                && method_kind == MethodKind::Regular
                && receiver_is_owned(&func.parameters);
            for declaration in &mut rust_interop {
                declaration.consumes_receiver = consumes_rust_receiver;
            }
            let mut python_interop = collect_python_method_declarations(
                &func.decorator_list,
                &func.parameters,
                func.is_async,
                ctx,
            );
            // Interop metadata must preserve the source receiver convention.
            // The Rust and Python declaration validators diagnose a consuming
            // policy whose source declaration does not use `own self`.
            if !python_interop.is_empty() && !is_python_opaque {
                ctx.error_with_code_at(
                    sifr_diagnostics::DiagnosticCode::PYIMP_INVALID_TARGET,
                    "`Self` Python declarations require an enclosing `@python.opaque` class"
                        .to_string(),
                    func.name.range(),
                );
            }
            if python_interop.first().is_some_and(|declaration| {
                declaration.kind == sifr_ir::PythonInteropDecoratorKind::Item
            }) && params.len() != 1
            {
                ctx.error_with_code_at(
                    sifr_diagnostics::DiagnosticCode::PYCALL_INVALID_SHAPE,
                    "`@python.item` requires exactly one key parameter after the receiver"
                        .to_string(),
                    func.name.range(),
                );
            }
            validate_python_interop_signature(&mut python_interop, &params, &return_ty, ctx);
            let skips_normal_body_lowering = if has_python_interop {
                classify_python_interop_stub_body(&func.body, true, ctx)
                    .skips_normal_body_lowering()
            } else {
                classify_rust_interop_stub_body(
                    &func.body,
                    has_rust_interop_decorator_syntax(&func.decorator_list),
                    ctx,
                )
                .skips_normal_body_lowering()
            };

            // Lower method body
            let previous_owner = ctx.current_owner.replace(class_name.clone());
            let previous_method = ctx.current_method.replace(method_name.clone());
            let previous_dynamic_python = ctx.current_function_trusts_dynamic_python;
            let previous_async = ctx.current_function_is_async;
            let previous_generator = ctx.current_function_is_generator;
            let previous_async_generator = ctx.current_function_is_async_generator;
            ctx.current_function_trusts_dynamic_python =
                has_decorator(func, "trust_python_dynamic");
            ctx.current_function_is_async = func.is_async;
            ctx.current_function_is_generator = function_body_contains_yield(&func.body);
            ctx.current_function_is_async_generator =
                func.is_async && function_body_contains_yield(&func.body);
            let previous_must_use_bindings = std::mem::take(&mut ctx.live_must_use_bindings);
            let previous_borrowed_params = std::mem::take(&mut ctx.borrowed_params);
            prepare_method_param_ownership(&params, &method_name, skips_normal_body_lowering, ctx);
            let mut body = if skips_normal_body_lowering {
                Vec::new()
            } else {
                lower_function_stmts(&func.body, &method_ft, ctx)
            };
            if method_name == "__init__" {
                if ctx.error_types.contains(&class_name) && !skips_normal_body_lowering {
                    super::error_message_contract::lower_root_initialization(&mut body);
                    super::error_message_contract::validate_constructor(
                        class_def,
                        func,
                        &body,
                        &own_fields,
                        &params,
                        parent_class_name
                            .as_deref()
                            .is_some_and(|parent| parent != "NonSend"),
                        ctx,
                    );
                }
                if let Some(gap) = constructor_uninitialized_storage_at_first_self_use(
                    &body,
                    &own_fields,
                    &params,
                    parent_class_name
                        .as_deref()
                        .is_some_and(|parent| parent != "NonSend"),
                ) {
                    let range = func
                        .body
                        .get(gap.statement_index)
                        .map_or_else(|| func.name.range(), Ranged::range);
                    ownership_diagnostics::constructor_storage_unavailable(
                        ctx,
                        &gap.missing_fields,
                        gap.missing_parent,
                        range,
                    );
                }
            }
            let mut live_must_use = ctx
                .live_must_use_bindings
                .iter()
                .filter(|(name, _)| {
                    ctx.scope.lookup(name.as_str()).is_some() && !ctx.scope.is_moved(name)
                })
                .map(|(name, obligation)| (name.clone(), obligation.clone()))
                .collect::<Vec<_>>();
            live_must_use.sort();
            for (name, obligation) in live_must_use {
                ctx.error_with_code_at(
                    sifr_diagnostics::DiagnosticCode::OWN_USE_AFTER_MOVE,
                    format!(
                        "must-use binding '{name}' owns {obligation} and must be closed or transferred before method exit"
                    ),
                    func.name.range(),
                );
            }
            ctx.live_must_use_bindings = previous_must_use_bindings;
            ctx.borrowed_params = previous_borrowed_params;
            ctx.current_function_is_async = previous_async;
            ctx.current_function_is_generator = previous_generator;
            ctx.current_function_is_async_generator = previous_async_generator;
            ctx.current_function_trusts_dynamic_python = previous_dynamic_python;
            ctx.current_method = previous_method;
            ctx.current_owner = previous_owner;

            ctx.scope.pop();
            ctx.self_annotation_available = false;
            ctx.current_class = None;
            ctx.current_parent_class = None;
            ctx.current_parent_type = None;

            // Collect user-defined decorators (excluding classmethod/staticmethod)
            let method_decorators: Vec<String> = func
                .decorator_list
                .iter()
                .filter_map(|d| {
                    if let Expr::Name(n) = &d.expression {
                        let name = n.id.to_string();
                        if name != "classmethod" && name != "staticmethod" {
                            Some(name)
                        } else {
                            None
                        }
                    } else if let Expr::Attribute(attribute) = &d.expression
                        && let Expr::Name(root) = attribute.value.as_ref()
                    {
                        Some(format!("{}.{}", root.id, attribute.attr))
                    } else {
                        None
                    }
                })
                .collect();

            let hir_func = HirFunction {
                name: if method_name == "__init__" {
                    "new".to_string()
                } else {
                    method_name.clone()
                },
                params,
                return_type: return_ty,
                body,
                is_async: func.is_async,
                method_kind,
                receiver: (method_kind == MethodKind::Regular && method_name != "__init__")
                    .then_some(method_ft.receiver)
                    .flatten(),
                decorators: method_decorators,
                rust_interop,
                python_interop,
                compiler_intrinsic: None,
                type_params: Vec::new(),
            };

            // Separate operator dunders from regular methods
            if is_operator_dunder(&method_name) {
                operator_impls.push((method_name, hir_func));
            } else {
                hir_methods.push(hir_func);
            }
        }
    }

    let rust_interop = collect_rust_interop_declarations(
        &class_def.decorator_list,
        RustInteropOwner::Class,
        ctx,
        false,
        false,
        false,
    );
    opaque::validate_rust_opaque_close_method(class_def, &hir_methods, &rust_interop, ctx);

    super::python_cleanup_validation::validate(class_def, &hir_methods, ctx);

    let is_error = ctx.error_types.contains(&class_name);
    if is_error && own_fields.is_empty() && !hir_methods.iter().any(|method| method.name == "new") {
        if let (Some(parent), Some(parent_ty)) = (&parent_class_name, &parent_type) {
            if let Some(constructor) = super::error_message_contract::inherited_constructor(
                class_def, parent, parent_ty, ctx,
            ) {
                hir_methods.push(constructor);
            }
        }
    }
    if is_error
        && !all_fields
            .iter()
            .all(|(_, field)| field.supports_debug_formatting())
    {
        unsupported_class_declaration(
            ctx,
            &class_name,
            "error fields must implement Debug so the generated Rust error satisfies std::error::Error",
            class_def.range,
        );
        return None;
    }

    // Check which protocols this class satisfies
    let mut implements_protocols = Vec::new();
    for (proto_name, proto_ty) in &ctx.class_types.clone() {
        if let Type::Protocol {
            methods: proto_methods,
            ..
        } = proto_ty
        {
            // Check if class has all required methods
            let satisfies = proto_methods
                .iter()
                .all(|(pname, _pft)| method_types.iter().any(|(mname, _)| mname == pname));
            if satisfies {
                implements_protocols.push(proto_name.clone());
            }
        }
    }

    // Collect PEP 695 type params for the class
    let class_type_params: Vec<String> = if let Some(ref type_params) = class_def.type_params {
        type_params
            .iter()
            .filter_map(|tp| {
                if let sifr_python_ast::TypeParam::TypeVar(tv) = tp {
                    Some(tv.name.to_string())
                } else {
                    None
                }
            })
            .collect()
    } else {
        Vec::new()
    };

    let python_opaque = ctx.python_opaque_classes.get(&class_name).cloned();
    if python_opaque.is_some() && !own_fields.is_empty() {
        ctx.error_with_code_at(
            sifr_diagnostics::DiagnosticCode::PYCALL_INVALID_SHAPE,
            "opaque Python classes cannot declare structural fields".to_string(),
            class_def.range,
        );
    }
    let kind = python_opaque.map_or(HirClassKind::Regular, HirClassKind::PythonOpaque);

    Some(HirClass {
        name: class_name.clone(),
        identity: None,
        fields: own_fields,
        field_defaults: field_defaults(ctx, &class_name),
        field_default_identities: field_default_identities(ctx, &class_name),
        declaration_metadata: declaration_metadata(ctx, &class_name),
        methods: hir_methods,
        is_hashable,
        is_error_type: is_error,
        kind,
        operator_impls,
        newtype_inner: None,
        implements_protocols,
        parent_class: parent_class_name,
        parent_type,
        type_params: class_type_params,
        enum_variants: Vec::new(),
        rust_interop,
    })
}

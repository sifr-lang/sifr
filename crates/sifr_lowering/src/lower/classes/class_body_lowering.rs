use super::{
    collect_enum_variants, get_newtype_inner, get_parent_class, has_decorator, is_enum_class,
    is_operator_dunder, is_protocol_class, lower_stmts, missing_method_param_annotation,
    resolve_annotation_expr, Expr, FunctionType, HirClass, HirClassKind, HirExpr, HirFunction,
    HirParam, HirPattern, HirStmt, HirTupleTargetBinding, LowerCtx, MethodKind, ParamConvention,
    Ranged, Stmt, StmtClassDef, Type,
};
use crate::lower::rust_interop::{collect_rust_interop_declarations, RustInteropOwner};
/// Second pass: lower class method bodies into `HirClass`.
pub(in crate::lower) fn lower_class(
    class_def: &StmtClassDef,
    ctx: &mut LowerCtx,
) -> Option<HirClass> {
    let class_name = class_def.name.to_string();
    let class_ty = ctx.class_types.get(&class_name)?.clone();
    let is_protocol = is_protocol_class(class_def);
    let newtype_inner = get_newtype_inner(class_def);

    // For protocol definitions, emit a HirClass with is_protocol=true
    if is_protocol {
        let methods_sigs = match &class_ty {
            Type::Protocol { methods, .. } => methods.clone(),
            _ => return None,
        };
        // Protocols have no fields, no body to lower -- just method signatures
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
                    decorators: vec![],
                    rust_interop: Vec::new(),
                    type_params: Vec::new(),
                }
            })
            .collect();

        return Some(HirClass {
            name: class_name,
            fields: vec![],
            methods: hir_methods,
            is_hashable: false,
            is_error_type: false,
            kind: HirClassKind::Protocol,
            operator_impls: Vec::new(),
            newtype_inner: None,
            implements_protocols: Vec::new(),
            parent_class: None,
            type_params: Vec::new(),
            enum_variants: Vec::new(),
            rust_interop: collect_rust_interop_declarations(
                &class_def.decorator_list,
                RustInteropOwner::Class,
                ctx,
                false,
                false,
            ),
        });
    }

    // For enum declarations, emit a HirClass with is_enum=true
    if is_enum_class(class_def) {
        let variants = collect_enum_variants(class_def);
        // Lower any methods defined in the enum body
        let mut hir_methods = Vec::new();
        ctx.current_class = Some(class_name.clone());
        for stmt in &class_def.body {
            if let Stmt::FunctionDef(func) = stmt {
                let method_name = func.name.to_string();
                ctx.scope.push();
                ctx.scope.define("self".to_string(), class_ty.clone());

                // Define method parameters (skip `self`)
                let mut params = Vec::new();
                for param in func.parameters.args.iter().skip(1) {
                    let param_name = param.parameter.name.to_string();
                    let param_ty = if let Some(ref ann) = param.parameter.annotation {
                        resolve_annotation_expr(ann, ctx)
                    } else {
                        Type::Any
                    };
                    ctx.scope.define(param_name.clone(), param_ty.clone());
                    params.push(HirParam {
                        name: param_name,
                        ty: param_ty,
                        default: None,
                        keyword_only: false,
                        convention: ParamConvention::default(),
                    });
                }

                let return_ty = if let Some(ref ret_ann) = func.returns {
                    resolve_annotation_expr(ret_ann, ctx)
                } else {
                    Type::None
                };

                let method_ft = FunctionType::new(
                    params
                        .iter()
                        .map(|p| (p.name.clone(), p.ty.clone()))
                        .collect(),
                    return_ty.clone(),
                );

                let previous_owner = ctx.current_owner.replace(class_name.clone());
                let previous_dynamic_python = ctx.current_function_trusts_dynamic_python;
                ctx.current_function_trusts_dynamic_python =
                    has_decorator(func, "trust_python_dynamic");
                let body = lower_stmts(&func.body, &method_ft, ctx);
                ctx.current_function_trusts_dynamic_python = previous_dynamic_python;
                ctx.current_owner = previous_owner;
                ctx.scope.pop();

                hir_methods.push(HirFunction {
                    name: method_name,
                    params,
                    return_type: return_ty,
                    body,
                    is_async: false,
                    method_kind: MethodKind::Regular,
                    decorators: vec![],
                    rust_interop: collect_rust_interop_declarations(
                        &func.decorator_list,
                        RustInteropOwner::Method,
                        ctx,
                        has_decorator(func, "blocking_io"),
                        has_decorator(func, "cpu_heavy"),
                    ),
                    type_params: Vec::new(),
                });
            }
        }
        ctx.current_class = None;
        return Some(HirClass {
            name: class_name,
            fields: vec![],
            methods: hir_methods,
            is_hashable: true,
            is_error_type: false,
            kind: HirClassKind::Enum,
            operator_impls: Vec::new(),
            newtype_inner: None,
            implements_protocols: Vec::new(),
            parent_class: None,
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
                ctx.scope.push();
                ctx.scope.define("self".to_string(), class_ty.clone());
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
                    ctx.scope.define(param_name.clone(), param_ty.clone());
                    params.push(HirParam {
                        name: param_name,
                        ty: param_ty,
                        default: None,
                        keyword_only: false,
                        convention: ParamConvention::default(),
                    });
                }
                let return_ty = if let Some(ref ret_ann) = func.returns {
                    resolve_annotation_expr(ret_ann, ctx)
                } else {
                    Type::None
                };
                let method_ft = FunctionType::new(
                    params
                        .iter()
                        .map(|p| (p.name.clone(), p.ty.clone()))
                        .collect(),
                    return_ty.clone(),
                );
                let previous_owner = ctx.current_owner.replace(class_name.clone());
                let previous_dynamic_python = ctx.current_function_trusts_dynamic_python;
                ctx.current_function_trusts_dynamic_python =
                    has_decorator(func, "trust_python_dynamic");
                let body = lower_stmts(&func.body, &method_ft, ctx);
                ctx.current_function_trusts_dynamic_python = previous_dynamic_python;
                ctx.current_owner = previous_owner;
                ctx.scope.pop();
                ctx.current_class = None;
                hir_methods.push(HirFunction {
                    name: method_name,
                    params,
                    return_type: return_ty,
                    body,
                    is_async: false,
                    method_kind: MethodKind::Regular,
                    decorators: vec![],
                    rust_interop: collect_rust_interop_declarations(
                        &func.decorator_list,
                        RustInteropOwner::Method,
                        ctx,
                        has_decorator(func, "blocking_io"),
                        has_decorator(func, "cpu_heavy"),
                    ),
                    type_params: Vec::new(),
                });
            }
        }

        return Some(HirClass {
            name: class_name,
            fields: vec![("0".to_string(), inner.clone())], // Single wrapped field
            methods: hir_methods,
            is_hashable: is_hashable_type(inner),
            is_error_type: false,
            kind: HirClassKind::Regular,
            operator_impls: Vec::new(),
            newtype_inner: Some(inner.clone()),
            parent_class: None,
            implements_protocols: Vec::new(),
            type_params: Vec::new(),
            enum_variants: Vec::new(),
            rust_interop: collect_rust_interop_declarations(
                &class_def.decorator_list,
                RustInteropOwner::Class,
                ctx,
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

    let parent_class_name = get_parent_class(class_def);

    // Separate own fields from inherited fields
    // For struct codegen, we only want the child's own fields (parent is embedded)
    let parent_field_names: Vec<String> = if let Some(Type::Class { fields: pf, .. }) =
        parent_class_name
            .as_ref()
            .and_then(|parent_name| ctx.class_types.get(parent_name))
    {
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
    let is_hashable = all_fields.iter().all(|(_, ty)| is_hashable_type(ty));

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

            // Set current class context for `self` resolution
            ctx.current_class = Some(class_name.clone());
            ctx.current_parent_class.clone_from(&parent_class_name);

            // Push a new scope for the method
            ctx.scope.push();

            // For static methods, don't skip any parameter (no self/cls)
            // For class methods, skip `cls` parameter
            // For regular methods, skip `self` parameter
            let skip_count = usize::from(!is_staticmethod); // classmethod has cls, regular has self

            // Define `self` in scope (for regular methods)
            if !is_staticmethod && !is_classmethod {
                ctx.scope.define("self".to_string(), class_ty.clone());
            }

            // Define method parameters (skip `self`/`cls`)
            let mut params = Vec::new();
            for param in func.parameters.args.iter().skip(skip_count) {
                let param_name = param.parameter.name.to_string();
                let param_ty = if let Some(ref ann) = param.parameter.annotation {
                    resolve_annotation_expr(ann, ctx)
                } else {
                    Type::Any
                };
                ctx.scope.define(param_name.clone(), param_ty.clone());
                params.push(HirParam {
                    name: param_name,
                    ty: param_ty,
                    default: None,
                    keyword_only: false,
                    convention: ParamConvention::default(),
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
            let method_ft = FunctionType::new(
                params
                    .iter()
                    .map(|p| (p.name.clone(), p.ty.clone()))
                    .collect(),
                return_ty.clone(),
            );

            // Lower method body
            let previous_owner = ctx.current_owner.replace(class_name.clone());
            let previous_dynamic_python = ctx.current_function_trusts_dynamic_python;
            ctx.current_function_trusts_dynamic_python =
                has_decorator(func, "trust_python_dynamic");
            let body = lower_stmts(&func.body, &method_ft, ctx);
            ctx.current_function_trusts_dynamic_python = previous_dynamic_python;
            ctx.current_owner = previous_owner;

            // Determine receiver mutability: if any statement assigns to self.field, it's &mut self
            let _is_mutating = method_name == "__init__" || body_contains_field_assign(&body);

            ctx.scope.pop();
            ctx.current_class = None;
            ctx.current_parent_class = None;

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
                decorators: method_decorators,
                rust_interop: collect_rust_interop_declarations(
                    &func.decorator_list,
                    RustInteropOwner::Method,
                    ctx,
                    has_decorator(func, "blocking_io"),
                    has_decorator(func, "cpu_heavy"),
                ),
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

    let is_error = ctx.error_types.contains(&class_name);

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

    Some(HirClass {
        name: class_name,
        fields: own_fields,
        methods: hir_methods,
        is_hashable,
        is_error_type: is_error,
        kind: HirClassKind::Regular,
        operator_impls,
        newtype_inner: None,
        implements_protocols,
        parent_class: parent_class_name,
        type_params: class_type_params,
        enum_variants: Vec::new(),
        rust_interop: collect_rust_interop_declarations(
            &class_def.decorator_list,
            RustInteropOwner::Class,
            ctx,
            false,
            false,
        ),
    })
}

/// Check if a type is hashable (can derive Hash + Eq).
pub(in crate::lower) fn is_hashable_type(ty: &Type) -> bool {
    match ty {
        Type::Int | Type::Bool | Type::Str | Type::None | Type::BigInt => true,
        Type::Float => false, // f64 doesn't implement Hash
        Type::LiteralInt(_) | Type::LiteralBool(_) | Type::LiteralStr(_) => true,
        Type::Tuple(elems) => elems.iter().all(is_hashable_type),
        Type::Class { fields, .. } => fields.iter().all(|(_, t)| is_hashable_type(t)),
        _ => false,
    }
}

/// Check if a method body contains any field assignments (self.field = ...).
pub(in crate::lower) fn body_contains_field_assign(stmts: &[HirStmt]) -> bool {
    fn stmt_contains_field_assign(stmt: &HirStmt) -> bool {
        match stmt {
            HirStmt::FieldAssign { .. } | HirStmt::NestedFieldAssign { .. } => true,
            HirStmt::TupleUnpack { targets, .. } => targets
                .iter()
                .any(|target| matches!(target.binding, HirTupleTargetBinding::Field { .. })),
            HirStmt::If {
                then_body,
                elif_clauses,
                else_body,
                ..
            } => {
                body_contains_field_assign(then_body)
                    || elif_clauses
                        .iter()
                        .any(|(_, body)| body_contains_field_assign(body))
                    || else_body
                        .as_ref()
                        .is_some_and(|body| body_contains_field_assign(body))
            }
            HirStmt::While {
                body, else_body, ..
            }
            | HirStmt::For {
                body, else_body, ..
            }
            | HirStmt::AsyncFor {
                body, else_body, ..
            } => {
                body_contains_field_assign(body)
                    || else_body
                        .as_ref()
                        .is_some_and(|body| body_contains_field_assign(body))
            }
            HirStmt::TryExcept { body, handlers, .. } => {
                body_contains_field_assign(body)
                    || handlers
                        .iter()
                        .any(|handler| body_contains_field_assign(&handler.body))
            }
            HirStmt::TryFinally { body, finalbody } => {
                body_contains_field_assign(body) || body_contains_field_assign(finalbody)
            }
            HirStmt::With { body, .. } | HirStmt::AsyncWith { body, .. } => {
                body_contains_field_assign(body)
            }
            HirStmt::Match { arms, .. } => {
                arms.iter().any(|arm| body_contains_field_assign(&arm.body))
            }
            _ => false,
        }
    }

    stmts.iter().any(stmt_contains_field_assign)
}

pub(in crate::lower) fn collect_literal_coverage(
    pattern: &HirPattern,
    covered_literal_strs: &mut std::collections::HashSet<String>,
    covered_literal_ints: &mut std::collections::HashSet<i64>,
    covered_literal_bools: &mut std::collections::HashSet<bool>,
) {
    if let HirPattern::Literal { value } = pattern {
        match value {
            HirExpr::StringLiteral(s) => {
                covered_literal_strs.insert(s.clone());
            }
            HirExpr::IntLiteral(n) => {
                covered_literal_ints.insert(*n);
            }
            HirExpr::BoolLiteral(b) => {
                covered_literal_bools.insert(*b);
            }
            _ => {}
        }
    }
}

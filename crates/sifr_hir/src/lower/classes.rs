use crate::hir_nodes::{
    HirClass, HirClassKind, HirExpr, HirFunction, HirParam, HirPattern, HirStmt,
    HirTupleTargetBinding, MethodKind,
};
use sifr_python_ast::{Expr, Number, Stmt, StmtClassDef, UnaryOp};
use sifr_type_system::{FunctionType, ParamConvention, Type};

use super::diagnostics::{
    collect_enum_variants, get_newtype_inner, get_parent_class, has_decorator, is_enum_class,
    is_error_class, is_operator_dunder, is_protocol_class,
};
use super::statements::lower_stmts;
use super::typing_and_functions::resolve_annotation_expr;
use super::{parse_typevar_bound_expr, LowerCtx};

fn class_method_signature<'a>(
    methods: &'a [(String, FunctionType)],
    method_name: &str,
) -> Option<&'a FunctionType> {
    methods.iter().find_map(
        |(name, ft)| {
            if name == method_name {
                Some(ft)
            } else {
                None
            }
        },
    )
}

fn option_member_type(ty: &Type) -> Option<Type> {
    let Type::Union(members) = ty.resolve_alias() else {
        return None;
    };
    let has_none = members
        .iter()
        .any(|member| matches!(member.resolve_alias(), Type::None));
    let non_none: Vec<Type> = members
        .iter()
        .filter(|member| !matches!(member.resolve_alias(), Type::None))
        .cloned()
        .collect();
    if has_none && non_none.len() == 1 {
        non_none.first().cloned()
    } else {
        None
    }
}

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

fn validate_iteration_protocol_methods(
    class_name: &str,
    methods: &[(String, FunctionType)],
    ctx: &mut LowerCtx,
) {
    if let Some(iter_ft) = class_method_signature(methods, "__iter__") {
        if !iter_ft.params.is_empty() {
            ctx.error(format!(
                "class '{class_name}.__iter__' must not declare parameters besides self"
            ));
        }
        if class_iter_element_type(class_name, methods).is_none() {
            ctx.error(format!(
                "class '{class_name}.__iter__' must return 'Iterator[T]' or 'Iterable[T]'"
            ));
        }
    }

    if let Some(next_ft) = class_method_signature(methods, "__next__") {
        if !next_ft.params.is_empty() {
            ctx.error(format!(
                "class '{class_name}.__next__' must not declare parameters besides self"
            ));
        }
        if class_next_element_type(class_name, methods).is_none() {
            ctx.error(format!(
                "class '{class_name}.__next__' must return 'T | None'"
            ));
        }
    }

    if let Some(reversed_ft) = class_method_signature(methods, "__reversed__") {
        if !reversed_ft.params.is_empty() {
            ctx.error(format!(
                "class '{class_name}.__reversed__' must not declare parameters besides self"
            ));
        }
        if class_reversed_element_type(class_name, methods).is_none() {
            ctx.error(format!(
                "class '{class_name}.__reversed__' must return 'Iterator[T]' or 'Iterable[T]'"
            ));
        }
    }

    if let (Some(iter_elem), Some(next_elem)) = (
        class_iter_element_type(class_name, methods),
        class_next_element_type(class_name, methods),
    ) {
        if !next_elem.is_assignable_to(&iter_elem) || !iter_elem.is_assignable_to(&next_elem) {
            ctx.error(format!(
                "class '{class_name}' iteration protocol mismatch: '__iter__' yields '{}' but '__next__' yields '{}'",
                iter_elem.display_name(),
                next_elem.display_name()
            ));
        }
    }

    if let (Some(iter_elem), Some(reversed_elem)) = (
        class_iter_element_type(class_name, methods),
        class_reversed_element_type(class_name, methods),
    ) {
        if !reversed_elem.is_assignable_to(&iter_elem)
            || !iter_elem.is_assignable_to(&reversed_elem)
        {
            ctx.error(format!(
                "class '{class_name}' iteration protocol mismatch: '__iter__' yields '{}' but '__reversed__' yields '{}'",
                iter_elem.display_name(),
                reversed_elem.display_name()
            ));
        }
    }
}

pub(super) fn collect_class_type(
    class_def: &StmtClassDef,
    ctx: &mut LowerCtx,
    validate_iteration_protocols: bool,
) {
    let class_name = class_def.name.to_string();
    let mut fields: Vec<(String, Type)> = Vec::new();
    let mut methods: Vec<(String, FunctionType)> = Vec::new();
    let is_error = is_error_class(class_def);
    let is_protocol = is_protocol_class(class_def);
    let newtype_inner = get_newtype_inner(class_def);

    // PEP 695: register inline type params (class C[T]) as type variables
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

    // For newtype declarations, register as a Newtype type
    if let Some(ref inner) = newtype_inner {
        let newtype_ty = Type::Newtype {
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
            for (vname, vval) in &variants {
                let val = vval.unwrap_or(0);
                if let Some(existing) = seen_values.get(&val) {
                    if vval.is_some() {
                        ctx.error(format!(
                            "enum '{class_name}' has duplicate value {val}: variants '{existing}' and '{vname}'"
                        ));
                    }
                } else if vval.is_some() {
                    seen_values.insert(val, vname.clone());
                }
            }
        }
        let enum_ty = Type::Enum {
            name: class_name.clone(),
            variants: variants.iter().map(|(n, v)| (n.clone(), *v)).collect(),
        };
        ctx.class_types.insert(class_name.clone(), enum_ty.clone());
        // Register each variant as a constant of the enum type
        for (variant_name, _) in &variants {
            ctx.functions.insert(
                format!("{class_name}.{variant_name}"),
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
                        Type::Any
                    };
                    params.push((param_name, param_ty));
                }
                let return_ty = if let Some(ref ret_ann) = func.returns {
                    resolve_annotation_expr(ret_ann, ctx)
                } else {
                    Type::None
                };
                let ft = FunctionType::new(params, return_ty);
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
                        Type::Any
                    };
                    params.push((param_name, param_ty));
                }
                let return_ty = if let Some(ref ret_ann) = func.returns {
                    resolve_annotation_expr(ret_ann, ctx)
                } else {
                    Type::None
                };
                methods.push((method_name, FunctionType::new(params, return_ty)));
            }
        }
        let proto_ty = Type::Protocol {
            name: class_name.clone(),
            methods: methods.clone(),
        };
        ctx.class_types.insert(class_name, proto_ty);
        return;
    }

    // For error types, ensure a 'message' field exists (add if not explicitly declared)
    // This will be checked after collecting all fields

    // Inherit parent fields and methods for single inheritance
    let parent_class_name = get_parent_class(class_def);
    let mut parent_class_chain: Option<String> = None;
    if let Some(ref parent_name) = parent_class_name {
        if let Some(parent_ty) = ctx.class_types.get(parent_name).cloned() {
            if let Type::Class {
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
                // Inherit parent methods
                for (mname, mft) in &parent_methods {
                    methods.push((mname.clone(), mft.clone()));
                }
                parent_class_chain = Some(if let Some(chain) = parent_parent_chain {
                    format!("{parent_name}|{chain}")
                } else {
                    parent_name.clone()
                });
            } else {
                ctx.error(format!("parent type '{parent_name}' is not a class"));
            }
        } else {
            ctx.error(format!("parent class '{parent_name}' not defined"));
        }
    }

    // Register a preliminary class type so self-referential annotations work
    // (e.g., `def distance(self, other: Point)` inside class Point)
    ctx.class_types.insert(
        class_name.clone(),
        Type::Class {
            name: class_name.clone(),
            fields: vec![],
            methods: vec![],
            parent_class: parent_class_chain.clone(),
        },
    );

    let mut field_defaults: Vec<(usize, HirExpr)> = Vec::new();

    for stmt in &class_def.body {
        match stmt {
            // Field annotations: `x: float` or `x: float = 0.0`
            Stmt::AnnAssign(ann) => {
                if let Expr::Name(name) = ann.target.as_ref() {
                    let ty = resolve_annotation_expr(&ann.annotation, ctx);
                    let field_idx = fields.len();
                    fields.push((name.id.clone(), ty));
                    // Collect default value if present (for auto-init default params)
                    if let Some(ref default_expr) = ann.value {
                        if let Some(hir_default) = lower_expr_simple(default_expr) {
                            field_defaults.push((field_idx, hir_default));
                        } else {
                            ctx.error(format!(
                                "class '{class_name}': unsupported default expression for field '{}'",
                                name.id
                            ));
                        }
                    }
                }
            }
            // Method definitions
            Stmt::FunctionDef(func) => {
                let method_name = func.name.to_string();
                if method_name == "__init__" {
                    // Constructor: extract params (skip `self`)
                    let mut params = Vec::new();
                    for param in func.parameters.args.iter().skip(1) {
                        let param_name = param.parameter.name.to_string();
                        let param_ty = if let Some(ref ann) = param.parameter.annotation {
                            resolve_annotation_expr(ann, ctx)
                        } else {
                            ctx.error(format!(
                                "parameter '{param_name}' in {class_name}.__init__ is missing a type annotation"
                            ));
                            Type::Any
                        };
                        params.push((param_name, param_ty));
                    }
                    // Constructor returns the class type (registered below)
                    // We store it as a function for call resolution
                    let constructor_ft = FunctionType::new(params.clone(), Type::None); // placeholder, updated below
                    ctx.functions.insert(class_name.clone(), constructor_ft);

                    // Collect defaults for constructor
                    let mut defaults = Vec::new();
                    for (i, param) in func.parameters.args.iter().skip(1).enumerate() {
                        if let Some(ref default_expr) = param.default {
                            if let Some(hir_default) = lower_expr_simple(default_expr) {
                                defaults.push((i, hir_default));
                            } else {
                                ctx.error(format!(
                                    "class '{class_name}.__init__': unsupported default argument expression for parameter '{}'",
                                    param.parameter.name
                                ));
                            }
                        }
                    }
                    if !defaults.is_empty() {
                        ctx.function_defaults.insert(class_name.clone(), defaults);
                    }
                } else {
                    // Regular/class/static method: extract params
                    // For @staticmethod, don't skip any params
                    // For @classmethod and regular methods, skip `self`/`cls`
                    let is_static = has_decorator(func, "staticmethod");
                    let skip_count = usize::from(!is_static);
                    let mut params = Vec::new();
                    for param in func.parameters.args.iter().skip(skip_count) {
                        let param_name = param.parameter.name.to_string();
                        let param_ty = if let Some(ref ann) = param.parameter.annotation {
                            resolve_annotation_expr(ann, ctx)
                        } else {
                            ctx.error(format!(
                                "parameter '{param_name}' in {class_name}.{method_name} is missing a type annotation"
                            ));
                            Type::Any
                        };
                        params.push((param_name, param_ty));
                    }
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
                                ctx.error(format!(
                                    "class '{class_name}.{method_name}': unsupported default argument expression for parameter '{}'",
                                    param.parameter.name
                                ));
                            }
                        }
                    }
                    if !defaults.is_empty() {
                        ctx.function_defaults
                            .insert(format!("{class_name}.{method_name}"), defaults);
                    }
                    methods.push((method_name, FunctionType::new(params, return_ty)));
                }
            }
            Stmt::Pass(_) => {} // Allow pass in class body
            _ => {
                ctx.error(format!(
                    "unsupported statement in class '{class_name}' body"
                ));
            }
        }
    }

    if validate_iteration_protocols {
        validate_iteration_protocol_methods(&class_name, &methods, ctx);
    }

    let class_ty = Type::Class {
        name: class_name.clone(),
        fields: fields.clone(),
        methods: methods.clone(),
        parent_class: parent_class_chain.clone(),
    };

    // Update the constructor function to return the class type
    if let Some(ft) = ctx.functions.get_mut(&class_name) {
        *ft.return_type = class_ty.clone();
    } else {
        // No __init__ defined -- create a default constructor from fields

        // Validate field ordering: required fields must come before defaulted fields
        let default_indices: std::collections::HashSet<usize> =
            field_defaults.iter().map(|(i, _)| *i).collect();
        let mut seen_default = false;
        for (i, (fname, _)) in fields.iter().enumerate() {
            if default_indices.contains(&i) {
                seen_default = true;
            } else if seen_default {
                ctx.error(format!(
                    "class '{class_name}': required field '{fname}' declared after field with default value"
                ));
            }
        }

        // Inheritance diagnostic: warn when child has own fields but no __init__ and extends a parent
        if parent_class_name.is_some() {
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
                ctx.error(format!(
                    "class '{class_name}' has fields but no __init__; parent fields will not be initialized. \
                     Define an explicit __init__ with super().__init__(...)"
                ));
            }
        }

        let params: Vec<(String, Type)> = fields.clone();
        let ft = FunctionType::new(params, class_ty.clone());
        ctx.functions.insert(class_name.clone(), ft);
        // Store field defaults for the auto-generated constructor
        if !field_defaults.is_empty() {
            ctx.function_defaults
                .insert(class_name.clone(), field_defaults);
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
    }

    ctx.class_types.insert(class_name, class_ty);
}

/// Second pass: lower class method bodies into `HirClass`.
pub(super) fn lower_class(class_def: &StmtClassDef, ctx: &mut LowerCtx) -> Option<HirClass> {
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
                    method_kind: MethodKind::Regular,
                    decorators: vec![],
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
                let body = lower_stmts(&func.body, &method_ft, ctx);
                ctx.current_owner = previous_owner;
                ctx.scope.pop();

                hir_methods.push(HirFunction {
                    name: method_name,
                    params,
                    return_type: return_ty,
                    body,
                    method_kind: MethodKind::Regular,
                    decorators: vec![],
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
            enum_variants: variants,
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
                let body = lower_stmts(&func.body, &method_ft, ctx);
                ctx.current_owner = previous_owner;
                ctx.scope.pop();
                ctx.current_class = None;
                hir_methods.push(HirFunction {
                    name: method_name,
                    params,
                    return_type: return_ty,
                    body,
                    method_kind: MethodKind::Regular,
                    decorators: vec![],
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
            let body = lower_stmts(&func.body, &method_ft, ctx);
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
                        let name = n.id.clone();
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
                method_kind,
                decorators: method_decorators,
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
    })
}

/// Check if a type is hashable (can derive Hash + Eq).
pub(super) fn is_hashable_type(ty: &Type) -> bool {
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
pub(super) fn body_contains_field_assign(stmts: &[HirStmt]) -> bool {
    fn stmt_contains_field_assign(stmt: &HirStmt) -> bool {
        match stmt {
            HirStmt::FieldAssign { .. } => true,
            HirStmt::TupleUnpack { targets, .. } => targets.iter().any(
                |target| matches!(target.binding, HirTupleTargetBinding::Field { .. }),
            ),
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
            HirStmt::With { body, .. } => body_contains_field_assign(body),
            HirStmt::Match { arms, .. } => arms.iter().any(|arm| body_contains_field_assign(&arm.body)),
            _ => false,
        }
    }

    stmts.iter().any(stmt_contains_field_assign)
}

pub(super) fn collect_literal_coverage(
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

/// Lower a simple expression (literal values only) without requiring a full `LowerCtx`.
/// Used for collecting default parameter values in the first pass.
pub(super) fn lower_expr_simple(expr: &Expr) -> Option<HirExpr> {
    match expr {
        Expr::NumberLiteral(num) => match &num.value {
            Number::Int(i) => Some(HirExpr::IntLiteral(i.as_i64()?)),
            Number::Float(f) => Some(HirExpr::FloatLiteral(*f)),
            Number::Complex { .. } => None,
        },
        Expr::StringLiteral(s) => Some(HirExpr::StringLiteral(s.value.to_str().to_string())),
        Expr::BytesLiteral(bytes) => {
            let mut elements = Vec::new();
            for part in &bytes.value {
                for value in part.as_slice() {
                    elements.push(HirExpr::IntLiteral(i64::from(*value)));
                }
            }
            Some(HirExpr::ListLiteral {
                elements,
                ty: Type::Bytes,
            })
        }
        Expr::BooleanLiteral(b) => Some(HirExpr::BoolLiteral(b.value)),
        Expr::NoneLiteral(_) => Some(HirExpr::NoneLiteral),
        Expr::UnaryOp(unary) if matches!(unary.op, UnaryOp::USub) => {
            // Handle negative literals like -1
            if let Some(inner) = lower_expr_simple(&unary.operand) {
                match inner {
                    HirExpr::IntLiteral(v) => Some(HirExpr::IntLiteral(-v)),
                    HirExpr::FloatLiteral(v) => Some(HirExpr::FloatLiteral(-v)),
                    _ => None,
                }
            } else {
                None
            }
        }
        Expr::List(list) => {
            let mut elements = Vec::new();
            let mut elem_ty: Option<Type> = None;
            for elt in &list.elts {
                let lowered = lower_expr_simple(elt)?;
                let lowered_ty = lowered.ty().clone();
                if let Some(ref expected) = elem_ty {
                    if !lowered_ty.is_assignable_to(expected) {
                        return None;
                    }
                } else {
                    elem_ty = Some(lowered_ty);
                }
                elements.push(lowered);
            }
            Some(HirExpr::ListLiteral {
                elements,
                ty: Type::List(Box::new(elem_ty.unwrap_or(Type::Any))),
            })
        }
        Expr::Set(set) => {
            let mut elements = Vec::new();
            let mut elem_ty: Option<Type> = None;
            for elt in &set.elts {
                let lowered = lower_expr_simple(elt)?;
                let lowered_ty = lowered.ty().clone();
                if let Some(ref expected) = elem_ty {
                    if !lowered_ty.is_assignable_to(expected) {
                        return None;
                    }
                } else {
                    elem_ty = Some(lowered_ty);
                }
                elements.push(lowered);
            }
            Some(HirExpr::SetLiteral {
                elements,
                ty: Type::Set(Box::new(elem_ty.unwrap_or(Type::Any))),
            })
        }
        Expr::Dict(dict) => {
            let mut keys = Vec::new();
            let mut values = Vec::new();
            let mut key_ty: Option<Type> = None;
            let mut val_ty: Option<Type> = None;

            for item in &dict.items {
                let key_expr = item.key.as_ref()?;
                let lowered_key = lower_expr_simple(key_expr)?;
                let lowered_val = lower_expr_simple(&item.value)?;
                let lowered_key_ty = lowered_key.ty().clone();
                let lowered_val_ty = lowered_val.ty().clone();

                if let Some(ref expected) = key_ty {
                    if !lowered_key_ty.is_assignable_to(expected) {
                        return None;
                    }
                } else {
                    key_ty = Some(lowered_key_ty);
                }

                if let Some(ref expected) = val_ty {
                    if !lowered_val_ty.is_assignable_to(expected) {
                        return None;
                    }
                } else {
                    val_ty = Some(lowered_val_ty);
                }

                keys.push(lowered_key);
                values.push(lowered_val);
            }

            Some(HirExpr::DictLiteral {
                keys,
                values,
                ty: Type::Dict(
                    Box::new(key_ty.unwrap_or(Type::Any)),
                    Box::new(val_ty.unwrap_or(Type::Any)),
                ),
            })
        }
        Expr::Tuple(tuple) => {
            let mut elements = Vec::new();
            let mut element_types = Vec::new();
            for elt in &tuple.elts {
                let lowered = lower_expr_simple(elt)?;
                element_types.push(lowered.ty().clone());
                elements.push(lowered);
            }
            Some(HirExpr::TupleLiteral {
                elements,
                ty: Type::Tuple(element_types),
            })
        }
        _ => None,
    }
}

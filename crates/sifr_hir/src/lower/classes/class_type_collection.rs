use super::str;
use crate::hir_nodes::HirExpr;
use ruff_text_size::{Ranged, TextRange};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::{Expr, Stmt, StmtClassDef};
use sifr_type_system::{FunctionType, Type};
use std::collections::HashMap;

use super::async_await::coroutine_result_type;
use super::class_field_inference::collect_constructor_self_field_assignments;
use super::diagnostics::{
    collect_enum_variants, get_newtype_inner, get_parent_class, has_decorator, is_enum_class,
    is_error_class, is_protocol_class,
};
use super::protocol_diagnostics;
use super::simple_expr::lower_expr_simple;
use super::typing_and_functions::resolve_annotation_expr;
use super::{parse_typevar_bound_expr, LowerCtx};

pub(super) fn class_method_signature<'a>(
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

pub(super) fn option_member_type(ty: &Type) -> Option<Type> {
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

pub(super) fn missing_method_param_annotation(
    ctx: &mut LowerCtx,
    class_name: &str,
    method_name: &str,
    param_name: &str,
    range: ruff_text_size::TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::TYPE_MISSING_ANNOTATION,
        format!(
            "parameter '{param_name}' in {class_name}.{method_name} is missing a type annotation"
        ),
        range,
    );
}

pub(super) fn invalid_class_base(
    ctx: &mut LowerCtx,
    class_name: &str,
    reason: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::CLASS_INVALID_BASE,
        format!("invalid base class for '{class_name}': {reason}"),
        range,
    );
}

pub(super) fn unsupported_class_declaration(
    ctx: &mut LowerCtx,
    class_name: &str,
    detail: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::CLASS_UNSUPPORTED_DECLARATION,
        format!("unsupported class declaration in '{class_name}': {detail}"),
        range,
    );
}

pub(super) fn parent_class_range(class_def: &StmtClassDef, parent_name: &str) -> TextRange {
    class_def
        .bases()
        .iter()
        .find_map(|base| match base {
            Expr::Name(name) if name.id.as_str() == parent_name => Some(name.range()),
            _ => None,
        })
        .unwrap_or_else(|| class_def.name.range())
}

pub(super) fn class_next_element_type(
    class_name: &str,
    methods: &[(String, FunctionType)],
) -> Option<Type> {
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

pub(super) fn class_iter_element_type(
    class_name: &str,
    methods: &[(String, FunctionType)],
) -> Option<Type> {
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

pub(super) fn class_reversed_element_type(
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

pub(in crate::lower) fn collect_class_type(
    class_def: &StmtClassDef,
    ctx: &mut LowerCtx,
    validate_iteration_protocols: bool,
) {
    let class_name = class_def.name.to_string();
    let mut fields: Vec<(String, Type)> = Vec::new();
    let mut methods: Vec<(String, FunctionType)> = Vec::new();
    let mut method_ranges: HashMap<String, ruff_text_size::TextRange> = HashMap::new();
    let is_error = is_error_class(class_def);
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
                    params.push((param_name, param_ty));
                }
                let return_ty = if let Some(ref ret_ann) = func.returns {
                    resolve_annotation_expr(ret_ann, ctx)
                } else {
                    Type::None
                };
                let ft = FunctionType::new(params, method_signature_return_type(func, return_ty));
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
                    params.push((param_name, param_ty));
                }
                let return_ty = if let Some(ref ret_ann) = func.returns {
                    resolve_annotation_expr(ret_ann, ctx)
                } else {
                    Type::None
                };
                methods.push((
                    method_name,
                    FunctionType::new(params, method_signature_return_type(func, return_ty)),
                ));
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
    let mut own_fields: Vec<(String, ruff_text_size::TextRange)> = Vec::new();
    let mut own_field_default_indices = std::collections::HashSet::new();

    for stmt in &class_def.body {
        match stmt {
            // Field annotations: `x: float` or `x: float = 0.0`
            Stmt::AnnAssign(ann) => {
                if let Expr::Name(name) = ann.target.as_ref() {
                    let ty = resolve_annotation_expr(&ann.annotation, ctx);
                    let field_idx = fields.len();
                    let own_field_idx = own_fields.len();
                    fields.push((name.id.to_string(), ty));
                    own_fields.push((name.id.to_string(), name.range()));
                    // Collect default value if present (for auto-init default params)
                    if let Some(ref default_expr) = ann.value {
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
            // Method definitions
            Stmt::FunctionDef(func) => {
                let method_name = func.name.to_string();
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
                        params.push((param_name, param_ty));
                    }
                    // Constructor return type is registered after field collection.
                    let constructor_ft = FunctionType::new(params.clone(), Type::None);
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
                    // Regular/class/static method: extract params
                    // For @staticmethod, don't skip any params
                    // For @classmethod and regular methods, skip `self`/`cls`
                    let is_static = has_decorator(func, "staticmethod");
                    let skip_count = usize::from(!is_static);
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
                    methods.push((
                        method_name,
                        FunctionType::new(params, method_signature_return_type(func, return_ty)),
                    ));

                    collect_constructor_self_field_assignments(
                        &func.body,
                        &mut method_locals,
                        &mut fields,
                        ctx,
                    );
                }
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

        // Inheritance diagnostic: warn when child has own fields but no __init__ and extends a parent
        if parent_class_name
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

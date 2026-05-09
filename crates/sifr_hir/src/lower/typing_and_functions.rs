use crate::hir_nodes::{HirFunction, HirParam, MethodKind};
use ruff_text_size::Ranged;
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::{
    AstParamConvention, AstParamMutability, AstParamOwnership, Expr, Number, Operator,
    StmtFunctionDef,
};
use sifr_type_system::infer::resolve_type_annotation;
use sifr_type_system::{
    make_union, FunctionType, OwnershipKind, ParamConvention, ParamMutability, ParamOwnership, Type,
};
use std::collections::HashMap;

use super::diagnostics::{format_type_name, is_valid_error_type};
use super::expressions::lower_expr;
use super::function_flow::{collect_yield_types, infer_function_return_type};
use super::nonlocal_support::collect_declared_nonlocals;
use super::simple_expr::lower_expr_simple;
use super::statements::lower_stmts;
use super::{substitute_type_vars, LowerCtx};

pub(super) fn register_builtins(ctx: &mut LowerCtx) {
    // print() accepts any single argument and returns None
    ctx.functions.insert(
        "print".to_string(),
        FunctionType::all_borrow(vec![("value".to_string(), Type::Any)], Type::None),
    );

    // Register built-in error classes.
    // These are compiler built-ins (like int, str, bool) — available without imports.
    // Error hierarchy: Error -> {IOError, ParseError, ValueError, ...}
    //                  IOError -> {FileNotFoundError, PermissionError, ...}

    // --- Base error class ---
    {
        let msg_fields = vec![("message".to_string(), Type::Str)];
        let class_ty = Type::Class {
            name: "Error".to_string(),
            fields: msg_fields.clone(),
            methods: vec![],
            parent_class: None,
        };
        ctx.class_types
            .insert("Error".to_string(), class_ty.clone());
        ctx.error_types.insert("Error".to_string());
        ctx.functions.insert(
            "Error".to_string(),
            FunctionType::new(vec![("message".to_string(), Type::Str)], class_ty),
        );
    }

    // --- Mid-level error classes (parent: Error) ---
    // IOError has an extra `kind` field for subclass dispatch; constructor accepts only message
    {
        let fields = vec![
            ("message".to_string(), Type::Str),
            ("kind".to_string(), Type::Str),
        ];
        let class_ty = Type::Class {
            name: "IOError".to_string(),
            fields: fields.clone(),
            methods: vec![],
            parent_class: Some("Error".to_string()),
        };
        ctx.class_types
            .insert("IOError".to_string(), class_ty.clone());
        ctx.error_types.insert("IOError".to_string());
        ctx.functions.insert(
            "IOError".to_string(),
            FunctionType::new(vec![("message".to_string(), Type::Str)], class_ty),
        );
    }
    let other_mid_level_errors = [
        "ParseError",
        "ValueError",
        "DivisionError",
        "KeyError",
        "OverflowError",
        "DecimalConversionError",
    ];
    for &error_name in &other_mid_level_errors {
        let fields = vec![("message".to_string(), Type::Str)];
        let class_ty = Type::Class {
            name: error_name.to_string(),
            fields: fields.clone(),
            methods: vec![],
            parent_class: Some("Error".to_string()),
        };
        ctx.class_types
            .insert(error_name.to_string(), class_ty.clone());
        ctx.error_types.insert(error_name.to_string());
        ctx.functions.insert(
            error_name.to_string(),
            FunctionType::new(vec![("message".to_string(), Type::Str)], class_ty),
        );
    }

    // --- JSON integer boundary errors (parent: Error, profile-specific fields) ---
    {
        let fields = vec![
            ("message".to_string(), Type::Str),
            ("path".to_string(), Type::Str),
            ("profile".to_string(), Type::Str),
        ];
        let class_ty = Type::Class {
            name: "JsonIntegerRangeError".to_string(),
            fields: fields.clone(),
            methods: vec![],
            parent_class: Some("Error".to_string()),
        };
        ctx.class_types
            .insert("JsonIntegerRangeError".to_string(), class_ty.clone());
        ctx.error_types.insert("JsonIntegerRangeError".to_string());
        ctx.functions.insert(
            "JsonIntegerRangeError".to_string(),
            FunctionType::new(vec![("message".to_string(), Type::Str)], class_ty),
        );
    }
    {
        let fields = vec![
            ("message".to_string(), Type::Str),
            ("limit".to_string(), Type::Int),
        ];
        let class_ty = Type::Class {
            name: "JsonLimitError".to_string(),
            fields: fields.clone(),
            methods: vec![],
            parent_class: Some("Error".to_string()),
        };
        ctx.class_types
            .insert("JsonLimitError".to_string(), class_ty.clone());
        ctx.error_types.insert("JsonLimitError".to_string());
        ctx.functions.insert(
            "JsonLimitError".to_string(),
            FunctionType::new(vec![("message".to_string(), Type::Str)], class_ty),
        );
    }

    // --- IOError subclasses (parent: IOError) ---
    let io_subclasses = [
        "FileNotFoundError",
        "PermissionError",
        "FileExistsError",
        "IsADirectoryError",
        "NotADirectoryError",
        "DirectoryNotEmptyError",
    ];
    for &error_name in &io_subclasses {
        let fields = vec![("message".to_string(), Type::Str)];
        let class_ty = Type::Class {
            name: error_name.to_string(),
            fields: fields.clone(),
            methods: vec![],
            parent_class: Some("IOError".to_string()),
        };
        ctx.class_types
            .insert(error_name.to_string(), class_ty.clone());
        ctx.error_types.insert(error_name.to_string());
        ctx.functions.insert(
            error_name.to_string(),
            FunctionType::new(vec![("message".to_string(), Type::Str)], class_ty),
        );
    }

    // --- JSONDecodeError (parent: Error, extra fields: line, column) ---
    // Constructor accepts only message; line/column are populated by intrinsics
    {
        let fields = vec![
            ("message".to_string(), Type::Str),
            ("line".to_string(), Type::Int),
            ("column".to_string(), Type::Int),
        ];
        let class_ty = Type::Class {
            name: "JSONDecodeError".to_string(),
            fields: fields.clone(),
            methods: vec![],
            parent_class: Some("Error".to_string()),
        };
        ctx.class_types
            .insert("JSONDecodeError".to_string(), class_ty.clone());
        ctx.error_types.insert("JSONDecodeError".to_string());
        ctx.functions.insert(
            "JSONDecodeError".to_string(),
            FunctionType::new(vec![("message".to_string(), Type::Str)], class_ty),
        );
    }

    // --- TOMLDecodeError (parent: Error, extra fields: line, column) ---
    // Constructor accepts only message; line/column are populated by intrinsics
    {
        let fields = vec![
            ("message".to_string(), Type::Str),
            ("line".to_string(), Type::Int),
            ("column".to_string(), Type::Int),
        ];
        let class_ty = Type::Class {
            name: "TOMLDecodeError".to_string(),
            fields: fields.clone(),
            methods: vec![],
            parent_class: Some("Error".to_string()),
        };
        ctx.class_types
            .insert("TOMLDecodeError".to_string(), class_ty.clone());
        ctx.error_types.insert("TOMLDecodeError".to_string());
        ctx.functions.insert(
            "TOMLDecodeError".to_string(),
            FunctionType::new(vec![("message".to_string(), Type::Str)], class_ty),
        );
    }

    // --- RegexError (parent: Error, extra field: detail) ---
    {
        let fields = vec![
            ("message".to_string(), Type::Str),
            ("detail".to_string(), Type::Str),
        ];
        let class_ty = Type::Class {
            name: "RegexError".to_string(),
            fields: fields.clone(),
            methods: vec![],
            parent_class: Some("Error".to_string()),
        };
        ctx.class_types
            .insert("RegexError".to_string(), class_ty.clone());
        ctx.error_types.insert("RegexError".to_string());
        // Constructor accepts only message; detail is populated by intrinsics
        ctx.functions.insert(
            "RegexError".to_string(),
            FunctionType::new(vec![("message".to_string(), Type::Str)], class_ty),
        );
    }

    // Build error hierarchy for exhaustiveness checking
    ctx.error_hierarchy.insert(
        "IOError".to_string(),
        vec![
            "FileNotFoundError".to_string(),
            "PermissionError".to_string(),
            "FileExistsError".to_string(),
            "IsADirectoryError".to_string(),
            "NotADirectoryError".to_string(),
            "DirectoryNotEmptyError".to_string(),
        ],
    );
}

pub(super) fn ast_convention_to_param(conv: AstParamConvention, ty: &Type) -> ParamConvention {
    let ownership = match conv.ownership {
        AstParamOwnership::Own => ParamOwnership::Own,
        AstParamOwnership::Borrow => {
            if matches!(ty, Type::TypeVar(_)) {
                ParamOwnership::Borrow
            } else if ty.ownership() == OwnershipKind::Copy {
                ParamOwnership::Own
            } else {
                ParamOwnership::Borrow
            }
        }
    };
    let mutability = match conv.mutability {
        AstParamMutability::Immutable => ParamMutability::Immutable,
        AstParamMutability::Mutable => ParamMutability::Mutable,
    };
    ParamConvention::new(ownership, mutability)
}

pub(super) fn function_type_to_callable_type(ft: &FunctionType) -> Type {
    Type::Callable(
        ft.params.iter().map(|(_, ty, _)| ty.clone()).collect(),
        ft.params
            .iter()
            .map(|(_, _, convention)| *convention)
            .collect(),
        ft.return_type.clone(),
    )
}

fn collect_function_defaults(
    func: &StmtFunctionDef,
    ctx: &mut LowerCtx,
) -> Vec<(usize, crate::hir_nodes::HirExpr)> {
    let mut defaults = Vec::new();

    for (index, param) in func.parameters.args.iter().enumerate() {
        if let Some(default_expr) = &param.default {
            if let Some(hir_default) = lower_expr_simple(default_expr) {
                defaults.push((index, hir_default));
            } else {
                ctx.error_with_code_at(
                    DiagnosticCode::TYPE_UNSUPPORTED_DEFAULT_ARGUMENT,
                    format!(
                        "function '{}': unsupported default argument expression for parameter '{}'",
                        func.name, param.parameter.name
                    ),
                    default_expr.range(),
                );
            }
        }
    }

    let regular_count = func.parameters.args.len() + usize::from(func.parameters.vararg.is_some());
    for (index, param) in func.parameters.kwonlyargs.iter().enumerate() {
        if let Some(default_expr) = &param.default {
            if let Some(hir_default) = lower_expr_simple(default_expr) {
                defaults.push((regular_count + index, hir_default));
            } else {
                ctx.error_with_code_at(
                    DiagnosticCode::TYPE_UNSUPPORTED_DEFAULT_ARGUMENT,
                    format!(
                        "function '{}': unsupported default argument expression for parameter '{}'",
                        func.name, param.parameter.name
                    ),
                    default_expr.range(),
                );
            }
        }
    }

    defaults
}

pub(super) fn register_local_function_signature(
    func: &StmtFunctionDef,
    ft: FunctionType,
    ctx: &mut LowerCtx,
) -> FunctionType {
    let function_name = func.name.to_string();
    let callable_ty = function_type_to_callable_type(&ft);
    let defaults = collect_function_defaults(func, ctx);

    if !defaults.is_empty() {
        ctx.function_defaults
            .insert(function_name.clone(), defaults);
    }
    ctx.scope.define(function_name.clone(), callable_ty);
    ctx.functions.insert(function_name.clone(), ft.clone());
    if func.parameters.vararg.is_some() {
        ctx.vararg_functions
            .insert(function_name, func.parameters.args.len());
    }

    ft
}

pub(super) fn register_local_function_symbol(
    func: &StmtFunctionDef,
    ctx: &mut LowerCtx,
) -> FunctionType {
    if let Some(existing) = ctx.functions.get(func.name.as_str()) {
        if ctx.scope.lookup(func.name.as_str()).is_some() {
            return existing.clone();
        }
    }

    register_local_function_signature(func, extract_function_type(func, ctx), ctx)
}

pub(super) fn extract_function_type(func: &StmtFunctionDef, ctx: &mut LowerCtx) -> FunctionType {
    let mut params: Vec<(String, Type, ParamConvention)> = Vec::new();

    for param in &func.parameters.args {
        let name = param.parameter.name.to_string();
        let ty = if let Some(annotation) = &param.parameter.annotation {
            resolve_annotation_expr(annotation, ctx)
        } else {
            ctx.error_with_code_at(
                DiagnosticCode::TYPE_MISSING_ANNOTATION,
                format!(
                    "parameter '{}' in function '{}' is missing a type annotation",
                    name, func.name
                ),
                param.parameter.name.range(),
            );
            Type::Any
        };
        let conv = ast_convention_to_param(param.parameter.convention, &ty);
        params.push((name, ty, conv));
    }

    // Vararg parameter (*args) -- becomes Vec<T>
    if let Some(ref vararg) = func.parameters.vararg {
        let name = vararg.name.to_string();
        let elem_ty = if let Some(ref annotation) = vararg.annotation {
            resolve_annotation_expr(annotation, ctx)
        } else {
            ctx.error_with_code_at(
                DiagnosticCode::TYPE_MISSING_ANNOTATION,
                format!(
                    "vararg parameter '{}' in function '{}' is missing a type annotation",
                    name, func.name
                ),
                vararg.name.range(),
            );
            Type::Any
        };
        let list_ty = Type::List(Box::new(elem_ty));
        let conv = ast_convention_to_param(vararg.convention, &list_ty);
        params.push((name, list_ty, conv));
    }

    // Also include keyword-only parameters
    for param in &func.parameters.kwonlyargs {
        let name = param.parameter.name.to_string();
        let ty = if let Some(annotation) = &param.parameter.annotation {
            resolve_annotation_expr(annotation, ctx)
        } else {
            ctx.error_with_code_at(
                DiagnosticCode::TYPE_MISSING_ANNOTATION,
                format!(
                    "parameter '{}' in function '{}' is missing a type annotation",
                    name, func.name
                ),
                param.parameter.name.range(),
            );
            Type::Any
        };
        let conv = ast_convention_to_param(param.parameter.convention, &ty);
        params.push((name, ty, conv));
    }

    let return_type = if let Some(returns) = &func.returns {
        resolve_annotation_expr(returns, ctx)
    } else {
        Type::Any // marker for "needs inference" -- will be inferred from body
    };

    FunctionType {
        params,
        return_type: Box::new(return_type),
    }
}

fn invalid_type_annotation(
    ctx: &mut LowerCtx,
    message: impl Into<String>,
    range: ruff_text_size::TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::TYPE_INVALID_ANNOTATION,
        message.into(),
        range,
    );
}

fn unknown_type(ctx: &mut LowerCtx, name: &str, range: ruff_text_size::TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::NAME_UNKNOWN_TYPE,
        format!("unknown type: '{name}'"),
        range,
    );
}

fn reserved_integer_width_name(ctx: &mut LowerCtx, name: &str, range: ruff_text_size::TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::INT_RESERVED_WIDTH_NAME,
        format!("reserved integer width name '{name}' is not supported yet"),
        range,
    );
}

pub(super) fn resolve_annotation_expr(expr: &Expr, ctx: &mut LowerCtx) -> Type {
    match expr {
        Expr::Name(name) => {
            // Check type variables first (e.g., T from TypeVar)
            if ctx.type_vars.contains(name.id.as_str()) {
                return Type::TypeVar(name.id.to_string());
            }
            // Check type aliases first
            if let Some(alias_ty) = ctx.scope.lookup_type_alias(&name.id) {
                return alias_ty.clone();
            }
            // Check class types
            if let Some(class_ty) = ctx.class_types.get(name.id.as_str()) {
                return class_ty.clone();
            }
            if matches!(name.id.as_str(), "int128" | "uint128") {
                reserved_integer_width_name(ctx, &name.id, name.range());
                return Type::Any;
            }
            if name.id.as_str() == "bigint" {
                ctx.warn_bigint_transition_alias(name.range());
                return Type::BigInt;
            }
            resolve_type_annotation(&name.id).unwrap_or_else(|| {
                unknown_type(ctx, &name.id, name.range());
                Type::Any
            })
        }
        Expr::NoneLiteral(_) => Type::None,
        // Union type syntax: int | str (parsed as BinOp with BitOr)
        Expr::BinOp(binop) if matches!(binop.op, Operator::BitOr) => {
            let left = resolve_annotation_expr(&binop.left, ctx);
            let right = resolve_annotation_expr(&binop.right, ctx);
            make_union(vec![left, right])
        }
        // Literal string in type position: "GET" | "POST"
        Expr::StringLiteral(s) => Type::LiteralStr(s.value.to_str().to_string()),
        // Literal int in type position: 200 | 404
        Expr::NumberLiteral(num) => {
            if let Number::Int(i) = &num.value {
                if let Some(val) = i.as_i64() {
                    Type::LiteralInt(val)
                } else {
                    invalid_type_annotation(
                        ctx,
                        "integer literal too large for type annotation",
                        num.range(),
                    );
                    Type::Any
                }
            } else {
                invalid_type_annotation(
                    ctx,
                    "only integer literals are supported in type annotations",
                    num.range(),
                );
                Type::Any
            }
        }
        // Literal bool in type position: True | False
        Expr::BooleanLiteral(b) => Type::LiteralBool(b.value),
        Expr::Subscript(sub) => {
            // Handle generic type annotations: list[int], dict[str, int], tuple[int, str]
            let base_name = if let Expr::Name(n) = sub.value.as_ref() {
                n.id.to_string()
            } else {
                invalid_type_annotation(ctx, "unsupported type annotation base", sub.value.range());
                return Type::Any;
            };
            match base_name.as_str() {
                "list" => {
                    let elem_ty = resolve_annotation_expr(&sub.slice, ctx);
                    Type::List(Box::new(elem_ty))
                }
                "set" => {
                    let elem_ty = resolve_annotation_expr(&sub.slice, ctx);
                    Type::Set(Box::new(elem_ty))
                }
                "dict" => {
                    // dict[K, V] -- the slice is a Tuple expression
                    if let Expr::Tuple(tuple) = sub.slice.as_ref() {
                        if tuple.elts.len() != 2 {
                            invalid_type_annotation(
                                ctx,
                                "dict type annotation requires exactly 2 type parameters",
                                sub.slice.range(),
                            );
                            return Type::Any;
                        }
                        let key_ty = resolve_annotation_expr(&tuple.elts[0], ctx);
                        let val_ty = resolve_annotation_expr(&tuple.elts[1], ctx);
                        Type::Dict(Box::new(key_ty), Box::new(val_ty))
                    } else {
                        invalid_type_annotation(
                            ctx,
                            "dict type annotation requires [K, V] syntax",
                            sub.slice.range(),
                        );
                        Type::Any
                    }
                }
                "tuple" => {
                    // tuple[A, B, ...] -- the slice is a Tuple expression
                    if let Expr::Tuple(tuple) = sub.slice.as_ref() {
                        let elem_types: Vec<Type> = tuple
                            .elts
                            .iter()
                            .map(|e| resolve_annotation_expr(e, ctx))
                            .collect();
                        Type::Tuple(elem_types)
                    } else {
                        // Single-element tuple: tuple[int]
                        let elem_ty = resolve_annotation_expr(&sub.slice, ctx);
                        Type::Tuple(vec![elem_ty])
                    }
                }
                "Iterable" => {
                    let elem_ty = resolve_annotation_expr(&sub.slice, ctx);
                    Type::Iterable(Box::new(elem_ty))
                }
                "Iterator" => {
                    let elem_ty = resolve_annotation_expr(&sub.slice, ctx);
                    Type::Iterator(Box::new(elem_ty))
                }
                "Awaitable" => {
                    let result_ty = resolve_annotation_expr(&sub.slice, ctx);
                    Type::Awaitable(Box::new(result_ty))
                }
                "Coroutine" | "Task" | "TaskResult" | "BlockingTask" | "AsyncIterator"
                | "AsyncGenerator" => {
                    let Expr::Tuple(tuple) = sub.slice.as_ref() else {
                        invalid_type_annotation(
                            ctx,
                            format!("{base_name} type annotation requires [T, E] syntax"),
                            sub.slice.range(),
                        );
                        return Type::Any;
                    };
                    if tuple.elts.len() != 2 {
                        invalid_type_annotation(
                            ctx,
                            format!(
                                "{base_name} type annotation requires exactly 2 type parameters"
                            ),
                            sub.slice.range(),
                        );
                        return Type::Any;
                    }
                    let ok_ty = resolve_annotation_expr(&tuple.elts[0], ctx);
                    let err_ty = resolve_annotation_expr(&tuple.elts[1], ctx);
                    match base_name.as_str() {
                        "Coroutine" => Type::Coroutine(Box::new(ok_ty), Box::new(err_ty)),
                        "Task" => Type::Task(Box::new(ok_ty), Box::new(err_ty)),
                        "TaskResult" => Type::TaskResult(Box::new(ok_ty), Box::new(err_ty)),
                        "BlockingTask" => Type::BlockingTask(Box::new(ok_ty), Box::new(err_ty)),
                        "AsyncIterator" => Type::AsyncIterator(Box::new(ok_ty), Box::new(err_ty)),
                        "AsyncGenerator" => Type::AsyncGenerator(Box::new(ok_ty), Box::new(err_ty)),
                        _ => Type::Any,
                    }
                }
                "Reversible" => {
                    let elem_ty = resolve_annotation_expr(&sub.slice, ctx);
                    Type::reversible(elem_ty)
                }
                "Result" => {
                    // Result[T, E] -- the slice is a Tuple expression
                    if let Expr::Tuple(tuple) = sub.slice.as_ref() {
                        if tuple.elts.len() != 2 {
                            invalid_type_annotation(
                                ctx,
                                "Result type annotation requires exactly 2 type parameters",
                                sub.slice.range(),
                            );
                            return Type::Any;
                        }
                        let ok_ty = resolve_annotation_expr(&tuple.elts[0], ctx);
                        let err_ty = resolve_annotation_expr(&tuple.elts[1], ctx);
                        // Enforce: E must be a class extending Error
                        if !is_valid_error_type(&err_ty, ctx) {
                            let err_name = format_type_name(&err_ty);
                            ctx.error_with_code_at(
                                DiagnosticCode::RESULT_INVALID_ERROR_TYPE,
                                format!(
                                "`{}` is not a valid error type in Result — use a class extending Error, e.g. `Result[{}, ValueError]`",
                                err_name,
                                format_type_name(&ok_ty),
                                ),
                                tuple.elts[1].range(),
                            );
                            return Type::Any;
                        }
                        Type::Result(Box::new(ok_ty), Box::new(err_ty))
                    } else {
                        invalid_type_annotation(
                            ctx,
                            "Result type annotation requires [T, E] syntax",
                            sub.slice.range(),
                        );
                        Type::Any
                    }
                }
                "Option" => {
                    // Option[T] -> T | None (sugar)
                    let inner_ty = resolve_annotation_expr(&sub.slice, ctx);
                    make_union(vec![inner_ty, Type::None])
                }
                "TypeGuard" => {
                    // TypeGuard[T] -- type predicate return type

                    // Store as the inner type; the function signature handler
                    // will recognize TypeGuard and mark it as a type predicate
                    resolve_annotation_expr(&sub.slice, ctx)
                }
                "Callable" => {
                    // Callable[[param_types], return_type]
                    // The slice is a Tuple of [List[param_types], return_type]
                    if let Expr::Tuple(tuple) = sub.slice.as_ref() {
                        if tuple.elts.len() != 2 {
                            invalid_type_annotation(
                                ctx,
                                "Callable type requires exactly 2 type parameters: [[param_types], return_type]",
                                sub.slice.range(),
                            );
                            return Type::Any;
                        }
                        // First element should be a list of parameter types
                        let param_types = if let Expr::List(list) = &tuple.elts[0] {
                            list.elts
                                .iter()
                                .map(|e| resolve_annotation_expr(e, ctx))
                                .collect::<Vec<_>>()
                        } else {
                            invalid_type_annotation(
                                ctx,
                                "Callable parameter types must be a list: Callable[[int, str], bool]",
                                tuple.elts[0].range(),
                            );
                            return Type::Any;
                        };
                        let return_type = resolve_annotation_expr(&tuple.elts[1], ctx);
                        let conventions = param_types
                            .iter()
                            .map(|ty| {
                                if ty.ownership() == OwnershipKind::Copy {
                                    ParamConvention::own()
                                } else {
                                    ParamConvention::borrow()
                                }
                            })
                            .collect();
                        Type::Callable(param_types, conventions, Box::new(return_type))
                    } else {
                        invalid_type_annotation(
                            ctx,
                            "Callable type requires [[param_types], return_type] syntax",
                            sub.slice.range(),
                        );
                        Type::Any
                    }
                }
                _ => {
                    // Check if it's a generic type alias (e.g., Pair[int])
                    if let Some((alias_params, alias_body)) =
                        ctx.scope.lookup_generic_type_alias(&base_name).cloned()
                    {
                        let type_args: Vec<Type> = match sub.slice.as_ref() {
                            Expr::Tuple(tup) => tup
                                .elts
                                .iter()
                                .map(|e| resolve_annotation_expr(e, ctx))
                                .collect(),
                            single => vec![resolve_annotation_expr(single, ctx)],
                        };
                        if alias_params.len() != type_args.len() {
                            invalid_type_annotation(
                                ctx,
                                format!(
                                "generic type alias '{base_name}' expects {} type argument(s), got {}",
                                alias_params.len(),
                                type_args.len()
                                ),
                                sub.slice.range(),
                            );
                            return Type::Any;
                        }
                        let mut bindings = HashMap::new();
                        for (i, tp) in alias_params.iter().enumerate() {
                            if let Some(arg) = type_args.get(i) {
                                bindings.insert(tp.clone(), arg.clone());
                            }
                        }
                        return substitute_type_vars(&alias_body, &bindings);
                    }
                    // Check if it's a generic class instantiation (e.g., Stack[int])
                    if let Some(class_ty) = ctx.class_types.get(&base_name).cloned() {
                        // Resolve type arguments and substitute into the class type
                        let type_args: Vec<Type> = match sub.slice.as_ref() {
                            Expr::Tuple(tup) => tup
                                .elts
                                .iter()
                                .map(|e| resolve_annotation_expr(e, ctx))
                                .collect(),
                            single => vec![resolve_annotation_expr(single, ctx)],
                        };
                        // Build substitution map from class type params to concrete args
                        if let Type::Class {
                            ref fields,
                            ref methods,
                            ref parent_class,
                            ..
                        } = class_ty
                        {
                            let class_type_params = ctx
                                .class_declared_type_params
                                .get(&base_name)
                                .cloned()
                                .unwrap_or_default();
                            if !type_args.is_empty() {
                                if class_type_params.is_empty() {
                                    invalid_type_annotation(
                                        ctx,
                                        format!(
                                        "class '{base_name}' does not declare type parameters; use `class {base_name}[T]: ...`"
                                        ),
                                        sub.value.range(),
                                    );
                                    return Type::Any;
                                }
                                if class_type_params.len() != type_args.len() {
                                    invalid_type_annotation(
                                        ctx,
                                        format!(
                                        "generic class '{base_name}' expects {} type argument(s), got {}",
                                        class_type_params.len(),
                                        type_args.len()
                                        ),
                                        sub.slice.range(),
                                    );
                                    return Type::Any;
                                }
                                let mut bindings = HashMap::new();
                                for (tp, arg) in class_type_params.iter().zip(type_args.iter()) {
                                    bindings.insert(tp.clone(), arg.clone());
                                }
                                let subst_fields: Vec<(String, Type)> = fields
                                    .iter()
                                    .map(|(n, t)| (n.clone(), substitute_type_vars(t, &bindings)))
                                    .collect();
                                let subst_methods: Vec<(String, FunctionType)> = methods
                                    .iter()
                                    .map(|(n, ft)| {
                                        let subst_params: Vec<(String, Type, ParamConvention)> = ft
                                            .params
                                            .iter()
                                            .map(|(pn, pt, pc)| {
                                                (
                                                    pn.clone(),
                                                    substitute_type_vars(pt, &bindings),
                                                    *pc,
                                                )
                                            })
                                            .collect();
                                        let subst_ret =
                                            substitute_type_vars(&ft.return_type, &bindings);
                                        (
                                            n.clone(),
                                            FunctionType {
                                                params: subst_params,
                                                return_type: Box::new(subst_ret),
                                            },
                                        )
                                    })
                                    .collect();
                                return Type::Class {
                                    name: base_name.clone(),
                                    fields: subst_fields,
                                    methods: subst_methods,
                                    parent_class: parent_class.clone(),
                                };
                            }
                        }
                        class_ty
                    } else {
                        unknown_type(ctx, &base_name, sub.value.range());
                        Type::Any
                    }
                }
            }
        }
        _ => {
            invalid_type_annotation(ctx, "unsupported type annotation expression", expr.range());
            Type::Any
        }
    }
}

pub(super) fn lower_function(func: &StmtFunctionDef, ctx: &mut LowerCtx) -> Option<HirFunction> {
    let ft = ctx.functions.get::<str>(func.name.as_ref())?.clone();

    ctx.enter_function_scope(collect_declared_nonlocals(&func.body));

    // Define parameters in scope, handling defaults
    let mut params = Vec::new();

    // Regular args
    for (i, param_def) in func.parameters.args.iter().enumerate() {
        let name = param_def.parameter.name.to_string();
        let ty = ft
            .params
            .get(i)
            .map(|(_, t, _)| t.clone())
            .unwrap_or(Type::Any);
        let convention = ast_convention_to_param(param_def.parameter.convention, &ty);
        ctx.scope
            .define_parameter(name.clone(), ty.clone(), convention.is_mutable());

        let default = param_def.default.as_ref().and_then(|d| lower_expr(d, ctx));

        params.push(HirParam {
            name,
            ty,
            default,
            keyword_only: false,
            convention,
        });
    }

    // Vararg parameter (*args) -- becomes Vec<T>
    if let Some(ref vararg) = func.parameters.vararg {
        let name = vararg.name.to_string();
        let regular_count = func.parameters.args.len();
        let ty = ft
            .params
            .get(regular_count)
            .map(|(_, t, _)| t.clone())
            .unwrap_or(Type::Any);
        let convention = ast_convention_to_param(vararg.convention, &ty);
        ctx.scope
            .define_parameter(name.clone(), ty.clone(), convention.is_mutable());
        params.push(HirParam {
            name,
            ty,
            default: None,
            keyword_only: false,
            convention,
        });
    }

    // Keyword-only args (after * separator)
    let regular_count = func.parameters.args.len() + usize::from(func.parameters.vararg.is_some());
    for (i, param_def) in func.parameters.kwonlyargs.iter().enumerate() {
        let name = param_def.parameter.name.to_string();
        let ty = ft
            .params
            .get(regular_count + i)
            .map(|(_, t, _)| t.clone())
            .unwrap_or(Type::Any);
        let convention = ast_convention_to_param(param_def.parameter.convention, &ty);
        ctx.scope
            .define_parameter(name.clone(), ty.clone(), convention.is_mutable());

        let default = param_def.default.as_ref().and_then(|d| lower_expr(d, ctx));

        params.push(HirParam {
            name,
            ty,
            default,
            keyword_only: true,
            convention,
        });
    }

    // Populate borrowed_params for escape analysis in lower_return / lower_let.
    // Any borrowed move-type parameter, shared or mutable, is escape-unsafe.
    // Exclude TypeVar parameters: generics are monomorphized by Rust and ownership is handled
    // by the Rust compiler, not by Sifr's escape analysis.
    ctx.borrowed_params.clear();
    for param in &params {
        if param.convention.is_borrowed()
            && param.ty.ownership() == OwnershipKind::Move
            && !matches!(param.ty, Type::TypeVar(_))
        {
            ctx.borrowed_params.insert(param.name.clone());
        }
    }

    // Lower body
    let previous_owner = ctx.current_owner.replace(func.name.to_string());
    let previous_async = ctx.current_function_is_async;
    ctx.current_function_is_async = func.is_async;
    let body = lower_stmts(&func.body, &ft, ctx);
    ctx.current_function_is_async = previous_async;
    ctx.current_owner = previous_owner;

    ctx.borrowed_params.clear();

    ctx.exit_function_scope();

    let has_yield = !collect_yield_types(&body).is_empty();
    if !has_yield && requires_exhaustive_return_annotation(func, ft.return_type.as_ref()) {
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            crate::cfg::flow_facts(&body).always_exits()
        })) {
            Ok(false) => {
                let return_type = ft.return_type.display_name();
                super::flow_diagnostics::missing_return_value(
                    ctx,
                    func.name.as_str(),
                    return_type.as_str(),
                    func.name.range(),
                );
            }
            Ok(true) => {}
            Err(_) => {
                // Fail closed: skipping return-completeness validation after an
                // invalid CFG would let an unsound function compile.
                ctx.error_with_code_at(
                    DiagnosticCode::INTERNAL_COMPILER_PANIC,
                    format!(
                        "internal compiler error: invalid control-flow graph while validating exhaustive return for '{}'",
                        func.name
                    ),
                    func.name.range(),
                );
            }
        }
    }

    let return_annotation_range = func
        .returns
        .as_ref()
        .map_or_else(|| func.name.range(), |returns| returns.range());
    let inferred_return_type = infer_function_return_type(
        func.name.as_ref(),
        ft.return_type.as_ref(),
        func.returns.is_some(),
        &body,
        |message| {
            ctx.error_with_code_at(
                DiagnosticCode::TYPE_MISMATCH,
                message,
                return_annotation_range,
            );
        },
    );

    // Collect user-defined decorators (excluding classmethod/staticmethod)
    let decorators: Vec<String> = func
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

    // Collect type parameters for generic functions
    let type_params = ctx
        .generic_functions
        .get::<str>(func.name.as_ref())
        .cloned()
        .unwrap_or_default();

    Some(HirFunction {
        name: func.name.to_string(),
        params,
        return_type: inferred_return_type,
        body,
        is_async: func.is_async,
        method_kind: MethodKind::Regular,
        decorators,
        type_params,
    })
}

fn requires_exhaustive_return_annotation(func: &StmtFunctionDef, return_type: &Type) -> bool {
    if func.returns.is_none() {
        return false;
    }
    match return_type.resolve_alias() {
        Type::None => false,
        Type::Union(members) => !members
            .iter()
            .any(|member| matches!(member.resolve_alias(), Type::None)),
        _ => true,
    }
}

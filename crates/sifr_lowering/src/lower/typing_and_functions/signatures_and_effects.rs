use super::{resolve_annotation_expr, str};
use ruff_text_size::{Ranged, TextRange};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::{
    AstParamConvention, AstParamMutability, AstParamOwnership, Expr, Stmt, StmtFunctionDef,
};
use sifr_type_system::{
    FunctionType, OwnershipKind, ParamConvention, ParamMutability, ParamOwnership, Type,
};

use super::simple_expr::lower_expr_simple;
use super::workload_annotations;
use super::LowerCtx;

pub(in crate::lower) fn register_builtins(ctx: &mut LowerCtx) {
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

    // --- Ownership marker classes ---
    // NonSend is a zero-runtime marker base used by task-boundary checking. Classes that inherit
    // from it, or structurally contain fields that do, cannot cross `scope.spawn`.
    {
        let class_ty = Type::Class {
            name: "NonSend".to_string(),
            fields: vec![],
            methods: vec![],
            parent_class: None,
        };
        ctx.class_types.insert("NonSend".to_string(), class_ty);
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
        "RustPanicError",
        "TimeoutError",
        "ScopeFailure",
        "TaskCancelled",
        "SecondaryError",
        "GeneratorCloseError",
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
    {
        let class_ty = Type::Class {
            name: "CancellationError".to_string(),
            fields: vec![("message".to_string(), Type::Str)],
            methods: vec![],
            parent_class: None,
        };
        ctx.class_types
            .insert("CancellationError".to_string(), class_ty.clone());
        ctx.functions.insert(
            "CancellationError".to_string(),
            FunctionType::new(vec![("message".to_string(), Type::Str)], class_ty),
        );
    }
    {
        let class_ty = Type::Class {
            name: "AsyncExitCause".to_string(),
            fields: vec![],
            methods: vec![],
            parent_class: None,
        };
        ctx.class_types
            .insert("AsyncExitCause".to_string(), class_ty);
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

pub(in crate::lower) fn ast_convention_to_param(
    conv: AstParamConvention,
    ty: &Type,
) -> ParamConvention {
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

pub(in crate::lower) fn function_type_to_callable_type(ft: &FunctionType) -> Type {
    Type::Callable(
        ft.params.iter().map(|(_, ty, _)| ty.clone()).collect(),
        ft.params
            .iter()
            .map(|(_, _, convention)| *convention)
            .collect(),
        ft.return_type.clone(),
    )
}

pub(super) fn collect_function_defaults(
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

pub(super) fn first_await_range_in_stmts(stmts: &[Stmt]) -> Option<TextRange> {
    stmts.iter().find_map(first_await_range_in_stmt)
}

pub(super) fn first_yield_range_in_stmts(stmts: &[Stmt]) -> Option<TextRange> {
    stmts.iter().find_map(first_yield_range_in_stmt)
}

pub(in crate::lower) fn function_body_contains_yield(stmts: &[Stmt]) -> bool {
    let mut visitor = sifr_python_ast::helpers::ReturnStatementVisitor::default();
    for stmt in stmts {
        sifr_python_ast::visitor::Visitor::visit_stmt(&mut visitor, stmt);
    }
    visitor.is_generator
}

pub(super) fn first_await_range_in_stmt(stmt: &Stmt) -> Option<TextRange> {
    match stmt {
        Stmt::Expr(expr_stmt) => first_await_range_in_expr(expr_stmt.value.as_ref()),
        Stmt::Return(ret) => ret
            .value
            .as_ref()
            .and_then(|expr| first_await_range_in_expr(expr.as_ref())),
        Stmt::AnnAssign(ann) => ann
            .value
            .as_ref()
            .and_then(|expr| first_await_range_in_expr(expr.as_ref())),
        Stmt::Assign(assign) => first_await_range_in_expr(assign.value.as_ref()),
        Stmt::AugAssign(aug) => first_await_range_in_expr(aug.value.as_ref()),
        Stmt::If(if_stmt) => first_await_range_in_expr(if_stmt.test.as_ref())
            .or_else(|| first_await_range_in_stmts(&if_stmt.body))
            .or_else(|| {
                if_stmt.elif_else_clauses.iter().find_map(|clause| {
                    clause
                        .test
                        .as_ref()
                        .and_then(first_await_range_in_expr)
                        .or_else(|| first_await_range_in_stmts(&clause.body))
                })
            }),
        Stmt::While(while_stmt) => first_await_range_in_expr(while_stmt.test.as_ref())
            .or_else(|| first_await_range_in_stmts(&while_stmt.body))
            .or_else(|| first_await_range_in_stmts(&while_stmt.orelse)),
        Stmt::For(for_stmt) => first_await_range_in_expr(for_stmt.iter.as_ref())
            .or_else(|| first_await_range_in_stmts(&for_stmt.body))
            .or_else(|| first_await_range_in_stmts(&for_stmt.orelse)),
        Stmt::With(with_stmt) => with_stmt
            .items
            .iter()
            .find_map(|item| first_await_range_in_expr(&item.context_expr))
            .or_else(|| first_await_range_in_stmts(&with_stmt.body)),
        Stmt::Try(try_stmt) => first_await_range_in_stmts(&try_stmt.body)
            .or_else(|| {
                try_stmt.handlers.iter().find_map(|handler| {
                    let sifr_python_ast::ExceptHandler::ExceptHandler(handler) = handler;
                    first_await_range_in_stmts(&handler.body)
                })
            })
            .or_else(|| first_await_range_in_stmts(&try_stmt.orelse))
            .or_else(|| first_await_range_in_stmts(&try_stmt.finalbody)),
        _ => None,
    }
}

pub(super) fn first_yield_range_in_stmt(stmt: &Stmt) -> Option<TextRange> {
    match stmt {
        Stmt::Expr(expr_stmt) => first_yield_range_in_expr(expr_stmt.value.as_ref()),
        Stmt::Return(ret) => ret
            .value
            .as_ref()
            .and_then(|expr| first_yield_range_in_expr(expr.as_ref())),
        Stmt::AnnAssign(ann) => ann
            .value
            .as_ref()
            .and_then(|expr| first_yield_range_in_expr(expr.as_ref())),
        Stmt::Assign(assign) => first_yield_range_in_expr(assign.value.as_ref()),
        Stmt::AugAssign(aug) => first_yield_range_in_expr(aug.value.as_ref()),
        Stmt::If(if_stmt) => first_yield_range_in_expr(if_stmt.test.as_ref())
            .or_else(|| first_yield_range_in_stmts(&if_stmt.body))
            .or_else(|| {
                if_stmt.elif_else_clauses.iter().find_map(|clause| {
                    clause
                        .test
                        .as_ref()
                        .and_then(first_yield_range_in_expr)
                        .or_else(|| first_yield_range_in_stmts(&clause.body))
                })
            }),
        Stmt::While(while_stmt) => first_yield_range_in_expr(while_stmt.test.as_ref())
            .or_else(|| first_yield_range_in_stmts(&while_stmt.body))
            .or_else(|| first_yield_range_in_stmts(&while_stmt.orelse)),
        Stmt::For(for_stmt) => first_yield_range_in_expr(for_stmt.iter.as_ref())
            .or_else(|| first_yield_range_in_stmts(&for_stmt.body))
            .or_else(|| first_yield_range_in_stmts(&for_stmt.orelse)),
        Stmt::With(with_stmt) => with_stmt
            .items
            .iter()
            .find_map(|item| first_yield_range_in_expr(&item.context_expr))
            .or_else(|| first_yield_range_in_stmts(&with_stmt.body)),
        Stmt::Try(try_stmt) => first_yield_range_in_stmts(&try_stmt.body)
            .or_else(|| {
                try_stmt.handlers.iter().find_map(|handler| {
                    let sifr_python_ast::ExceptHandler::ExceptHandler(handler) = handler;
                    first_yield_range_in_stmts(&handler.body)
                })
            })
            .or_else(|| first_yield_range_in_stmts(&try_stmt.orelse))
            .or_else(|| first_yield_range_in_stmts(&try_stmt.finalbody)),
        _ => None,
    }
}

pub(super) fn first_await_range_in_expr(expr: &Expr) -> Option<TextRange> {
    match expr {
        Expr::Await(await_expr) => Some(await_expr.range()),
        Expr::Call(call) => first_await_range_in_expr(call.func.as_ref()).or_else(|| {
            call.arguments
                .args
                .iter()
                .find_map(first_await_range_in_expr)
                .or_else(|| {
                    call.arguments
                        .keywords
                        .iter()
                        .find_map(|keyword| first_await_range_in_expr(&keyword.value))
                })
        }),
        Expr::Attribute(attr) => first_await_range_in_expr(attr.value.as_ref()),
        Expr::Subscript(sub) => first_await_range_in_expr(sub.value.as_ref())
            .or_else(|| first_await_range_in_expr(sub.slice.as_ref())),
        Expr::BinOp(bin) => first_await_range_in_expr(bin.left.as_ref())
            .or_else(|| first_await_range_in_expr(bin.right.as_ref())),
        Expr::BoolOp(bool_op) => bool_op.values.iter().find_map(first_await_range_in_expr),
        Expr::UnaryOp(unary) => first_await_range_in_expr(unary.operand.as_ref()),
        Expr::Compare(compare) => first_await_range_in_expr(compare.left.as_ref()).or_else(|| {
            compare
                .comparators
                .iter()
                .find_map(first_await_range_in_expr)
        }),
        Expr::If(if_expr) => first_await_range_in_expr(if_expr.test.as_ref())
            .or_else(|| first_await_range_in_expr(if_expr.body.as_ref()))
            .or_else(|| first_await_range_in_expr(if_expr.orelse.as_ref())),
        Expr::List(list) => list.elts.iter().find_map(first_await_range_in_expr),
        Expr::Tuple(tuple) => tuple.elts.iter().find_map(first_await_range_in_expr),
        Expr::Set(set) => set.elts.iter().find_map(first_await_range_in_expr),
        Expr::Dict(dict) => dict.items.iter().find_map(|item| {
            item.key
                .as_ref()
                .and_then(first_await_range_in_expr)
                .or_else(|| first_await_range_in_expr(&item.value))
        }),
        _ => None,
    }
}

pub(super) fn first_yield_range_in_expr(expr: &Expr) -> Option<TextRange> {
    match expr {
        Expr::Yield(yield_expr) => Some(yield_expr.range()),
        Expr::YieldFrom(yield_from) => Some(yield_from.range()),
        Expr::Call(call) => first_yield_range_in_expr(call.func.as_ref()).or_else(|| {
            call.arguments
                .args
                .iter()
                .find_map(first_yield_range_in_expr)
                .or_else(|| {
                    call.arguments
                        .keywords
                        .iter()
                        .find_map(|keyword| first_yield_range_in_expr(&keyword.value))
                })
        }),
        Expr::Attribute(attr) => first_yield_range_in_expr(attr.value.as_ref()),
        Expr::Subscript(sub) => first_yield_range_in_expr(sub.value.as_ref())
            .or_else(|| first_yield_range_in_expr(sub.slice.as_ref())),
        Expr::BinOp(bin) => first_yield_range_in_expr(bin.left.as_ref())
            .or_else(|| first_yield_range_in_expr(bin.right.as_ref())),
        Expr::BoolOp(bool_op) => bool_op.values.iter().find_map(first_yield_range_in_expr),
        Expr::UnaryOp(unary) => first_yield_range_in_expr(unary.operand.as_ref()),
        Expr::Compare(compare) => first_yield_range_in_expr(compare.left.as_ref()).or_else(|| {
            compare
                .comparators
                .iter()
                .find_map(first_yield_range_in_expr)
        }),
        Expr::If(if_expr) => first_yield_range_in_expr(if_expr.test.as_ref())
            .or_else(|| first_yield_range_in_expr(if_expr.body.as_ref()))
            .or_else(|| first_yield_range_in_expr(if_expr.orelse.as_ref())),
        Expr::List(list) => list.elts.iter().find_map(first_yield_range_in_expr),
        Expr::Tuple(tuple) => tuple.elts.iter().find_map(first_yield_range_in_expr),
        Expr::Set(set) => set.elts.iter().find_map(first_yield_range_in_expr),
        Expr::Dict(dict) => dict.items.iter().find_map(|item| {
            item.key
                .as_ref()
                .and_then(first_yield_range_in_expr)
                .or_else(|| first_yield_range_in_expr(&item.value))
        }),
        _ => None,
    }
}

pub(in crate::lower) fn register_local_function_signature(
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
    if let Some(workload) =
        workload_annotations::annotation_for_decorators(func.decorator_list.iter())
    {
        ctx.function_workload_annotations
            .insert(function_name.clone(), workload);
    }
    if func.parameters.vararg.is_some() {
        ctx.vararg_functions
            .insert(function_name, func.parameters.args.len());
    }

    ft
}

pub(in crate::lower) fn register_local_function_symbol(
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

pub(in crate::lower) fn extract_function_type(
    func: &StmtFunctionDef,
    ctx: &mut LowerCtx,
) -> FunctionType {
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

pub(super) fn invalid_type_annotation(
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

pub(super) fn unknown_type(ctx: &mut LowerCtx, name: &str, range: ruff_text_size::TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::NAME_UNKNOWN_TYPE,
        format!("unknown type: '{name}'"),
        range,
    );
}

pub(super) fn reserved_integer_width_name(
    ctx: &mut LowerCtx,
    name: &str,
    range: ruff_text_size::TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::INT_RESERVED_WIDTH_NAME,
        format!("reserved integer width name '{name}' is not supported yet"),
        range,
    );
}

use crate::{
    HirExpr, HirFunction, HirModule, HirStmt, RustEmitter, RustExpr, RustItem, RustLiteral,
    RustStmt, Type, Visibility,
};

impl RustEmitter {
    pub(crate) fn reject_invalid_codegen_module_types(&mut self, module: &HirModule) -> bool {
        let Err(error) = validate_codegen_module_types(module) else {
            return false;
        };
        self.lowering_stats.item_lowering_errors += 1;
        self.body_items.push(RustItem::Fn {
            name: format!(
                "__sifr_codegen_type_error_{}",
                self.lowering_stats.item_lowering_errors
            ),
            visibility: Visibility::Private,
            type_params: Vec::new(),
            params: Vec::new(),
            ret: None,
            body: vec![RustStmt::Expr(RustExpr::MacroCall {
                name: "compile_error".to_string(),
                args: vec![RustExpr::Literal(RustLiteral::Str(error.to_string()))],
            })],
            is_async: false,
        });
        true
    }
}

pub(crate) fn validate_codegen_module_types(module: &HirModule) -> Result<(), crate::CodegenError> {
    for (_, ty, value) in &module.constants {
        validate_codegen_type(ty)?;
        validate_expr_tree(value)?;
    }
    for class in &module.classes {
        for (_, field) in &class.fields {
            validate_codegen_type(field)?;
        }
        if let Some(parent) = &class.parent_type {
            validate_codegen_type(parent)?;
        }
        if let Some(inner) = &class.newtype_inner {
            validate_codegen_type(inner)?;
        }
        for (_, default) in &class.field_defaults {
            validate_expr_tree(default)?;
        }
        for method in &class.methods {
            validate_function_types(method)?;
        }
        for (_, method) in &class.operator_impls {
            validate_function_types(method)?;
        }
    }
    for function in &module.functions {
        validate_function_types(function)?;
    }
    Ok(())
}

fn validate_function_types(function: &HirFunction) -> Result<(), crate::CodegenError> {
    for param in &function.params {
        validate_codegen_type(&param.ty)?;
        if let Some(default) = &param.default {
            validate_expr_tree(default)?;
        }
    }
    validate_codegen_type(&function.return_type)?;

    let mut statement_error = None;
    crate::hir_analysis::traversal::walk_stmts(
        &function.body,
        crate::hir_analysis::traversal::TraversalConfig::INCLUDE_NESTED_FUNCTIONS,
        &mut |statement| {
            if statement_error.is_none() {
                statement_error = validate_stmt_types(statement).err();
            }
        },
        &mut |_| {},
    );
    if let Some(error) = statement_error {
        return Err(error);
    }

    let mut expression_error = None;
    crate::hir_analysis::traversal::walk_stmts(
        &function.body,
        crate::hir_analysis::traversal::TraversalConfig::INCLUDE_NESTED_FUNCTIONS,
        &mut |_| {},
        &mut |expr| {
            if expression_error.is_none() {
                expression_error = validate_expr_types(expr).err();
            }
        },
    );
    expression_error.map_or(Ok(()), Err)
}

fn validate_expr_tree(expr: &HirExpr) -> Result<(), crate::CodegenError> {
    let mut error = None;
    crate::hir_analysis::traversal::walk_expr(expr, &mut |node| {
        if error.is_none() {
            error = validate_expr_types(node).err();
        }
    });
    error.map_or(Ok(()), Err)
}

fn validate_expr_types(expr: &HirExpr) -> Result<(), crate::CodegenError> {
    validate_codegen_type(expr.ty())?;
    if let HirExpr::GenericCall { type_args, .. } = expr {
        for type_arg in type_args {
            validate_codegen_type(type_arg)?;
        }
    }
    Ok(())
}

fn validate_stmt_types(stmt: &HirStmt) -> Result<(), crate::CodegenError> {
    match stmt {
        HirStmt::Let { ty, .. } => validate_codegen_type(ty)?,
        HirStmt::For { target_ty, .. } => validate_codegen_type(target_ty)?,
        HirStmt::AsyncFor {
            target_ty,
            iter_error_ty,
            close_error_ty,
            ..
        } => {
            validate_codegen_type(target_ty)?;
            validate_codegen_type(iter_error_ty)?;
            if let Some(close_error_ty) = close_error_ty {
                validate_codegen_type(close_error_ty)?;
            }
        }
        HirStmt::TupleUnpack { targets, .. } => {
            for target in targets {
                validate_codegen_type(&target.ty)?;
            }
        }
        HirStmt::StarUnpack {
            before,
            star,
            after,
            ..
        } => {
            for (_, ty) in before.iter().chain(std::iter::once(star)).chain(after) {
                validate_codegen_type(ty)?;
            }
        }
        HirStmt::TryExcept {
            handlers,
            body_error_types,
            ..
        } => {
            for ty in body_error_types {
                validate_codegen_type(ty)?;
            }
            for handler in handlers {
                if let Some(ty) = &handler.error_resolved_type {
                    validate_codegen_type(ty)?;
                }
            }
        }
        HirStmt::FieldAssign { field_ty, .. } => validate_codegen_type(field_ty)?,
        HirStmt::NestedFieldAssign {
            field_ty,
            nested_field_ty,
            ..
        } => {
            validate_codegen_type(field_ty)?;
            validate_codegen_type(nested_field_ty)?;
        }
        HirStmt::SubscriptAssign { object_ty, .. }
        | HirStmt::NestedSubscriptAssign { object_ty, .. } => {
            validate_codegen_type(object_ty)?;
        }
        HirStmt::AttributeNestedSubscriptAssign { field_ty, .. }
        | HirStmt::AttributeSubscriptAssign { field_ty, .. } => {
            validate_codegen_type(field_ty)?;
        }
        HirStmt::SubscriptAugAssign {
            object_ty,
            missing_key_error,
            ..
        } => {
            validate_codegen_type(object_ty)?;
            if let Some(error) = missing_key_error {
                validate_codegen_type(error)?;
            }
        }
        HirStmt::NestedFunction { func, .. } => {
            for param in &func.params {
                validate_codegen_type(&param.ty)?;
            }
            validate_codegen_type(&func.return_type)?;
        }
        HirStmt::Match { subject_ty, .. } => validate_codegen_type(subject_ty)?,
        _ => {}
    }
    Ok(())
}

pub(super) fn validate_codegen_type(ty: &Type) -> Result<(), crate::CodegenError> {
    match ty {
        Type::Callable(params, conventions, ret)
        | Type::AsyncCallable(params, conventions, ret) => {
            if params.len() != conventions.len() {
                return Err(crate::CodegenError::new(format!(
                    "unsupported callable type: {} parameters but {} conventions",
                    params.len(),
                    conventions.len()
                )));
            }
            for param in params {
                validate_codegen_type(param)?;
            }
            validate_codegen_type(ret)
        }
        Type::Function(function) | Type::AsyncFunction(function) => {
            for (_, param, _) in &function.params {
                validate_codegen_type(param)?;
            }
            validate_codegen_type(&function.return_type)
        }
        Type::List(inner)
        | Type::Set(inner)
        | Type::Iterable(inner)
        | Type::Iterator(inner)
        | Type::Awaitable(inner)
        | Type::Failure(inner)
        | Type::TimeoutResult(inner)
        | Type::PythonBuffer(inner)
        | Type::PythonDlpackTensor(inner)
        | Type::Newtype { inner, .. }
        | Type::Alias { body: inner, .. } => validate_codegen_type(inner),
        Type::Dict(left, right)
        | Type::Result(left, right)
        | Type::Coroutine(left, right)
        | Type::Task(left, right)
        | Type::TaskResult(left, right)
        | Type::Select2(left, right)
        | Type::BlockingTask(left, right)
        | Type::JoinSet(left, right)
        | Type::AsyncIterator(left, right)
        | Type::AsyncGenerator(left, right) => {
            validate_codegen_type(left)?;
            validate_codegen_type(right)
        }
        Type::Tuple(items)
        | Type::Union(items)
        | Type::Intersection(items)
        | Type::Template(items) => {
            for item in items {
                validate_codegen_type(item)?;
            }
            Ok(())
        }
        Type::StructuralRecord(record) => {
            for field in record.fields() {
                validate_codegen_type(field.ty())?;
            }
            Ok(())
        }
        Type::Class {
            type_args,
            fields,
            methods,
            ..
        } => {
            for type_arg in type_args {
                validate_codegen_type(type_arg)?;
            }
            for (_, field) in fields {
                validate_codegen_type(field)?;
            }
            for (_, method) in methods {
                validate_codegen_type(&Type::Function(method.clone()))?;
            }
            Ok(())
        }
        Type::Protocol { methods, .. } => {
            for (_, method) in methods {
                validate_codegen_type(&Type::Function(method.clone()))?;
            }
            Ok(())
        }
        Type::Int
        | Type::FixedInt(_)
        | Type::Float
        | Type::Bool
        | Type::Str
        | Type::Bytes
        | Type::None
        | Type::PythonArrow(_)
        | Type::PythonDlpackStream
        | Type::Range
        | Type::Any
        | Type::Never
        | Type::LiteralInt(_)
        | Type::LiteralStr(_)
        | Type::LiteralBool(_)
        | Type::Unknown
        | Type::TypeVar(_)
        | Type::Enum { .. }
        | Type::Decimal
        | Type::BigDecimal => Ok(()),
    }
}

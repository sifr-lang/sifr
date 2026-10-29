use sifr_ir::{HirExpr, HirFStringPart, HirModule, HirStmt};
use sifr_type_system::Type;
use std::collections::HashSet;

mod checked_places;
mod conversions;
pub(crate) use conversions::ErrorConversionDemand;

#[derive(Default)]
struct ErrorReferences {
    builtins: HashSet<String>,
    conversions: ErrorConversionDemand,
}

pub(crate) fn collect_error_conversion_demand(
    module: &HirModule,
    module_name: Option<&str>,
) -> ErrorConversionDemand {
    let mut conversions =
        collect_error_references(module, "", &HashSet::new(), false, &[]).conversions;
    conversions.record_classes(module, module_name);
    conversions
}

pub(crate) fn collect_referenced_builtin_error_classes(
    module: &HirModule,
    stdlib_preamble: &str,
    intrinsic_functions: &HashSet<String>,
    needs_file_handles: bool,
    builtin_error_classes: &[&str],
) -> HashSet<String> {
    collect_error_references(
        module,
        stdlib_preamble,
        intrinsic_functions,
        needs_file_handles,
        builtin_error_classes,
    )
    .builtins
}

fn collect_error_references(
    module: &HirModule,
    stdlib_preamble: &str,
    intrinsic_functions: &HashSet<String>,
    needs_file_handles: bool,
    builtin_error_classes: &[&str],
) -> ErrorReferences {
    let mut referenced = ErrorReferences::default();

    for func in &module.functions {
        for param in &func.params {
            collect_type_error_refs(&param.ty, &mut referenced, builtin_error_classes);
        }
        collect_type_error_refs(&func.return_type, &mut referenced, builtin_error_classes);
        collect_stmt_error_refs(&func.body, &mut referenced, builtin_error_classes);
    }
    for class in &module.classes {
        for (_, field_ty) in &class.fields {
            collect_type_error_refs(field_ty, &mut referenced, builtin_error_classes);
        }
        for method in &class.methods {
            for param in &method.params {
                collect_type_error_refs(&param.ty, &mut referenced, builtin_error_classes);
            }
            collect_type_error_refs(&method.return_type, &mut referenced, builtin_error_classes);
            collect_stmt_error_refs(&method.body, &mut referenced, builtin_error_classes);
        }
        for (_, operator) in &class.operator_impls {
            for param in &operator.params {
                collect_type_error_refs(&param.ty, &mut referenced, builtin_error_classes);
            }
            collect_type_error_refs(
                &operator.return_type,
                &mut referenced,
                builtin_error_classes,
            );
            collect_stmt_error_refs(&operator.body, &mut referenced, builtin_error_classes);
        }
    }
    for (_, ty, value) in &module.constants {
        collect_type_error_refs(ty, &mut referenced, builtin_error_classes);
        collect_expr_error_refs(value, &mut referenced, builtin_error_classes);
    }

    collect_text_error_refs(stdlib_preamble, &mut referenced, builtin_error_classes);

    if needs_file_handles {
        referenced.builtins.insert("IOError".to_string());
    }

    // Intrinsics can produce these builtin errors through generated helper code.
    if !intrinsic_functions.is_empty() {
        for error_name in [
            "IOError",
            "ParseError",
            "ValueError",
            "JSONDecodeError",
            "JsonIntegerRangeError",
            "JsonLimitError",
            "TOMLDecodeError",
            "RegexError",
            "TimeoutError",
            "ScopeFailure",
        ] {
            referenced.builtins.insert(error_name.to_string());
        }
    }

    referenced
}

pub(crate) fn collect_module_intrinsic_function_names(module: &HirModule) -> HashSet<String> {
    fn collect_expr(expr: &HirExpr, names: &mut HashSet<String>) {
        crate::hir_analysis::traversal::walk_expr(expr, &mut |expr| {
            if let HirExpr::IntrinsicCall { intrinsic, .. } = expr {
                names.insert(intrinsic.declaration_name().to_string());
            }
        });
    }

    fn collect_body(body: &[HirStmt], names: &mut HashSet<String>) {
        crate::hir_analysis::traversal::walk_stmts(
            body,
            crate::hir_analysis::traversal::TraversalConfig::INCLUDE_NESTED_FUNCTIONS,
            &mut |_| {},
            &mut |expr| {
                if let HirExpr::IntrinsicCall { intrinsic, .. } = expr {
                    names.insert(intrinsic.declaration_name().to_string());
                }
            },
        );
    }

    let mut names = HashSet::new();
    for function in &module.functions {
        for param in &function.params {
            if let Some(default) = &param.default {
                collect_expr(default, &mut names);
            }
        }
        collect_body(&function.body, &mut names);
    }
    for class in &module.classes {
        for (_, default) in &class.field_defaults {
            collect_expr(default, &mut names);
        }
        for method in &class.methods {
            for param in &method.params {
                if let Some(default) = &param.default {
                    collect_expr(default, &mut names);
                }
            }
            collect_body(&method.body, &mut names);
        }
        for (_, operator) in &class.operator_impls {
            for param in &operator.params {
                if let Some(default) = &param.default {
                    collect_expr(default, &mut names);
                }
            }
            collect_body(&operator.body, &mut names);
        }
    }
    for (_, _, value) in &module.constants {
        collect_expr(value, &mut names);
    }
    names
}

fn collect_type_error_refs(
    ty: &Type,
    referenced: &mut ErrorReferences,
    builtin_error_classes: &[&str],
) {
    referenced.conversions.record_type(ty);
    match ty {
        Type::Class { name, .. } | Type::Protocol { name, .. } | Type::Enum { name, .. } => {
            if builtin_error_classes.contains(&name.as_str()) {
                referenced.builtins.insert(name.clone());
            }
        }
        Type::List(inner)
        | Type::PythonBuffer(inner)
        | Type::PythonDlpackTensor(inner)
        | Type::Set(inner)
        | Type::Iterable(inner)
        | Type::Iterator(inner)
        | Type::Newtype { inner, .. } => {
            collect_type_error_refs(inner, referenced, builtin_error_classes);
        }
        Type::Alias {
            type_args, body, ..
        } => {
            for arg in type_args {
                collect_type_error_refs(arg, referenced, builtin_error_classes);
            }
            collect_type_error_refs(body, referenced, builtin_error_classes);
        }
        Type::Dict(key, value)
        | Type::Result(key, value)
        | Type::Coroutine(key, value)
        | Type::Task(key, value)
        | Type::TaskResult(key, value)
        | Type::Select2(key, value)
        | Type::BlockingTask(key, value)
        | Type::JoinSet(key, value)
        | Type::AsyncIterator(key, value)
        | Type::AsyncGenerator(key, value) => {
            collect_type_error_refs(key, referenced, builtin_error_classes);
            collect_type_error_refs(value, referenced, builtin_error_classes);
        }
        Type::Failure(err) => {
            collect_type_error_refs(err, referenced, builtin_error_classes);
            if builtin_error_classes.contains(&"SecondaryError") {
                referenced.builtins.insert("SecondaryError".to_string());
            }
        }
        Type::TimeoutResult(err) => {
            collect_type_error_refs(err, referenced, builtin_error_classes);
        }
        Type::Awaitable(inner) => collect_type_error_refs(inner, referenced, builtin_error_classes),
        Type::Tuple(items)
        | Type::Union(items)
        | Type::Intersection(items)
        | Type::Template(items) => {
            for item in items {
                collect_type_error_refs(item, referenced, builtin_error_classes);
            }
        }
        Type::StructuralRecord(record) => {
            for field in record.fields() {
                collect_type_error_refs(field.ty(), referenced, builtin_error_classes);
            }
        }
        Type::Function(sig) | Type::AsyncFunction(sig) => {
            for (_, param_ty, _) in &sig.params {
                collect_type_error_refs(param_ty, referenced, builtin_error_classes);
            }
            collect_type_error_refs(&sig.return_type, referenced, builtin_error_classes);
        }
        Type::Callable(params, _, ret) | Type::AsyncCallable(params, _, ret) => {
            for param_ty in params {
                collect_type_error_refs(param_ty, referenced, builtin_error_classes);
            }
            collect_type_error_refs(ret, referenced, builtin_error_classes);
        }
        Type::Int
        | Type::FixedInt(_)
        | Type::Float
        | Type::Bool
        | Type::Str
        | Type::Bytes
        | Type::None
        | Type::Range
        | Type::Any
        | Type::Never
        | Type::LiteralInt(_)
        | Type::LiteralStr(_)
        | Type::LiteralBool(_)
        | Type::Unknown
        | Type::TypeVar(_)
        | Type::PythonArrow(_)
        | Type::PythonDlpackStream
        | Type::Decimal
        | Type::BigDecimal => {}
    }
}

fn collect_stmt_error_refs(
    stmts: &[HirStmt],
    referenced: &mut ErrorReferences,
    builtin_error_classes: &[&str],
) {
    for stmt in stmts {
        if checked_places::collect_checked_place_stmt_error_refs(
            stmt,
            referenced,
            builtin_error_classes,
        ) {
            continue;
        }
        match stmt {
            HirStmt::Let { value, .. }
            | HirStmt::Assign { value, .. }
            | HirStmt::AugAssign { value, .. }
            | HirStmt::AttributeAugAssign { value, .. }
            | HirStmt::FieldAssign { value, .. }
            | HirStmt::NestedFieldAssign { value, .. }
            | HirStmt::Raise { value }
            | HirStmt::Yield { value } => {
                collect_expr_error_refs(value, referenced, builtin_error_classes);
            }
            HirStmt::Return { value } => {
                if let Some(value) = value {
                    collect_expr_error_refs(value, referenced, builtin_error_classes);
                }
            }
            HirStmt::Expr { expr } => {
                collect_expr_error_refs(expr, referenced, builtin_error_classes);
            }
            HirStmt::If {
                condition,
                then_body,
                elif_clauses,
                else_body,
            } => {
                collect_expr_error_refs(condition, referenced, builtin_error_classes);
                collect_stmt_error_refs(then_body, referenced, builtin_error_classes);
                for (elif_cond, elif_body) in elif_clauses {
                    collect_expr_error_refs(elif_cond, referenced, builtin_error_classes);
                    collect_stmt_error_refs(elif_body, referenced, builtin_error_classes);
                }
                if let Some(else_body) = else_body {
                    collect_stmt_error_refs(else_body, referenced, builtin_error_classes);
                }
            }
            HirStmt::While {
                condition,
                body,
                else_body,
            } => {
                collect_expr_error_refs(condition, referenced, builtin_error_classes);
                collect_stmt_error_refs(body, referenced, builtin_error_classes);
                if let Some(else_body) = else_body {
                    collect_stmt_error_refs(else_body, referenced, builtin_error_classes);
                }
            }
            HirStmt::For {
                iter,
                body,
                else_body,
                ..
            } => {
                collect_expr_error_refs(iter, referenced, builtin_error_classes);
                collect_stmt_error_refs(body, referenced, builtin_error_classes);
                if let Some(else_body) = else_body {
                    collect_stmt_error_refs(else_body, referenced, builtin_error_classes);
                }
            }
            HirStmt::AsyncFor {
                iter,
                iter_error_ty,
                body,
                else_body,
                ..
            } => {
                collect_expr_error_refs(iter, referenced, builtin_error_classes);
                collect_type_error_refs(iter_error_ty, referenced, builtin_error_classes);
                collect_stmt_error_refs(body, referenced, builtin_error_classes);
                if let Some(else_body) = else_body {
                    collect_stmt_error_refs(else_body, referenced, builtin_error_classes);
                }
            }
            HirStmt::TupleUnpack { value, .. } => {
                collect_expr_error_refs(value, referenced, builtin_error_classes);
            }
            HirStmt::StarUnpack { .. }
            | HirStmt::SubscriptAssign { .. }
            | HirStmt::NestedSubscriptAssign { .. }
            | HirStmt::AttributeNestedSubscriptAssign { .. }
            | HirStmt::SubscriptAugAssign { .. }
            | HirStmt::AttributeSubscriptAssign { .. }
            | HirStmt::Delete { .. } => {}
            HirStmt::Assert { test, msg } => {
                collect_expr_error_refs(test, referenced, builtin_error_classes);
                if let Some(msg) = msg {
                    collect_expr_error_refs(msg, referenced, builtin_error_classes);
                }
            }
            HirStmt::TryExcept { body, handlers, .. } => {
                collect_stmt_error_refs(body, referenced, builtin_error_classes);
                for handler in handlers {
                    if let Some(error_type) = &handler.error_type {
                        if builtin_error_classes.contains(&error_type.as_str()) {
                            referenced.builtins.insert(error_type.clone());
                        }
                    }
                    collect_stmt_error_refs(&handler.body, referenced, builtin_error_classes);
                }
            }
            HirStmt::TryFinally { body, finalbody } => {
                collect_stmt_error_refs(body, referenced, builtin_error_classes);
                collect_stmt_error_refs(finalbody, referenced, builtin_error_classes);
            }
            HirStmt::With { items, body } => {
                for item in items {
                    collect_expr_error_refs(&item.context, referenced, builtin_error_classes);
                }
                collect_stmt_error_refs(body, referenced, builtin_error_classes);
            }
            HirStmt::AsyncWith { kind, body, .. } => {
                match kind {
                    sifr_ir::HirAsyncWithKind::TaskTimeout { duration } => {
                        referenced.builtins.insert("TimeoutError".to_string());
                        collect_expr_error_refs(duration, referenced, builtin_error_classes);
                    }
                    sifr_ir::HirAsyncWithKind::UserDefined { context, .. }
                    | sifr_ir::HirAsyncWithKind::Python { context, .. } => {
                        collect_expr_error_refs(context, referenced, builtin_error_classes);
                    }
                    sifr_ir::HirAsyncWithKind::TaskGroup {
                        context: Some(context),
                    } => {
                        referenced.builtins.insert("ScopeFailure".to_string());
                        collect_expr_error_refs(context, referenced, builtin_error_classes);
                    }
                    sifr_ir::HirAsyncWithKind::TaskScope
                    | sifr_ir::HirAsyncWithKind::TaskGroup { context: None } => {
                        referenced.builtins.insert("ScopeFailure".to_string());
                    }
                }
                collect_stmt_error_refs(body, referenced, builtin_error_classes);
            }
            HirStmt::NestedFunction { func, .. } => {
                collect_stmt_error_refs(&func.body, referenced, builtin_error_classes);
            }
            HirStmt::Match { subject, arms, .. } => {
                collect_expr_error_refs(subject, referenced, builtin_error_classes);
                for arm in arms {
                    if let Some(guard) = &arm.guard {
                        collect_expr_error_refs(guard, referenced, builtin_error_classes);
                    }
                    collect_stmt_error_refs(&arm.body, referenced, builtin_error_classes);
                }
            }
            HirStmt::Pass | HirStmt::Break | HirStmt::Continue => {}
        }
    }
}

fn collect_expr_error_refs(
    expr: &HirExpr,
    referenced: &mut ErrorReferences,
    builtin_error_classes: &[&str],
) {
    collect_type_error_refs(expr.ty(), referenced, builtin_error_classes);

    match expr {
        HirExpr::Call { func, args, .. }
        | HirExpr::GenericCall { func, args, .. }
        | HirExpr::PythonCall { func, args, .. } => {
            if builtin_error_classes.contains(&func.as_str()) {
                referenced.builtins.insert(func.clone());
            }
            for arg in args {
                collect_expr_error_refs(arg, referenced, builtin_error_classes);
            }
        }
        HirExpr::IntrinsicCall { args, .. } | HirExpr::IteratorCall { args, .. } => {
            for arg in args {
                collect_expr_error_refs(arg, referenced, builtin_error_classes);
            }
        }
        HirExpr::ConstructorCall {
            class_name, args, ..
        } => {
            if builtin_error_classes.contains(&class_name.as_str()) {
                referenced.builtins.insert(class_name.clone());
            }
            for arg in args {
                collect_expr_error_refs(arg, referenced, builtin_error_classes);
            }
        }
        HirExpr::BinOp { left, right, .. } => {
            collect_expr_error_refs(left, referenced, builtin_error_classes);
            collect_expr_error_refs(right, referenced, builtin_error_classes);
        }
        HirExpr::UnaryOp { operand, .. }
        | HirExpr::Await { value: operand, .. }
        | HirExpr::QuestionMark { expr: operand, .. }
        | HirExpr::OkWrap { value: operand, .. }
        | HirExpr::ErrWrap { value: operand, .. }
        | HirExpr::WalrusExpr { value: operand, .. }
        | HirExpr::StructuralRecordProject {
            source: operand, ..
        }
        | HirExpr::FieldAccess {
            object: operand, ..
        } => {
            collect_expr_error_refs(operand, referenced, builtin_error_classes);
        }
        HirExpr::Compare {
            left, comparators, ..
        } => {
            collect_expr_error_refs(left, referenced, builtin_error_classes);
            for comparator in comparators {
                collect_expr_error_refs(comparator, referenced, builtin_error_classes);
            }
        }
        HirExpr::BoolOp { values, .. }
        | HirExpr::ListLiteral {
            elements: values, ..
        }
        | HirExpr::SetLiteral {
            elements: values, ..
        }
        | HirExpr::TupleLiteral {
            elements: values, ..
        } => {
            for value in values {
                collect_expr_error_refs(value, referenced, builtin_error_classes);
            }
        }
        HirExpr::IfExpr {
            condition,
            then_expr,
            else_expr,
            ..
        } => {
            collect_expr_error_refs(condition, referenced, builtin_error_classes);
            collect_expr_error_refs(then_expr, referenced, builtin_error_classes);
            collect_expr_error_refs(else_expr, referenced, builtin_error_classes);
        }
        HirExpr::RangeLiteral {
            start, end, step, ..
        } => {
            collect_expr_error_refs(start, referenced, builtin_error_classes);
            collect_expr_error_refs(end, referenced, builtin_error_classes);
            if let Some(step) = step {
                collect_expr_error_refs(step, referenced, builtin_error_classes);
            }
        }
        HirExpr::DictLiteral { keys, values, .. } => {
            for key in keys {
                collect_expr_error_refs(key, referenced, builtin_error_classes);
            }
            for value in values {
                collect_expr_error_refs(value, referenced, builtin_error_classes);
            }
        }
        HirExpr::Index { object, index, .. }
        | HirExpr::ContainsOp {
            element: index,
            collection: object,
            ..
        } => {
            collect_expr_error_refs(object, referenced, builtin_error_classes);
            collect_expr_error_refs(index, referenced, builtin_error_classes);
            if let HirExpr::Index { ty, .. } = expr
                && !crate::helpers::is_option_type(ty)
            {
                match object.ty().resolve_alias() {
                    Type::List(_) | Type::Bytes | Type::Str => {
                        referenced.builtins.insert("IndexError".to_string());
                    }
                    Type::Dict(_, _) => {
                        referenced.builtins.insert("KeyError".to_string());
                    }
                    _ => {}
                }
            }
        }
        HirExpr::MethodCall { object, args, .. } => {
            collect_expr_error_refs(object, referenced, builtin_error_classes);
            for arg in args {
                collect_expr_error_refs(arg, referenced, builtin_error_classes);
            }
        }
        HirExpr::FString { parts, .. } => {
            for part in parts {
                if let HirFStringPart::Expr(expr) = part {
                    collect_expr_error_refs(expr, referenced, builtin_error_classes);
                }
            }
        }
        HirExpr::TemplateString(template) => template.for_each_value(&mut |expr| {
            collect_expr_error_refs(expr, referenced, builtin_error_classes);
        }),
        HirExpr::Slice {
            object,
            start,
            stop,
            step,
            ..
        } => {
            collect_expr_error_refs(object, referenced, builtin_error_classes);
            if let Some(start) = start {
                collect_expr_error_refs(start, referenced, builtin_error_classes);
            }
            if let Some(stop) = stop {
                collect_expr_error_refs(stop, referenced, builtin_error_classes);
            }
            if let Some(step) = step {
                collect_expr_error_refs(step, referenced, builtin_error_classes);
            }
        }
        HirExpr::SuperCall { args, .. } => {
            for arg in args {
                collect_expr_error_refs(arg, referenced, builtin_error_classes);
            }
        }
        HirExpr::Lambda { body, .. } => {
            collect_expr_error_refs(body, referenced, builtin_error_classes);
        }
        HirExpr::ListComp {
            expr, generators, ..
        }
        | HirExpr::SetComp {
            expr, generators, ..
        } => {
            collect_expr_error_refs(expr, referenced, builtin_error_classes);
            for (_, iter_expr, filter) in generators {
                collect_expr_error_refs(iter_expr, referenced, builtin_error_classes);
                if let Some(filter) = filter {
                    collect_expr_error_refs(filter, referenced, builtin_error_classes);
                }
            }
        }
        HirExpr::DictComp {
            key_expr,
            val_expr,
            generators,
            ..
        } => {
            collect_expr_error_refs(key_expr, referenced, builtin_error_classes);
            collect_expr_error_refs(val_expr, referenced, builtin_error_classes);
            for (_, iter_expr, filter) in generators {
                collect_expr_error_refs(iter_expr, referenced, builtin_error_classes);
                if let Some(filter) = filter {
                    collect_expr_error_refs(filter, referenced, builtin_error_classes);
                }
            }
        }
        HirExpr::GeneratorExpr {
            expr, iter, filter, ..
        } => {
            collect_expr_error_refs(expr, referenced, builtin_error_classes);
            collect_expr_error_refs(iter, referenced, builtin_error_classes);
            if let Some(filter) = filter {
                collect_expr_error_refs(filter, referenced, builtin_error_classes);
            }
        }
        HirExpr::IntLiteral(_)
        | HirExpr::LargeIntLiteral(_)
        | HirExpr::FloatLiteral(_)
        | HirExpr::StringLiteral(_)
        | HirExpr::BoolLiteral(_)
        | HirExpr::NoneLiteral
        | HirExpr::Name { .. }
        | HirExpr::EnumVariant { .. } => {}
    }
}

fn collect_text_error_refs(
    text: &str,
    referenced: &mut ErrorReferences,
    builtin_error_classes: &[&str],
) {
    referenced
        .builtins
        .extend(collect_source_builtin_error_classes(
            text,
            builtin_error_classes,
        ));
}

pub(crate) use sifr_generated_rust::collect_source_builtin_error_classes;

#[cfg(test)]
#[path = "../tests/unit/error_refs/tests.rs"]
mod tests;

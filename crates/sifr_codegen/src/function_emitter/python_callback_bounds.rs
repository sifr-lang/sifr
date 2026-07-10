use super::{HashSet, HirExpr, HirFunction, HirStmt, Type};

pub(super) fn python_callback_bound_param_names(func: &HirFunction) -> HashSet<String> {
    let callable_params = func
        .params
        .iter()
        .filter(|param| matches!(param.ty.resolve_alias(), Type::Callable(..)))
        .map(|param| param.name.clone())
        .collect::<HashSet<_>>();
    if callable_params.is_empty() {
        return HashSet::new();
    }

    let mut names = HashSet::new();
    for stmt in &func.body {
        collect_python_callback_bound_names_stmt(stmt, &callable_params, &mut names);
    }
    names
}

fn collect_python_callback_bound_names_stmt(
    stmt: &HirStmt,
    callable_params: &HashSet<String>,
    names: &mut HashSet<String>,
) {
    match stmt {
        HirStmt::Let { value, .. }
        | HirStmt::Assign { value, .. }
        | HirStmt::AugAssign { value, .. }
        | HirStmt::Return { value: Some(value) }
        | HirStmt::Expr { expr: value }
        | HirStmt::Raise { value }
        | HirStmt::Yield { value }
        | HirStmt::TupleUnpack { value, .. }
        | HirStmt::StarUnpack { value, .. } => {
            collect_python_callback_bound_names_expr(value, callable_params, names);
        }
        HirStmt::If {
            condition,
            then_body,
            elif_clauses,
            else_body,
        } => {
            collect_python_callback_bound_names_expr(condition, callable_params, names);
            for nested in then_body {
                collect_python_callback_bound_names_stmt(nested, callable_params, names);
            }
            for (elif_condition, elif_body) in elif_clauses {
                collect_python_callback_bound_names_expr(elif_condition, callable_params, names);
                for nested in elif_body {
                    collect_python_callback_bound_names_stmt(nested, callable_params, names);
                }
            }
            if let Some(else_body) = else_body {
                for nested in else_body {
                    collect_python_callback_bound_names_stmt(nested, callable_params, names);
                }
            }
        }
        HirStmt::While {
            condition, body, ..
        } => {
            collect_python_callback_bound_names_expr(condition, callable_params, names);
            for nested in body {
                collect_python_callback_bound_names_stmt(nested, callable_params, names);
            }
        }
        HirStmt::For { iter, body, .. } | HirStmt::AsyncFor { iter, body, .. } => {
            collect_python_callback_bound_names_expr(iter, callable_params, names);
            for nested in body {
                collect_python_callback_bound_names_stmt(nested, callable_params, names);
            }
        }
        HirStmt::TryExcept { body, handlers, .. } => {
            for nested in body {
                collect_python_callback_bound_names_stmt(nested, callable_params, names);
            }
            for handler in handlers {
                for nested in &handler.body {
                    collect_python_callback_bound_names_stmt(nested, callable_params, names);
                }
            }
        }
        HirStmt::TryFinally { body, finalbody } => {
            for nested in body.iter().chain(finalbody) {
                collect_python_callback_bound_names_stmt(nested, callable_params, names);
            }
        }
        HirStmt::With { items, body } => {
            for (_, context, _) in items {
                collect_python_callback_bound_names_expr(context, callable_params, names);
            }
            for nested in body {
                collect_python_callback_bound_names_stmt(nested, callable_params, names);
            }
        }
        HirStmt::AsyncWith { body, .. } => {
            for nested in body {
                collect_python_callback_bound_names_stmt(nested, callable_params, names);
            }
        }
        HirStmt::Match { subject, arms, .. } => {
            collect_python_callback_bound_names_expr(subject, callable_params, names);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    collect_python_callback_bound_names_expr(guard, callable_params, names);
                }
                for nested in &arm.body {
                    collect_python_callback_bound_names_stmt(nested, callable_params, names);
                }
            }
        }
        HirStmt::Assert { test, msg } => {
            collect_python_callback_bound_names_expr(test, callable_params, names);
            if let Some(msg) = msg {
                collect_python_callback_bound_names_expr(msg, callable_params, names);
            }
        }
        HirStmt::FieldAssign { value, .. }
        | HirStmt::NestedFieldAssign { value, .. }
        | HirStmt::SubscriptAssign { value, .. }
        | HirStmt::NestedSubscriptAssign { value, .. }
        | HirStmt::AttributeNestedSubscriptAssign { value, .. }
        | HirStmt::SubscriptAugAssign { value, .. }
        | HirStmt::AttributeAugAssign { value, .. }
        | HirStmt::AttributeSubscriptAssign { value, .. } => {
            collect_python_callback_bound_names_expr(value, callable_params, names);
        }
        HirStmt::Delete { object, index } => {
            collect_python_callback_bound_names_expr(object, callable_params, names);
            collect_python_callback_bound_names_expr(index, callable_params, names);
        }
        HirStmt::Return { value: None }
        | HirStmt::Break
        | HirStmt::Continue
        | HirStmt::Pass
        | HirStmt::NestedFunction { .. } => {}
    }
}

fn collect_python_callback_bound_names_expr(
    expr: &HirExpr,
    callable_params: &HashSet<String>,
    names: &mut HashSet<String>,
) {
    match expr {
        HirExpr::Call { func, args, .. }
            if matches!(
                func.as_str(),
                "py_local_callback" | "py_threadsafe_callback"
            ) =>
        {
            for arg in args {
                collect_callable_names_from_callback_arg(arg, callable_params, names);
            }
        }
        HirExpr::Call { args, .. }
        | HirExpr::IteratorCall { args, .. }
        | HirExpr::TupleLiteral { elements: args, .. }
        | HirExpr::ListLiteral { elements: args, .. }
        | HirExpr::SetLiteral { elements: args, .. }
        | HirExpr::ConstructorCall { args, .. }
        | HirExpr::SuperCall { args, .. } => {
            for arg in args {
                collect_python_callback_bound_names_expr(arg, callable_params, names);
            }
        }
        HirExpr::Lambda { body, .. }
        | HirExpr::QuestionMark { expr: body, .. }
        | HirExpr::OkWrap { value: body, .. }
        | HirExpr::ErrWrap { value: body, .. }
        | HirExpr::Await { value: body, .. }
        | HirExpr::UnaryOp { operand: body, .. }
        | HirExpr::FieldAccess { object: body, .. }
        | HirExpr::WalrusExpr { value: body, .. } => {
            collect_python_callback_bound_names_expr(body, callable_params, names);
        }
        HirExpr::BinOp { left, right, .. } => {
            collect_python_callback_bound_names_expr(left, callable_params, names);
            collect_python_callback_bound_names_expr(right, callable_params, names);
        }
        HirExpr::Compare {
            left, comparators, ..
        } => {
            collect_python_callback_bound_names_expr(left, callable_params, names);
            for comparator in comparators {
                collect_python_callback_bound_names_expr(comparator, callable_params, names);
            }
        }
        HirExpr::BoolOp { values, .. } => {
            for value in values {
                collect_python_callback_bound_names_expr(value, callable_params, names);
            }
        }
        HirExpr::IfExpr {
            condition,
            then_expr,
            else_expr,
            ..
        } => {
            collect_python_callback_bound_names_expr(condition, callable_params, names);
            collect_python_callback_bound_names_expr(then_expr, callable_params, names);
            collect_python_callback_bound_names_expr(else_expr, callable_params, names);
        }
        HirExpr::Index { object, index, .. } => {
            collect_python_callback_bound_names_expr(object, callable_params, names);
            collect_python_callback_bound_names_expr(index, callable_params, names);
        }
        HirExpr::MethodCall { object, args, .. } => {
            collect_python_callback_bound_names_expr(object, callable_params, names);
            for arg in args {
                collect_python_callback_bound_names_expr(arg, callable_params, names);
            }
        }
        HirExpr::DictLiteral { keys, values, .. } => {
            for value in keys.iter().chain(values.iter()) {
                collect_python_callback_bound_names_expr(value, callable_params, names);
            }
        }
        HirExpr::FString { parts, .. } => {
            for part in parts {
                if let sifr_ir::HirFStringPart::Expr(value) = part {
                    collect_python_callback_bound_names_expr(value, callable_params, names);
                }
            }
        }
        HirExpr::Slice {
            object,
            start,
            stop,
            step,
            ..
        } => {
            collect_python_callback_bound_names_expr(object, callable_params, names);
            if let Some(value) = start {
                collect_python_callback_bound_names_expr(value, callable_params, names);
            }
            if let Some(value) = stop {
                collect_python_callback_bound_names_expr(value, callable_params, names);
            }
            if let Some(value) = step {
                collect_python_callback_bound_names_expr(value, callable_params, names);
            }
        }
        HirExpr::RangeLiteral {
            start, end, step, ..
        } => {
            collect_python_callback_bound_names_expr(start, callable_params, names);
            collect_python_callback_bound_names_expr(end, callable_params, names);
            if let Some(value) = step {
                collect_python_callback_bound_names_expr(value, callable_params, names);
            }
        }
        HirExpr::ContainsOp {
            element,
            collection,
            ..
        } => {
            collect_python_callback_bound_names_expr(element, callable_params, names);
            collect_python_callback_bound_names_expr(collection, callable_params, names);
        }
        HirExpr::ListComp {
            expr, generators, ..
        }
        | HirExpr::SetComp {
            expr, generators, ..
        } => {
            collect_python_callback_bound_names_expr(expr, callable_params, names);
            for (_, iter, filter) in generators {
                collect_python_callback_bound_names_expr(iter, callable_params, names);
                if let Some(filter) = filter {
                    collect_python_callback_bound_names_expr(filter, callable_params, names);
                }
            }
        }
        HirExpr::DictComp {
            key_expr,
            val_expr,
            generators,
            ..
        } => {
            collect_python_callback_bound_names_expr(key_expr, callable_params, names);
            collect_python_callback_bound_names_expr(val_expr, callable_params, names);
            for (_, iter, filter) in generators {
                collect_python_callback_bound_names_expr(iter, callable_params, names);
                if let Some(filter) = filter {
                    collect_python_callback_bound_names_expr(filter, callable_params, names);
                }
            }
        }
        HirExpr::GeneratorExpr {
            expr, iter, filter, ..
        } => {
            collect_python_callback_bound_names_expr(expr, callable_params, names);
            collect_python_callback_bound_names_expr(iter, callable_params, names);
            if let Some(filter) = filter {
                collect_python_callback_bound_names_expr(filter, callable_params, names);
            }
        }
        HirExpr::Name { .. }
        | HirExpr::IntLiteral(_)
        | HirExpr::LargeIntLiteral(_)
        | HirExpr::FloatLiteral(_)
        | HirExpr::StringLiteral(_)
        | HirExpr::BoolLiteral(_)
        | HirExpr::NoneLiteral
        | HirExpr::EnumVariant { .. } => {}
    }
}

fn collect_callable_names_from_callback_arg(
    expr: &HirExpr,
    callable_params: &HashSet<String>,
    names: &mut HashSet<String>,
) {
    match expr {
        HirExpr::Name { name, .. } if callable_params.contains(name) => {
            names.insert(name.clone());
        }
        HirExpr::Lambda { body, .. } => {
            collect_callable_param_name_refs(body, callable_params, names);
        }
        other => collect_python_callback_bound_names_expr(other, callable_params, names),
    }
}

fn collect_callable_param_name_refs(
    expr: &HirExpr,
    callable_params: &HashSet<String>,
    names: &mut HashSet<String>,
) {
    match expr {
        HirExpr::Name { name, .. } if callable_params.contains(name) => {
            names.insert(name.clone());
        }
        HirExpr::Call { args, .. }
        | HirExpr::IteratorCall { args, .. }
        | HirExpr::TupleLiteral { elements: args, .. }
        | HirExpr::ListLiteral { elements: args, .. }
        | HirExpr::SetLiteral { elements: args, .. }
        | HirExpr::ConstructorCall { args, .. }
        | HirExpr::SuperCall { args, .. } => {
            for arg in args {
                collect_callable_param_name_refs(arg, callable_params, names);
            }
        }
        HirExpr::Lambda { body, .. }
        | HirExpr::QuestionMark { expr: body, .. }
        | HirExpr::OkWrap { value: body, .. }
        | HirExpr::ErrWrap { value: body, .. }
        | HirExpr::Await { value: body, .. }
        | HirExpr::UnaryOp { operand: body, .. }
        | HirExpr::FieldAccess { object: body, .. }
        | HirExpr::WalrusExpr { value: body, .. } => {
            collect_callable_param_name_refs(body, callable_params, names);
        }
        HirExpr::BinOp { left, right, .. } => {
            collect_callable_param_name_refs(left, callable_params, names);
            collect_callable_param_name_refs(right, callable_params, names);
        }
        HirExpr::Compare {
            left, comparators, ..
        } => {
            collect_callable_param_name_refs(left, callable_params, names);
            for comparator in comparators {
                collect_callable_param_name_refs(comparator, callable_params, names);
            }
        }
        HirExpr::BoolOp { values, .. } => {
            for value in values {
                collect_callable_param_name_refs(value, callable_params, names);
            }
        }
        HirExpr::IfExpr {
            condition,
            then_expr,
            else_expr,
            ..
        } => {
            collect_callable_param_name_refs(condition, callable_params, names);
            collect_callable_param_name_refs(then_expr, callable_params, names);
            collect_callable_param_name_refs(else_expr, callable_params, names);
        }
        HirExpr::Index { object, index, .. } => {
            collect_callable_param_name_refs(object, callable_params, names);
            collect_callable_param_name_refs(index, callable_params, names);
        }
        HirExpr::MethodCall { object, args, .. } => {
            collect_callable_param_name_refs(object, callable_params, names);
            for arg in args {
                collect_callable_param_name_refs(arg, callable_params, names);
            }
        }
        HirExpr::DictLiteral { keys, values, .. } => {
            for value in keys.iter().chain(values.iter()) {
                collect_callable_param_name_refs(value, callable_params, names);
            }
        }
        HirExpr::FString { parts, .. } => {
            for part in parts {
                if let sifr_ir::HirFStringPart::Expr(value) = part {
                    collect_callable_param_name_refs(value, callable_params, names);
                }
            }
        }
        HirExpr::Slice {
            object,
            start,
            stop,
            step,
            ..
        } => {
            collect_callable_param_name_refs(object, callable_params, names);
            if let Some(value) = start {
                collect_callable_param_name_refs(value, callable_params, names);
            }
            if let Some(value) = stop {
                collect_callable_param_name_refs(value, callable_params, names);
            }
            if let Some(value) = step {
                collect_callable_param_name_refs(value, callable_params, names);
            }
        }
        HirExpr::RangeLiteral {
            start, end, step, ..
        } => {
            collect_callable_param_name_refs(start, callable_params, names);
            collect_callable_param_name_refs(end, callable_params, names);
            if let Some(value) = step {
                collect_callable_param_name_refs(value, callable_params, names);
            }
        }
        HirExpr::ContainsOp {
            element,
            collection,
            ..
        } => {
            collect_callable_param_name_refs(element, callable_params, names);
            collect_callable_param_name_refs(collection, callable_params, names);
        }
        HirExpr::ListComp {
            expr, generators, ..
        }
        | HirExpr::SetComp {
            expr, generators, ..
        } => {
            collect_callable_param_name_refs(expr, callable_params, names);
            for (_, iter, filter) in generators {
                collect_callable_param_name_refs(iter, callable_params, names);
                if let Some(filter) = filter {
                    collect_callable_param_name_refs(filter, callable_params, names);
                }
            }
        }
        HirExpr::DictComp {
            key_expr,
            val_expr,
            generators,
            ..
        } => {
            collect_callable_param_name_refs(key_expr, callable_params, names);
            collect_callable_param_name_refs(val_expr, callable_params, names);
            for (_, iter, filter) in generators {
                collect_callable_param_name_refs(iter, callable_params, names);
                if let Some(filter) = filter {
                    collect_callable_param_name_refs(filter, callable_params, names);
                }
            }
        }
        HirExpr::GeneratorExpr {
            expr, iter, filter, ..
        } => {
            collect_callable_param_name_refs(expr, callable_params, names);
            collect_callable_param_name_refs(iter, callable_params, names);
            if let Some(filter) = filter {
                collect_callable_param_name_refs(filter, callable_params, names);
            }
        }
        HirExpr::Name { .. }
        | HirExpr::IntLiteral(_)
        | HirExpr::LargeIntLiteral(_)
        | HirExpr::FloatLiteral(_)
        | HirExpr::StringLiteral(_)
        | HirExpr::BoolLiteral(_)
        | HirExpr::NoneLiteral
        | HirExpr::EnumVariant { .. } => {}
    }
}

use super::{
    HirExpr, HirIteratorOp, HirStmt, RustEmitter,
    performance_lowering_gate::stmt_needs_performance_lowering,
};
use sifr_type_system::Type;

#[derive(Default)]
struct StringLoopTargetUse {
    found: bool,
    valid: bool,
}

fn is_string_like_set_type(ty: &Type) -> bool {
    match ty.resolve_alias() {
        Type::Any | Type::Unknown => true,
        Type::Set(elem) => matches!(
            elem.as_ref().resolve_alias(),
            Type::Str | Type::LiteralStr(_) | Type::Any | Type::Unknown | Type::TypeVar(_)
        ),
        _ => false,
    }
}

fn is_target_name(expr: &HirExpr, target: &str) -> bool {
    matches!(expr, HirExpr::Name { name, .. } if name == target)
}

fn is_direct_string_type(ty: &Type) -> bool {
    matches!(ty.resolve_alias(), Type::Str | Type::LiteralStr(_))
}

fn is_string_for_loop_iter(iter: &HirExpr) -> bool {
    match iter {
        HirExpr::IteratorCall { op, args, .. } if *op == HirIteratorOp::Iter && args.len() == 1 => {
            is_direct_string_type(args[0].ty())
        }
        HirExpr::Call { func, args, .. } if func == "iter" && args.len() == 1 => {
            is_direct_string_type(args[0].ty())
        }
        _ => is_direct_string_type(iter.ty()),
    }
}

fn collect_string_loop_target_use_expr(
    expr: &HirExpr,
    target: &str,
    usage: &mut StringLoopTargetUse,
) {
    if !usage.valid {
        return;
    }
    match expr {
        HirExpr::Name { name, .. } if name == target => {
            usage.valid = false;
        }
        HirExpr::ContainsOp {
            element,
            collection,
            ..
        } if is_target_name(element, target) && is_string_like_set_type(collection.ty()) => {
            usage.found = true;
            collect_string_loop_target_use_expr(collection, target, usage);
        }
        HirExpr::MethodCall {
            object,
            method,
            args,
            ..
        } if matches!(
            method.as_str(),
            "add" | "insert" | "remove" | "discard" | "contains"
        ) && !matches!(
            object.ty().resolve_alias(),
            Type::Class { .. } | Type::List(_)
        ) && args.len() == 1
            && is_target_name(&args[0], target) =>
        {
            usage.found = true;
            collect_string_loop_target_use_expr(object, target, usage);
        }
        HirExpr::BinOp { left, right, .. } => {
            collect_string_loop_target_use_expr(left, target, usage);
            collect_string_loop_target_use_expr(right, target, usage);
        }
        HirExpr::UnaryOp { operand, .. }
        | HirExpr::Await { value: operand, .. }
        | HirExpr::QuestionMark { expr: operand, .. }
        | HirExpr::OkWrap { value: operand, .. }
        | HirExpr::ErrWrap { value: operand, .. }
        | HirExpr::StructuralRecordProject {
            source: operand, ..
        } => {
            collect_string_loop_target_use_expr(operand, target, usage);
        }
        HirExpr::Compare {
            left, comparators, ..
        } => {
            collect_string_loop_target_use_expr(left, target, usage);
            for comparator in comparators {
                collect_string_loop_target_use_expr(comparator, target, usage);
            }
        }
        HirExpr::BoolOp { values, .. }
        | HirExpr::Call { args: values, .. }
        | HirExpr::GenericCall { args: values, .. }
        | HirExpr::PythonCall { args: values, .. }
        | HirExpr::IntrinsicCall { args: values, .. }
        | HirExpr::IteratorCall { args: values, .. }
        | HirExpr::ListLiteral {
            elements: values, ..
        }
        | HirExpr::SetLiteral {
            elements: values, ..
        }
        | HirExpr::TupleLiteral {
            elements: values, ..
        }
        | HirExpr::ConstructorCall { args: values, .. }
        | HirExpr::SuperCall { args: values, .. } => {
            for value in values {
                collect_string_loop_target_use_expr(value, target, usage);
            }
        }
        HirExpr::IfExpr {
            condition,
            then_expr,
            else_expr,
            ..
        } => {
            collect_string_loop_target_use_expr(condition, target, usage);
            collect_string_loop_target_use_expr(then_expr, target, usage);
            collect_string_loop_target_use_expr(else_expr, target, usage);
        }
        HirExpr::RangeLiteral {
            start, end, step, ..
        } => {
            collect_string_loop_target_use_expr(start, target, usage);
            collect_string_loop_target_use_expr(end, target, usage);
            if let Some(step) = step {
                collect_string_loop_target_use_expr(step, target, usage);
            }
        }
        HirExpr::DictLiteral { keys, values, .. } => {
            for value in keys.iter().chain(values.iter()) {
                collect_string_loop_target_use_expr(value, target, usage);
            }
        }
        HirExpr::Index { object, index, .. } => {
            collect_string_loop_target_use_expr(object, target, usage);
            collect_string_loop_target_use_expr(index, target, usage);
        }
        HirExpr::MethodCall { object, args, .. } => {
            collect_string_loop_target_use_expr(object, target, usage);
            for arg in args {
                collect_string_loop_target_use_expr(arg, target, usage);
            }
        }
        HirExpr::ContainsOp {
            element,
            collection,
            ..
        } => {
            collect_string_loop_target_use_expr(element, target, usage);
            collect_string_loop_target_use_expr(collection, target, usage);
        }
        HirExpr::FString { parts, .. } => {
            for part in parts {
                if let sifr_ir::HirFStringPart::Expr(expr) = part {
                    collect_string_loop_target_use_expr(expr, target, usage);
                }
            }
        }
        HirExpr::TemplateString(template) => template.for_each_value(&mut |expr| {
            collect_string_loop_target_use_expr(expr, target, usage);
        }),
        HirExpr::Slice {
            object,
            start,
            stop,
            step,
            ..
        } => {
            collect_string_loop_target_use_expr(object, target, usage);
            if let Some(start) = start {
                collect_string_loop_target_use_expr(start, target, usage);
            }
            if let Some(stop) = stop {
                collect_string_loop_target_use_expr(stop, target, usage);
            }
            if let Some(step) = step {
                collect_string_loop_target_use_expr(step, target, usage);
            }
        }
        HirExpr::WalrusExpr { name, value, .. } => {
            if name == target {
                usage.valid = false;
                return;
            }
            collect_string_loop_target_use_expr(value, target, usage);
        }
        HirExpr::FieldAccess { object, .. } | HirExpr::Lambda { body: object, .. } => {
            collect_string_loop_target_use_expr(object, target, usage);
        }
        HirExpr::ListComp {
            expr, generators, ..
        }
        | HirExpr::SetComp {
            expr, generators, ..
        } => {
            collect_string_loop_target_use_expr(expr, target, usage);
            for (name, iter, filter) in generators {
                if name == target {
                    usage.valid = false;
                    return;
                }
                collect_string_loop_target_use_expr(iter, target, usage);
                if let Some(filter) = filter {
                    collect_string_loop_target_use_expr(filter, target, usage);
                }
            }
        }
        HirExpr::DictComp {
            key_expr,
            val_expr,
            generators,
            ..
        } => {
            collect_string_loop_target_use_expr(key_expr, target, usage);
            collect_string_loop_target_use_expr(val_expr, target, usage);
            for (name, iter, filter) in generators {
                if name == target {
                    usage.valid = false;
                    return;
                }
                collect_string_loop_target_use_expr(iter, target, usage);
                if let Some(filter) = filter {
                    collect_string_loop_target_use_expr(filter, target, usage);
                }
            }
        }
        HirExpr::GeneratorExpr {
            expr,
            var,
            iter,
            filter,
            ..
        } => {
            if var == target {
                usage.valid = false;
                return;
            }
            collect_string_loop_target_use_expr(expr, target, usage);
            collect_string_loop_target_use_expr(iter, target, usage);
            if let Some(filter) = filter {
                collect_string_loop_target_use_expr(filter, target, usage);
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

fn collect_string_loop_target_use_stmt(
    stmt: &HirStmt,
    target: &str,
    usage: &mut StringLoopTargetUse,
) {
    if !usage.valid {
        return;
    }
    match stmt {
        HirStmt::Let { name, value, .. } | HirStmt::Assign { name, value } => {
            if name == target {
                usage.valid = false;
                return;
            }
            collect_string_loop_target_use_expr(value, target, usage);
        }
        HirStmt::AugAssign { name, value, .. }
        | HirStmt::SubscriptAugAssign {
            object: name,
            value,
            ..
        }
        | HirStmt::AttributeAugAssign {
            object: name,
            value,
            ..
        } => {
            if name == target {
                usage.valid = false;
                return;
            }
            collect_string_loop_target_use_expr(value, target, usage);
        }
        HirStmt::Return { value } => {
            if let Some(value) = value {
                collect_string_loop_target_use_expr(value, target, usage);
            }
        }
        HirStmt::Expr { expr }
        | HirStmt::Assert {
            test: expr,
            msg: None,
        }
        | HirStmt::Raise { value: expr }
        | HirStmt::Yield { value: expr } => {
            collect_string_loop_target_use_expr(expr, target, usage);
        }
        HirStmt::Assert {
            test,
            msg: Some(msg),
        } => {
            collect_string_loop_target_use_expr(test, target, usage);
            collect_string_loop_target_use_expr(msg, target, usage);
        }
        HirStmt::If {
            condition,
            then_body,
            elif_clauses,
            else_body,
        } => {
            collect_string_loop_target_use_expr(condition, target, usage);
            for stmt in then_body {
                collect_string_loop_target_use_stmt(stmt, target, usage);
            }
            for (condition, body) in elif_clauses {
                collect_string_loop_target_use_expr(condition, target, usage);
                for stmt in body {
                    collect_string_loop_target_use_stmt(stmt, target, usage);
                }
            }
            if let Some(else_body) = else_body {
                for stmt in else_body {
                    collect_string_loop_target_use_stmt(stmt, target, usage);
                }
            }
        }
        HirStmt::While {
            condition, body, ..
        } => {
            collect_string_loop_target_use_expr(condition, target, usage);
            for stmt in body {
                collect_string_loop_target_use_stmt(stmt, target, usage);
            }
        }
        HirStmt::For {
            target: loop_target,
            iter,
            body,
            ..
        }
        | HirStmt::AsyncFor {
            target: loop_target,
            iter,
            body,
            ..
        } => {
            if loop_target == target {
                usage.valid = false;
                return;
            }
            collect_string_loop_target_use_expr(iter, target, usage);
            for stmt in body {
                collect_string_loop_target_use_stmt(stmt, target, usage);
            }
        }
        HirStmt::TupleUnpack { targets, value } => {
            if targets.iter().any(|tuple_target| {
                matches!(
                    &tuple_target.binding,
                    sifr_ir::HirTupleTargetBinding::Name(name) if name == target
                )
            }) {
                usage.valid = false;
                return;
            }
            collect_string_loop_target_use_expr(value, target, usage);
        }
        HirStmt::StarUnpack {
            before,
            star,
            after,
            value,
            ..
        } => {
            if before
                .iter()
                .chain(std::iter::once(star))
                .chain(after)
                .any(|unpack_target| {
                    matches!(
                        &unpack_target.binding,
                        sifr_ir::HirTupleTargetBinding::Name(name) if name == target
                    )
                })
            {
                usage.valid = false;
                return;
            }
            collect_string_loop_target_use_expr(value, target, usage);
        }
        HirStmt::FieldAssign { object, value, .. }
        | HirStmt::SubscriptAssign { object, value, .. }
        | HirStmt::NestedSubscriptAssign { object, value, .. }
        | HirStmt::AttributeNestedSubscriptAssign { object, value, .. } => {
            if object == target {
                usage.valid = false;
                return;
            }
            collect_string_loop_target_use_expr(value, target, usage);
        }
        HirStmt::NestedFieldAssign { object, value, .. }
        | HirStmt::AttributeSubscriptAssign { object, value, .. } => {
            if object == target {
                usage.valid = false;
                return;
            }
            collect_string_loop_target_use_expr(value, target, usage);
        }
        HirStmt::Delete { object, index, .. } => {
            collect_string_loop_target_use_expr(object, target, usage);
            collect_string_loop_target_use_expr(index, target, usage);
        }
        HirStmt::TryExcept { body, handlers, .. } => {
            for stmt in body {
                collect_string_loop_target_use_stmt(stmt, target, usage);
            }
            for handler in handlers {
                if handler.name.as_deref() == Some(target) {
                    usage.valid = false;
                    return;
                }
                for stmt in &handler.body {
                    collect_string_loop_target_use_stmt(stmt, target, usage);
                }
            }
        }
        HirStmt::TryFinally { body, finalbody } => {
            for stmt in body.iter().chain(finalbody.iter()) {
                collect_string_loop_target_use_stmt(stmt, target, usage);
            }
        }
        HirStmt::With { items, body } => {
            for item in items {
                if item.target == target {
                    usage.valid = false;
                    return;
                }
                collect_string_loop_target_use_expr(&item.context, target, usage);
            }
            for stmt in body {
                collect_string_loop_target_use_stmt(stmt, target, usage);
            }
        }
        HirStmt::AsyncWith {
            target: binding,
            body,
            ..
        } => {
            if binding.as_deref() == Some(target) {
                usage.valid = false;
                return;
            }
            for stmt in body {
                collect_string_loop_target_use_stmt(stmt, target, usage);
            }
        }
        HirStmt::Match { subject, arms, .. } => {
            collect_string_loop_target_use_expr(subject, target, usage);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    collect_string_loop_target_use_expr(guard, target, usage);
                }
                for stmt in &arm.body {
                    collect_string_loop_target_use_stmt(stmt, target, usage);
                }
            }
        }
        HirStmt::Pass | HirStmt::Break | HirStmt::Continue | HirStmt::NestedFunction { .. } => {}
    }
}

impl RustEmitter {
    pub(crate) fn try_lower_simple_block_stmt_for_ir(
        &self,
        stmt: &HirStmt,
    ) -> Result<Option<Vec<crate::RustStmt>>, crate::CodegenError> {
        if (self.try_closure_depth > 0 && !matches!(stmt, HirStmt::AttributeAugAssign { .. }))
            || !self.active_timeout_durations.is_empty()
            || self.stmt_uses_checked_place_read_witness(stmt)
            || Self::stmt_defines_nonempty_list(stmt)
        {
            return Ok(None);
        }
        crate::try_lower_simple_stmt_with_scope_result_and_bindings(
            stmt,
            &self.mutated_vars,
            &self.borrowed_params,
            &self.mut_borrowed_params,
            &self.local_binding_types,
            &self.recursive_fields,
            &self.stmt_block_scope_context(),
        )
    }

    pub(crate) fn try_lower_stmt_block_for_ir(
        &mut self,
        stmts: &[HirStmt],
    ) -> Result<Option<Vec<crate::RustStmt>>, crate::CodegenError> {
        self.stmt_block_depth += 1;
        let result = self.try_lower_stmt_block_for_ir_inner(stmts);
        self.stmt_block_depth -= 1;
        result
    }

    pub(crate) fn try_lower_scoped_stmt_block_for_ir(
        &mut self,
        stmts: &[HirStmt],
    ) -> Result<Option<Vec<crate::RustStmt>>, crate::CodegenError> {
        let string_char_cache_vars = self.string_char_cache_vars.clone();
        let callable_var_conventions = self.callable_var_conventions.clone();
        let nested_fn_captures = self.nested_fn_captures.clone();
        let nonempty_list_bindings = self.nonempty_list_bindings.clone();
        let checked_place_read_witnesses = self.checked_place_read_witnesses.clone();
        let recursive_option_borrowed_views = self.recursive_option_borrowed_views.clone();
        let result = self.try_lower_stmt_block_for_ir(stmts);
        self.string_char_cache_vars = string_char_cache_vars;
        self.callable_var_conventions = callable_var_conventions;
        self.nested_fn_captures = nested_fn_captures;
        self.nonempty_list_bindings = nonempty_list_bindings;
        self.checked_place_read_witnesses = checked_place_read_witnesses;
        self.recursive_option_borrowed_views = recursive_option_borrowed_views;
        result
    }

    pub(crate) fn stmt_block_scope_context(&self) -> crate::ScopeContext {
        crate::ScopeContext {
            function_return_type: self.current_return_type.clone(),
            in_generator_closure: self.emission_ctx.in_generator_closure,
            in_display_impl: self.emission_ctx.in_display_impl,
            in_loop_with_else: self.current_loop_has_else(),
            class_scope: if self.current_class_name.is_some() {
                crate::ClassScope::Inside
            } else {
                crate::ClassScope::Outside
            },
        }
    }

    pub(crate) fn should_bypass_simple_block_lowering(&self, stmt: &HirStmt) -> bool {
        matches!(
            stmt,
            HirStmt::NestedFunction { .. }
                | HirStmt::Let { .. }
                | HirStmt::Assign { .. }
                | HirStmt::Expr { .. }
                | HirStmt::If { .. }
                | HirStmt::While { .. }
                | HirStmt::For { .. }
                | HirStmt::AsyncFor { .. }
                | HirStmt::Delete { .. }
                | HirStmt::SubscriptAssign { .. }
                | HirStmt::NestedSubscriptAssign { .. }
                | HirStmt::AttributeNestedSubscriptAssign { .. }
                | HirStmt::AttributeSubscriptAssign { .. }
                | HirStmt::SubscriptAugAssign { .. }
                | HirStmt::StarUnpack { .. }
                | HirStmt::Raise { .. }
        ) || matches!(stmt, HirStmt::Let { ty, .. } if self.type_contains_generic_class(ty))
            || matches!(stmt, HirStmt::TupleUnpack { targets, .. } if targets.iter().any(|target| {
                let sifr_ir::HirTupleTargetBinding::Name(name) = &target.binding else {
                    return false;
                };
                self.string_char_cache_required_names.contains(name)
                    || matches!(
                        crate::resolve_alias_type_for_plain_call(&target.ty),
                        Type::Str | Type::LiteralStr(_)
                    )
            }))
            || matches!(stmt, HirStmt::StarUnpack { before, star, after, .. } if before
            .iter()
            .chain(std::iter::once(star))
            .chain(after)
            .any(|target| {
                let sifr_ir::HirTupleTargetBinding::Name(name) = &target.binding else {
                    return false;
                };
                self.string_char_cache_required_names.contains(name)
                    || matches!(
                        crate::resolve_alias_type_for_plain_call(&target.ty),
                        Type::Str | Type::LiteralStr(_)
                    )
            }))
            || stmt_needs_performance_lowering(stmt)
            || self.body_analysis.aggregate_statement_has_last_use(stmt)
    }

    pub(crate) fn try_lower_borrowed_move_name_clone_for_ir(
        &self,
        effective_ty: &Type,
        value: &HirExpr,
    ) -> Option<crate::RustExpr> {
        if crate::helpers::is_copy_type_for_codegen(effective_ty) {
            return None;
        }
        let HirExpr::Name { name, .. } = value else {
            return None;
        };
        if self.borrowed_params.contains(name)
            || self.mut_borrowed_params.contains(name)
            || self.recursive_option_borrowed_views.contains(name)
        {
            Some(crate::ownership_plan::materialize_owned_value(
                effective_ty,
                crate::RustExpr::Ident(name.clone()),
            ))
        } else {
            None
        }
    }

    pub(crate) fn is_borrowed_empty_list_get_expr_for_ir(value: &crate::RustExpr) -> bool {
        matches!(
            value,
            crate::RustExpr::MethodCall { method, args, receiver }
                if method == "unwrap_or"
                    && matches!(args.as_slice(), [crate::RustExpr::Ident(default)] if default == "&[]")
                    && matches!(receiver.as_ref(), crate::RustExpr::MethodCall { method, args, .. }
                        if method == "map"
                            && matches!(args.as_slice(), [crate::RustExpr::Path(path)] if path == &["Vec".to_string(), "as_slice".to_string()]))
        )
    }

    pub(crate) fn should_lower_string_set_loop_target_as_char(
        target: &str,
        target_ty: &Type,
        iter: &HirExpr,
        body: &[HirStmt],
    ) -> bool {
        if target.contains(',')
            || !matches!(
                target_ty.resolve_alias(),
                Type::Str | Type::LiteralStr(_) | Type::Any | Type::Unknown | Type::TypeVar(_)
            )
            || !is_string_for_loop_iter(iter)
        {
            return false;
        }
        let mut usage = StringLoopTargetUse {
            found: false,
            valid: true,
        };
        for stmt in body {
            collect_string_loop_target_use_stmt(stmt, target, &mut usage);
            if !usage.valid {
                return false;
            }
        }
        usage.found
    }
}

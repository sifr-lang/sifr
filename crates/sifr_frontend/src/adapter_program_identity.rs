//! Cache-key bridge from early class adaptation to static specialization.

use sifr_lowering::{
    HirAsyncWithKind, HirExpr, HirFunction, HirModule, HirPattern, HirStmt, LoweringResult,
    PythonArrowSchemaMode, PythonDlpackStreamMode, PythonInteropDeclaration,
    RustInteropDeclaration, RustInteropValue,
};
use std::fmt::Write;

pub(crate) fn canonical_const_functions(module: &HirModule) -> String {
    let mut functions = module.functions.clone();
    functions.sort_by(|left, right| left.name.cmp(&right.name));
    for function in &mut functions {
        strip_function_locations(function);
    }
    format!("{functions:#?}")
}

fn strip_function_locations(function: &mut HirFunction) {
    for parameter in &mut function.params {
        if let Some(default) = &mut parameter.default {
            strip_expr_locations(default);
        }
    }
    strip_stmt_locations(&mut function.body);
    for declaration in &mut function.rust_interop {
        strip_rust_interop_locations(declaration);
    }
    for declaration in &mut function.python_interop {
        strip_python_interop_locations(declaration);
    }
}

fn strip_rust_interop_locations(declaration: &mut RustInteropDeclaration) {
    declaration.span = ruff_text_size::TextRange::default();
    if let Some(target) = &mut declaration.target {
        target.span = ruff_text_size::TextRange::default();
    }
    for argument in &mut declaration.arguments {
        argument.span = ruff_text_size::TextRange::default();
        strip_rust_interop_value_locations(&mut argument.value);
    }
}

fn strip_rust_interop_value_locations(value: &mut RustInteropValue) {
    match value {
        RustInteropValue::PolicyCall { argument, span, .. } => {
            *span = ruff_text_size::TextRange::default();
            strip_rust_interop_value_locations(argument);
        }
        RustInteropValue::TargetPath(target) => {
            target.span = ruff_text_size::TextRange::default();
        }
        RustInteropValue::Boolean(_)
        | RustInteropValue::Symbol(_)
        | RustInteropValue::Integer(_)
        | RustInteropValue::IntegerList(_) => {}
    }
}

fn strip_python_interop_locations(declaration: &mut PythonInteropDeclaration) {
    declaration.span = ruff_text_size::TextRange::default();
    if let Some(target) = &mut declaration.target {
        target.span = ruff_text_size::TextRange::default();
    }
    for parameter in &mut declaration.parameters {
        parameter.span = ruff_text_size::TextRange::default();
    }
    for callback in &mut declaration.callbacks {
        callback.span = ruff_text_size::TextRange::default();
    }
    if let Some(arrow) = &mut declaration.arrow {
        if let PythonArrowSchemaMode::Parameter { span, .. } = &mut arrow.schema {
            *span = ruff_text_size::TextRange::default();
        }
    }
    if let Some(dlpack) = &mut declaration.dlpack {
        if let PythonDlpackStreamMode::Parameter { span, .. } = &mut dlpack.stream {
            *span = ruff_text_size::TextRange::default();
        }
    }
}

fn strip_stmt_locations(statements: &mut [HirStmt]) {
    for statement in statements {
        match statement {
            HirStmt::Let { value, .. }
            | HirStmt::Assign { value, .. }
            | HirStmt::AugAssign { value, .. }
            | HirStmt::Raise { value }
            | HirStmt::Yield { value } => strip_expr_locations(value),
            HirStmt::Return { value } => {
                if let Some(value) = value {
                    strip_expr_locations(value);
                }
            }
            HirStmt::Expr { expr } => strip_expr_locations(expr),
            HirStmt::If {
                condition,
                then_body,
                elif_clauses,
                else_body,
            } => {
                strip_expr_locations(condition);
                strip_stmt_locations(then_body);
                for (condition, body) in elif_clauses {
                    strip_expr_locations(condition);
                    strip_stmt_locations(body);
                }
                if let Some(body) = else_body {
                    strip_stmt_locations(body);
                }
            }
            HirStmt::While {
                condition,
                body,
                else_body,
            } => {
                strip_expr_locations(condition);
                strip_stmt_locations(body);
                if let Some(body) = else_body {
                    strip_stmt_locations(body);
                }
            }
            HirStmt::For {
                iter,
                body,
                else_body,
                ..
            }
            | HirStmt::AsyncFor {
                iter,
                body,
                else_body,
                ..
            } => {
                strip_expr_locations(iter);
                strip_stmt_locations(body);
                if let Some(body) = else_body {
                    strip_stmt_locations(body);
                }
            }
            HirStmt::TupleUnpack { value, .. } | HirStmt::StarUnpack { value, .. } => {
                strip_expr_locations(value);
            }
            HirStmt::Assert { test, msg } => {
                strip_expr_locations(test);
                if let Some(message) = msg {
                    strip_expr_locations(message);
                }
            }
            HirStmt::TryExcept { body, handlers, .. } => {
                strip_stmt_locations(body);
                for handler in handlers {
                    strip_stmt_locations(&mut handler.body);
                }
            }
            HirStmt::TryFinally { body, finalbody } => {
                strip_stmt_locations(body);
                strip_stmt_locations(finalbody);
            }
            HirStmt::FieldAssign { value, .. }
            | HirStmt::NestedFieldAssign { value, .. }
            | HirStmt::AttributeAugAssign { value, .. } => strip_expr_locations(value),
            HirStmt::SubscriptAssign { index, value, .. }
            | HirStmt::SubscriptAugAssign { index, value, .. }
            | HirStmt::AttributeSubscriptAssign { index, value, .. } => {
                strip_expr_locations(index);
                strip_expr_locations(value);
            }
            HirStmt::NestedSubscriptAssign {
                outer_index,
                inner_index,
                value,
                ..
            }
            | HirStmt::AttributeNestedSubscriptAssign {
                outer_index,
                inner_index,
                value,
                ..
            } => {
                strip_expr_locations(outer_index);
                strip_expr_locations(inner_index);
                strip_expr_locations(value);
            }
            HirStmt::Delete { object, index } => {
                strip_expr_locations(object);
                strip_expr_locations(index);
            }
            HirStmt::With { items, body } => {
                for item in items {
                    strip_expr_locations(&mut item.context);
                }
                strip_stmt_locations(body);
            }
            HirStmt::AsyncWith { kind, body, .. } => {
                strip_async_with_locations(kind);
                strip_stmt_locations(body);
            }
            HirStmt::NestedFunction { func, .. } => strip_function_locations(func),
            HirStmt::Match { subject, arms, .. } => {
                strip_expr_locations(subject);
                for arm in arms {
                    strip_pattern_locations(&mut arm.pattern);
                    if let Some(guard) = &mut arm.guard {
                        strip_expr_locations(guard);
                    }
                    strip_stmt_locations(&mut arm.body);
                }
            }
            HirStmt::Break | HirStmt::Continue | HirStmt::Pass => {}
        }
    }
}

fn strip_expr_locations(expression: &mut HirExpr) {
    match expression {
        HirExpr::IntLiteral(_)
        | HirExpr::LargeIntLiteral(_)
        | HirExpr::FloatLiteral(_)
        | HirExpr::StringLiteral(_)
        | HirExpr::BoolLiteral(_)
        | HirExpr::NoneLiteral
        | HirExpr::EnumVariant { .. } => {}
        HirExpr::Name { binding_id, .. } => *binding_id = None,
        HirExpr::BinOp { left, right, .. } => {
            strip_expr_locations(left);
            strip_expr_locations(right);
        }
        HirExpr::UnaryOp { operand, .. }
        | HirExpr::Await { value: operand, .. }
        | HirExpr::QuestionMark { expr: operand, .. }
        | HirExpr::OkWrap { value: operand, .. }
        | HirExpr::ErrWrap { value: operand, .. } => strip_expr_locations(operand),
        HirExpr::Compare {
            left, comparators, ..
        } => {
            strip_expr_locations(left);
            strip_expr_list_locations(comparators);
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
        } => strip_expr_list_locations(values),
        HirExpr::Call {
            args,
            mutable_arg_places,
            ..
        }
        | HirExpr::GenericCall {
            args,
            mutable_arg_places,
            ..
        }
        | HirExpr::IteratorCall {
            args,
            mutable_arg_places,
            ..
        } => {
            strip_expr_list_locations(args);
            mutable_arg_places.clear();
        }
        HirExpr::ConstructorCall { args, .. } | HirExpr::SuperCall { args, .. } => {
            strip_expr_list_locations(args);
        }
        HirExpr::PythonCall {
            args,
            record_expansions,
            ..
        } => {
            strip_expr_list_locations(args);
            for expansion in record_expansions {
                expansion.span = ruff_text_size::TextRange::default();
            }
        }
        HirExpr::IntrinsicCall {
            args,
            call_range,
            arg_ranges,
            ..
        } => {
            strip_expr_list_locations(args);
            *call_range = ruff_text_size::TextRange::default();
            arg_ranges.clear();
        }
        HirExpr::IfExpr {
            condition,
            then_expr,
            else_expr,
            ..
        } => {
            strip_expr_locations(condition);
            strip_expr_locations(then_expr);
            strip_expr_locations(else_expr);
        }
        HirExpr::RangeLiteral {
            start, end, step, ..
        } => {
            strip_expr_locations(start);
            strip_expr_locations(end);
            if let Some(step) = step {
                strip_expr_locations(step);
            }
        }
        HirExpr::DictLiteral { keys, values, .. } => {
            strip_expr_list_locations(keys);
            strip_expr_list_locations(values);
        }
        HirExpr::Index { object, index, .. }
        | HirExpr::ContainsOp {
            element: object,
            collection: index,
            ..
        } => {
            strip_expr_locations(object);
            strip_expr_locations(index);
        }
        HirExpr::MethodCall {
            object,
            args,
            receiver_target,
            mutable_arg_places,
            source,
            ..
        } => {
            strip_expr_locations(object);
            strip_expr_list_locations(args);
            *receiver_target = None;
            mutable_arg_places.clear();
            *source = None;
        }
        HirExpr::FString { parts, .. } => {
            for part in parts {
                if let sifr_lowering::HirFStringPart::Expr(expression) = part {
                    strip_expr_locations(expression);
                }
            }
        }
        HirExpr::TemplateString(template) => {
            template.for_each_value_mut(&mut strip_expr_locations);
            template.source_range = ruff_text_size::TextRange::default();
            for segment in &mut template.segments {
                segment.mappings.clear();
            }
            for interpolation in &mut template.interpolations {
                interpolation.source_range = ruff_text_size::TextRange::default();
                interpolation.expression_range = ruff_text_size::TextRange::default();
            }
        }
        HirExpr::Slice {
            object,
            start,
            stop,
            step,
            ..
        } => {
            strip_expr_locations(object);
            for bound in [start, stop, step].into_iter().flatten() {
                strip_expr_locations(bound);
            }
        }
        HirExpr::WalrusExpr { value, .. }
        | HirExpr::FieldAccess { object: value, .. }
        | HirExpr::StructuralRecordProject { source: value, .. } => {
            strip_expr_locations(value);
        }
        HirExpr::Lambda { params, body, .. } => {
            for parameter in params {
                if let Some(default) = &mut parameter.default {
                    strip_expr_locations(default);
                }
            }
            strip_expr_locations(body);
        }
        HirExpr::ListComp {
            expr, generators, ..
        }
        | HirExpr::SetComp {
            expr, generators, ..
        } => {
            strip_expr_locations(expr);
            strip_generator_locations(generators);
        }
        HirExpr::DictComp {
            key_expr,
            val_expr,
            generators,
            ..
        } => {
            strip_expr_locations(key_expr);
            strip_expr_locations(val_expr);
            strip_generator_locations(generators);
        }
        HirExpr::GeneratorExpr {
            expr, iter, filter, ..
        } => {
            strip_expr_locations(expr);
            strip_expr_locations(iter);
            if let Some(filter) = filter {
                strip_expr_locations(filter);
            }
        }
    }
}

fn strip_expr_list_locations(expressions: &mut [HirExpr]) {
    for expression in expressions {
        strip_expr_locations(expression);
    }
}

fn strip_generator_locations(generators: &mut [(String, HirExpr, Option<HirExpr>)]) {
    for (_, iter, filter) in generators {
        strip_expr_locations(iter);
        if let Some(filter) = filter {
            strip_expr_locations(filter);
        }
    }
}

fn strip_async_with_locations(kind: &mut HirAsyncWithKind) {
    match kind {
        HirAsyncWithKind::TaskScope => {}
        HirAsyncWithKind::TaskGroup { context } => {
            if let Some(context) = context {
                strip_expr_locations(context);
            }
        }
        HirAsyncWithKind::TaskTimeout { duration } => strip_expr_locations(duration),
        HirAsyncWithKind::UserDefined { context, .. }
        | HirAsyncWithKind::Python { context, .. } => strip_expr_locations(context),
    }
}

fn strip_pattern_locations(pattern: &mut HirPattern) {
    match pattern {
        HirPattern::Literal { value } => strip_expr_locations(value),
        HirPattern::Or { patterns } => {
            for pattern in patterns {
                strip_pattern_locations(pattern);
            }
        }
        HirPattern::Class { fields, .. } => {
            for (_, pattern) in fields {
                strip_pattern_locations(pattern);
            }
        }
        HirPattern::Tuple { elements } => {
            for pattern in elements {
                strip_pattern_locations(pattern);
            }
        }
        HirPattern::Wildcard
        | HirPattern::Capture { .. }
        | HirPattern::None
        | HirPattern::Value { .. } => {}
    }
}

pub(crate) fn post_adapter_hex(result: &LoweringResult, owner: &str) -> String {
    result
        .class_adapter_selections
        .iter()
        .find(|selection| selection.owner == owner)
        .map_or_else(String::new, |selection| {
            selection.post_adapter_identity.iter().fold(
                String::with_capacity(64),
                |mut encoded, byte| {
                    let _ = write!(encoded, "{byte:02x}");
                    encoded
                },
            )
        })
}

#[cfg(test)]
mod tests {
    use super::canonical_const_functions;
    use ruff_text_size::{TextRange, TextSize};
    use sifr_lowering::{
        BindingId, HirExpr, HirFunction, HirModule, HirStmt, MethodKind,
        RustInteropAbiRequirements, RustInteropDeclaration, RustInteropDecoratorKind,
        RustInteropEffect, RustTargetPath,
    };
    use sifr_type_system::Type;
    use std::collections::HashMap;

    fn function(body: Vec<HirStmt>) -> HirFunction {
        HirFunction {
            name: "adapt".to_string(),
            params: Vec::new(),
            return_type: Type::Int,
            body,
            is_async: false,
            method_kind: MethodKind::Regular,
            receiver: None,
            decorators: vec!["const_eval".to_string()],
            rust_interop: Vec::new(),
            python_interop: Vec::new(),
            compiler_intrinsic: None,
            type_params: Vec::new(),
        }
    }

    fn module(function: HirFunction) -> HirModule {
        HirModule {
            functions: vec![function],
            classes: Vec::new(),
            imports: Vec::new(),
            constants: Vec::new(),
            generic_functions: HashMap::new(),
            type_param_bounds: HashMap::new(),
        }
    }

    #[test]
    fn provider_identity_ignores_lowering_binding_ids() {
        let with_id = |id| {
            module(function(vec![HirStmt::Return {
                value: Some(HirExpr::Name {
                    name: "value".to_string(),
                    binding_id: Some(BindingId(id)),
                    ty: Type::Int,
                }),
            }]))
        };

        assert_eq!(
            canonical_const_functions(&with_id(4)),
            canonical_const_functions(&with_id(91))
        );
    }

    #[test]
    fn provider_identity_keeps_interop_semantics_but_ignores_interop_spans() {
        let range = |start, end| TextRange::new(TextSize::new(start), TextSize::new(end));
        let with_target = |segments: &[&str], span| {
            let mut provider = function(Vec::new());
            provider.rust_interop.push(RustInteropDeclaration {
                kind: RustInteropDecoratorKind::Function,
                target: Some(RustTargetPath {
                    segments: segments
                        .iter()
                        .map(|segment| (*segment).to_string())
                        .collect(),
                    span,
                }),
                arguments: Vec::new(),
                span,
                effect: RustInteropEffect::Sync,
                abi_requirements: RustInteropAbiRequirements::default(),
                consumes_receiver: false,
            });
            module(provider)
        };

        let base = canonical_const_functions(&with_target(&["bridge", "adapt"], range(1, 8)));
        let moved = canonical_const_functions(&with_target(&["bridge", "adapt"], range(41, 48)));
        let changed = canonical_const_functions(&with_target(&["bridge", "other"], range(41, 48)));

        assert_eq!(base, moved);
        assert_ne!(base, changed);
    }
}

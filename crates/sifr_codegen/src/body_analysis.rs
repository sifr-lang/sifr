#[path = "body_analysis/call_conventions.rs"]
mod call_conventions;
#[path = "body_analysis/discarded_bindings.rs"]
mod discarded_bindings;
#[path = "body_analysis/last_use.rs"]
mod last_use;
#[path = "body_analysis/read_regions.rs"]
mod read_regions;

use crate::hir_analysis::traversal;
use crate::{HirExpr, HirFunction, HirStmt, ModuleFuncSignatures, Type};
use call_conventions::{CallParamConventions, collect_call_param_conventions};
use sifr_ir::{HirAsyncWithKind, HirIteratorOp, HirPattern, HirTupleTargetBinding};
use sifr_type_system::{ParamConvention, ReceiverConvention};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Default)]
pub(crate) struct BodySummary {
    pub(crate) mutated: HashSet<String>,
    pub(crate) proven_reads: Vec<HirExpr>,
    atomic_proven_reads: Vec<HirExpr>,
    pub(crate) checked_read_keys: HashSet<String>,
    referenced: HashMap<String, usize>,
}

impl BodySummary {
    pub(crate) fn uses_binding(&self, name: &str) -> bool {
        self.referenced.contains_key(name) || self.mutated.contains(name)
    }

    fn merge(&mut self, other: &Self) {
        self.mutated.extend(other.mutated.iter().cloned());
        self.proven_reads.extend(other.proven_reads.iter().cloned());
        self.atomic_proven_reads
            .extend(other.atomic_proven_reads.iter().cloned());
        self.checked_read_keys
            .extend(other.checked_read_keys.iter().cloned());
        for (name, count) in &other.referenced {
            *self.referenced.entry(name.clone()).or_default() += count;
        }
    }
}

#[derive(Clone, Default)]
pub(crate) struct BodyAnalysis {
    blocks: HashMap<(usize, usize), BodySummary>,
    statements: HashMap<usize, BodySummary>,
    nested_captures: HashMap<usize, HashSet<String>>,
    last_use_statements: HashSet<usize>,
    unused_projection_statements: HashMap<usize, discarded_bindings::UnusedProjection>,
}

impl BodyAnalysis {
    pub(crate) fn build(
        func: &HirFunction,
        func_signatures: &ModuleFuncSignatures,
    ) -> (Self, HashSet<usize>) {
        Self::build_with_borrowed(func, func_signatures, &HashSet::new())
    }

    pub(crate) fn build_with_borrowed(
        func: &HirFunction,
        func_signatures: &ModuleFuncSignatures,
        extra_borrowed: &HashSet<String>,
    ) -> (Self, HashSet<usize>) {
        let mut analysis = Self::default();
        let call_param_conventions = collect_call_param_conventions(&func.body, func_signatures);
        analysis.analyze_block(&func.body, &call_param_conventions);
        let mut defined = func
            .params
            .iter()
            .map(|param| param.name.clone())
            .collect::<HashSet<_>>();
        let mut borrowed = func
            .params
            .iter()
            .filter(|param| param.convention.is_borrowed())
            .map(|param| param.name.clone())
            .collect::<HashSet<_>>();
        borrowed.extend(extra_borrowed.iter().cloned());
        let mut moves = HashSet::new();
        analysis.mark_last_uses(
            &func.body,
            &mut defined,
            &HashSet::new(),
            &borrowed,
            &mut moves,
        );
        (analysis, moves)
    }

    pub(crate) fn summary(&self, stmts: &[HirStmt]) -> Option<&BodySummary> {
        self.blocks.get(&block_key(stmts))
    }

    pub(crate) fn owned_value_statement_has_last_use(&self, stmt: &HirStmt) -> bool {
        if !self.last_use_statements.contains(&stmt_key(stmt)) {
            return false;
        }
        let value = match stmt {
            HirStmt::Let { value, .. }
            | HirStmt::Assign { value, .. }
            | HirStmt::Return { value: Some(value) }
            | HirStmt::Yield { value } => value,
            _ => return false,
        };
        matches!(
            value,
            HirExpr::Name { .. }
                | HirExpr::ListLiteral { .. }
                | HirExpr::TupleLiteral { .. }
                | HirExpr::DictLiteral { .. }
                | HirExpr::SetLiteral { .. }
        )
    }

    pub(crate) fn mutated_in(&self, stmts: &[HirStmt]) -> HashSet<String> {
        if let Some(summary) = self.summary(stmts) {
            return summary.mutated.clone();
        }
        let mut mutated = HashSet::new();
        for stmt in stmts {
            if let Some(summary) = self.statements.get(&stmt_key(stmt)) {
                mutated.extend(summary.mutated.iter().cloned());
            }
        }
        mutated
    }

    pub(crate) fn proven_reads_in(&self, stmts: &[HirStmt]) -> Vec<HirExpr> {
        if let Some(summary) = self.summary(stmts) {
            return summary.proven_reads.clone();
        }
        let mut reads = Vec::new();
        for stmt in stmts {
            if let Some(summary) = self.statements.get(&stmt_key(stmt)) {
                reads.extend(summary.proven_reads.iter().cloned());
            }
        }
        reads.sort_by_key(index_depth);
        reads
    }

    pub(crate) fn references_outside_checked_read(
        &self,
        stmt: &HirStmt,
        owner: &str,
        key: &str,
    ) -> bool {
        let Some(summary) = self.statements.get(&stmt_key(stmt)) else {
            return true;
        };
        let mut witnessed_references = 0;
        for read in &summary.proven_reads {
            let HirExpr::Index { object, index, .. } = read else {
                continue;
            };
            if crate::checked_place::checked_place_read_key(object, index).as_deref() != Some(key) {
                continue;
            }
            traversal::walk_expr(read, &mut |expr| {
                if matches!(expr, HirExpr::Name { name, .. } if name == owner) {
                    witnessed_references += 1;
                }
            });
        }
        summary.mutated.contains(owner)
            || summary.referenced.get(owner).copied().unwrap_or_default() > witnessed_references
    }

    pub(crate) fn atomic_proven_reads_in(&self, stmt: &HirStmt) -> Vec<HirExpr> {
        self.statements
            .get(&stmt_key(stmt))
            .map(|summary| summary.atomic_proven_reads.clone())
            .unwrap_or_default()
    }

    pub(crate) fn checked_read_is_used(&self, stmts: &[HirStmt], key: &str) -> bool {
        if let Some(summary) = self.summary(stmts) {
            return summary.checked_read_keys.contains(key);
        }
        stmts.iter().any(|stmt| {
            self.statements
                .get(&stmt_key(stmt))
                .is_some_and(|summary| summary.checked_read_keys.contains(key))
        })
    }

    fn analyze_block(
        &mut self,
        stmts: &[HirStmt],
        call_param_conventions: &CallParamConventions,
    ) -> BodySummary {
        let mut block = BodySummary::default();
        for stmt in stmts {
            let mut summary = direct_stmt_summary(stmt, call_param_conventions);
            self.merge_child_blocks(stmt, &mut summary, call_param_conventions);
            let nested_captures = self.nested_captures_introduced_by(stmt);
            if !nested_captures.is_empty() {
                self.nested_captures.insert(stmt_key(stmt), nested_captures);
            }
            summary.proven_reads.sort_by_key(index_depth);
            summary.atomic_proven_reads.sort_by_key(index_depth);
            block.merge(&summary);
            self.statements.insert(stmt_key(stmt), summary);
        }
        block = self.remove_unused_projection_summaries(stmts, block);
        block.proven_reads.sort_by_key(index_depth);
        block.atomic_proven_reads.sort_by_key(index_depth);
        self.blocks.insert(block_key(stmts), block.clone());
        block
    }

    fn merge_child_blocks(
        &mut self,
        stmt: &HirStmt,
        summary: &mut BodySummary,
        call_param_conventions: &CallParamConventions,
    ) {
        match stmt {
            HirStmt::If {
                then_body,
                elif_clauses,
                else_body,
                ..
            } => {
                summary.merge(&self.analyze_block(then_body, call_param_conventions));
                for (_, body) in elif_clauses {
                    summary.merge(&self.analyze_block(body, call_param_conventions));
                }
                if let Some(body) = else_body {
                    summary.merge(&self.analyze_block(body, call_param_conventions));
                }
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
                summary.merge(&self.analyze_block(body, call_param_conventions));
                if let Some(body) = else_body {
                    summary.merge(&self.analyze_block(body, call_param_conventions));
                }
            }
            HirStmt::TryExcept { body, handlers, .. } => {
                summary.merge(&self.analyze_block(body, call_param_conventions));
                for handler in handlers {
                    summary.merge(&self.analyze_block(&handler.body, call_param_conventions));
                }
            }
            HirStmt::TryFinally { body, finalbody } => {
                summary.merge(&self.analyze_block(body, call_param_conventions));
                summary.merge(&self.analyze_block(finalbody, call_param_conventions));
            }
            HirStmt::With { body, .. } | HirStmt::AsyncWith { body, .. } => {
                summary.merge(&self.analyze_block(body, call_param_conventions));
            }
            HirStmt::NestedFunction { func, .. } => {
                let nested = self.analyze_block(&func.body, call_param_conventions);
                let params = func
                    .params
                    .iter()
                    .map(|param| param.name.as_str())
                    .collect::<HashSet<_>>();
                let locals = crate::helpers::collect_locally_defined_vars(&func.body);
                let captured_mutations = nested
                    .mutated
                    .into_iter()
                    .filter(|name| !params.contains(name.as_str()) && !locals.contains(name))
                    .collect::<HashSet<_>>();
                let captured_references = nested
                    .referenced
                    .into_iter()
                    .filter(|(name, _)| !params.contains(name.as_str()) && !locals.contains(name))
                    .collect::<HashMap<_, _>>();
                let mut captures = captured_mutations.clone();
                captures.extend(captured_references.keys().cloned());
                self.nested_captures.insert(stmt_key(stmt), captures);
                summary.mutated.extend(captured_mutations);
                for (name, count) in captured_references {
                    *summary.referenced.entry(name).or_default() += count;
                }
            }
            HirStmt::Match { arms, .. } => {
                for arm in arms {
                    summary.merge(&self.analyze_block(&arm.body, call_param_conventions));
                }
            }
            _ => {}
        }
    }

    fn nested_captures_introduced_by(&self, stmt: &HirStmt) -> HashSet<String> {
        let mut captures = self
            .nested_captures
            .get(&stmt_key(stmt))
            .cloned()
            .unwrap_or_default();
        // A stored lazy iterator can borrow its input until the iterator is
        // consumed or dropped, even after its final explicit next() call.
        let stored_value = match stmt {
            HirStmt::Let { value, .. }
            | HirStmt::Assign { value, .. }
            | HirStmt::FieldAssign { value, .. }
            | HirStmt::NestedFieldAssign { value, .. }
            | HirStmt::SubscriptAssign { value, .. } => Some(value),
            _ => None,
        };
        if let Some(value) = stored_value
            && matches!(
                value.ty().resolve_alias(),
                sifr_type_system::Type::Iterator(_)
                    | sifr_type_system::Type::AsyncIterator(_, _)
                    | sifr_type_system::Type::AsyncGenerator(_, _)
            )
        {
            traversal::walk_expr(value, &mut |expr| {
                if let HirExpr::Name { name, .. } = expr {
                    captures.insert(name.clone());
                }
            });
        }
        let mut extend_from = |body: &[HirStmt]| {
            for child in body {
                if let Some(child_captures) = self.nested_captures.get(&stmt_key(child)) {
                    captures.extend(child_captures.iter().cloned());
                }
            }
        };
        match stmt {
            HirStmt::If {
                then_body,
                elif_clauses,
                else_body,
                ..
            } => {
                extend_from(then_body);
                for (_, body) in elif_clauses {
                    extend_from(body);
                }
                if let Some(body) = else_body {
                    extend_from(body);
                }
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
                extend_from(body);
                if let Some(body) = else_body {
                    extend_from(body);
                }
            }
            HirStmt::TryExcept { body, handlers, .. } => {
                extend_from(body);
                for handler in handlers {
                    extend_from(&handler.body);
                }
            }
            HirStmt::TryFinally { body, finalbody } => {
                extend_from(body);
                extend_from(finalbody);
            }
            HirStmt::With { body, .. } | HirStmt::AsyncWith { body, .. } => extend_from(body),
            HirStmt::Match { arms, .. } => {
                for arm in arms {
                    extend_from(&arm.body);
                }
            }
            _ => {}
        }
        captures
    }
}

fn direct_stmt_summary(
    stmt: &HirStmt,
    call_param_conventions: &CallParamConventions,
) -> BodySummary {
    let mut summary = BodySummary::default();
    direct_stmt_mutations(stmt, &mut summary.mutated);
    // Augmented assignment reads its existing value through the target name,
    // which is not represented by a Name expression in the RHS traversal.
    if let HirStmt::AugAssign { name, .. } = stmt {
        *summary.referenced.entry(name.clone()).or_default() += 1;
    }
    let excluded_optional_read = match stmt {
        HirStmt::Let { ty, value, .. } if ty.optional_member_type().is_some() => {
            Some(std::ptr::from_ref(value))
        }
        _ => None,
    };
    let conditional_reads = read_regions::conditional_reads(stmt);
    walk_direct_stmt_exprs(stmt, &mut |expr| {
        traversal::walk_expr(expr, &mut |candidate| {
            if let HirExpr::Name { name, .. } = candidate {
                *summary.referenced.entry(name.clone()).or_default() += 1;
            }
            collect_expr_mutation(candidate, call_param_conventions, &mut summary.mutated);
            let HirExpr::Index { object, index, ty } = candidate else {
                return;
            };
            if let Some(key) = crate::checked_place::checked_place_read_key(object, index) {
                summary.checked_read_keys.insert(key);
            }
            if excluded_optional_read == Some(std::ptr::from_ref(candidate))
                || crate::helpers::is_option_type(ty)
                || !matches!(
                    crate::resolve_alias_type_for_plain_call(object.ty()),
                    Type::Dict(_, _) | Type::List(_) | Type::Bytes | Type::Str
                )
            {
                return;
            }
            summary.proven_reads.push(candidate.clone());
            if !conditional_reads.contains(&expr_key(candidate)) {
                summary.atomic_proven_reads.push(candidate.clone());
            }
        });
    });
    summary
}

fn collect_expr_mutation(
    expr: &HirExpr,
    call_param_conventions: &CallParamConventions,
    mutated: &mut HashSet<String>,
) {
    match expr {
        HirExpr::MethodCall {
            object,
            method,
            receiver_convention,
            args,
            mutable_arg_places,
            ..
        } => {
            if *receiver_convention == Some(ReceiverConvention::MutableBorrow)
                && let Some(name) = expression_root_name(object)
            {
                mutated.insert(name.to_string());
            }
            collect_mutable_args(args, mutable_arg_places, mutated);
            if let Type::Class { name, .. } = object.ty().resolve_alias()
                && let Some(conventions) = call_param_conventions.get(&format!("{name}::{method}"))
            {
                collect_signature_mutable_args(args, conventions, false, mutated);
            }
        }
        HirExpr::Call {
            func,
            args,
            mutable_arg_places,
            ..
        }
        | HirExpr::GenericCall {
            func,
            args,
            mutable_arg_places,
            ..
        } => {
            collect_mutable_args(args, mutable_arg_places, mutated);
            let canonical = crate::stmt_support_emitter::canonical_plain_call_name_for_ir(func);
            let conventions = call_param_conventions
                .get(func)
                .or_else(|| call_param_conventions.get(canonical));
            if let Some(conventions) = conventions {
                collect_signature_mutable_args(args, conventions, true, mutated);
            }
            // Match the same builtin identity and arity as async-call emission.
            if func == "anext"
                && args.len() == 1
                && let Some(name) = args.first().and_then(expression_root_name)
            {
                mutated.insert(name.to_string());
            }
            if matches!(
                canonical.rsplit('.').next(),
                Some("heappush" | "heappop" | "heapify" | "heapreplace" | "heappushpop")
            ) && let Some(name) = args.first().and_then(expression_root_name)
            {
                mutated.insert(name.to_string());
            }
        }
        HirExpr::IteratorCall {
            op: HirIteratorOp::Next,
            args,
            ..
        } => {
            if let Some(name) = args.first().and_then(expression_root_name) {
                mutated.insert(name.to_string());
            }
        }
        _ => {}
    }
}

fn collect_signature_mutable_args(
    args: &[HirExpr],
    conventions: &[ParamConvention],
    include_owned: bool,
    mutated: &mut HashSet<String>,
) {
    for (arg, convention) in args.iter().zip(conventions) {
        let invalidates_witness = if include_owned {
            convention.is_mutable()
        } else {
            convention.is_mut_borrow()
        };
        if invalidates_witness && let Some(name) = expression_root_name(arg) {
            mutated.insert(name.to_string());
        }
    }
}

fn collect_mutable_args(
    args: &[HirExpr],
    places: &[Option<sifr_ir::MutableArgumentTarget>],
    mutated: &mut HashSet<String>,
) {
    for (arg, place) in args.iter().zip(places) {
        if place.is_some()
            && let Some(name) = expression_root_name(arg)
        {
            mutated.insert(name.to_string());
        }
    }
}

fn direct_stmt_mutations(stmt: &HirStmt, mutated: &mut HashSet<String>) {
    match stmt {
        HirStmt::Assign { name, .. } | HirStmt::AugAssign { name, .. } => {
            mutated.insert(name.clone());
        }
        HirStmt::TupleUnpack { targets, .. } => {
            collect_rebound_tuple_targets(targets.iter(), mutated);
        }
        HirStmt::StarUnpack {
            before,
            star,
            after,
            ..
        } => {
            collect_rebound_tuple_targets(
                before.iter().chain(std::iter::once(star)).chain(after),
                mutated,
            );
        }
        HirStmt::SubscriptAssign { object, .. }
        | HirStmt::NestedSubscriptAssign { object, .. }
        | HirStmt::AttributeNestedSubscriptAssign { object, .. }
        | HirStmt::SubscriptAugAssign { object, .. }
        | HirStmt::AttributeAugAssign { object, .. }
        | HirStmt::FieldAssign { object, .. }
        | HirStmt::NestedFieldAssign { object, .. }
        | HirStmt::AttributeSubscriptAssign { object, .. } => {
            mutated.insert(object.clone());
        }
        HirStmt::AsyncFor {
            iter: HirExpr::Name { name, .. },
            ..
        }
        | HirStmt::Delete {
            object: HirExpr::Name { name, .. },
            ..
        } => {
            mutated.insert(name.clone());
        }
        HirStmt::AsyncWith {
            kind:
                HirAsyncWithKind::UserDefined {
                    context: HirExpr::Name { name, .. },
                    ..
                }
                | HirAsyncWithKind::Python {
                    context: HirExpr::Name { name, .. },
                    ..
                },
            ..
        } => {
            mutated.insert(name.clone());
        }
        _ => {}
    }
}

fn collect_rebound_tuple_targets<'a>(
    targets: impl Iterator<Item = &'a sifr_ir::HirTupleTarget>,
    mutated: &mut HashSet<String>,
) {
    for target in targets {
        if target.rebind_existing
            && let HirTupleTargetBinding::Name(name) = &target.binding
        {
            mutated.insert(name.clone());
        }
    }
}

fn walk_direct_stmt_exprs(stmt: &HirStmt, visit: &mut impl FnMut(&HirExpr)) {
    match stmt {
        HirStmt::Let { value, .. }
        | HirStmt::Assign { value, .. }
        | HirStmt::AugAssign { value, .. }
        | HirStmt::Expr { expr: value }
        | HirStmt::Raise { value }
        | HirStmt::Yield { value }
        | HirStmt::FieldAssign { value, .. }
        | HirStmt::NestedFieldAssign { value, .. } => visit(value),
        HirStmt::Return { value } => value.iter().for_each(&mut *visit),
        HirStmt::If {
            condition,
            elif_clauses,
            ..
        } => {
            visit(condition);
            for (condition, _) in elif_clauses {
                visit(condition);
            }
        }
        HirStmt::While { condition, .. } => visit(condition),
        HirStmt::For { iter, .. } | HirStmt::AsyncFor { iter, .. } => visit(iter),
        HirStmt::TupleUnpack { value, .. } | HirStmt::StarUnpack { value, .. } => visit(value),
        HirStmt::Assert { test, msg } => {
            visit(test);
            msg.iter().for_each(&mut *visit);
        }
        HirStmt::SubscriptAssign { index, value, .. }
        | HirStmt::SubscriptAugAssign { index, value, .. }
        | HirStmt::AttributeSubscriptAssign { index, value, .. } => {
            visit(index);
            visit(value);
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
            visit(outer_index);
            visit(inner_index);
            visit(value);
        }
        HirStmt::AttributeAugAssign { value, .. } => visit(value),
        HirStmt::Delete { object, index, .. } => {
            visit(object);
            visit(index);
        }
        HirStmt::With { items, .. } => items.iter().for_each(|item| visit(&item.context)),
        HirStmt::AsyncWith { kind, .. } => match kind {
            HirAsyncWithKind::TaskGroup { context } => context.iter().for_each(&mut *visit),
            HirAsyncWithKind::TaskTimeout { duration } => visit(duration),
            HirAsyncWithKind::UserDefined { context, .. }
            | HirAsyncWithKind::Python { context, .. } => visit(context),
            HirAsyncWithKind::TaskScope => {}
        },
        HirStmt::Match { subject, arms, .. } => {
            visit(subject);
            for arm in arms {
                arm.guard.iter().for_each(&mut *visit);
                walk_pattern_exprs(&arm.pattern, visit);
            }
        }
        HirStmt::TryExcept { .. }
        | HirStmt::TryFinally { .. }
        | HirStmt::NestedFunction { .. }
        | HirStmt::Break
        | HirStmt::Continue
        | HirStmt::Pass => {}
    }
}

fn walk_pattern_exprs(pattern: &HirPattern, visit: &mut impl FnMut(&HirExpr)) {
    match pattern {
        HirPattern::Literal { value } => visit(value),
        HirPattern::Or { patterns } => patterns
            .iter()
            .for_each(|pattern| walk_pattern_exprs(pattern, visit)),
        HirPattern::Class { fields, .. } => fields
            .iter()
            .for_each(|(_, pattern)| walk_pattern_exprs(pattern, visit)),
        HirPattern::Tuple { elements } => elements
            .iter()
            .for_each(|pattern| walk_pattern_exprs(pattern, visit)),
        HirPattern::Wildcard
        | HirPattern::Capture { .. }
        | HirPattern::None
        | HirPattern::Value { .. } => {}
    }
}

fn register_stmt_definitions(stmt: &HirStmt, defined: &mut HashSet<String>) {
    match stmt {
        HirStmt::Let { name, .. }
        | HirStmt::Assign { name, .. }
        | HirStmt::AugAssign { name, .. } => {
            defined.insert(name.clone());
        }
        HirStmt::TupleUnpack { targets, .. } => {
            for target in targets {
                if let HirTupleTargetBinding::Name(name) = &target.binding {
                    defined.insert(name.clone());
                }
            }
        }
        HirStmt::StarUnpack {
            before,
            star,
            after,
            ..
        } => {
            for target in before.iter().chain(std::iter::once(star)).chain(after) {
                if let HirTupleTargetBinding::Name(name) = &target.binding {
                    defined.insert(name.clone());
                }
            }
        }
        HirStmt::NestedFunction { func, .. } => {
            defined.insert(func.name.clone());
        }
        _ => {}
    }
}

fn register_pattern_definitions(pattern: &HirPattern, defined: &mut HashSet<String>) {
    match pattern {
        HirPattern::Capture { name, .. } => {
            defined.insert(name.clone());
        }
        HirPattern::Or { patterns } => patterns
            .iter()
            .for_each(|pattern| register_pattern_definitions(pattern, defined)),
        HirPattern::Class { fields, .. } => fields
            .iter()
            .for_each(|(_, pattern)| register_pattern_definitions(pattern, defined)),
        HirPattern::Tuple { elements } => elements
            .iter()
            .for_each(|pattern| register_pattern_definitions(pattern, defined)),
        _ => {}
    }
}

fn subtract_counts(remaining: &mut HashMap<String, usize>, current: &HashMap<String, usize>) {
    for (name, count) in current {
        let remove = if let Some(remaining_count) = remaining.get_mut(name) {
            *remaining_count = remaining_count.saturating_sub(*count);
            *remaining_count == 0
        } else {
            false
        };
        if remove {
            remaining.remove(name);
        }
    }
}

fn expression_root_name(expr: &HirExpr) -> Option<&str> {
    match expr {
        HirExpr::Name { name, .. } => Some(name),
        HirExpr::FieldAccess { object, .. } | HirExpr::Index { object, .. } => {
            expression_root_name(object)
        }
        _ => None,
    }
}

fn index_depth(expr: &HirExpr) -> usize {
    let mut depth = 0;
    let mut current = expr;
    while let HirExpr::Index { object, .. } = current {
        depth += 1;
        current = object;
    }
    depth
}

fn block_key(stmts: &[HirStmt]) -> (usize, usize) {
    // HIR nodes are immutable for the complete build-and-emission interval. These identity keys
    // intentionally fail closed (no cached optimization) if that ownership invariant changes.
    (stmts.as_ptr() as usize, stmts.len())
}
fn stmt_key(stmt: &HirStmt) -> usize {
    std::ptr::from_ref(stmt) as usize
}
pub(crate) fn expr_key(expr: &HirExpr) -> usize {
    std::ptr::from_ref(expr) as usize
}

#[cfg(test)]
#[path = "body_analysis/async_advance_tests.rs"]
mod async_advance_tests;

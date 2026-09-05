pub(super) fn remove_last_use_clones(statements: &mut [syn::Stmt]) {
    let mut locally_owned = HashSet::new();
    let mut locally_borrowed = HashSet::new();
    for statement in statements.iter() {
        let syn::Stmt::Local(local) = statement else {
            continue;
        };
        let produces_borrow = local.init.as_ref().is_some_and(|init| {
            expression_produces_borrowed_binding(&init.expr)
                || expression_root_name(&init.expr)
                    .is_some_and(|name| locally_borrowed.contains(&name))
        });
        if produces_borrow {
            collect_owned_pattern_names(&local.pat, &mut locally_borrowed);
        } else {
            let mut bindings = HashSet::new();
            collect_owned_pattern_names(&local.pat, &mut bindings);
            let self_shadowing = local.init.as_ref().is_some_and(|init| {
                bindings
                    .iter()
                    .any(|name| expression_mentions_name(&init.expr, name))
            });
            let cloned_from_borrow = local.init.as_ref().is_some_and(|init| {
                init.diverge.is_some()
                    && matches!(init.expr.as_ref(), syn::Expr::MethodCall(clone)
                        if clone.method == "clone"
                            && clone.args.is_empty()
                            && matches!(clone.receiver.as_ref(), syn::Expr::Unary(dereference)
                                if matches!(dereference.op, syn::UnOp::Deref(_))))
            });
            if !self_shadowing || cloned_from_borrow {
                locally_owned.extend(bindings);
            }
        }
    }
    remove_last_use_clones_with_owned(statements, &locally_owned, true);
    let mut return_cleanup = ReturnExpressionCloneRemover {
        owned: &locally_owned,
    };
    for statement in statements {
        return_cleanup.visit_stmt_mut(statement);
    }
}

pub(super) fn remove_needless_collected_length_bindings(statements: &mut Vec<syn::Stmt>) {
    let mut index = 0;
    while index < statements.len() {
        let candidate = match &statements[index] {
            syn::Stmt::Local(local) => {
                let name = simple_pattern_name(&local.pat);
                let producer = local.init.as_ref().and_then(|init| {
                    let syn::Expr::MethodCall(collect) = init.expr.as_ref() else {
                        return None;
                    };
                    (collect.method == "collect" && collect.args.is_empty())
                        .then(|| collect.receiver.as_ref().clone())
                });
                name.zip(producer)
            }
            _ => None,
        };
        let Some((name, producer)) = candidate else {
            index += 1;
            continue;
        };
        let mut uses = IdentifierUseCounter::default();
        for statement in &statements[index + 1..] {
            uses.visit_stmt(statement);
        }
        if uses.counts.get(&name).copied().unwrap_or(0) != 1 {
            index += 1;
            continue;
        }
        let mut rewriter = CollectedLengthUseRewriter {
            name: &name,
            producer: &producer,
            replaced: false,
        };
        for statement in &mut statements[index + 1..] {
            rewriter.visit_stmt_mut(statement);
        }
        if rewriter.replaced {
            statements.remove(index);
        } else {
            index += 1;
        }
    }
}

struct CollectedLengthUseRewriter<'value> {
    name: &'value str,
    producer: &'value syn::Expr,
    replaced: bool,
}

impl VisitMut for CollectedLengthUseRewriter<'_> {
    fn visit_expr_mut(&mut self, expression: &mut syn::Expr) {
        if let syn::Expr::MethodCall(call) = expression
            && call.method == "len"
            && call.args.is_empty()
            && matches!(call.receiver.as_ref(), syn::Expr::Path(path) if path.path.is_ident(self.name))
        {
            let producer = self.producer;
            *expression = syn::parse_quote!((#producer).count());
            self.replaced = true;
            return;
        }
        visit_mut::visit_expr_mut(self, expression);
    }

    fn visit_item_mut(&mut self, _item: &mut syn::Item) {}

    fn visit_macro_mut(&mut self, rust_macro: &mut syn::Macro) {
        let Ok(mut arguments) = rust_macro.parse_body_with(
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
        ) else {
            return;
        };
        for argument in &mut arguments {
            self.visit_expr_mut(argument);
        }
        rust_macro.tokens = arguments.to_token_stream();
    }
}

fn expression_mentions_name(expression: &syn::Expr, name: &str) -> bool {
    let mut uses = IdentifierUseCounter::default();
    uses.visit_expr(expression);
    uses.counts.contains_key(name)
}

pub(super) fn remove_last_use_parameter_clones(signature: &syn::Signature, body: &mut syn::Block) {
    let owned_parameters = signature
        .inputs
        .iter()
        .filter_map(|argument| {
            let syn::FnArg::Typed(parameter) = argument else {
                return None;
            };
            if matches!(parameter.ty.as_ref(), syn::Type::Reference(_)) {
                return None;
            }
            simple_pattern_name(&parameter.pat)
        })
        .collect::<HashSet<_>>();
    remove_shadowing_condition_clones(&mut body.stmts, &owned_parameters);
    remove_last_use_clones_with_owned(&mut body.stmts, &owned_parameters, false);
    remove_clones_from_return_expressions(body, &owned_parameters);
    if let Some(syn::Stmt::Expr(tail, None)) = body.stmts.last_mut() {
        remove_terminal_expression_clones(tail, &owned_parameters);
    }
}

fn remove_terminal_expression_clones(expression: &mut syn::Expr, owned: &HashSet<String>) {
    match expression {
        syn::Expr::Block(block) => {
            if let Some(syn::Stmt::Expr(tail, None)) = block.block.stmts.last_mut() {
                remove_terminal_expression_clones(tail, owned);
            }
        }
        syn::Expr::If(branch) => {
            if let Some(syn::Stmt::Expr(tail, None)) = branch.then_branch.stmts.last_mut() {
                remove_terminal_expression_clones(tail, owned);
            }
            if let Some((_, alternative)) = &mut branch.else_branch {
                remove_terminal_expression_clones(alternative, owned);
            }
        }
        syn::Expr::Match(match_) => {
            for arm in &mut match_.arms {
                remove_terminal_expression_clones(&mut arm.body, owned);
            }
        }
        syn::Expr::MethodCall(clone)
            if clone.method == "clone"
                && clone.args.is_empty()
                && expression_root_name(&clone.receiver)
                    .is_some_and(|name| owned.contains(&name)) =>
        {
            *expression = clone.receiver.as_ref().clone();
        }
        syn::Expr::Return(returned) => {
            if let Some(value) = &mut returned.expr {
                remove_terminal_expression_clones(value, owned);
            }
        }
        _ => {}
    }
}

pub(super) fn remove_last_use_closure_input_clones(closure: &mut syn::ExprClosure) {
    let owned = closure
        .inputs
        .iter()
        .filter_map(|pattern| {
            if let Some(name) = simple_pattern_name(pattern)
                && (name.starts_with("sifr_generated_try_err")
                    || name.starts_with("sifr_generated_checked_value"))
            {
                return Some(name);
            }
            let syn::Pat::Type(typed) = pattern else {
                return None;
            };
            (!matches!(typed.ty.as_ref(), syn::Type::Reference(_)))
                .then(|| simple_pattern_name(&typed.pat))
                .flatten()
        })
        .collect::<HashSet<_>>();
    if owned.is_empty() {
        return;
    }
    if let syn::Expr::Block(body) = closure.body.as_mut() {
        remove_last_use_clones_with_owned(&mut body.block.stmts, &owned, false);
        return;
    }
    let mut counts = IdentifierUseCounter::default();
    counts.visit_expr(&closure.body);
    LastUseCloneRemover {
        movable: &owned,
        remaining: counts.counts,
    }
    .visit_expr_mut(&mut closure.body);
}

fn remove_clones_from_return_expressions(body: &mut syn::Block, owned: &HashSet<String>) {
    ReturnExpressionCloneRemover { owned }.visit_block_mut(body);
}

pub(super) fn replace_unused_underscore_bindings(statements: &mut [syn::Stmt]) {
    let mut uses = IdentifierUseCounter::default();
    for statement in statements.iter() {
        uses.visit_stmt(statement);
    }
    for statement in statements {
        let syn::Stmt::Local(local) = statement else {
            continue;
        };
        let Some(name) = simple_pattern_name(&local.pat) else {
            continue;
        };
        if name.starts_with('_')
            && uses.counts.get(&name).copied().unwrap_or(0) == 0
            && local.init.as_ref().is_some_and(|init| {
                crate::discardability::syntax_expression_is_discardable(&init.expr)
            })
        {
            local.pat = syn::parse_quote!(_);
        }
    }
}

fn remove_last_use_clones_with_owned(
    statements: &mut [syn::Stmt],
    owned_names: &HashSet<String>,
    rewrite_shadow_conditions: bool,
) {
    let mut loop_control = LoopControlCollector { found: false };
    for statement in statements.iter() {
        loop_control.visit_stmt(statement);
    }
    if loop_control.found {
        if let Some(last_loop_control) = statements.iter().rposition(|statement| {
            let mut collector = LoopControlCollector { found: false };
            collector.visit_stmt(statement);
            collector.found
        }) {
            remove_last_use_clones_with_owned(
                &mut statements[last_loop_control.saturating_add(1)..],
                owned_names,
                rewrite_shadow_conditions,
            );
        }
        return;
    }
    let mut captures = ClosureCaptureCollector::default();
    for statement in statements.iter() {
        captures.visit_stmt(statement);
    }
    let mut used_later = HashSet::new();
    for statement in statements.iter_mut().rev() {
        let unused_local = if let syn::Stmt::Local(local) = statement {
            let names =
                crate::generated_rust_canonicalizer::syntax_cleanup::identifier_names_in_pattern(
                    &local.pat,
                );
            names.iter().all(|name| !used_later.contains(name))
        } else {
            false
        };
        if let syn::Stmt::Local(local) = statement {
            let mut bindings = HashSet::new();
            collect_owned_pattern_names(&local.pat, &mut bindings);
            used_later.retain(|name| !bindings.contains(name));
        }
        if rewrite_shadow_conditions {
            remove_shadowing_condition_clone(statement, &used_later, owned_names);
        }
        let mut counts = IdentifierUseCounter::default();
        counts.visit_stmt(statement);
        let movable = counts
            .counts
            .iter()
            .filter_map(|(name, count)| {
                (*count >= 1
                    && owned_names.contains(name)
                    && !captures.names.contains(name)
                    && !used_later.contains(name))
                .then_some(name.clone())
            })
            .collect::<HashSet<_>>();
        if let syn::Stmt::Expr(syn::Expr::If(branch), _) = statement {
            rewrite_terminal_branch(branch, &movable, counts.counts.clone());
        } else if !unused_local {
            LastUseCloneRemover {
                movable: &movable,
                remaining: counts.counts.clone(),
            }
            .visit_stmt_mut(statement);
        }
        used_later.extend(counts.counts.into_keys());
    }
}

fn rewrite_terminal_branch(
    branch: &mut syn::ExprIf,
    movable: &HashSet<String>,
    counts: HashMap<String, usize>,
) {
    // Only a complete statement has no expression siblings which could still
    // borrow its inputs. Mutually exclusive branch bodies each get one walk.
    LastUseCloneRemover {
        movable,
        remaining: counts,
    }
    .visit_expr_mut(&mut branch.cond);
    let mut then_owned = movable.clone();
    let mut condition = IdentifierUseCounter::default();
    condition.visit_condition(&branch.cond);
    for name in &condition.shadowed {
        then_owned.remove(name);
    }
    remove_last_use_clones_with_owned(&mut branch.then_branch.stmts, &then_owned, false);
    if let Some((_, alternative)) = &mut branch.else_branch {
        match alternative.as_mut() {
            syn::Expr::Block(block) => {
                remove_last_use_clones_with_owned(&mut block.block.stmts, movable, false);
            }
            syn::Expr::If(next) => {
                let mut counts = IdentifierUseCounter::default();
                counts.visit_expr_if(next);
                rewrite_terminal_branch(next, movable, counts.counts);
            }
            _ => {}
        }
    }
}

fn remove_shadowing_condition_clone(
    statement: &mut syn::Stmt,
    used_later: &HashSet<String>,
    owned_names: &HashSet<String>,
) {
    let syn::Stmt::Expr(syn::Expr::If(branch), _) = statement else {
        return;
    };
    remove_shadowing_branch_condition_clones(branch, used_later, owned_names);
}

fn remove_shadowing_branch_condition_clones(
    branch: &mut syn::ExprIf,
    used_later: &HashSet<String>,
    owned_names: &HashSet<String>,
) {
    remove_shadowing_clone_from_condition(&mut branch.cond, used_later, owned_names);
    let mut condition_owned = HashSet::new();
    collect_moved_condition_bindings(&branch.cond, &mut condition_owned);
    remove_last_use_clones_with_owned(&mut branch.then_branch.stmts, &condition_owned, true);
    if let Some((_, alternative)) = &mut branch.else_branch
        && let syn::Expr::If(next) = alternative.as_mut()
    {
        remove_shadowing_branch_condition_clones(next, used_later, owned_names);
    }
}

fn collect_moved_condition_bindings(condition: &syn::Expr, owned: &mut HashSet<String>) {
    match condition {
        syn::Expr::Binary(binary) if matches!(binary.op, syn::BinOp::And(_)) => {
            collect_moved_condition_bindings(&binary.left, owned);
            collect_moved_condition_bindings(&binary.right, owned);
        }
        syn::Expr::Let(let_) if matches!(let_.expr.as_ref(), syn::Expr::Path(_)) => {
            if let Some(binding) = option_pattern_binding(&let_.pat) {
                owned.insert(binding);
            }
        }
        syn::Expr::Paren(paren) => collect_moved_condition_bindings(&paren.expr, owned),
        _ => {}
    }
}

fn remove_shadowing_clone_from_condition(
    condition: &mut syn::Expr,
    used_later: &HashSet<String>,
    owned_names: &HashSet<String>,
) {
    match condition {
        syn::Expr::Binary(binary) if matches!(binary.op, syn::BinOp::And(_)) => {
            remove_shadowing_clone_from_condition(&mut binary.left, used_later, owned_names);
            remove_shadowing_clone_from_condition(&mut binary.right, used_later, owned_names);
        }
        syn::Expr::Let(let_) => {
            let Some(binding) = option_pattern_binding(&let_.pat) else {
                return;
            };
            let syn::Expr::MethodCall(clone) = let_.expr.as_ref() else {
                return;
            };
            if clone.method != "clone" || !clone.args.is_empty() {
                return;
            }
            let syn::Expr::Path(source) = clone.receiver.as_ref() else {
                return;
            };
            if source.path.is_ident(&binding)
                && owned_names.contains(&binding)
                && !used_later.contains(&binding)
            {
                let_.expr = clone.receiver.clone();
            }
        }
        syn::Expr::Paren(paren) => {
            remove_shadowing_clone_from_condition(&mut paren.expr, used_later, owned_names);
        }
        _ => {}
    }
}

fn option_pattern_binding(pattern: &syn::Pat) -> Option<String> {
    let syn::Pat::TupleStruct(tuple) = pattern else {
        return None;
    };
    if tuple.path.segments.last()?.ident != "Some" || tuple.elems.len() != 1 {
        return None;
    }
    simple_pattern_name(tuple.elems.first()?)
}

fn simple_pattern_name(pattern: &syn::Pat) -> Option<String> {
    match pattern {
        syn::Pat::Ident(binding) if binding.subpat.is_none() => Some(binding.ident.to_string()),
        syn::Pat::Type(typed) => simple_pattern_name(&typed.pat),
        syn::Pat::Paren(paren) => simple_pattern_name(&paren.pat),
        _ => None,
    }
}

include!("lexical_uses.rs");

struct LastUseCloneRemover<'names> {
    movable: &'names HashSet<String>,
    remaining: HashMap<String, usize>,
}

struct ReturnExpressionCloneRemover<'names> {
    owned: &'names HashSet<String>,
}

impl VisitMut for ReturnExpressionCloneRemover<'_> {
    fn visit_expr_if_mut(&mut self, branch: &mut syn::ExprIf) {
        self.visit_expr_mut(&mut branch.cond);
        self.visit_block_mut(&mut branch.then_branch);
        if block_ends_control_flow(&branch.then_branch) {
            remove_last_use_clones_with_owned(&mut branch.then_branch.stmts, self.owned, false);
        }
        if let Some((_, alternative)) = &mut branch.else_branch {
            self.visit_expr_mut(alternative);
            if let syn::Expr::Block(block) = alternative.as_mut()
                && block_ends_control_flow(&block.block)
            {
                remove_last_use_clones_with_owned(&mut block.block.stmts, self.owned, false);
            }
        }
    }

    fn visit_expr_return_mut(&mut self, returned: &mut syn::ExprReturn) {
        let Some(expression) = &mut returned.expr else {
            return;
        };
        let mut counts = IdentifierUseCounter::default();
        counts.visit_expr(expression);
        LastUseCloneRemover {
            movable: self.owned,
            remaining: counts.counts,
        }
        .visit_expr_mut(expression);
    }

    fn visit_expr_closure_mut(&mut self, _closure: &mut syn::ExprClosure) {}

    fn visit_item_mut(&mut self, _item: &mut syn::Item) {}
}

fn block_ends_control_flow(block: &syn::Block) -> bool {
    matches!(
        block.stmts.last(),
        Some(syn::Stmt::Expr(
            syn::Expr::Return(_) | syn::Expr::Break(_) | syn::Expr::Continue(_),
            _
        ))
    )
}

pub(super) fn remove_last_use_if_let_clones(branch: &mut syn::ExprIf) {
    let syn::Expr::Let(condition) = branch.cond.as_ref() else {
        return;
    };
    if expression_produces_borrowed_binding(&condition.expr) {
        let mut borrowed = HashSet::new();
        collect_owned_pattern_names(&condition.pat, &mut borrowed);
        LoopBindingUseRewriter {
            owned: &HashSet::new(),
            borrowed: &borrowed,
        }
        .visit_block_mut(&mut branch.then_branch);
        return;
    }
    if !expression_produces_owned_if_let_binding(&condition.expr) {
        return;
    }
    let mut owned = HashSet::new();
    collect_owned_pattern_names(&condition.pat, &mut owned);
    remove_last_use_clones_with_owned(&mut branch.then_branch.stmts, &owned, true);
}

pub(super) fn remove_owned_generated_error_arm_clones(match_: &mut syn::ExprMatch) {
    if !matches!(match_.expr.as_ref(), syn::Expr::Path(path)
        if path.qself.is_none()
            && path.path.segments.len() == 1
            && matches!(path.path.segments[0].ident.to_string().as_str(), name
                if name.starts_with("sifr_generated_try_err")
                    || name.starts_with("sifr_generated_try_res")))
    {
        return;
    }
    for arm in &mut match_.arms {
        let mut owned = HashSet::new();
        collect_owned_pattern_names(&arm.pat, &mut owned);
        if let syn::Expr::Block(body) = arm.body.as_mut() {
            remove_last_use_clones_with_owned(&mut body.block.stmts, &owned, false);
        }
    }
}

fn expression_produces_owned_if_let_binding(expression: &syn::Expr) -> bool {
    matches!(expression, syn::Expr::MethodCall(call)
        if matches!(call.method.to_string().as_str(), "clone" | "to_owned" | "to_string")
            && call.args.is_empty())
        || matches!(expression, syn::Expr::Path(path)
            if path.qself.is_none()
                && path.path.segments.len() == 1
                && path.path.segments[0]
                    .ident
                    .to_string()
                    .starts_with("sifr_generated_try_res"))
}

fn expression_produces_borrowed_binding(expression: &syn::Expr) -> bool {
    matches!(expression, syn::Expr::Reference(_))
        || matches!(expression, syn::Expr::MethodCall(call)
            if matches!(call.method.to_string().as_str(),
                "as_ref" | "as_mut" | "as_deref" | "as_deref_mut"))
}

fn collect_owned_pattern_names(pattern: &syn::Pat, owned: &mut HashSet<String>) {
    match pattern {
        syn::Pat::Ident(binding) if binding.by_ref.is_none() => {
            owned.insert(binding.ident.to_string());
            if let Some((_, subpattern)) = &binding.subpat {
                collect_owned_pattern_names(subpattern, owned);
            }
        }
        syn::Pat::Or(or) => {
            for case in &or.cases {
                collect_owned_pattern_names(case, owned);
            }
        }
        syn::Pat::Paren(paren) => collect_owned_pattern_names(&paren.pat, owned),
        syn::Pat::Reference(_) => {}
        syn::Pat::Slice(slice) => {
            for element in &slice.elems {
                collect_owned_pattern_names(element, owned);
            }
        }
        syn::Pat::Struct(struct_) => {
            for field in &struct_.fields {
                collect_owned_pattern_names(&field.pat, owned);
            }
        }
        syn::Pat::Tuple(tuple) => {
            for element in &tuple.elems {
                collect_owned_pattern_names(element, owned);
            }
        }
        syn::Pat::TupleStruct(tuple) => {
            for element in &tuple.elems {
                collect_owned_pattern_names(element, owned);
            }
        }
        syn::Pat::Type(typed) => collect_owned_pattern_names(&typed.pat, owned),
        _ => {}
    }
}

include!("borrowed_loop_binding.rs");

impl VisitMut for LastUseCloneRemover<'_> {
    fn visit_block_mut(&mut self, block: &mut syn::Block) {
        let mut movable = self.movable.clone();
        for statement in &mut block.stmts {
            let mut nested = LastUseCloneRemover {
                movable: &movable,
                remaining: std::mem::take(&mut self.remaining),
            };
            nested.visit_stmt_mut(statement);
            self.remaining = nested.remaining;
            if let syn::Stmt::Local(local) = statement {
                for name in
                    crate::generated_rust_canonicalizer::syntax_cleanup::identifier_names_in_pattern(
                        &local.pat,
                    )
                {
                    movable.remove(&name);
                }
            }
        }
    }

    fn visit_expr_if_mut(&mut self, branch: &mut syn::ExprIf) {
        self.visit_expr_mut(&mut branch.cond);
        let mut movable = self.movable.clone();
        let mut condition = IdentifierUseCounter::default();
        condition.visit_condition(&branch.cond);
        for name in &condition.shadowed {
            movable.remove(name);
        }
        let mut nested = LastUseCloneRemover {
            movable: &movable,
            remaining: std::mem::take(&mut self.remaining),
        };
        nested.visit_block_mut(&mut branch.then_branch);
        self.remaining = nested.remaining;
        if let Some((_, alternative)) = &mut branch.else_branch {
            self.visit_expr_mut(alternative);
        }
    }

    fn visit_local_mut(&mut self, local: &mut syn::Local) {
        // An unused clone can call user code, and a named unused binding retains
        // the value until scope exit. Neither is an ownership-transfer proof.
        if simple_pattern_name(&local.pat).is_some_and(|name| name.starts_with('_'))
            || matches!(local.pat, syn::Pat::Wild(_))
        {
            return;
        }
        visit_mut::visit_local_mut(self, local);
    }
    fn visit_expr_mut(&mut self, expression: &mut syn::Expr) {
        visit_mut::visit_expr_mut(self, expression);
        let syn::Expr::MethodCall(call) = expression else {
            return;
        };
        if call.method != "clone" || !call.args.is_empty() {
            return;
        }
        let Some(root) = expression_root_name(&call.receiver) else {
            return;
        };
        if self.movable.contains(&root) && self.remaining.get(&root).copied().unwrap_or(0) == 0 {
            *expression = call.receiver.as_ref().clone();
        }
    }

    fn visit_expr_path_mut(&mut self, path: &mut syn::ExprPath) {
        if path.qself.is_none()
            && path.path.segments.len() == 1
            && let Some(segment) = path.path.segments.first()
            && self.movable.contains(&segment.ident.to_string())
            && let Some(remaining) = self.remaining.get_mut(&segment.ident.to_string())
        {
            *remaining = remaining.saturating_sub(1);
        }
    }

    fn visit_expr_call_mut(&mut self, call: &mut syn::ExprCall) {
        if let Some(closure) = immediately_called_closure(&call.func) {
            let mut closure = closure.clone();
            // Arguments can still borrow a capture before the call starts.
            let mut movable = self.movable.clone();
            for argument in &call.args {
                movable.retain(|name| !expression_mentions_name(argument, name));
            }
            for input in &closure.inputs {
                for name in
                    crate::generated_rust_canonicalizer::syntax_cleanup::identifier_names_in_pattern(
                        input,
                    )
                {
                    movable.remove(&name);
                }
            }
            let mut nested = LastUseCloneRemover {
                movable: &movable,
                remaining: std::mem::take(&mut self.remaining),
            };
            nested.visit_expr_mut(&mut closure.body);
            self.remaining = nested.remaining;
            *call.func = syn::parse_quote!((#closure));
        }
        self.visit_expr_mut(&mut call.func);
        let protected = call
            .args
            .iter()
            .enumerate()
            .map(|(index, argument)| {
                terminal_clone_root(argument).is_some_and(|name| {
                    call.args.iter().enumerate().any(|(other, sibling)| {
                        other != index
                            && expression_mentions_name(sibling, &name)
                            && !clones_disjoint_fields(argument, sibling)
                    })
                })
            })
            .collect::<Vec<_>>();
        for (argument, protected) in call.args.iter_mut().zip(protected) {
            if protected && let syn::Expr::MethodCall(clone) = argument {
                self.visit_expr_mut(&mut clone.receiver);
            } else {
                self.visit_expr_mut(argument);
            }
        }
    }

    fn visit_item_mut(&mut self, _item: &mut syn::Item) {}

    fn visit_expr_for_loop_mut(&mut self, for_loop: &mut syn::ExprForLoop) {
        // The iterator expression executes once; the body may execute repeatedly.
        self.visit_expr_mut(&mut for_loop.expr);
    }

    fn visit_expr_while_mut(&mut self, _while_loop: &mut syn::ExprWhile) {}

    fn visit_expr_loop_mut(&mut self, _loop: &mut syn::ExprLoop) {}

    fn visit_expr_closure_mut(&mut self, _closure: &mut syn::ExprClosure) {}

    fn visit_macro_mut(&mut self, rust_macro: &mut syn::Macro) {
        let Ok(mut arguments) = rust_macro.parse_body_with(
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
        ) else {
            return;
        };
        for argument in &mut arguments {
            self.visit_expr_mut(argument);
        }
        rust_macro.tokens = arguments.to_token_stream();
    }
}

fn clones_disjoint_fields(left: &syn::Expr, right: &syn::Expr) -> bool {
    fn field(expression: &syn::Expr) -> Option<&syn::ExprField> {
        let syn::Expr::MethodCall(call) = expression else {
            return None;
        };
        if call.method != "clone" || !call.args.is_empty() {
            return None;
        }
        if let syn::Expr::Field(field) = call.receiver.as_ref() {
            Some(field)
        } else {
            None
        }
    }
    matches!((field(left), field(right)), (Some(left), Some(right))
        if left.base.to_token_stream().to_string() == right.base.to_token_stream().to_string()
            && left.member != right.member)
}

fn immediately_called_closure(expression: &syn::Expr) -> Option<&syn::ExprClosure> {
    match expression {
        syn::Expr::Closure(closure) => Some(closure),
        syn::Expr::Paren(paren) => immediately_called_closure(&paren.expr),
        syn::Expr::Group(group) => immediately_called_closure(&group.expr),
        _ => None,
    }
}

fn terminal_clone_root(expression: &syn::Expr) -> Option<String> {
    let syn::Expr::MethodCall(call) = expression else {
        return None;
    };
    (call.method == "clone" && call.args.is_empty())
        .then(|| expression_root_name(&call.receiver))
        .flatten()
}

struct LoopControlCollector {
    found: bool,
}

impl Visit<'_> for LoopControlCollector {
    fn visit_expr_break(&mut self, _expression: &syn::ExprBreak) {
        self.found = true;
    }

    fn visit_expr_continue(&mut self, _expression: &syn::ExprContinue) {
        self.found = true;
    }

    fn visit_item(&mut self, _item: &syn::Item) {}
}

fn expression_root_name(expression: &syn::Expr) -> Option<String> {
    match expression {
        syn::Expr::Path(path) if path.qself.is_none() && path.path.segments.len() == 1 => path
            .path
            .segments
            .first()
            .map(|segment| segment.ident.to_string()),
        syn::Expr::Field(field) => expression_root_name(&field.base),
        syn::Expr::Paren(paren) => expression_root_name(&paren.expr),
        syn::Expr::Reference(reference) => expression_root_name(&reference.expr),
        _ => None,
    }
}

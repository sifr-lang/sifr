pub(super) fn remove_last_use_clones(statements: &mut [syn::Stmt]) {
    let mut locally_owned = HashSet::new();
    for statement in statements.iter() {
        let syn::Stmt::Local(local) = statement else {
            continue;
        };
        if local
            .init
            .as_ref()
            .is_none_or(|init| !expression_produces_borrowed_binding(&init.expr))
        {
            let mut bindings = HashSet::new();
            collect_owned_pattern_names(&local.pat, &mut bindings);
            let self_shadowing = local.init.as_ref().is_some_and(|init| {
                bindings
                    .iter()
                    .any(|name| expression_mentions_name(&init.expr, name))
            });
            if !self_shadowing {
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
        rewrite_discarded_error_message_copy(local);
        let Some(name) = simple_pattern_name(&local.pat) else {
            continue;
        };
        if name.starts_with('_') && uses.counts.get(&name).copied().unwrap_or(0) == 0 {
            local.pat = syn::parse_quote!(_);
            rewrite_discarded_error_message_copy(local);
        }
    }
}

pub(super) fn add_complex_local_type_expectation(local: &mut syn::Local) {
    local.attrs.retain(|attribute| {
        !attribute.path().is_ident("expect")
            || !attribute.meta.to_token_stream().to_string().contains(
                "this generated carrier preserves nested typed Sifr error and tuple structure",
            ) && !attribute
                .meta
                .to_token_stream()
                .to_string()
                .contains("generated Rust preserves the typed Sifr mapping key")
    });
    if let Some(init) = &local.init
        && simple_pattern_name(&local.pat).as_deref() == Some("sifr_generated_assign_key")
        && let syn::Expr::MethodCall(conversion) = init.expr.as_ref()
        && conversion.args.is_empty()
        && let syn::Expr::Path(source) = conversion.receiver.as_ref()
        && source.path.is_ident("option_name")
    {
        let reason = syn::LitStr::new(
            "language necessity: generated Rust preserves the typed Sifr mapping key while control-flow ownership remains branch-local; owner Item 12; remove when keyed assignment lowering carries path-sensitive last-use proof",
            proc_macro2::Span::call_site(),
        );
        local
            .attrs
            .push(syn::parse_quote!(#[expect(clippy::redundant_clone, reason = #reason)]));
        if conversion.method == "to_owned" || conversion.method == "to_string" {
            local
                .attrs
                .push(syn::parse_quote!(#[expect(clippy::implicit_clone, reason = #reason)]));
        }
    }
    if let syn::Pat::Type(typed) = &local.pat
        && type_contains_large_nested_result_tuple(&typed.ty)
    {
        let reason = syn::LitStr::new(
            "language necessity: this generated carrier preserves nested typed Sifr error and tuple structure; owner Item 12; remove when the carrier representation changes",
            proc_macro2::Span::call_site(),
        );
        local
            .attrs
            .push(syn::parse_quote!(#[expect(clippy::type_complexity, reason = #reason)]));
    }
}

fn type_contains_large_nested_result_tuple(ty: &syn::Type) -> bool {
    let syn::Type::Path(outer) = ty else {
        return false;
    };
    let Some(outer_result) = outer.path.segments.last() else {
        return false;
    };
    let syn::PathArguments::AngleBracketed(outer_arguments) = &outer_result.arguments else {
        return false;
    };
    let Some(syn::GenericArgument::Type(syn::Type::Path(inner))) = outer_arguments.args.first()
    else {
        return false;
    };
    let Some(inner_result) = inner.path.segments.last() else {
        return false;
    };
    let syn::PathArguments::AngleBracketed(inner_arguments) = &inner_result.arguments else {
        return false;
    };
    matches!(inner_arguments.args.first(), Some(syn::GenericArgument::Type(syn::Type::Tuple(tuple))) if tuple.elems.len() >= 6)
}

fn rewrite_discarded_error_message_copy(local: &mut syn::Local) {
    if matches!(local.pat, syn::Pat::Wild(_))
        && let Some(init) = &mut local.init
        && let syn::Expr::MethodCall(call) = init.expr.as_ref()
        && call.method == "to_string"
        && call.args.is_empty()
        && matches!(call.receiver.as_ref(), syn::Expr::Field(field)
            if matches!(&field.member, syn::Member::Named(name) if name == "message"))
    {
        init.expr = call.receiver.clone();
    } else if matches!(local.pat, syn::Pat::Wild(_))
        && let Some(init) = &mut local.init
        && let syn::Expr::MethodCall(call) = init.expr.as_ref()
        && call.method == "clone"
        && call.args.is_empty()
        && matches!(call.receiver.as_ref(), syn::Expr::Path(path)
            if path.qself.is_none()
                && path.path.segments.len() == 1
                && path.path.segments[0]
                    .ident
                    .to_string()
                    .starts_with("sifr_generated_try_err"))
    {
        init.expr = call.receiver.clone();
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
        LastUseCloneRemover {
            movable: &movable,
            remaining: counts.counts.clone(),
        }
        .visit_stmt_mut(statement);
        used_later.extend(counts.counts.into_keys());
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

#[derive(Default)]
struct IdentifierUseCounter {
    counts: HashMap<String, usize>,
}

#[derive(Default)]
struct ClosureCaptureCollector {
    names: HashSet<String>,
}

impl Visit<'_> for ClosureCaptureCollector {
    fn visit_expr_closure(&mut self, closure: &syn::ExprClosure) {
        let mut uses = IdentifierUseCounter::default();
        uses.visit_expr(&closure.body);
        self.names.extend(uses.counts.into_keys());
        visit::visit_expr_closure(self, closure);
    }

    fn visit_item(&mut self, _item: &syn::Item) {}
}

impl Visit<'_> for IdentifierUseCounter {
    fn visit_expr_path(&mut self, path: &syn::ExprPath) {
        if path.qself.is_none()
            && path.path.segments.len() == 1
            && let Some(segment) = path.path.segments.first()
        {
            *self.counts.entry(segment.ident.to_string()).or_default() += 1;
        }
        visit::visit_expr_path(self, path);
    }

    fn visit_item(&mut self, _item: &syn::Item) {}

    fn visit_macro(&mut self, rust_macro: &syn::Macro) {
        if let Ok(arguments) = rust_macro.parse_body_with(
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
        ) {
            for argument in &arguments {
                self.visit_expr(argument);
            }
        }
    }
}

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
            if matches!(call.method.to_string().as_str(), "as_ref" | "as_mut"))
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

struct BorrowOnlyLoopBindingUse<'binding> {
    binding: &'binding str,
    owned_use: bool,
}

struct BindingShadowCollector<'binding> {
    binding: &'binding str,
    found: bool,
}

impl Visit<'_> for BindingShadowCollector<'_> {
    fn visit_expr_let(&mut self, let_: &syn::ExprLet) {
        self.visit_expr(&let_.expr);
        if pattern_contains_name(&let_.pat, self.binding) {
            self.found = true;
        }
    }

    fn visit_local(&mut self, local: &syn::Local) {
        if pattern_contains_name(&local.pat, self.binding) {
            self.found = true;
        }
        visit::visit_local(self, local);
    }

    fn visit_item(&mut self, _item: &syn::Item) {}
}

impl Visit<'_> for BorrowOnlyLoopBindingUse<'_> {
    fn visit_expr_if(&mut self, branch: &syn::ExprIf) {
        if let syn::Expr::Let(let_) = branch.cond.as_ref() {
            self.visit_expr(&let_.expr);
            if !pattern_contains_name(&let_.pat, self.binding) {
                self.visit_block(&branch.then_branch);
            }
            if let Some((_, alternative)) = &branch.else_branch {
                self.visit_expr(alternative);
            }
            return;
        }
        visit::visit_expr_if(self, branch);
    }

    fn visit_expr_method_call(&mut self, call: &syn::ExprMethodCall) {
        if matches!(call.receiver.as_ref(), syn::Expr::Path(path)
            if path.path.is_ident(self.binding))
        {
            for argument in &call.args {
                self.visit_expr(argument);
            }
            return;
        }
        visit::visit_expr_method_call(self, call);
    }

    fn visit_expr_binary(&mut self, binary: &syn::ExprBinary) {
        if comparison_operator(&binary.op) {
            return;
        }
        visit::visit_expr_binary(self, binary);
    }

    fn visit_expr_reference(&mut self, reference: &syn::ExprReference) {
        if matches!(reference.expr.as_ref(), syn::Expr::Path(path)
            if path.path.is_ident(self.binding))
        {
            return;
        }
        visit::visit_expr_reference(self, reference);
    }

    fn visit_expr_path(&mut self, path: &syn::ExprPath) {
        if path.qself.is_none() && path.path.is_ident(self.binding) {
            self.owned_use = true;
            return;
        }
        visit::visit_expr_path(self, path);
    }

    fn visit_macro(&mut self, rust_macro: &syn::Macro) {
        if rust_macro
            .tokens
            .to_string()
            .split_whitespace()
            .any(|token| token == self.binding)
        {
            self.owned_use = true;
            return;
        }
        visit::visit_macro(self, rust_macro);
    }

    fn visit_item(&mut self, _item: &syn::Item) {}
}

fn pattern_contains_name(pattern: &syn::Pat, expected: &str) -> bool {
    match pattern {
        syn::Pat::Ident(binding) => binding.ident == expected,
        syn::Pat::Paren(paren) => pattern_contains_name(&paren.pat, expected),
        syn::Pat::Reference(reference) => pattern_contains_name(&reference.pat, expected),
        syn::Pat::Type(typed) => pattern_contains_name(&typed.pat, expected),
        syn::Pat::Tuple(tuple) => tuple
            .elems
            .iter()
            .any(|element| pattern_contains_name(element, expected)),
        syn::Pat::TupleStruct(tuple) => tuple
            .elems
            .iter()
            .any(|element| pattern_contains_name(element, expected)),
        _ => false,
    }
}

struct BorrowedLoopComparisonRewriter<'binding> {
    binding: &'binding str,
}

impl VisitMut for BorrowedLoopComparisonRewriter<'_> {
    fn visit_expr_binary_mut(&mut self, binary: &mut syn::ExprBinary) {
        visit_mut::visit_expr_binary_mut(self, binary);
        if !comparison_operator(&binary.op) {
            return;
        }
        if matches!(binary.left.as_ref(), syn::Expr::Path(path) if path.path.is_ident(self.binding))
            && let syn::Expr::Unary(right) = binary.right.as_ref()
            && matches!(right.op, syn::UnOp::Deref(_))
        {
            binary.right = right.expr.clone();
        }
        if matches!(binary.right.as_ref(), syn::Expr::Path(path) if path.path.is_ident(self.binding))
            && let syn::Expr::Unary(left) = binary.left.as_ref()
            && matches!(left.op, syn::UnOp::Deref(_))
        {
            binary.left = left.expr.clone();
        }
    }

    fn visit_item_mut(&mut self, _item: &mut syn::Item) {}
}

impl VisitMut for LastUseCloneRemover<'_> {
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
            && let Some(remaining) = self.remaining.get_mut(&segment.ident.to_string())
        {
            *remaining = remaining.saturating_sub(1);
        }
    }

    fn visit_item_mut(&mut self, _item: &mut syn::Item) {}

    fn visit_expr_for_loop_mut(&mut self, _for_loop: &mut syn::ExprForLoop) {}

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

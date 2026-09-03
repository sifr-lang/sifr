use quote::ToTokens;
use std::collections::HashSet;
use syn::punctuated::Punctuated;
use syn::visit::Visit;

pub(super) fn factor_shared_if_prefix(expression: &mut syn::Expr) {
    let binding = unused_internal_binding(expression, "sifr_generated_shared_branch_condition");
    let value_binding = unused_internal_binding(expression, "sifr_generated_shared_branch_value");
    let syn::Expr::If(branch) = expression else {
        return;
    };
    let Some((_, alternative)) = &branch.else_branch else {
        return;
    };
    let syn::Expr::Block(else_block) = alternative.as_ref() else {
        return;
    };
    let (Some(then_first), Some(else_first)) = (
        branch.then_branch.stmts.first(),
        else_block.block.stmts.first(),
    ) else {
        return;
    };
    if then_first.to_token_stream().to_string() != else_first.to_token_stream().to_string() {
        return;
    }
    let shared = then_first;
    let then_rest = &branch.then_branch.stmts[1..];
    let else_rest = &else_block.block.stmts[1..];
    if let syn::Expr::Let(let_condition) = branch.cond.as_ref() {
        let mut condition_bindings = HashSet::new();
        collect_pattern_bindings(&let_condition.pat, &mut condition_bindings);
        let mut shared_identifiers = HashSet::new();
        collect_statement_identifiers(shared, &mut shared_identifiers);
        if !condition_bindings.is_disjoint(&shared_identifiers) {
            return;
        }
        let pattern = let_condition.pat.as_ref();
        let value = let_condition.expr.as_ref();
        *expression = syn::parse_quote!({
            let #value_binding = #value;
            #shared
            if let #pattern = #value_binding { #(#then_rest)* } else { #(#else_rest)* }
        });
        return;
    }
    if condition_contains_let(branch.cond.as_ref()) {
        return;
    }
    let condition = branch.cond.as_ref();
    *expression = syn::parse_quote!({
        let #binding = #condition;
        #shared
        if #binding { #(#then_rest)* } else { #(#else_rest)* }
    });
}

fn condition_contains_let(condition: &syn::Expr) -> bool {
    struct LetDetector(bool);

    impl<'ast> Visit<'ast> for LetDetector {
        fn visit_expr_let(&mut self, _expression: &'ast syn::ExprLet) {
            self.0 = true;
        }
    }

    let mut detector = LetDetector(false);
    detector.visit_expr(condition);
    detector.0
}

fn unused_internal_binding(expression: &syn::Expr, base: &str) -> syn::Ident {
    let mut identifiers = HashSet::new();
    collect_identifiers(expression.to_token_stream(), &mut identifiers);
    let mut candidate = base.to_string();
    let mut suffix = 2_usize;
    while identifiers.contains(&candidate) {
        candidate = format!("{base}_{suffix}");
        suffix += 1;
    }
    syn::Ident::new(&candidate, proc_macro2::Span::call_site())
}

fn collect_identifiers(tokens: proc_macro2::TokenStream, identifiers: &mut HashSet<String>) {
    for token in tokens {
        match token {
            proc_macro2::TokenTree::Ident(identifier) => {
                identifiers.insert(identifier.to_string());
            }
            proc_macro2::TokenTree::Group(group) => {
                collect_identifiers(group.stream(), identifiers);
            }
            _ => {}
        }
    }
}

pub(super) fn factor_shared_if_suffix(expression: &mut syn::Expr) {
    let syn::Expr::If(branch) = expression else {
        return;
    };
    let Some((_, alternative)) = &branch.else_branch else {
        return;
    };
    let syn::Expr::Block(else_block) = alternative.as_ref() else {
        return;
    };
    let shared_count = branch
        .then_branch
        .stmts
        .iter()
        .rev()
        .zip(else_block.block.stmts.iter().rev())
        .take_while(|(left, right)| {
            statement_is_movable_suffix(left)
                && left.to_token_stream().to_string() == right.to_token_stream().to_string()
        })
        .count();
    if shared_count == 0 {
        return;
    }
    let then_split = branch.then_branch.stmts.len() - shared_count;
    let else_split = else_block.block.stmts.len() - shared_count;
    let shared = branch.then_branch.stmts[then_split..].to_vec();
    let mut branch_bindings = HashSet::new();
    collect_statement_bindings(
        &branch.then_branch.stmts[..then_split],
        &mut branch_bindings,
    );
    collect_statement_bindings(&else_block.block.stmts[..else_split], &mut branch_bindings);
    // Moving the suffix outside either branch also moves it past the drop of
    // every branch-local binding. Without resolved types, no local's `Drop`
    // implementation can be assumed unobservable. Statement macros are also
    // excluded because they can expand to a binding whose lexical lifetime is
    // not visible in the parsed syntax tree.
    let branch_prefixes_are_lifetime_transparent = branch_bindings.is_empty()
        && branch.then_branch.stmts[..then_split]
            .iter()
            .chain(&else_block.block.stmts[..else_split])
            .all(statement_cannot_introduce_branch_local);
    let mut shared_identifiers = HashSet::new();
    for statement in &shared {
        collect_statement_identifiers(statement, &mut shared_identifiers);
    }
    let condition_bindings = condition_binding_names(&branch.cond);
    // A let-chain binding lives through the selected branch in the original
    // expression. Hoisting any suffix would make that binding drop first.
    if !branch_prefixes_are_lifetime_transparent
        || !condition_bindings.is_empty()
        || !branch_bindings.is_disjoint(&shared_identifiers)
        || !condition_bindings.is_disjoint(&shared_identifiers)
    {
        return;
    }
    let condition = branch.cond.as_ref();
    let then_prefix = &branch.then_branch.stmts[..then_split];
    let else_prefix = &else_block.block.stmts[..else_split];
    *expression = syn::parse_quote!({
        if #condition { #(#then_prefix)* } else { #(#else_prefix)* }
        #(#shared)*
    });
}

fn collect_statement_identifiers(statement: &syn::Stmt, identifiers: &mut HashSet<String>) {
    collect_identifiers(statement.to_token_stream(), identifiers);
    FormatCaptureCollector { names: identifiers }.visit_stmt(statement);
}

struct FormatCaptureCollector<'names> {
    names: &'names mut HashSet<String>,
}

impl<'ast> Visit<'ast> for FormatCaptureCollector<'_> {
    fn visit_macro(&mut self, rust_macro: &'ast syn::Macro) {
        self.names
            .extend(crate::generated_rust_canonicalizer::format_capture::names(
                rust_macro,
            ));
        syn::visit::visit_macro(self, rust_macro);
    }
}

fn condition_binding_names(condition: &syn::Expr) -> HashSet<String> {
    struct ConditionBindingCollector {
        names: HashSet<String>,
    }

    impl<'ast> Visit<'ast> for ConditionBindingCollector {
        fn visit_expr_let(&mut self, expression: &'ast syn::ExprLet) {
            collect_pattern_bindings(&expression.pat, &mut self.names);
            syn::visit::visit_expr_let(self, expression);
        }
    }

    let mut collector = ConditionBindingCollector {
        names: HashSet::new(),
    };
    collector.visit_expr(condition);
    collector.names
}

fn statement_is_movable_suffix(statement: &syn::Stmt) -> bool {
    matches!(statement, syn::Stmt::Expr(_, Some(_)))
}

fn statement_cannot_introduce_branch_local(statement: &syn::Stmt) -> bool {
    matches!(statement, syn::Stmt::Expr(_, Some(_)))
}

fn collect_statement_bindings(statements: &[syn::Stmt], bindings: &mut HashSet<String>) {
    for statement in statements {
        let syn::Stmt::Local(local) = statement else {
            continue;
        };
        collect_pattern_bindings(&local.pat, bindings);
    }
}

fn collect_pattern_bindings(pattern: &syn::Pat, bindings: &mut HashSet<String>) {
    match pattern {
        syn::Pat::Ident(binding) => {
            bindings.insert(binding.ident.to_string());
        }
        syn::Pat::Tuple(tuple) => {
            for element in &tuple.elems {
                collect_pattern_bindings(element, bindings);
            }
        }
        syn::Pat::TupleStruct(tuple) => {
            for element in &tuple.elems {
                collect_pattern_bindings(element, bindings);
            }
        }
        syn::Pat::Or(or) => {
            for alternative in &or.cases {
                collect_pattern_bindings(alternative, bindings);
            }
        }
        syn::Pat::Reference(reference) => collect_pattern_bindings(&reference.pat, bindings),
        syn::Pat::Slice(slice) => {
            for element in &slice.elems {
                collect_pattern_bindings(element, bindings);
            }
        }
        syn::Pat::Struct(struct_) => {
            for field in &struct_.fields {
                collect_pattern_bindings(&field.pat, bindings);
            }
        }
        syn::Pat::Type(typed) => collect_pattern_bindings(&typed.pat, bindings),
        syn::Pat::Paren(paren) => collect_pattern_bindings(&paren.pat, bindings),
        _ => {}
    }
}

pub(super) fn collapse_nested_if(expression: &mut syn::Expr) {
    let syn::Expr::If(outer) = expression else {
        return;
    };
    if outer.else_branch.is_some() {
        return;
    }
    let [syn::Stmt::Expr(syn::Expr::If(inner), _)] = outer.then_branch.stmts.as_slice() else {
        return;
    };
    if inner.else_branch.is_some() {
        return;
    }
    let outer_condition = outer.cond.clone();
    let inner_condition = inner.cond.clone();
    *outer.cond = combine_and_conditions(*outer_condition, *inner_condition);
    outer.then_branch = inner.then_branch.clone();
}

fn combine_and_conditions(left: syn::Expr, right: syn::Expr) -> syn::Expr {
    let mut terms = Vec::new();
    collect_and_terms(left, &mut terms);
    collect_and_terms(right, &mut terms);
    let mut terms = terms.into_iter();
    let Some(first) = terms.next() else {
        return syn::parse_quote!(true);
    };
    terms.fold(first, |left, right| {
        syn::Expr::Binary(syn::ExprBinary {
            attrs: Vec::new(),
            left: Box::new(left),
            op: syn::BinOp::And(syn::token::AndAnd::default()),
            right: Box::new(right),
        })
    })
}

fn collect_and_terms(expression: syn::Expr, terms: &mut Vec<syn::Expr>) {
    match expression {
        syn::Expr::Binary(binary) if matches!(binary.op, syn::BinOp::And(_)) => {
            collect_and_terms(*binary.left, terms);
            collect_and_terms(*binary.right, terms);
        }
        other => terms.push(other),
    }
}

pub(super) fn flatten_or_pattern(pattern: &mut syn::Pat) {
    let syn::Pat::Or(or_pattern) = pattern else {
        return;
    };
    let mut cases = Punctuated::new();
    for case in std::mem::take(&mut or_pattern.cases) {
        if let syn::Pat::Or(nested) = case {
            cases.extend(nested.cases);
        } else {
            cases.push(case);
        }
    }
    or_pattern.cases = cases;
}

pub(super) fn factor_tuple_struct_or_pattern(pattern: &syn::Pat) -> Option<syn::Pat> {
    let syn::Pat::Or(or_pattern) = pattern else {
        return None;
    };
    let mut cases = or_pattern.cases.iter();
    let syn::Pat::TupleStruct(first) = cases.next()? else {
        return None;
    };
    if first.elems.len() != 1 {
        return None;
    }
    let mut inner = Punctuated::<syn::Pat, syn::Token![|]>::new();
    inner.push(first.elems.first()?.clone());
    for case in cases {
        let syn::Pat::TupleStruct(tuple) = case else {
            return None;
        };
        if tuple.path.to_token_stream().to_string() != first.path.to_token_stream().to_string()
            || tuple.elems.len() != 1
        {
            return None;
        }
        inner.push(tuple.elems.first()?.clone());
    }
    let path = first.path.clone();
    Some(syn::parse_quote!(#path(#inner)))
}

pub(super) fn collapse_else_if(expression: &mut syn::Expr) {
    let syn::Expr::If(branch) = expression else {
        return;
    };
    let Some((_, else_expression)) = &mut branch.else_branch else {
        return;
    };
    let syn::Expr::Block(block) = else_expression.as_ref() else {
        return;
    };
    let [syn::Stmt::Expr(syn::Expr::If(nested), None)] = block.block.stmts.as_slice() else {
        return;
    };
    **else_expression = syn::Expr::If(nested.clone());
}

pub(super) fn collapse_identical_if_else_branches(expression: &mut syn::Expr) {
    let syn::Expr::If(branch) = expression else {
        return;
    };
    let Some((_, else_expression)) = &branch.else_branch else {
        return;
    };
    let syn::Expr::If(nested) = else_expression.as_ref() else {
        return;
    };
    if branch.then_branch.to_token_stream().to_string()
        != nested.then_branch.to_token_stream().to_string()
    {
        return;
    }

    let first = branch.cond.as_ref();
    let second = nested.cond.as_ref();
    let combined: syn::Expr = syn::parse_quote!((#first) || (#second));
    *branch.cond = combined;
    branch.else_branch = nested.else_branch.clone();
}

pub(super) fn invert_negative_condition_with_else(expression: &mut syn::Expr) {
    let syn::Expr::If(branch) = expression else {
        return;
    };
    let Some((else_token, else_expression)) = branch.else_branch.take() else {
        return;
    };
    let positive_condition = match branch.cond.as_mut() {
        syn::Expr::Binary(condition) => {
            let syn::BinOp::Ne(not_equal) = condition.op else {
                branch.else_branch = Some((else_token, else_expression));
                return;
            };
            condition.op = syn::BinOp::Eq(syn::token::EqEq(not_equal.spans));
            None
        }
        syn::Expr::Unary(condition) if matches!(condition.op, syn::UnOp::Not(_)) => {
            Some(condition.expr.as_ref().clone())
        }
        _ => {
            branch.else_branch = Some((else_token, else_expression));
            return;
        }
    };
    if let Some(positive_condition) = positive_condition {
        *branch.cond = positive_condition;
    }
    let previous_then = std::mem::replace(
        &mut branch.then_branch,
        super::super::expression_into_block(else_expression),
    );
    branch.else_branch = Some((
        else_token,
        Box::new(syn::Expr::Block(syn::ExprBlock {
            attrs: Vec::new(),
            label: None,
            block: previous_then,
        })),
    ));
}

pub(super) fn remove_single_expression_block(expression: &mut syn::Expr) {
    let syn::Expr::Block(block) = expression else {
        return;
    };
    if block.label.is_some() || !block.attrs.is_empty() {
        return;
    }
    let [syn::Stmt::Expr(inner, None)] = block.block.stmts.as_slice() else {
        return;
    };
    *expression = inner.clone();
}

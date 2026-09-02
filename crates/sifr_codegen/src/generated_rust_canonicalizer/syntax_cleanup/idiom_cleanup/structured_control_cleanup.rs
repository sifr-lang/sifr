use quote::ToTokens;
use syn::punctuated::Punctuated;

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

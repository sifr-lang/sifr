pub(super) fn fold_initial_assignments(
    statements: &mut Vec<syn::Stmt>,
    discardable: &impl Fn(&syn::Local) -> bool,
    references: &super::super::identifier_collection::InitializerReferences,
) {
    let mut index = 0;
    while index + 1 < statements.len() {
        let Some(name) = initializable_local_name(&statements[index], discardable) else {
            index += 1;
            continue;
        };
        let Some(default) = (match &statements[index] {
            syn::Stmt::Local(local) => local.init.as_ref().map(|init| init.expr.as_ref()),
            _ => None,
        }) else {
            index += 1;
            continue;
        };
        let Some(replacement) =
            initialization_value(&statements[index + 1], &name, &statements[index], references)
                .or_else(|| conditional_initial_assignment(&statements[index + 1], &name, default, references))
        else {
            index += 1;
            continue;
        };
        let syn::Stmt::Local(local) = &mut statements[index] else {
            index += 1;
            continue;
        };
        if let Some(init) = &mut local.init {
            *init.expr = replacement;
        }
        statements.remove(index + 1);
    }
}

fn conditional_initial_assignment(
    statement: &syn::Stmt,
    name: &str,
    default: &syn::Expr,
    references: &super::super::identifier_collection::InitializerReferences,
) -> Option<syn::Expr> {
    let syn::Stmt::Expr(syn::Expr::If(branch), _) = statement else {
        return None;
    };
    if branch.else_branch.is_some() || branch.then_branch.stmts.len() != 1 {
        return None;
    }
    let value = direct_assignment_value(branch.then_branch.stmts.first()?, name, references)?;
    let condition = branch.cond.as_ref();
    if references.expression(condition, name) {
        return None;
    }
    Some(syn::parse_quote!(if #condition { #value } else { #default }))
}

pub(super) fn fold_tail_bindings(statements: &mut Vec<syn::Stmt>) {
    let [
        prefix @ ..,
        syn::Stmt::Local(local),
        syn::Stmt::Expr(tail, semicolon),
    ] = statements.as_slice()
    else {
        return;
    };
    let Some(name) = simple_binding_name(&local.pat) else {
        return;
    };
    if !local.attrs.is_empty()
        || local
            .init
            .as_ref()
            .is_none_or(|init| init.diverge.is_some())
        || !matches!(tail, syn::Expr::Path(path) if path.qself.is_none() && path.path.is_ident(&name))
    {
        return;
    }
    let Some(init) = &local.init else {
        return;
    };
    let mut value = init.expr.as_ref().clone();
    if let syn::Pat::Type(typed) = &local.pat
        && let syn::Expr::Call(call) = &mut value
        && call.args.is_empty()
        && let syn::Expr::Path(callee) = call.func.as_mut()
        && let syn::Type::Path(declared) = typed.ty.as_ref()
        && callee.path.segments.len() == 2
        && declared.path.segments.len() == 1
        && callee.path.segments[0].ident == declared.path.segments[0].ident
        && matches!(
            callee.path.segments[1].ident.to_string().as_str(),
            "new" | "default"
        )
    {
        // The local may be the only element-type evidence, notably in an
        // empty-list equality where SifrInt has several PartialEq impls.
        let mut arguments = declared.path.segments[0].arguments.clone();
        if let syn::PathArguments::AngleBracketed(arguments) = &mut arguments {
            arguments.colon2_token = Some(Default::default());
        }
        callee.path.segments[0].arguments = arguments;
    }
    let replacement = syn::Stmt::Expr(value, *semicolon);
    let prefix_len = prefix.len();
    statements.truncate(prefix_len);
    statements.push(replacement);
}

pub(super) fn fold_delayed_initializations(
    statements: &mut Vec<syn::Stmt>,
    mutating_methods: &HashSet<String>,
    discardable: &impl Fn(&syn::Local) -> bool,
    references: &super::super::identifier_collection::InitializerReferences,
) {
    let mut declaration_index = 0;
    while declaration_index < statements.len() {
        let Some((name, mut pattern, attrs)) =
            movable_default_declaration(&statements[declaration_index], discardable)
        else {
            declaration_index += 1;
            continue;
        };
        let mut assignment_index = declaration_index + 1;
        while assignment_index < statements.len()
            && !references.statement(&statements[assignment_index], &name)
        {
            assignment_index += 1;
        }
        if assignment_index == statements.len() {
            declaration_index += 1;
            continue;
        }
        let Some(value) = initialization_value(
            &statements[assignment_index],
            &name,
            &statements[declaration_index],
            references,
        ) else {
            declaration_index += 1;
            continue;
        };
        if !super::super::mutability_cleanup::statements_mutate_name(
            &statements[assignment_index + 1..],
            &name,
            mutating_methods,
        ) {
            remove_pattern_mutability(&mut pattern);
        }
        statements[assignment_index] = syn::parse_quote!(#(#attrs)* let #pattern = #value;);
        statements.remove(declaration_index);
    }
}

fn movable_default_declaration(
    statement: &syn::Stmt,
    discardable: &impl Fn(&syn::Local) -> bool,
) -> Option<(String, syn::Pat, Vec<syn::Attribute>)> {
    let syn::Stmt::Local(local) = statement else {
        return None;
    };
    let name = simple_binding_name(&local.pat)?;
    discardable(local).then(|| (name, local.pat.clone(), local.attrs.clone()))
}

fn direct_assignment_value(statement: &syn::Stmt, name: &str,
    references: &super::super::identifier_collection::InitializerReferences,
) -> Option<syn::Expr> {
    if let syn::Stmt::Expr(syn::Expr::Block(block), _) = statement {
        let [inner] = block.block.stmts.as_slice() else {
            return None;
        };
        return direct_assignment_value(inner, name, references);
    }
    if let syn::Stmt::Expr(syn::Expr::If(branch), _) = statement {
        let [then_statement] = branch.then_branch.stmts.as_slice() else {
            return None;
        };
        let then_value = direct_assignment_value(then_statement, name, references)?;
        let (_, alternative) = branch.else_branch.as_ref()?;
        if !matches!(alternative.as_ref(), syn::Expr::Block(block)
            if matches!(block.block.stmts.last(), Some(syn::Stmt::Expr(syn::Expr::Return(_), _))))
        {
            return None;
        }
        let condition = branch.cond.as_ref();
        if references.expression(condition, name) { return None; }
        return Some(syn::parse_quote!(if #condition { #then_value } else #alternative));
    }
    let syn::Stmt::Expr(syn::Expr::Assign(assignment), Some(_)) = statement else {
        return None;
    };
    if !matches!(assignment.left.as_ref(), syn::Expr::Path(path) if path.path.is_ident(name))
        || references.expression(&assignment.right, name)
    {
        return None;
    }
    Some(assignment.right.as_ref().clone())
}

fn initialization_value(
    statement: &syn::Stmt,
    name: &str,
    declaration: &syn::Stmt,
    references: &super::super::identifier_collection::InitializerReferences,
) -> Option<syn::Expr> {
    if let Some(value) = direct_assignment_value(statement, name, references) {
        return Some(value);
    }
    // SifrInt's Clone implementation has value semantics. Do not substitute
    // arbitrary user clone_from implementations at an initialization boundary.
    let syn::Stmt::Local(local) = declaration else {
        return None;
    };
    let syn::Pat::Type(typed) = &local.pat else {
        return None;
    };
    if !matches!(typed.ty.as_ref(), syn::Type::Path(path) if path.path.is_ident("SifrInt")) {
        return None;
    }
    let syn::Stmt::Expr(syn::Expr::MethodCall(call), Some(_)) = statement else {
        return None;
    };
    if call.method != "clone_from"
        || call.args.len() != 1
        || !matches!(call.receiver.as_ref(), syn::Expr::Path(path) if path.path.is_ident(name))
    {
        return None;
    }
    let syn::Expr::Reference(reference) = &call.args[0] else {
        return None;
    };
    if references.expression(&reference.expr, name) {
        return None;
    }
    let source = &reference.expr;
    Some(syn::parse_quote!((#source).clone()))
}


fn remove_pattern_mutability(pattern: &mut syn::Pat) {
    match pattern {
        syn::Pat::Ident(binding) => binding.mutability = None,
        syn::Pat::Type(typed) => remove_pattern_mutability(&mut typed.pat),
        syn::Pat::Paren(paren) => remove_pattern_mutability(&mut paren.pat),
        _ => {}
    }
}

fn initializable_local_name(
    statement: &syn::Stmt,
    discardable: &impl Fn(&syn::Local) -> bool,
) -> Option<String> {
    let syn::Stmt::Local(local) = statement else {
        return None;
    };
    let name = simple_binding_name(&local.pat)?;
    discardable(local).then_some(name)
}

use std::collections::HashSet;
use syn::visit::{self, Visit};
use syn::visit_mut::{self, VisitMut};

pub(super) fn disambiguate_similar_parameter_names(
    signature: &mut syn::Signature,
    body: &mut syn::Block,
) {
    let names = signature
        .inputs
        .iter()
        .enumerate()
        .flat_map(|(index, input)| match input {
            syn::FnArg::Receiver(_) => Vec::new(),
            syn::FnArg::Typed(typed) => pattern_binding_names(&typed.pat)
                .into_iter()
                .map(|name| (index, name))
                .collect(),
        })
        .collect::<Vec<_>>();
    let mut occupied = names
        .iter()
        .map(|(_, name)| name.clone())
        .collect::<HashSet<_>>();
    let mut body_bindings = PatternBindingCollector { names: Vec::new() };
    body_bindings.visit_block(body);
    occupied.extend(body_bindings.names);

    for right in 1..names.len() {
        let (argument_index, name) = &names[right];
        if name.starts_with("sifr_generated_")
            || is_disambiguated_name(name)
            || !names[..right]
                .iter()
                .any(|(_, other)| source_names_are_too_similar(other, name))
        {
            continue;
        }
        let replacement = unoccupied_parameter_name(name, &mut occupied);
        let renamed = match &mut signature.inputs[*argument_index] {
            syn::FnArg::Receiver(_) => false,
            syn::FnArg::Typed(typed) => rename_pattern_binding(&mut typed.pat, name, &replacement),
        };
        if renamed {
            LocalReferenceRenamer {
                from: name,
                to: &replacement,
            }
            .visit_block_mut(body);
        }
    }
}

pub(super) fn disambiguate_similar_local_names(statements: &mut [syn::Stmt]) {
    let names = statements
        .iter()
        .enumerate()
        .flat_map(|(index, statement)| {
            let syn::Stmt::Local(local) = statement else {
                return Vec::new();
            };
            pattern_binding_names(&local.pat)
                .into_iter()
                .map(|name| (index, name))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let mut occupied = names
        .iter()
        .map(|(_, name)| name.clone())
        .collect::<HashSet<_>>();
    for right in 1..names.len() {
        let (statement_index, name) = &names[right];
        if name.starts_with("sifr_generated_")
            || is_disambiguated_name(name)
            || !names[..right]
                .iter()
                .any(|(_, other)| source_names_are_too_similar(other, name))
        {
            continue;
        }
        let replacement = unoccupied_value_name(name, &mut occupied);
        let renamed = if let syn::Stmt::Local(local) = &mut statements[*statement_index] {
            rename_pattern_binding(&mut local.pat, name, &replacement)
        } else {
            false
        };
        if !renamed {
            continue;
        }
        let mut renamer = LocalReferenceRenamer {
            from: name,
            to: &replacement,
        };
        for statement in &mut statements[*statement_index + 1..] {
            renamer.visit_stmt_mut(statement);
            if matches!(statement, syn::Stmt::Local(local) if pattern_binds_name(&local.pat, name))
            {
                break;
            }
        }
    }
}

pub(super) fn disambiguate_similar_names_across_nested_scopes(
    signature: &syn::Signature,
    body: &mut syn::Block,
) {
    let mut occupied = signature
        .inputs
        .iter()
        .flat_map(|input| match input {
            syn::FnArg::Receiver(_) => Vec::new(),
            syn::FnArg::Typed(typed) => pattern_binding_names(&typed.pat),
        })
        .collect::<HashSet<_>>();
    FunctionWideNameDisambiguator {
        occupied: &mut occupied,
        depth: 0,
    }
    .visit_block_mut(body);
}

struct FunctionWideNameDisambiguator<'names> {
    occupied: &'names mut HashSet<String>,
    depth: usize,
}

impl VisitMut for FunctionWideNameDisambiguator<'_> {
    fn visit_block_mut(&mut self, block: &mut syn::Block) {
        let should_rename = self.depth > 0;
        self.depth += 1;
        for index in 0..block.stmts.len() {
            let names = match &block.stmts[index] {
                syn::Stmt::Local(local) => pattern_binding_names(&local.pat),
                _ => Vec::new(),
            };
            for name in names {
                if name.starts_with("sifr_generated_") || is_disambiguated_name(&name) {
                    self.occupied.insert(name);
                    continue;
                }
                let is_too_similar = should_rename
                    && self
                        .occupied
                        .iter()
                        .any(|other| other != &name && source_names_are_too_similar(other, &name));
                let replacement =
                    is_too_similar.then(|| unoccupied_value_name(&name, self.occupied));
                if let Some(replacement) = replacement
                    && let syn::Stmt::Local(local) = &mut block.stmts[index]
                    && rename_pattern_binding(&mut local.pat, &name, &replacement)
                {
                    let mut renamer = LocalReferenceRenamer {
                        from: &name,
                        to: &replacement,
                    };
                    for statement in &mut block.stmts[index + 1..] {
                        renamer.visit_stmt_mut(statement);
                        if matches!(statement, syn::Stmt::Local(local)
                            if pattern_binds_name(&local.pat, &name))
                        {
                            break;
                        }
                    }
                } else {
                    self.occupied.insert(name);
                }
            }
            self.visit_stmt_mut(&mut block.stmts[index]);
        }
        self.depth -= 1;
    }
}

fn unoccupied_value_name(name: &str, occupied: &mut HashSet<String>) -> String {
    let fingerprint = stable_name_fingerprint(name);
    let mut replacement = format!("{name}_value_{fingerprint:016x}");
    while occupied.contains(&replacement) {
        replacement.push_str("_binding");
    }
    occupied.insert(replacement.clone());
    replacement
}

fn unoccupied_parameter_name(name: &str, occupied: &mut HashSet<String>) -> String {
    let fingerprint = stable_name_fingerprint(name);
    let mut replacement = format!("{name}_argument_{fingerprint:016x}");
    while occupied.contains(&replacement) {
        replacement.push_str("_binding");
    }
    occupied.insert(replacement.clone());
    replacement
}

fn stable_name_fingerprint(name: &str) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
    name.bytes().fold(FNV_OFFSET_BASIS, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(FNV_PRIME)
    })
}

fn is_disambiguated_name(name: &str) -> bool {
    name.ends_with("_value") || name.contains("_value_") || name.contains("_argument_")
}

fn source_names_are_too_similar(left: &str, right: &str) -> bool {
    if left.starts_with("sifr_generated_") || right.starts_with("sifr_generated_") {
        return false;
    }
    if left.len().min(right.len()) < 4 {
        return false;
    }
    let length_difference = left.len().abs_diff(right.len());
    (length_difference <= 2
        && common_prefix_length(left, right) >= 4
        && common_prefix_length(left, right) >= left.len().min(right.len()).saturating_sub(2))
        || names_differ_by_one_inserted_character(left, right)
        || names_differ_by_one_substituted_character(left, right)
}

fn names_differ_by_one_substituted_character(left: &str, right: &str) -> bool {
    left.len() == right.len()
        && left.len() >= 4
        && left
            .bytes()
            .zip(right.bytes())
            .filter(|(left, right)| left != right)
            .count()
            == 1
}

fn names_differ_by_one_inserted_character(left: &str, right: &str) -> bool {
    let (shorter, longer) = if left.len() <= right.len() {
        (left.as_bytes(), right.as_bytes())
    } else {
        (right.as_bytes(), left.as_bytes())
    };
    if longer.len() != shorter.len() + 1 {
        return false;
    }
    let mut short_index = 0;
    let mut long_index = 0;
    let mut skipped = false;
    while short_index < shorter.len() && long_index < longer.len() {
        if shorter[short_index] == longer[long_index] {
            short_index += 1;
            long_index += 1;
        } else if skipped {
            return false;
        } else {
            skipped = true;
            long_index += 1;
        }
    }
    true
}

fn common_prefix_length(left: &str, right: &str) -> usize {
    left.bytes()
        .zip(right.bytes())
        .take_while(|(left, right)| left == right)
        .count()
}

fn pattern_binding_names(pattern: &syn::Pat) -> Vec<String> {
    let mut collector = PatternBindingCollector { names: Vec::new() };
    collector.visit_pat(pattern);
    collector.names
}

fn pattern_binds_name(pattern: &syn::Pat, name: &str) -> bool {
    pattern_binding_names(pattern)
        .iter()
        .any(|binding| binding == name)
}

struct PatternBindingCollector {
    names: Vec<String>,
}

impl<'ast> Visit<'ast> for PatternBindingCollector {
    fn visit_pat_ident(&mut self, pattern: &'ast syn::PatIdent) {
        self.names.push(pattern.ident.to_string());
        visit::visit_pat_ident(self, pattern);
    }
}

fn rename_pattern_binding(pattern: &mut syn::Pat, name: &str, replacement: &str) -> bool {
    struct PatternBindingRenamer<'name> {
        name: &'name str,
        replacement: &'name str,
        renamed: bool,
    }
    impl VisitMut for PatternBindingRenamer<'_> {
        fn visit_field_pat_mut(&mut self, field: &mut syn::FieldPat) {
            if field.colon_token.is_none()
                && matches!(&field.member, syn::Member::Named(member) if member == self.name)
                && pattern_binds_name(&field.pat, self.name)
            {
                field.colon_token = Some(syn::token::Colon::default());
            }
            visit_mut::visit_field_pat_mut(self, field);
        }

        fn visit_pat_ident_mut(&mut self, binding: &mut syn::PatIdent) {
            if binding.ident == self.name {
                binding.ident = proc_macro2::Ident::new(self.replacement, binding.ident.span());
                self.renamed = true;
            }
            visit_mut::visit_pat_ident_mut(self, binding);
        }
    }
    let mut renamer = PatternBindingRenamer {
        name,
        replacement,
        renamed: false,
    };
    renamer.visit_pat_mut(pattern);
    renamer.renamed
}

struct LocalReferenceRenamer<'name> {
    from: &'name str,
    to: &'name str,
}

impl LocalReferenceRenamer<'_> {
    fn rename_tokens(&self, tokens: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        tokens
            .into_iter()
            .map(|token| match token {
                proc_macro2::TokenTree::Ident(identifier) if identifier == self.from => {
                    proc_macro2::TokenTree::Ident(proc_macro2::Ident::new(
                        self.to,
                        identifier.span(),
                    ))
                }
                proc_macro2::TokenTree::Group(group) => {
                    let mut renamed = proc_macro2::Group::new(
                        group.delimiter(),
                        self.rename_tokens(group.stream()),
                    );
                    renamed.set_span(group.span());
                    proc_macro2::TokenTree::Group(renamed)
                }
                token => token,
            })
            .collect()
    }

    fn rename_format_capture(&self, rust_macro: &mut syn::Macro) {
        super::format_capture::rename(rust_macro, self.from, self.to);
    }
}

impl VisitMut for LocalReferenceRenamer<'_> {
    fn visit_block_mut(&mut self, block: &mut syn::Block) {
        for statement in &mut block.stmts {
            self.visit_stmt_mut(statement);
            if matches!(statement, syn::Stmt::Local(local) if pattern_binds_name(&local.pat, self.from))
            {
                break;
            }
        }
    }

    fn visit_expr_if_mut(&mut self, branch: &mut syn::ExprIf) {
        for attribute in &mut branch.attrs {
            self.visit_attribute_mut(attribute);
        }
        let shadows = self.visit_condition_mut(&mut branch.cond);
        if !shadows {
            self.visit_block_mut(&mut branch.then_branch);
        }
        if let Some((_, alternative)) = &mut branch.else_branch {
            self.visit_expr_mut(alternative);
        }
    }

    fn visit_expr_while_mut(&mut self, loop_: &mut syn::ExprWhile) {
        for attribute in &mut loop_.attrs {
            self.visit_attribute_mut(attribute);
        }
        let shadows = self.visit_condition_mut(&mut loop_.cond);
        if !shadows {
            self.visit_block_mut(&mut loop_.body);
        }
    }

    fn visit_expr_match_mut(&mut self, match_: &mut syn::ExprMatch) {
        for attribute in &mut match_.attrs {
            self.visit_attribute_mut(attribute);
        }
        self.visit_expr_mut(&mut match_.expr);
        for arm in &mut match_.arms {
            for attribute in &mut arm.attrs {
                self.visit_attribute_mut(attribute);
            }
            if pattern_binds_name(&arm.pat, self.from) {
                continue;
            }
            if let syn::Pat::Guard(guard) = &mut arm.pat {
                self.visit_expr_mut(&mut guard.guard);
            }
            self.visit_expr_mut(&mut arm.body);
        }
    }

    fn visit_expr_closure_mut(&mut self, closure: &mut syn::ExprClosure) {
        for attribute in &mut closure.attrs {
            self.visit_attribute_mut(attribute);
        }
        if !closure
            .inputs
            .iter()
            .any(|input| pattern_binds_name(input, self.from))
        {
            self.visit_expr_mut(&mut closure.body);
        }
    }

    fn visit_expr_for_loop_mut(&mut self, loop_: &mut syn::ExprForLoop) {
        for attribute in &mut loop_.attrs {
            self.visit_attribute_mut(attribute);
        }
        self.visit_expr_mut(&mut loop_.expr);
        if !pattern_binds_name(&loop_.pat, self.from) {
            self.visit_block_mut(&mut loop_.body);
        }
    }

    fn visit_expr_path_mut(&mut self, path: &mut syn::ExprPath) {
        if path.qself.is_none()
            && path.path.segments.len() == 1
            && let Some(segment) = path.path.segments.first_mut()
            && segment.ident == self.from
        {
            segment.ident = proc_macro2::Ident::new(self.to, segment.ident.span());
        }
        visit_mut::visit_expr_path_mut(self, path);
    }

    fn visit_macro_mut(&mut self, rust_macro: &mut syn::Macro) {
        rust_macro.tokens = self.rename_tokens(rust_macro.tokens.clone());
        self.rename_format_capture(rust_macro);
        visit_mut::visit_macro_mut(self, rust_macro);
    }
}

impl LocalReferenceRenamer<'_> {
    fn visit_condition_mut(&mut self, condition: &mut syn::Expr) -> bool {
        let syn::Expr::Binary(binary) = condition else {
            if let syn::Expr::Let(let_) = condition {
                self.visit_expr_mut(&mut let_.expr);
                return pattern_binds_name(&let_.pat, self.from);
            }
            self.visit_expr_mut(condition);
            return false;
        };
        if !matches!(binary.op, syn::BinOp::And(_)) {
            self.visit_expr_mut(condition);
            return false;
        }
        let left_shadows = self.visit_condition_mut(&mut binary.left);
        let right_shadows = if left_shadows {
            condition_binds_name(&binary.right, self.from)
        } else {
            self.visit_condition_mut(&mut binary.right)
        };
        left_shadows || right_shadows
    }
}

fn condition_binds_name(condition: &syn::Expr, name: &str) -> bool {
    match condition {
        syn::Expr::Let(let_) => pattern_binds_name(&let_.pat, name),
        syn::Expr::Binary(binary) if matches!(binary.op, syn::BinOp::And(_)) => {
            condition_binds_name(&binary.left, name) || condition_binds_name(&binary.right, name)
        }
        _ => false,
    }
}

use crate::CompilerFragment;
use crate::ir_validate::{IrValidationIssue, IrValidationKind};
use proc_macro2::{Delimiter, TokenStream, TokenTree};
use syn::visit::{self, Visit};

pub(crate) fn assert_generated_source_is_safe(source: &str, context: &str) {
    let errors = match validate_generated_rust_source(source) {
        Ok(()) => return,
        Err(errors) => errors,
    };
    assert!(
        errors.is_empty(),
        "generated Rust validation failed ({context}): {}",
        errors.join(" | ")
    );
}

/// Validate a complete generated Rust source file before materialization.
pub fn validate_generated_rust_source(source: &str) -> Result<(), Vec<String>> {
    let issues = validate_generated_source(source);
    if issues.is_empty() {
        return Ok(());
    }
    Err(issues.into_iter().map(|issue| issue.message).collect())
}

pub(crate) fn validate_generated_source(source: &str) -> Vec<IrValidationIssue> {
    let file = match syn::parse_file(source) {
        Ok(file) => file,
        Err(error) => {
            return vec![IrValidationIssue {
                kind: IrValidationKind::InvalidGeneratedSource,
                message: format!("generated Rust source is invalid: {error}"),
            }];
        }
    };
    forbidden_file_issues(&file, None, IrValidationKind::ForbiddenGeneratedSource)
}

#[derive(Default)]
struct ForbiddenGeneratedRust {
    constructs: Vec<&'static str>,
}

impl<'ast> Visit<'ast> for ForbiddenGeneratedRust {
    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let syn::Expr::Path(path) = node.func.as_ref()
            && let Some(segment) = path.path.segments.last()
        {
            match segment.ident.to_string().as_str() {
                "unwrap" => self.constructs.push("unwrap("),
                "expect" => self.constructs.push("expect("),
                _ => {}
            }
        }
        visit::visit_expr_call(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        match node.method.to_string().as_str() {
            "unwrap" => self.constructs.push(".unwrap("),
            "expect" => self.constructs.push(".expect("),
            _ => {}
        }
        visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_macro(&mut self, node: &'ast syn::ExprMacro) {
        if let Some(segment) = node.mac.path.segments.last() {
            match segment.ident.to_string().as_str() {
                "panic" => self.constructs.push("panic!"),
                "todo" => self.constructs.push("todo!"),
                "unimplemented" => self.constructs.push("unimplemented!"),
                _ => {}
            }
        }
        visit::visit_expr_macro(self, node);
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        if let Some(segment) = node.path.segments.last() {
            match segment.ident.to_string().as_str() {
                "panic" => self.constructs.push("panic!"),
                "todo" => self.constructs.push("todo!"),
                "unimplemented" => self.constructs.push("unimplemented!"),
                _ => {}
            }
        }
        scan_macro_tokens(&node.tokens, &mut self.constructs);
        visit::visit_macro(self, node);
    }

    fn visit_expr_unsafe(&mut self, node: &'ast syn::ExprUnsafe) {
        self.constructs.push("unsafe block");
        visit::visit_expr_unsafe(self, node);
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        if matches!(&node.sig.safety, syn::Safety::Unsafe(_)) {
            self.constructs.push("unsafe fn");
        }
        visit::visit_item_fn(self, node);
    }

    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        if node.unsafety.is_some() {
            self.constructs.push("unsafe impl");
        }
        visit::visit_item_impl(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        if matches!(&node.sig.safety, syn::Safety::Unsafe(_)) {
            self.constructs.push("unsafe impl method");
        }
        visit::visit_impl_item_fn(self, node);
    }

    fn visit_item_trait(&mut self, node: &'ast syn::ItemTrait) {
        if node.unsafety.is_some() {
            self.constructs.push("unsafe trait");
        }
        visit::visit_item_trait(self, node);
    }

    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        if matches!(&node.sig.safety, syn::Safety::Unsafe(_)) {
            self.constructs.push("unsafe trait method");
        }
        visit::visit_trait_item_fn(self, node);
    }

    fn visit_item_foreign_mod(&mut self, node: &'ast syn::ItemForeignMod) {
        if node.unsafety.is_some() {
            self.constructs.push("unsafe extern block");
        }
        visit::visit_item_foreign_mod(self, node);
    }

    fn visit_foreign_item_fn(&mut self, node: &'ast syn::ForeignItemFn) {
        if matches!(&node.sig.safety, syn::Safety::Unsafe(_)) {
            self.constructs.push("unsafe foreign function");
        }
        visit::visit_foreign_item_fn(self, node);
    }

    fn visit_attribute(&mut self, node: &'ast syn::Attribute) {
        if node.path().is_ident("allow") || meta_tokens_contain_ident(&node.meta, "allow") {
            self.constructs.push("#[allow(...)]");
        }
        if node.path().is_ident("expect") || meta_tokens_contain_ident(&node.meta, "expect") {
            self.constructs.push("#[expect(...)]");
        }
        visit::visit_attribute(self, node);
    }
}

fn meta_tokens_contain_ident(meta: &syn::Meta, expected: &str) -> bool {
    let syn::Meta::List(list) = meta else {
        return false;
    };
    token_stream_contains_ident(&list.tokens, expected)
}

fn token_stream_contains_ident(tokens: &TokenStream, expected: &str) -> bool {
    tokens.clone().into_iter().any(|token| match token {
        TokenTree::Ident(ident) => ident == expected,
        TokenTree::Group(group) => token_stream_contains_ident(&group.stream(), expected),
        TokenTree::Punct(_) | TokenTree::Literal(_) => false,
    })
}

fn scan_macro_tokens(tokens: &TokenStream, constructs: &mut Vec<&'static str>) {
    let tokens = tokens.clone().into_iter().collect::<Vec<_>>();
    for token in &tokens {
        if let TokenTree::Group(group) = token {
            scan_macro_tokens(&group.stream(), constructs);
        }
        if matches!(token, TokenTree::Ident(ident) if ident == "unsafe") {
            constructs.push("unsafe macro token");
        }
    }
    for window in tokens.windows(2) {
        if let [TokenTree::Punct(hash), TokenTree::Group(group)] = window
            && hash.as_char() == '#'
            && group.delimiter() == Delimiter::Bracket
            && group
                .stream()
                .into_iter()
                .next()
                .is_some_and(|token| matches!(token, TokenTree::Ident(ident) if ident == "allow" || ident == "expect"))
        {
            constructs.push("lint-suppression attribute macro token");
        }
        if let [TokenTree::Ident(ident), TokenTree::Punct(punct)] = window
            && punct.as_char() == '!'
        {
            match ident.to_string().as_str() {
                "panic" => constructs.push("panic! macro token"),
                "todo" => constructs.push("todo! macro token"),
                "unimplemented" => constructs.push("unimplemented! macro token"),
                _ => {}
            }
        }
        if let [TokenTree::Ident(ident), TokenTree::Group(group)] = window
            && group.delimiter() == Delimiter::Parenthesis
        {
            match ident.to_string().as_str() {
                "unwrap" => constructs.push("unwrap( macro token"),
                "expect" => constructs.push("expect( macro token"),
                _ => {}
            }
        }
    }
    for window in tokens.windows(3) {
        if let [
            TokenTree::Punct(dot),
            TokenTree::Ident(ident),
            TokenTree::Group(group),
        ] = window
            && dot.as_char() == '.'
            && group.delimiter() == Delimiter::Parenthesis
        {
            match ident.to_string().as_str() {
                "unwrap" => constructs.push(".unwrap( macro token"),
                "expect" => constructs.push(".expect( macro token"),
                _ => {}
            }
        }
    }
}

pub(crate) fn forbidden_file_issues(
    syntax: &syn::File,
    fragment: Option<&CompilerFragment>,
    kind: IrValidationKind,
) -> Vec<IrValidationIssue> {
    let mut visitor = ForbiddenGeneratedRust::default();
    visitor.visit_file(syntax);
    forbidden_issues_from_visitor(visitor, fragment, kind)
}

pub(crate) fn forbidden_expr_issues(
    syntax: &syn::Expr,
    fragment: &CompilerFragment,
    kind: IrValidationKind,
) -> Vec<IrValidationIssue> {
    let mut visitor = ForbiddenGeneratedRust::default();
    visitor.visit_expr(syntax);
    forbidden_issues_from_visitor(visitor, Some(fragment), kind)
}

fn forbidden_issues_from_visitor(
    mut visitor: ForbiddenGeneratedRust,
    fragment: Option<&CompilerFragment>,
    kind: IrValidationKind,
) -> Vec<IrValidationIssue> {
    visitor.constructs.sort_unstable();
    visitor.constructs.dedup();
    visitor
        .constructs
        .into_iter()
        .map(|construct| {
            let origin = fragment.map_or_else(String::new, |fragment| {
                let origin = fragment.origin();
                format!(
                    " at {}:{}:{}",
                    origin.file(),
                    origin.line(),
                    origin.column()
                )
            });
            IrValidationIssue {
                kind,
                message: format!("forbidden generated Rust construct `{construct}`{origin}"),
            }
        })
        .collect()
}

use quote::ToTokens;
use std::collections::HashSet;
use syn::visit::{self, Visit};

const EXPECTATION_REASON_MARKER: &str =
    "generated Rust preserves this exact typed Sifr source contract";
const EXPECTATION_REASON: &str = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics";

pub(super) fn refresh_function_expectations(
    attrs: &mut Vec<syn::Attribute>,
    signature: &syn::Signature,
    body: &syn::Block,
    owner_has_display: bool,
    trait_impl: bool,
    restricted_api: bool,
) {
    remove_generated_expectations(attrs);
    let mut shape = FunctionShape::default();
    shape.visit_signature(signature);
    shape.visit_block(body);
    if bool_parameter_count(signature) > 3 {
        add_expectation(attrs, "fn_params_excessive_bools");
    }
    if shape.asserts_constant {
        add_expectation(attrs, "assertions_on_constants");
    }
    if shape.has_approximate_constant {
        add_expectation(attrs, "approx_constant");
    }
    if signature.ident == "to_string" && owner_has_display {
        add_expectation(attrs, "inherent_to_string_shadow_display");
    }
    if !is_snake_case(&signature.ident.to_string()) {
        add_rust_expectation(attrs, "non_snake_case");
    }
    if returns_option(signature) && body_is_single_some(body) {
        add_expectation(attrs, "unnecessary_wraps");
    }
    if single_character_binding_count(signature, body) > 4 {
        add_expectation(attrs, "many_single_char_names");
    }
    if restricted_api && signature.inputs.iter().any(argument_is_ref_option) {
        add_expectation(attrs, "ref_option");
    }
    if signature.inputs.iter().any(|argument| {
        let syn::FnArg::Typed(parameter) = argument else {
            return false;
        };
        !matches!(parameter.ty.as_ref(), syn::Type::Reference(_))
            && !type_is_bare_generic(&parameter.ty, signature)
            && !type_is_copy_value(&parameter.ty)
            && !pattern_is_mutable(&parameter.pat)
            && simple_pattern_name(&parameter.pat).is_some_and(|name| {
                parameter_is_only_borrowed(body, &name)
                    && !parameter_has_known_consuming_method(body, &name, &parameter.ty)
            })
    }) {
        add_expectation(attrs, "needless_pass_by_value");
    }
    if signature.inputs.iter().any(|argument| {
        matches!(argument, syn::FnArg::Typed(parameter)
            if parameter.ty.to_token_stream().to_string().ends_with("SifrInt")
                && simple_pattern_name(&parameter.pat).is_some())
    }) && body_calls_function(body, &signature.ident)
    {
        add_expectation(attrs, "needless_pass_by_value");
    }
    if signature.asyncness.is_some() && !shape.has_await {
        add_expectation(attrs, "unused_async");
        if trait_impl || signature.receiver().is_some_and(receiver_is_mutable) {
            add_expectation(attrs, "unused_async_trait_impl");
        }
    }
    if signature.ident == "isclose" || shape.has_nonzero_float_equality {
        add_expectation(attrs, "float_cmp");
    }
    if shape.has_suboptimal_float_power {
        add_expectation(attrs, "suboptimal_flops");
    }
    if restricted_api
        && !trait_impl
        && signature.asyncness.is_none()
        && signature.receiver().is_some()
        && !body_mentions_self(body)
    {
        add_expectation(attrs, "unused_self");
    }
    let rendered_signature = signature.to_token_stream().to_string();
    if rendered_signature.contains("SifrGeneratedSecondaryErrorKind")
        || rendered_signature.contains("SifrGeneratedChannelPushState")
        || rendered_signature.contains("SifrGeneratedChannelPopState")
    {
        add_rust_expectation(attrs, "private_interfaces");
    }
}

fn receiver_is_mutable(receiver: &syn::Receiver) -> bool {
    receiver.mutability.is_some()
        || receiver
            .to_token_stream()
            .to_string()
            .contains("& mut self")
}

fn type_is_copy_value(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Path(path) if path.qself.is_none()
        && path.path.segments.last().is_some_and(|segment| matches!(segment.ident.to_string().as_str(),
            "bool" | "char" | "f32" | "f64" | "i8" | "i16" | "i32" | "i64" | "i128"
                | "isize" | "u8" | "u16" | "u32" | "u64" | "u128" | "usize")))
}

pub(super) fn refresh_struct_expectations(attrs: &mut Vec<syn::Attribute>, fields: &syn::Fields) {
    remove_generated_expectations(attrs);
    let named = fields.iter().collect::<Vec<_>>();
    if named.iter().filter(|field| type_is_bool(&field.ty)).count() > 3 {
        add_expectation(attrs, "struct_excessive_bools");
    }
    if named.iter().any(|field| {
        !matches!(field.vis, syn::Visibility::Inherited) && {
            let ty = field.ty.to_token_stream().to_string();
            ty.contains("SifrGeneratedSecondaryErrorKind")
                || ty.contains("SifrGeneratedChannelState")
        }
    }) {
        add_rust_expectation(attrs, "private_interfaces");
    }
    if named.iter().any(|field| {
        let ty = field.ty.to_token_stream().to_string();
        ty.contains("dyn :: std :: future :: Future") && ty.len() > 200
    }) {
        add_expectation(attrs, "type_complexity");
    }
}

fn parameter_has_known_consuming_method(body: &syn::Block, name: &str, ty: &syn::Type) -> bool {
    if !ty.to_token_stream().to_string().contains("TcpStream") {
        return false;
    }
    let mut use_ = ConsumingMethodUse { name, found: false };
    use_.visit_block(body);
    use_.found
}

struct ConsumingMethodUse<'name> {
    name: &'name str,
    found: bool,
}

impl Visit<'_> for ConsumingMethodUse<'_> {
    fn visit_expr_method_call(&mut self, call: &syn::ExprMethodCall) {
        if matches!(call.receiver.as_ref(), syn::Expr::Path(path) if path.path.is_ident(self.name))
            && call.method == "split"
        {
            self.found = true;
            return;
        }
        visit::visit_expr_method_call(self, call);
    }
}

fn type_is_bare_generic(ty: &syn::Type, signature: &syn::Signature) -> bool {
    let syn::Type::Path(path) = ty else {
        return false;
    };
    let Some(identifier) = path.path.get_ident() else {
        return false;
    };
    signature
        .generics
        .type_params()
        .any(|parameter| parameter.ident == *identifier)
}

fn pattern_is_mutable(pattern: &syn::Pat) -> bool {
    match pattern {
        syn::Pat::Ident(binding) => binding.mutability.is_some(),
        syn::Pat::Type(typed) => pattern_is_mutable(&typed.pat),
        _ => false,
    }
}

pub(super) fn refresh_const_expectations(attrs: &mut Vec<syn::Attribute>, expression: &syn::Expr) {
    remove_generated_expectations(attrs);
    let mut shape = FunctionShape::default();
    shape.visit_expr(expression);
    if shape.has_approximate_constant {
        add_expectation(attrs, "approx_constant");
    }
}

fn remove_generated_expectations(attrs: &mut Vec<syn::Attribute>) {
    attrs.retain(|attribute| {
        !attribute.path().is_ident("expect")
            || !attribute
                .meta
                .to_token_stream()
                .to_string()
                .contains(EXPECTATION_REASON_MARKER)
    });
}

fn argument_is_ref_option(argument: &syn::FnArg) -> bool {
    let syn::FnArg::Typed(parameter) = argument else {
        return false;
    };
    matches!(parameter.ty.as_ref(), syn::Type::Reference(reference)
        if matches!(reference.elem.as_ref(), syn::Type::Path(path)
            if path.path.segments.last().is_some_and(|part| part.ident == "Option")))
        && simple_pattern_name(&parameter.pat).is_some()
}

pub(super) fn parameter_is_only_borrowed(body: &syn::Block, name: &str) -> bool {
    let mut uses = ParameterBorrowUse {
        name,
        borrowed: 0,
        other: 0,
    };
    uses.visit_block(body);
    uses.borrowed > 0 && uses.other == 0
}

struct ParameterBorrowUse<'name> {
    name: &'name str,
    borrowed: usize,
    other: usize,
}

impl Visit<'_> for ParameterBorrowUse<'_> {
    fn visit_expr_async(&mut self, asynchronous: &syn::ExprAsync) {
        let mut uses = ParameterNameUse {
            name: self.name,
            found: false,
        };
        uses.visit_block(&asynchronous.block);
        if uses.found {
            self.other += 1;
            return;
        }
        visit::visit_expr_async(self, asynchronous);
    }

    fn visit_block(&mut self, block: &syn::Block) {
        for statement in &block.stmts {
            if let syn::Stmt::Local(local) = statement {
                if let Some(init) = &local.init {
                    self.visit_expr(&init.expr);
                    if let Some((_, diverge)) = &init.diverge {
                        self.visit_expr(diverge);
                    }
                }
                if pattern_binds_name(&local.pat, self.name) {
                    return;
                }
            } else {
                self.visit_stmt(statement);
            }
        }
    }

    fn visit_expr_if(&mut self, branch: &syn::ExprIf) {
        if let syn::Expr::Let(let_) = branch.cond.as_ref() {
            self.visit_expr(&let_.expr);
            if !pattern_binds_name(&let_.pat, self.name) {
                self.visit_block(&branch.then_branch);
            }
            if let Some((_, alternative)) = &branch.else_branch {
                self.visit_expr(alternative);
            }
            return;
        }
        visit::visit_expr_if(self, branch);
    }

    fn visit_expr_closure(&mut self, closure: &syn::ExprClosure) {
        if closure.capture.is_some() {
            let mut uses = ParameterNameUse {
                name: self.name,
                found: false,
            };
            uses.visit_expr(&closure.body);
            if uses.found {
                self.other += 1;
                return;
            }
        }
        visit::visit_expr_closure(self, closure);
    }

    fn visit_expr_binary(&mut self, binary: &syn::ExprBinary) {
        if matches!(
            binary.op,
            syn::BinOp::Eq(_)
                | syn::BinOp::Ne(_)
                | syn::BinOp::Lt(_)
                | syn::BinOp::Le(_)
                | syn::BinOp::Gt(_)
                | syn::BinOp::Ge(_)
        ) {
            for operand in [&binary.left, &binary.right] {
                if matches!(operand.as_ref(), syn::Expr::Path(path) if path.path.is_ident(self.name))
                {
                    self.borrowed += 1;
                } else {
                    self.visit_expr(operand);
                }
            }
            return;
        }
        visit::visit_expr_binary(self, binary);
    }

    fn visit_expr_reference(&mut self, reference: &syn::ExprReference) {
        if matches!(reference.expr.as_ref(), syn::Expr::Path(path)
            if path.path.is_ident(self.name))
        {
            self.borrowed += 1;
            return;
        }
        visit::visit_expr_reference(self, reference);
    }

    fn visit_expr_method_call(&mut self, call: &syn::ExprMethodCall) {
        if matches!(call.receiver.as_ref(), syn::Expr::Path(path) if path.path.is_ident(self.name))
            && !call.method.to_string().starts_with("into_")
            && !matches!(
                call.method.to_string().as_str(),
                "and_then"
                    | "map"
                    | "map_or"
                    | "map_or_else"
                    | "ok_or"
                    | "ok_or_else"
                    | "unwrap"
                    | "unwrap_or"
                    | "unwrap_or_else"
            )
        {
            self.borrowed += 1;
            for argument in &call.args {
                self.visit_expr(argument);
            }
            return;
        }
        visit::visit_expr_method_call(self, call);
    }

    fn visit_expr_path(&mut self, path: &syn::ExprPath) {
        if path.path.is_ident(self.name) {
            self.other += 1;
            return;
        }
        visit::visit_expr_path(self, path);
    }

    fn visit_item(&mut self, _item: &syn::Item) {}
}

fn pattern_binds_name(pattern: &syn::Pat, name: &str) -> bool {
    match pattern {
        syn::Pat::Ident(binding) => binding.ident == name,
        syn::Pat::Paren(paren) => pattern_binds_name(&paren.pat, name),
        syn::Pat::Reference(reference) => pattern_binds_name(&reference.pat, name),
        syn::Pat::Type(typed) => pattern_binds_name(&typed.pat, name),
        syn::Pat::Tuple(tuple) => tuple
            .elems
            .iter()
            .any(|element| pattern_binds_name(element, name)),
        syn::Pat::TupleStruct(tuple) => tuple
            .elems
            .iter()
            .any(|element| pattern_binds_name(element, name)),
        _ => false,
    }
}

fn body_mentions_self(body: &syn::Block) -> bool {
    let mut use_ = ParameterNameUse {
        name: "self",
        found: false,
    };
    use_.visit_block(body);
    use_.found
}

struct ParameterNameUse<'name> {
    name: &'name str,
    found: bool,
}

impl Visit<'_> for ParameterNameUse<'_> {
    fn visit_expr_path(&mut self, path: &syn::ExprPath) {
        if path.path.is_ident(self.name) {
            self.found = true;
            return;
        }
        visit::visit_expr_path(self, path);
    }

    fn visit_item(&mut self, _item: &syn::Item) {}

    fn visit_macro(&mut self, rust_macro: &syn::Macro) {
        if rust_macro
            .tokens
            .to_string()
            .split_whitespace()
            .any(|token| token == self.name)
        {
            self.found = true;
        }
    }
}

fn add_expectation(attrs: &mut Vec<syn::Attribute>, lint: &str) {
    let Ok(lint_path) = syn::parse_str::<syn::Path>(&format!("clippy::{lint}")) else {
        return;
    };
    let rendered_lint = lint_path.to_token_stream().to_string();
    if attrs.iter().any(|attribute| {
        attribute.path().is_ident("expect")
            && attribute
                .meta
                .to_token_stream()
                .to_string()
                .contains(&rendered_lint)
    }) {
        return;
    }
    let reason = syn::LitStr::new(EXPECTATION_REASON, proc_macro2::Span::call_site());
    attrs.push(syn::parse_quote!(#[expect(#lint_path, reason = #reason)]));
}

pub(super) fn body_calls_function(body: &syn::Block, name: &proc_macro2::Ident) -> bool {
    struct CallFinder<'name> {
        name: &'name proc_macro2::Ident,
        found: bool,
    }

    impl Visit<'_> for CallFinder<'_> {
        fn visit_expr_call(&mut self, call: &syn::ExprCall) {
            if matches!(call.func.as_ref(), syn::Expr::Path(path)
                if path.qself.is_none() && path.path.is_ident(self.name))
            {
                self.found = true;
                return;
            }
            visit::visit_expr_call(self, call);
        }

        fn visit_item(&mut self, _item: &syn::Item) {}
    }

    let mut finder = CallFinder { name, found: false };
    finder.visit_block(body);
    finder.found
}

fn add_rust_expectation(attrs: &mut Vec<syn::Attribute>, lint: &str) {
    let Ok(lint_path) = syn::parse_str::<syn::Path>(lint) else {
        return;
    };
    let reason = syn::LitStr::new(EXPECTATION_REASON, proc_macro2::Span::call_site());
    attrs.push(syn::parse_quote!(#[expect(#lint_path, reason = #reason)]));
}

fn bool_parameter_count(signature: &syn::Signature) -> usize {
    signature
        .inputs
        .iter()
        .filter(|argument| matches!(argument, syn::FnArg::Typed(typed) if type_is_bool(&typed.ty)))
        .count()
}

fn single_character_binding_count(signature: &syn::Signature, body: &syn::Block) -> usize {
    let mut counter = SingleCharacterBindingCounter {
        scopes: vec![HashSet::new()],
        maximum: 0,
    };
    for argument in &signature.inputs {
        if let syn::FnArg::Typed(typed) = argument {
            counter.record_pattern(&typed.pat);
        }
    }
    counter.visit_block(body);
    counter.maximum
}

#[derive(Default)]
struct SingleCharacterBindingCounter {
    scopes: Vec<HashSet<String>>,
    maximum: usize,
}

impl SingleCharacterBindingCounter {
    fn record_pattern(&mut self, pattern: &syn::Pat) {
        let mut names = Vec::new();
        collect_pattern_names(pattern, &mut names);
        for name in names {
            if name.chars().count() == 1
                && !self.scopes.iter().any(|scope| scope.contains(&name))
                && let Some(scope) = self.scopes.last_mut()
            {
                scope.insert(name);
            }
        }
        self.maximum = self.maximum.max(self.scopes.iter().map(HashSet::len).sum());
    }
}

impl<'ast> Visit<'ast> for SingleCharacterBindingCounter {
    fn visit_local(&mut self, local: &'ast syn::Local) {
        if let Some(init) = &local.init {
            self.visit_expr(&init.expr);
            if let Some((_, diverge)) = &init.diverge {
                self.visit_expr(diverge);
            }
        }
        self.record_pattern(&local.pat);
    }

    fn visit_block(&mut self, block: &'ast syn::Block) {
        self.scopes.push(HashSet::new());
        visit::visit_block(self, block);
        self.scopes.pop();
    }

    fn visit_arm(&mut self, arm: &'ast syn::Arm) {
        self.scopes.push(HashSet::new());
        self.record_pattern(&arm.pat);
        if let syn::Pat::Guard(guard) = &arm.pat {
            self.visit_expr(&guard.guard);
        }
        self.visit_expr(&arm.body);
        self.scopes.pop();
    }

    fn visit_item(&mut self, _item: &'ast syn::Item) {}
}

fn collect_pattern_names(pattern: &syn::Pat, names: &mut Vec<String>) {
    match pattern {
        syn::Pat::Ident(binding) => names.push(binding.ident.to_string()),
        syn::Pat::Guard(guard) => collect_pattern_names(&guard.pat, names),
        syn::Pat::Paren(paren) => collect_pattern_names(&paren.pat, names),
        syn::Pat::Reference(reference) => collect_pattern_names(&reference.pat, names),
        syn::Pat::Slice(slice) => {
            for element in &slice.elems {
                collect_pattern_names(element, names);
            }
        }
        syn::Pat::Struct(struct_) => {
            for field in &struct_.fields {
                collect_pattern_names(&field.pat, names);
            }
        }
        syn::Pat::Tuple(tuple) => {
            for element in &tuple.elems {
                collect_pattern_names(element, names);
            }
        }
        syn::Pat::TupleStruct(tuple) => {
            for element in &tuple.elems {
                collect_pattern_names(element, names);
            }
        }
        syn::Pat::Type(typed) => collect_pattern_names(&typed.pat, names),
        _ => {}
    }
}

fn simple_pattern_name(pattern: &syn::Pat) -> Option<String> {
    match pattern {
        syn::Pat::Ident(binding) if binding.subpat.is_none() => Some(binding.ident.to_string()),
        syn::Pat::Type(typed) => simple_pattern_name(&typed.pat),
        syn::Pat::Paren(paren) => simple_pattern_name(&paren.pat),
        _ => None,
    }
}

fn type_is_bool(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Path(path) if path.path.is_ident("bool"))
}

fn returns_option(signature: &syn::Signature) -> bool {
    matches!(&signature.output,
        syn::ReturnType::Type(_, ty)
            if matches!(ty.as_ref(), syn::Type::Path(path)
                if path.path.segments.last().is_some_and(|segment| segment.ident == "Option")))
}

fn body_is_single_some(body: &syn::Block) -> bool {
    let [syn::Stmt::Expr(syn::Expr::Call(call), _)] = body.stmts.as_slice() else {
        return false;
    };
    matches!(call.func.as_ref(), syn::Expr::Path(path) if path.path.is_ident("Some"))
}

fn is_snake_case(name: &str) -> bool {
    !name.chars().any(char::is_uppercase)
}

#[derive(Default)]
struct FunctionShape {
    asserts_constant: bool,
    has_approximate_constant: bool,
    has_await: bool,
    has_arithmetic_binary: bool,
    has_nonzero_float_equality: bool,
    has_suboptimal_float_power: bool,
    float_bindings: HashSet<String>,
}

impl<'ast> Visit<'ast> for FunctionShape {
    fn visit_expr_await(&mut self, expression: &'ast syn::ExprAwait) {
        self.has_await = true;
        visit::visit_expr_await(self, expression);
    }

    fn visit_expr_binary(&mut self, expression: &'ast syn::ExprBinary) {
        if matches!(
            expression.op,
            syn::BinOp::Add(_) | syn::BinOp::Sub(_) | syn::BinOp::Mul(_)
        ) {
            self.has_arithmetic_binary = true;
        }
        if matches!(expression.op, syn::BinOp::Eq(_) | syn::BinOp::Ne(_))
            && !is_zero_float_literal(&expression.left)
            && !is_zero_float_literal(&expression.right)
            && (nonzero_float_literal(&expression.left)
                || nonzero_float_literal(&expression.right)
                || expression_is_float_binding(&expression.left, &self.float_bindings)
                || expression_is_float_binding(&expression.right, &self.float_bindings))
        {
            self.has_nonzero_float_equality = true;
        }
        visit::visit_expr_binary(self, expression);
    }

    fn visit_local(&mut self, local: &'ast syn::Local) {
        if let Some(init) = &local.init {
            self.visit_expr(&init.expr);
            if let Some((_, diverge)) = &init.diverge {
                self.visit_expr(diverge);
            }
        }
        if let syn::Pat::Type(typed) = &local.pat
            && matches!(typed.ty.as_ref(), syn::Type::Path(path) if path.path.is_ident("f64"))
            && let Some(name) = simple_pattern_name(&typed.pat)
        {
            self.float_bindings.insert(name);
        }
    }

    fn visit_expr_method_call(&mut self, expression: &'ast syn::ExprMethodCall) {
        if matches!(expression.method.to_string().as_str(), "powf" | "powi") {
            self.has_suboptimal_float_power = true;
        }
        visit::visit_expr_method_call(self, expression);
    }

    fn visit_lit_float(&mut self, literal: &'ast syn::LitFloat) {
        if let Ok(value) = literal.base10_parse::<f64>()
            && [
                std::f64::consts::PI,
                std::f64::consts::E,
                std::f64::consts::TAU,
            ]
            .iter()
            .any(|constant| (value.abs() - constant).abs() < 0.01)
        {
            self.has_approximate_constant = true;
        }
    }

    fn visit_macro(&mut self, rust_macro: &'ast syn::Macro) {
        if rust_macro.tokens.to_string().contains("await") {
            self.has_await = true;
        }
        let first_argument_is_constant = rust_macro
            .parse_body_with(
                syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
            )
            .ok()
            .and_then(|arguments| arguments.first().cloned())
            .is_some_and(|expression| expression_is_syntactic_constant(&expression));
        if rust_macro.path.is_ident("assert") && first_argument_is_constant {
            self.asserts_constant = true;
        }
        if let Ok(arguments) = rust_macro.parse_body_with(
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
        ) {
            for argument in &arguments {
                self.visit_expr(argument);
            }
        }
        visit::visit_macro(self, rust_macro);
    }
}

fn nonzero_float_literal(expression: &syn::Expr) -> bool {
    matches!(expression, syn::Expr::Lit(literal)
        if matches!(&literal.lit, syn::Lit::Float(value)
            if value.base10_parse::<f64>().is_ok_and(|number| number != 0.0)))
}

fn is_zero_float_literal(expression: &syn::Expr) -> bool {
    matches!(expression, syn::Expr::Lit(literal)
        if matches!(&literal.lit, syn::Lit::Float(value)
            if value.base10_parse::<f64>().is_ok_and(|number| number == 0.0)))
}

fn expression_is_float_binding(expression: &syn::Expr, names: &HashSet<String>) -> bool {
    matches!(expression, syn::Expr::Path(path)
        if path.qself.is_none()
            && path.path.get_ident().is_some_and(|name| names.contains(&name.to_string())))
}

fn expression_is_syntactic_constant(expression: &syn::Expr) -> bool {
    match expression {
        syn::Expr::Lit(_) => true,
        syn::Expr::Path(path) => path.path.segments.last().is_some_and(|segment| {
            let name = segment.ident.to_string();
            name.chars().any(|character| character.is_ascii_uppercase())
                && name.chars().all(|character| {
                    character == '_' || character.is_ascii_uppercase() || character.is_ascii_digit()
                })
        }),
        syn::Expr::Paren(paren) => expression_is_syntactic_constant(&paren.expr),
        syn::Expr::Group(group) => expression_is_syntactic_constant(&group.expr),
        syn::Expr::Unary(unary) => expression_is_syntactic_constant(&unary.expr),
        syn::Expr::Binary(binary) => {
            expression_is_syntactic_constant(&binary.left)
                && expression_is_syntactic_constant(&binary.right)
        }
        _ => false,
    }
}

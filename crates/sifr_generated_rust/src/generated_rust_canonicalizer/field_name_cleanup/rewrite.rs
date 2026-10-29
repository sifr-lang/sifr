use super::registry::{Registry, Ty, qualify};
use quote::ToTokens;
use std::collections::BTreeMap;
use syn::visit_mut::{self, VisitMut};

pub(super) struct Rewriter<'a> {
    registry: &'a Registry,
    module: String,
    owner: Option<String>,
    locals: BTreeMap<String, Ty>,
    pub(super) error: Option<String>,
}

impl<'a> Rewriter<'a> {
    pub(super) fn new(registry: &'a Registry, module: &str) -> Self {
        Self {
            registry,
            module: module.to_string(),
            owner: None,
            locals: BTreeMap::new(),
            error: None,
        }
    }

    fn expression_type(&self, expression: &syn::Expr) -> Ty {
        match expression {
            syn::Expr::Path(path) if path.path.is_ident("self") => self
                .owner
                .as_ref()
                .map_or(Ty::Unknown, |owner| Ty::Named(owner.clone(), Vec::new())),
            syn::Expr::Path(path) => path
                .path
                .get_ident()
                .and_then(|name| self.locals.get(&name.to_string()))
                .cloned()
                .unwrap_or_default(),
            syn::Expr::Struct(value) => Ty::Named(
                self.registry
                    .path(&self.module, &value.path, self.owner.as_deref()),
                Vec::new(),
            ),
            syn::Expr::Reference(value) => self.expression_type(&value.expr),
            syn::Expr::Paren(value) => self.expression_type(&value.expr),
            syn::Expr::Group(value) => self.expression_type(&value.expr),
            syn::Expr::Unary(value) => self.expression_type(&value.expr),
            syn::Expr::Try(value) => self.expression_type(&value.expr).inner(),
            syn::Expr::Index(value) => self.expression_type(&value.expr).inner(),
            syn::Expr::Await(value) => self.expression_type(&value.base),
            syn::Expr::Field(value) => self
                .registry
                .field_type(&self.expression_type(&value.base), &value.member),
            syn::Expr::Tuple(value) => Ty::Tuple(
                value
                    .elems
                    .iter()
                    .map(|value| self.expression_type(value))
                    .collect(),
            ),
            syn::Expr::Call(call) => {
                let mut callee = call.func.as_ref();
                while let syn::Expr::Paren(paren) = callee {
                    callee = &paren.expr;
                }
                if let syn::Expr::Closure(closure) = callee {
                    return match &closure.output {
                        syn::ReturnType::Type(_, ty) => {
                            self.registry.ty(&self.module, ty, self.owner.as_deref())
                        }
                        syn::ReturnType::Default => self.expression_type(&closure.body),
                    };
                }
                let syn::Expr::Path(path) = callee else {
                    return Ty::Unknown;
                };
                let name = self
                    .registry
                    .path(&self.module, &path.path, self.owner.as_deref());
                if self.registry.fields.contains_key(&name) {
                    return Ty::Named(name, Vec::new());
                }
                if matches!(name.as_str(), "Some" | "Ok" | "Err" | "Box::new") {
                    return Ty::Named(
                        name,
                        call.args
                            .first()
                            .map(|value| vec![self.expression_type(value)])
                            .unwrap_or_default(),
                    );
                }
                let owner = name.rsplit_once("::").map(|(owner, _)| owner);
                self.registry.return_type(&name, owner)
            }
            syn::Expr::MethodCall(call) => {
                let receiver = self.expression_type(&call.receiver);
                let declared = receiver.owner().map_or(Ty::Unknown, |owner| {
                    self.registry
                        .return_type(&qualify(owner, &call.method.to_string()), Some(owner))
                });
                if !matches!(declared, Ty::Unknown) {
                    return declared;
                }
                if matches!(call.method.to_string().as_str(), "map" | "map_err")
                    && let Some(syn::Expr::Closure(closure)) = call.args.first()
                    && self.is_standard_container(&receiver)
                {
                    let index = usize::from(call.method == "map_err");
                    let mut scope = Self::new(self.registry, &self.module);
                    scope.owner.clone_from(&self.owner);
                    scope.locals.clone_from(&self.locals);
                    for pattern in &closure.inputs {
                        scope.bind(pattern, receiver.argument(index));
                    }
                    let mapped = scope.expression_type(&closure.body);
                    if let Ty::Named(name, mut args) = receiver {
                        if let Some(argument) = args.get_mut(index) {
                            *argument = mapped;
                        }
                        return Ty::Named(name, args);
                    }
                }
                match call.method.to_string().as_str() {
                    "clone" | "to_owned" | "as_ref" | "as_mut" => receiver,
                    "unwrap" | "expect" | "unwrap_or_else" | "unwrap_or_default" => {
                        receiver.inner()
                    }
                    "iter" | "iter_mut" | "into_iter" => {
                        Ty::Named("Iterator".to_string(), vec![receiver.inner()])
                    }
                    "copied" | "cloned" | "rev" => receiver,
                    "next" => Ty::Named("Option".to_string(), vec![receiver.inner()]),
                    _ => receiver.owner().map_or(Ty::Unknown, |owner| {
                        self.registry
                            .return_type(&qualify(owner, &call.method.to_string()), Some(owner))
                    }),
                }
            }
            syn::Expr::Block(value) => self.block_type(&value.block),
            syn::Expr::Macro(value) if value.mac.path.is_ident("vec") => {
                let tokens = &value.mac.tokens;
                let element = if let Ok(repeat) =
                    syn::parse2::<syn::ExprRepeat>(quote::quote!([#tokens]))
                {
                    self.expression_type(&repeat.expr)
                } else if let Ok(array) = syn::parse2::<syn::ExprArray>(quote::quote!([#tokens])) {
                    array
                        .elems
                        .first()
                        .map_or(Ty::Unknown, |element| self.expression_type(element))
                } else {
                    Ty::Unknown
                };
                Ty::Named("Vec".to_string(), vec![element])
            }
            syn::Expr::If(value) => self.block_type(&value.then_branch),
            syn::Expr::Match(value) => value
                .arms
                .first()
                .map_or(Ty::Unknown, |arm| self.expression_type(&arm.body)),
            _ => Ty::Unknown,
        }
    }

    fn block_type(&self, block: &syn::Block) -> Ty {
        let mut scope = Self::new(self.registry, &self.module);
        scope.owner.clone_from(&self.owner);
        scope.locals.clone_from(&self.locals);
        for statement in &block.stmts {
            if let syn::Stmt::Local(local) = statement {
                let ty = local
                    .init
                    .as_ref()
                    .map_or(Ty::Unknown, |init| scope.expression_type(&init.expr));
                scope.bind(&local.pat, ty);
            }
        }
        match block.stmts.last() {
            Some(syn::Stmt::Expr(value, None)) => scope.expression_type(value),
            _ => Ty::Unknown,
        }
    }

    fn is_standard_container(&self, ty: &Ty) -> bool {
        ty.owner().is_some_and(|owner| {
            !self.registry.fields.contains_key(owner)
                && matches!(
                    owner.trim_start_matches("::"),
                    "Result"
                        | "Option"
                        | "Iterator"
                        | "std::result::Result"
                        | "core::result::Result"
                        | "std::option::Option"
                        | "core::option::Option"
                )
        })
    }

    fn bind(&mut self, pattern: &syn::Pat, ty: Ty) {
        match pattern {
            syn::Pat::Ident(binding) => {
                self.locals.insert(binding.ident.to_string(), ty);
            }
            syn::Pat::Type(binding) => {
                let ty = if matches!(binding.ty.as_ref(), syn::Type::Infer(_)) {
                    ty
                } else {
                    self.registry
                        .ty(&self.module, &binding.ty, self.owner.as_deref())
                };
                self.bind(&binding.pat, ty);
            }
            syn::Pat::Reference(binding) => self.bind(&binding.pat, ty),
            syn::Pat::Paren(binding) => self.bind(&binding.pat, ty),
            syn::Pat::Tuple(binding) => {
                for (index, pattern) in binding.elems.iter().enumerate() {
                    let ty = match &ty {
                        Ty::Tuple(types) => types.get(index).cloned().unwrap_or_default(),
                        _ => Ty::Unknown,
                    };
                    self.bind(pattern, ty);
                }
            }
            syn::Pat::TupleStruct(binding) => {
                let owner = Ty::Named(
                    self.registry
                        .path(&self.module, &binding.path, self.owner.as_deref()),
                    Vec::new(),
                );
                for (position, pattern) in binding.elems.iter().enumerate() {
                    let value = if owner
                        .owner()
                        .is_some_and(|owner| self.registry.fields.contains_key(owner))
                    {
                        self.registry
                            .field_type(&owner, &syn::Member::Unnamed(syn::Index::from(position)))
                    } else {
                        let index = usize::from(binding.path.is_ident("Err"));
                        ty.argument(index)
                    };
                    self.bind(pattern, value);
                }
            }
            syn::Pat::Guard(binding) => self.bind(&binding.pat, ty),
            syn::Pat::Struct(binding) => {
                let owner = Ty::Named(
                    self.registry
                        .path(&self.module, &binding.path, self.owner.as_deref()),
                    Vec::new(),
                );
                for field in &binding.fields {
                    self.bind(&field.pat, self.registry.field_type(&owner, &field.member));
                }
            }
            syn::Pat::Or(binding) => {
                if let Some(pattern) = binding.cases.first() {
                    self.bind(pattern, ty);
                }
            }
            _ => {}
        }
    }

    fn member(&mut self, owner: &Ty, member: &mut syn::Member) -> bool {
        if let Some(names) = owner
            .owner()
            .and_then(|owner| self.registry.fields.get(owner))
        {
            return super::rename(member, names);
        }
        if matches!(owner, Ty::Unknown)
            && let syn::Member::Named(name) = member
            && self
                .registry
                .fields
                .values()
                .any(|fields| fields.contains_key(&name.to_string()))
        {
            self.error.get_or_insert_with(|| {
                format!(
                    "cannot resolve generated field owner for {} in module {}",
                    name, self.module
                )
            });
        }
        false
    }

    fn signature(&mut self, signature: &syn::Signature) {
        for argument in &signature.inputs {
            if let syn::FnArg::Typed(argument) = argument {
                self.bind(
                    &argument.pat,
                    self.registry
                        .ty(&self.module, &argument.ty, self.owner.as_deref()),
                );
            }
        }
    }
}

impl VisitMut for Rewriter<'_> {
    fn visit_item_mod_mut(&mut self, item: &mut syn::ItemMod) {
        let previous = self.module.clone();
        self.module = qualify(&previous, &item.ident.to_string());
        let locals = std::mem::take(&mut self.locals);
        visit_mut::visit_item_mod_mut(self, item);
        self.locals = locals;
        self.module = previous;
    }

    fn visit_item_struct_mut(&mut self, item: &mut syn::ItemStruct) {
        let owner = Ty::Named(qualify(&self.module, &item.ident.to_string()), Vec::new());
        for field in &mut item.fields {
            if let Some(identifier) = &mut field.ident {
                let mut member = syn::Member::Named(identifier.clone());
                self.member(&owner, &mut member);
                if let syn::Member::Named(name) = member {
                    *identifier = name;
                }
            }
        }
    }

    fn visit_item_enum_mut(&mut self, item: &mut syn::ItemEnum) {
        for variant in &mut item.variants {
            let owner = Ty::Named(
                qualify(
                    &qualify(&self.module, &item.ident.to_string()),
                    &variant.ident.to_string(),
                ),
                Vec::new(),
            );
            for field in &mut variant.fields {
                if let Some(identifier) = &mut field.ident {
                    let mut member = syn::Member::Named(identifier.clone());
                    self.member(&owner, &mut member);
                    if let syn::Member::Named(name) = member {
                        *identifier = name;
                    }
                }
            }
        }
    }

    fn visit_item_impl_mut(&mut self, item: &mut syn::ItemImpl) {
        let previous = self.owner.clone();
        self.owner = self
            .registry
            .ty(&self.module, &item.self_ty, None)
            .owner()
            .map(str::to_owned);
        visit_mut::visit_item_impl_mut(self, item);
        self.owner = previous;
    }

    fn visit_item_fn_mut(&mut self, item: &mut syn::ItemFn) {
        let locals = std::mem::take(&mut self.locals);
        self.signature(&item.sig);
        self.visit_block_mut(&mut item.block);
        self.locals = locals;
    }

    fn visit_impl_item_fn_mut(&mut self, item: &mut syn::ImplItemFn) {
        let locals = std::mem::take(&mut self.locals);
        self.signature(&item.sig);
        self.visit_block_mut(&mut item.block);
        self.locals = locals;
    }

    fn visit_block_mut(&mut self, block: &mut syn::Block) {
        let locals = self.locals.clone();
        visit_mut::visit_block_mut(self, block);
        self.locals = locals;
    }

    fn visit_local_mut(&mut self, local: &mut syn::Local) {
        let ty = local
            .init
            .as_ref()
            .map_or(Ty::Unknown, |init| self.expression_type(&init.expr));
        // The initializer sees the previous binding; the pattern introduces the new one.
        if let Some(init) = &mut local.init {
            self.visit_expr_mut(&mut init.expr);
            if let Some((_, diverge)) = &mut init.diverge {
                self.visit_expr_mut(diverge);
            }
        }
        self.bind(&local.pat, ty);
        self.visit_pat_mut(&mut local.pat);
    }

    fn visit_expr_struct_mut(&mut self, value: &mut syn::ExprStruct) {
        let owner = Ty::Named(
            self.registry
                .path(&self.module, &value.path, self.owner.as_deref()),
            Vec::new(),
        );
        for field in &mut value.fields {
            if self.member(&owner, &mut field.member) {
                field.colon_token = Some(Default::default());
            }
            self.visit_expr_mut(&mut field.expr);
        }
        if let Some(rest) = &mut value.rest {
            self.visit_expr_mut(rest);
        }
    }

    fn visit_pat_struct_mut(&mut self, pattern: &mut syn::PatStruct) {
        let owner = Ty::Named(
            self.registry
                .path(&self.module, &pattern.path, self.owner.as_deref()),
            Vec::new(),
        );
        for field in &mut pattern.fields {
            if self.member(&owner, &mut field.member) {
                field.colon_token = Some(Default::default());
            }
            self.visit_pat_mut(&mut field.pat);
        }
    }

    fn visit_expr_field_mut(&mut self, value: &mut syn::ExprField) {
        let owner = self.expression_type(&value.base);
        self.member(&owner, &mut value.member);
        self.visit_expr_mut(&mut value.base);
    }

    fn visit_expr_match_mut(&mut self, value: &mut syn::ExprMatch) {
        let ty = self.expression_type(&value.expr);
        self.visit_expr_mut(&mut value.expr);
        for arm in &mut value.arms {
            let locals = self.locals.clone();
            self.bind(&arm.pat, ty.clone());
            self.visit_arm_mut(arm);
            self.locals = locals;
        }
    }

    fn visit_expr_if_mut(&mut self, value: &mut syn::ExprIf) {
        let locals = self.locals.clone();
        self.visit_expr_mut(&mut value.cond);
        self.visit_block_mut(&mut value.then_branch);
        self.locals = locals;
        if let Some((_, branch)) = &mut value.else_branch {
            self.visit_expr_mut(branch);
        }
    }

    fn visit_expr_let_mut(&mut self, value: &mut syn::ExprLet) {
        let ty = self.expression_type(&value.expr);
        self.visit_expr_mut(&mut value.expr);
        self.bind(&value.pat, ty);
        self.visit_pat_mut(&mut value.pat);
    }

    fn visit_expr_closure_mut(&mut self, value: &mut syn::ExprClosure) {
        let locals = self.locals.clone();
        for pattern in &value.inputs {
            self.bind(pattern, Ty::Unknown);
        }
        visit_mut::visit_expr_closure_mut(self, value);
        self.locals = locals;
    }

    fn visit_expr_method_call_mut(&mut self, call: &mut syn::ExprMethodCall) {
        let receiver = self.expression_type(&call.receiver);
        let closure_input = match call.method.to_string().as_str() {
            "map_err" => Some(receiver.argument(1)),
            "map" | "and_then" | "filter" | "for_each" => Some(receiver.inner()),
            _ => None,
        }
        .filter(|_| self.is_standard_container(&receiver));
        self.visit_expr_mut(&mut call.receiver);
        for argument in &mut call.args {
            if let (Some(ty), syn::Expr::Closure(closure)) = (&closure_input, &mut *argument) {
                let locals = self.locals.clone();
                for pattern in &closure.inputs {
                    self.bind(pattern, ty.clone());
                }
                self.visit_expr_mut(&mut closure.body);
                self.locals = locals;
            } else {
                self.visit_expr_mut(argument);
            }
        }
    }

    fn visit_expr_for_loop_mut(&mut self, value: &mut syn::ExprForLoop) {
        let element = self.expression_type(&value.expr).inner();
        self.visit_expr_mut(&mut value.expr);
        let locals = self.locals.clone();
        self.bind(&value.pat, element);
        self.visit_pat_mut(&mut value.pat);
        self.visit_block_mut(&mut value.body);
        self.locals = locals;
    }

    fn visit_expr_while_mut(&mut self, value: &mut syn::ExprWhile) {
        let locals = self.locals.clone();
        self.visit_expr_mut(&mut value.cond);
        self.visit_block_mut(&mut value.body);
        self.locals = locals;
    }

    fn visit_macro_mut(&mut self, value: &mut syn::Macro) {
        if let Ok(mut arguments) = value.parse_body_with(
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
        ) {
            for argument in &mut arguments {
                self.visit_expr_mut(argument);
            }
            value.tokens = arguments.into_token_stream();
        } else if value.path.is_ident("vec") {
            let tokens = &value.tokens;
            if let Ok(mut expression) = syn::parse2::<syn::ExprRepeat>(quote::quote!([#tokens])) {
                self.visit_expr_mut(&mut expression.expr);
                self.visit_expr_mut(&mut expression.len);
                let element = expression.expr;
                let length = expression.len;
                value.tokens = quote::quote!(#element; #length);
            }
        }
    }
}

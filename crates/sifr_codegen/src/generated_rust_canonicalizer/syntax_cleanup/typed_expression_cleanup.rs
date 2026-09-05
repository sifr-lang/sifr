//! Cleanup that requires the receiving type and lexical binding identity.
use std::collections::HashMap;

use quote::ToTokens;
use syn::visit_mut::{self, VisitMut};

#[derive(Clone)]
struct Callable {
    signature: syn::Signature,
}

pub(super) fn rewrite(file: &mut syn::File) {
    let facts = collect_project_facts(std::slice::from_ref(file));
    rewrite_with_facts(file, &facts);
}

pub(crate) struct ProjectTypeFacts {
    functions: HashMap<String, Callable>,
    structures: HashMap<String, syn::ItemStruct>,
}

pub(super) fn collect_project_facts(files: &[syn::File]) -> ProjectTypeFacts {
    let mut functions = HashMap::new();
    let mut structures = HashMap::new();
    for file in files {
        collect(&file.items, &[], &mut functions);
        collect_structures(&file.items, &[], &mut structures);
        exclude_drop_structures(&file.items, &[], &mut structures);
    }
    let combined = syn::File {
        frontmatter: None,
        shebang: None,
        attrs: Vec::new(),
        items: files
            .iter()
            .flat_map(|file| file.items.iter().cloned())
            .collect(),
    };
    super::scoped_imports::expand(&combined, &mut functions);
    super::scoped_imports::expand(&combined, &mut structures);
    ProjectTypeFacts {
        functions,
        structures,
    }
}

pub(super) fn rewrite_with_facts(file: &mut syn::File, facts: &ProjectTypeFacts) {
    Rewriter {
        functions: &facts.functions,
        structures: &facts.structures,
        self_type: None,
        scope: Vec::new(),
        module_depth: 0,
        bindings: HashMap::new(),
    }
    .visit_file_mut(file);
}

include!("typed_expression_facts.rs");
include!("typed_expression_types.rs");
include!("typed_iterator_facts.rs");

struct Rewriter<'facts> {
    functions: &'facts HashMap<String, Callable>,
    structures: &'facts HashMap<String, syn::ItemStruct>,
    self_type: Option<syn::Type>,
    scope: Vec<String>,
    module_depth: usize,
    // Unknown shadowing bindings have a None entry, never an outer type.
    bindings: HashMap<String, Option<syn::Type>>,
}

fn same_type(left: &syn::Type, right: &syn::Type) -> bool {
    left.to_token_stream().to_string() == right.to_token_stream().to_string()
}

fn named(ty: &syn::Type, name: &str) -> bool {
    matches!(ty, syn::Type::Path(path) if path.path.is_ident(name))
}

fn generic<'a>(ty: &'a syn::Type, name: &str) -> Option<&'a syn::Type> {
    let syn::Type::Path(path) = ty else {
        return None;
    };
    let segment = path.path.segments.last()?;
    if segment.ident != name {
        return None;
    }
    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };
    match args.args.first()? {
        syn::GenericArgument::Type(ty) => Some(ty),
        _ => None,
    }
}

fn unreference(ty: &syn::Type) -> &syn::Type {
    if let syn::Type::Reference(reference) = ty {
        unreference(&reference.elem)
    } else {
        ty
    }
}

fn callable_inputs(ty: &syn::Type) -> Option<Vec<syn::Type>> {
    match ty {
        syn::Type::Reference(reference) => callable_inputs(&reference.elem),
        syn::Type::FnPtr(function) => Some(
            function
                .inputs
                .iter()
                .map(|input| input.ty.clone())
                .collect(),
        ),
        syn::Type::ImplTrait(trait_) => bound_inputs(&trait_.bounds),
        syn::Type::TraitObject(trait_) => bound_inputs(&trait_.bounds),
        _ => generic(ty, "Box").and_then(callable_inputs),
    }
}

fn bound_inputs(
    bounds: &syn::punctuated::Punctuated<syn::TypeParamBound, syn::Token![+]>,
) -> Option<Vec<syn::Type>> {
    bounds.iter().find_map(|bound| {
        let syn::TypeParamBound::Trait(bound) = bound else {
            return None;
        };
        let segment = bound.path.segments.last()?;
        if !matches!(
            segment.ident.to_string().as_str(),
            "Fn" | "FnMut" | "FnOnce"
        ) {
            return None;
        }
        let syn::PathArguments::Parenthesized(arguments) = &segment.arguments else {
            return None;
        };
        Some(
            arguments
                .inputs
                .iter()
                .map(|input| input.ty.clone())
                .collect(),
        )
    })
}

impl Rewriter<'_> {
    fn resolve(&self, path: &syn::Path) -> Option<&Callable> {
        if path.leading_colon.is_some() {
            return None;
        }
        let parts = path
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect::<Vec<_>>();
        if parts.len() == 1 && self.bindings.contains_key(&parts[0]) {
            return None;
        }
        if parts.len() > 1 {
            let qualified =
                super::scoped_imports::qualified_path(&self.scope[..self.module_depth], &parts)?;
            return self.functions.get(&qualified.join("::"));
        }
        for depth in (self.module_depth..=self.scope.len()).rev() {
            let mut key = self.scope[..depth].to_vec();
            key.extend(parts.iter().cloned());
            if let Some(function) = self.functions.get(&key.join("::")) {
                return Some(function);
            }
        }
        None
    }

    fn bind(&mut self, pattern: &syn::Pat, ty: Option<syn::Type>) {
        match pattern {
            syn::Pat::Ident(binding) => {
                let ty = if binding.by_ref.is_some() {
                    ty.map(|ty| syn::parse_quote!(&#ty))
                } else {
                    ty
                };
                self.bindings.insert(binding.ident.to_string(), ty);
                if let Some((_, sub)) = &binding.subpat {
                    self.bind(sub, None);
                }
            }
            syn::Pat::Type(typed) => self.bind(&typed.pat, Some(*typed.ty.clone())),
            syn::Pat::Paren(paren) => self.bind(&paren.pat, ty),
            syn::Pat::Tuple(tuple) => {
                for (index, pattern) in tuple.elems.iter().enumerate() {
                    let element = ty.as_ref().and_then(|ty| match unreference(ty) {
                        syn::Type::Tuple(tuple) => tuple.elems.iter().nth(index).cloned(),
                        _ => None,
                    });
                    let element = element.map(|element| {
                        if matches!(ty, Some(syn::Type::Reference(_))) {
                            syn::parse_quote!(&#element)
                        } else {
                            element
                        }
                    });
                    self.bind(pattern, element);
                }
            }
            syn::Pat::Slice(pattern) => {
                let element = ty.as_ref().and_then(|ty| {
                    if let syn::Type::Slice(slice) = unreference(ty) {
                        Some(*slice.elem.clone())
                    } else {
                        None
                    }
                });
                for part in &pattern.elems {
                    let rest = matches!(part, syn::Pat::Ident(binding) if binding.subpat.as_ref().is_some_and(|(_, pat)| matches!(pat.as_ref(), syn::Pat::Rest(_))));
                    let part_ty = element
                        .clone()
                        .map(|element| {
                            if rest {
                                syn::parse_quote!([#element])
                            } else {
                                element
                            }
                        })
                        .map(|part| {
                            if matches!(ty, Some(syn::Type::Reference(_))) {
                                syn::parse_quote!(&#part)
                            } else {
                                part
                            }
                        });
                    self.bind(part, part_ty);
                }
            }
            syn::Pat::TupleStruct(tuple)
                if (tuple.path.is_ident("Some")
                    || tuple.path.is_ident("Ok")
                    || tuple.path.is_ident("Err"))
                    && tuple.elems.len() == 1 =>
            {
                let inner = ty
                    .as_ref()
                    .and_then(|ty| {
                        if tuple.path.is_ident("Some") {
                            generic(unreference(ty), "Option")
                        } else if tuple.path.is_ident("Ok") {
                            generic(unreference(ty), "Result")
                        } else {
                            result_error(unreference(ty))
                        }
                    })
                    .cloned();
                let inner = if matches!(ty, Some(syn::Type::Reference(_))) {
                    inner.map(|ty| syn::parse_quote!(&#ty))
                } else {
                    inner
                };
                self.bind(&tuple.elems[0], inner);
            }
            syn::Pat::TupleStruct(tuple) => {
                let fields = self
                    .resolve(&tuple.path)
                    .filter(|callable| {
                        let syn::ReturnType::Type(_, output) = &callable.signature.output else {
                            return false;
                        };
                        let Some(ty) = ty.as_ref() else { return false };
                        let syn::Type::Path(path) = unreference(ty) else {
                            return false;
                        };
                        let key = path.path.to_token_stream().to_string().replace(' ', "");
                        (0..=self.scope.len()).rev().any(|depth| {
                            let mut candidate = self.scope[..depth].to_vec();
                            candidate.push(key.clone());
                            output.to_token_stream().to_string().replace(' ', "")
                                == candidate.join("::")
                        })
                    })
                    .map(|callable| {
                        callable
                            .signature
                            .inputs
                            .iter()
                            .filter_map(|input| {
                                if let syn::FnArg::Typed(input) = input {
                                    Some(*input.ty.clone())
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>()
                    });
                for (index, part) in tuple.elems.iter().enumerate() {
                    let field = fields
                        .as_ref()
                        .and_then(|fields| fields.get(index))
                        .cloned()
                        .map(|field| {
                            if matches!(ty, Some(syn::Type::Reference(_))) {
                                syn::parse_quote!(&#field)
                            } else {
                                field
                            }
                        });
                    self.bind(part, field);
                }
            }
            _ => {
                for name in super::identifier_names_in_pattern(pattern) {
                    self.bindings.insert(name, None);
                }
            }
        }
    }

    fn expected(&mut self, expr: &mut syn::Expr, expected: &syn::Type) {
        if let syn::Expr::Block(block) = expr {
            self.expected_block(&mut block.block, expected);
            return;
        }
        if let syn::Expr::Closure(closure) = expr
            && let Some(inputs) = callable_inputs(expected)
            && inputs.len() == closure.inputs.len()
        {
            let outer = self.bindings.clone();
            for (pattern, ty) in closure.inputs.iter().zip(inputs) {
                self.bind(pattern, Some(ty));
            }
            self.visit_expr_mut(&mut closure.body);
            self.bindings = outer;
        }
        self.visit_expr_mut(expr);
        self.rewrite_vector_collect(expr, expected);
        if let syn::Expr::Call(call) = expr
            && matches!(call.func.as_ref(), syn::Expr::Path(path) if path.path.is_ident("Some"))
            && call.args.len() == 1
            && let Some(inner) = generic(expected, "Option")
        {
            self.expected(&mut call.args[0], inner);
        }
        let syn::Type::Reference(target) = expected else {
            return;
        };
        if target.mutability.is_some() {
            return;
        }
        if named(&target.elem, "str") && self.ty(expr).is_some_and(|ty| named(&ty, "String")) {
            if let syn::Expr::MethodCall(conversion) = expr
                && conversion.args.is_empty()
                && matches!(
                    conversion.method.to_string().as_str(),
                    "to_string" | "to_owned" | "clone"
                )
                && let Some(ty) = self.ty(&conversion.receiver)
            {
                if named(unreference(&ty), "str") {
                    *expr = *conversion.receiver.clone();
                    return;
                }
                if named(unreference(&ty), "String") {
                    let receiver = &conversion.receiver;
                    *expr = syn::parse_quote!(#receiver.as_str());
                    return;
                }
            }
            let value = expr.clone();
            *expr = syn::parse_quote!(#value.as_str());
            return;
        }
        let syn::Expr::Reference(reference) = expr else {
            return;
        };
        if reference.mutability.is_some() {
            return;
        }
        if self.ty(&reference.expr).is_some_and(|ty| {
            same_type(&ty, expected) || shared_vector_slice_coercion(&ty, expected)
        }) {
            *expr = *reference.expr.clone();
            return;
        }
        if named(&target.elem, "str")
            && let syn::Expr::MethodCall(conversion) = reference.expr.as_ref()
            && matches!(
                conversion.method.to_string().as_str(),
                "to_string" | "to_owned" | "clone"
            )
            && conversion.args.is_empty()
            && let Some(ty) = self.ty(&conversion.receiver)
        {
            if named(unreference(&ty), "str") {
                *expr = *conversion.receiver.clone();
            } else if named(unreference(&ty), "String") {
                let receiver = &conversion.receiver;
                *expr = syn::parse_quote!(#receiver.as_str());
            }
        }
    }

    fn condition(&mut self, expr: &mut syn::Expr) {
        if let syn::Expr::Binary(binary) = expr
            && matches!(binary.op, syn::BinOp::And(_))
        {
            self.condition(&mut binary.left);
            self.condition(&mut binary.right);
        } else if let syn::Expr::Let(let_) = expr {
            self.visit_expr_mut(&mut let_.expr);
            self.bind(&let_.pat, self.ty(&let_.expr));
        } else {
            self.visit_expr_mut(expr);
        }
    }
}

impl VisitMut for Rewriter<'_> {
    fn visit_item_impl_mut(&mut self, implementation: &mut syn::ItemImpl) {
        let previous = self.self_type.replace(*implementation.self_ty.clone());
        visit_mut::visit_item_impl_mut(self, implementation);
        self.self_type = previous;
    }
    fn visit_item_mod_mut(&mut self, module: &mut syn::ItemMod) {
        self.scope.push(module.ident.to_string());
        let previous = std::mem::replace(&mut self.module_depth, self.scope.len());
        visit_mut::visit_item_mod_mut(self, module);
        self.module_depth = previous;
        self.scope.pop();
    }

    fn visit_item_fn_mut(&mut self, function: &mut syn::ItemFn) {
        let outer = std::mem::take(&mut self.bindings);
        self.scope.push(function.sig.ident.to_string());
        for input in &function.sig.inputs {
            if let syn::FnArg::Typed(input) = input {
                self.bind(&input.pat, Some(*input.ty.clone()));
            }
        }
        self.visit_block_mut(&mut function.block);
        self.scope.pop();
        self.bindings = outer;
    }

    fn visit_impl_item_fn_mut(&mut self, function: &mut syn::ImplItemFn) {
        let outer = std::mem::take(&mut self.bindings);
        if let Some(ty) = &self.self_type
            && function.sig.receiver().is_some()
        {
            self.bindings
                .insert("self".to_string(), Some(syn::parse_quote!(&#ty)));
        }
        for input in &function.sig.inputs {
            if let syn::FnArg::Typed(input) = input {
                self.bind(&input.pat, Some(*input.ty.clone()));
            }
        }
        self.visit_block_mut(&mut function.block);
        self.bindings = outer;
    }

    fn visit_block_mut(&mut self, block: &mut syn::Block) {
        let outer = self.bindings.clone();
        let mut owned_locals = std::collections::HashSet::new();
        for index in 0..block.stmts.len() {
            let (processed, remaining) = block.stmts.split_at_mut(index + 1);
            let statement = &mut processed[index];
            if let syn::Stmt::Local(local) = statement {
                if let Some(init) = &mut local.init {
                    if let syn::Pat::Type(typed) = &local.pat {
                        self.expected(&mut init.expr, &typed.ty);
                    } else {
                        self.visit_expr_mut(&mut init.expr);
                    }
                    if let Some((_, diverge)) = &mut init.diverge {
                        self.visit_expr_mut(diverge);
                    }
                }
                let ty = local.init.as_ref().and_then(|init| self.ty(&init.expr));
                self.move_unused_string_copy(local, remaining, &owned_locals);
                self.bind(&local.pat, ty);
                for name in super::identifier_names_in_pattern(&local.pat) {
                    owned_locals.remove(&name);
                    if self.bindings.get(&name).is_some_and(|ty| {
                        ty.as_ref()
                            .is_some_and(|ty| !matches!(ty, syn::Type::Reference(_)))
                    }) {
                        owned_locals.insert(name);
                    }
                }
            } else {
                self.visit_stmt_mut(statement);
            }
        }
        self.bindings = outer;
    }

    fn visit_expr_if_mut(&mut self, branch: &mut syn::ExprIf) {
        let outer = self.bindings.clone();
        self.condition(&mut branch.cond);
        self.visit_block_mut(&mut branch.then_branch);
        if let syn::Expr::Let(let_) = branch.cond.as_ref()
            && let syn::Pat::TupleStruct(pattern) = let_.pat.as_ref()
            && pattern.path.is_ident("Some")
            && pattern.elems.len() == 1
            && let syn::Pat::Ident(binding) = &pattern.elems[0]
            && binding.by_ref.is_none()
            && let Some(Some(ty)) = self.bindings.get(&binding.ident.to_string())
            && !matches!(ty, syn::Type::Reference(_))
        {
            let name = &binding.ident;
            let signature = syn::parse_quote!(fn binding(#name: #ty));
            super::idiom_cleanup::remove_last_use_parameter_clones(
                &signature,
                &mut branch.then_branch,
            );
        }
        self.bindings = outer;
        if let Some((_, alternative)) = &mut branch.else_branch {
            self.visit_expr_mut(alternative);
        }
    }

    fn visit_expr_while_mut(&mut self, loop_: &mut syn::ExprWhile) {
        let outer = self.bindings.clone();
        self.condition(&mut loop_.cond);
        self.visit_block_mut(&mut loop_.body);
        self.bindings = outer;
    }

    fn visit_expr_match_mut(&mut self, match_: &mut syn::ExprMatch) {
        self.visit_expr_mut(&mut match_.expr);
        let ty = self.ty(&match_.expr);
        for arm in &mut match_.arms {
            let outer = self.bindings.clone();
            self.bind(&arm.pat, ty.clone());
            self.visit_expr_mut(&mut arm.body);
            self.bindings = outer;
        }
    }

    fn visit_expr_for_loop_mut(&mut self, loop_: &mut syn::ExprForLoop) {
        self.visit_expr_mut(&mut loop_.expr);
        let outer = self.bindings.clone();
        self.bind(&loop_.pat, self.iterator_element(&loop_.expr));
        self.visit_block_mut(&mut loop_.body);
        self.bindings = outer;
    }

    fn visit_expr_closure_mut(&mut self, closure: &mut syn::ExprClosure) {
        let outer = self.bindings.clone();
        for input in &closure.inputs {
            self.bind(input, None);
        }
        self.visit_expr_mut(&mut closure.body);
        self.bindings = outer;
    }

    fn visit_expr_call_mut(&mut self, call: &mut syn::ExprCall) {
        let runtime_inputs = if let syn::Expr::Path(path) = call.func.as_ref() {
            runtime_call_inputs(&path.path)
        } else {
            None
        };
        let signature = if let syn::Expr::Path(path) = call.func.as_ref() {
            self.resolve(&path.path).map(|f| f.signature.clone())
        } else {
            None
        };
        let local_inputs = self.ty(&call.func).as_ref().and_then(callable_inputs);
        self.visit_expr_mut(&mut call.func);
        for (index, argument) in call.args.iter_mut().enumerate() {
            if let Some(syn::FnArg::Typed(parameter)) = signature
                .as_ref()
                .and_then(|sig| sig.inputs.iter().nth(index))
            {
                self.expected(argument, &parameter.ty);
            } else if let Some(ty) = local_inputs
                .as_ref()
                .or(runtime_inputs.as_ref())
                .and_then(|inputs| inputs.get(index))
            {
                self.expected(argument, ty);
            } else {
                self.visit_expr_mut(argument);
            }
        }
        if matches!(call.func.as_ref(), syn::Expr::Path(path)
            if matches!(path.path.to_token_stream().to_string().replace(' ', "").as_str(),
                "::std::ops::Add::add" | "::std::ops::Sub::sub" | "::std::ops::Mul::mul" | "::std::ops::Neg::neg"))
        {
            for argument in &mut call.args {
                if let syn::Expr::Reference(reference) = argument
                    && self
                        .ty(&reference.expr)
                        .is_some_and(|ty| same_type(&ty, &syn::parse_quote!(&SifrInt)))
                {
                    *argument = *reference.expr.clone();
                }
            }
        }
    }

    fn visit_expr_method_call_mut(&mut self, call: &mut syn::ExprMethodCall) {
        self.visit_expr_mut(&mut call.receiver);
        let ty = self.ty(&call.receiver);
        let parameters = ty
            .as_ref()
            .and_then(|ty| self.declared_method(ty, &call.method))
            .filter(|method| method.signature.generics.params.is_empty())
            .map(|method| {
                method
                    .signature
                    .inputs
                    .iter()
                    .filter_map(|input| {
                        if let syn::FnArg::Typed(input) = input {
                            Some(*input.ty.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            });
        if ty
            .as_ref()
            .is_some_and(|ty| named(unreference(ty), "SifrInt"))
            && matches!(
                call.method.to_string().as_str(),
                "shl_known_valid" | "shr_known_valid"
            )
            && let syn::Expr::MethodCall(clone) = call.receiver.as_ref()
            && clone.method == "clone"
            && clone.args.is_empty()
        {
            call.receiver = clone.receiver.clone();
        }
        for (index, argument) in call.args.iter_mut().enumerate() {
            if let Some(expected) = parameters
                .as_ref()
                .and_then(|parameters| parameters.get(index))
            {
                self.expected(argument, expected);
            } else if call.method == "replace"
                && index < 2
                && ty.as_ref().is_some_and(|ty| {
                    named(unreference(ty), "String") || named(unreference(ty), "str")
                })
            {
                self.expected(argument, &syn::parse_quote!(&str));
            } else if call.method == "clone_from"
                && let Some(ty) = &ty
            {
                let base = unreference(ty);
                self.expected(argument, &syn::parse_quote!(&#base));
            } else if let Some(ty) = &ty
                && let Some(error) = result_error(unreference(ty))
                && matches!(
                    call.method.to_string().as_str(),
                    "unwrap_or_else" | "map_err"
                )
                && index == 0
                && let syn::Expr::Closure(closure) = argument
                && closure.inputs.len() == 1
            {
                let outer = self.bindings.clone();
                self.bind(&closure.inputs[0], Some(error.clone()));
                self.visit_expr_mut(&mut closure.body);
                self.bindings = outer;
            } else if let Some(ty) = &ty
                && let Some(inner) = generic(unreference(ty), "Option")
                && ((matches!(call.method.to_string().as_str(), "map" | "and_then") && index == 0)
                    || (matches!(call.method.to_string().as_str(), "map_or" | "map_or_else")
                        && index == 1))
                && let syn::Expr::Closure(closure) = argument
                && closure.inputs.len() == 1
            {
                let outer = self.bindings.clone();
                self.bind(&closure.inputs[0], Some(inner.clone()));
                self.visit_expr_mut(&mut closure.body);
                if named(inner, "String")
                    && let syn::Expr::MethodCall(clone) = closure.body.as_ref()
                    && clone.method == "clone"
                    && clone.args.is_empty()
                    && matches!((&closure.inputs[0], clone.receiver.as_ref()), (syn::Pat::Ident(binding), syn::Expr::Path(path)) if path.path.is_ident(&binding.ident))
                {
                    closure.body = clone.receiver.clone();
                }
                self.bindings = outer;
            } else if index == 0
                && ty.as_ref().is_some_and(|ty| {
                    let base = unreference(ty);
                    (named(base, "String")
                        && matches!(
                            call.method.to_string().as_str(),
                            "contains" | "starts_with" | "ends_with" | "find"
                        ))
                        || (matches!(
                            call.method.to_string().as_str(),
                            "get" | "contains_key" | "remove"
                        ) && generic(base, "HashMap").is_some_and(|ty| named(ty, "String")))
                        || (matches!(call.method.to_string().as_str(), "contains" | "remove")
                            && generic(base, "HashSet").is_some_and(|ty| named(ty, "String")))
                })
            {
                self.expected(argument, &syn::parse_quote!(&str));
            } else {
                self.visit_expr_mut(argument);
            }
        }
    }

    fn visit_expr_mut(&mut self, expression: &mut syn::Expr) {
        visit_mut::visit_expr_mut(self, expression);
        if let Some(ty) = self.ty(expression) {
            self.rewrite_vector_collect(expression, &ty);
        }
        if let syn::Expr::Closure(closure) = expression
            && closure.asyncness.is_none()
            && let syn::Expr::Call(call) = closure.body.as_ref()
            && let syn::Expr::Path(path) = call.func.as_ref()
            && let Some(function) = self.resolve(&path.path)
            && function.signature.generics.params.is_empty()
            && closure.inputs.len() == call.args.len()
            && closure.inputs.iter().zip(&call.args).all(|(input, arg)| {
                matches!((input, arg), (syn::Pat::Ident(binding), syn::Expr::Path(path))
                    if binding.by_ref.is_none() && binding.subpat.is_none() && path.path.is_ident(&binding.ident))
            })
        { *expression = *call.func.clone(); return; }
        let syn::Expr::MethodCall(call) = expression else {
            return;
        };
        if call.method == "unwrap_or_else"
            && call.args.len() == 1
            && let syn::Expr::Closure(fallback) = &call.args[0]
            && fallback.asyncness.is_none()
            && matches!(fallback.body.as_ref(), syn::Expr::Path(path) if path.qself.is_none() && path.path.is_ident("None"))
            && let Some(receiver) = self.ty(&call.receiver)
            && ((generic(unreference(&receiver), "Option").is_some() && fallback.inputs.is_empty())
                || (generic(unreference(&receiver), "Result").is_some()
                    && fallback.inputs.len() == 1
                    && matches!(
                        &fallback.inputs[0],
                        syn::Pat::Wild(_)
                            | syn::Pat::Ident(syn::PatIdent {
                                by_ref: None,
                                subpat: None,
                                ..
                            })
                    )))
            && let Some(value) = generic(unreference(&receiver), "Result")
                .or_else(|| generic(unreference(&receiver), "Option"))
            && generic(value, "Option").is_some()
        {
            call.method = syn::Ident::new("unwrap_or", call.method.span());
            call.args[0] = syn::parse_quote!(None);
        }
        if call.args.is_empty()
            && matches!(call.method.to_string().as_str(), "clone" | "as_str")
            && self
                .ty(&call.receiver)
                .is_some_and(|ty| same_type(&ty, &syn::parse_quote!(&str)))
        {
            *expression = *call.receiver.clone();
            return;
        }
        if call.args.is_empty()
            && matches!(call.method.to_string().as_str(), "is_empty" | "len")
            && let syn::Expr::MethodCall(view) = call.receiver.as_ref()
            && view.method == "as_str"
            && view.args.is_empty()
            && self
                .ty(&view.receiver)
                .is_some_and(|ty| named(unreference(&ty), "String"))
        {
            call.receiver = view.receiver.clone();
        }
        if call.method == "to_vec"
            && call.args.is_empty()
            && self
                .ty(&call.receiver)
                .is_some_and(|ty| generic(&ty, "Vec").is_some())
        {
            call.method = syn::Ident::new("clone", call.method.span());
        }
        if call.method == "unwrap_or_else"
            && call.args.len() == 1
            && let syn::Expr::MethodCall(map) = call.receiver.as_ref()
            && map.method == "map"
            && map.args.len() == 1
            && self
                .ty(&map.receiver)
                .is_some_and(|ty| generic(&ty, "Option").is_some())
        {
            let receiver = &map.receiver;
            let mapper = &map.args[0];
            let fallback = &call.args[0];
            *expression = syn::parse_quote!(#receiver.map_or_else(#fallback, #mapper));
            return;
        }
        if call.args.is_empty()
            && call.method == "clone"
            && matches!(
                call.receiver.as_ref(),
                syn::Expr::Call(_) | syn::Expr::MethodCall(_)
            )
            && self
                .ty(&call.receiver)
                .is_some_and(|ty| named(&ty, "SifrInt") || named(&ty, "String"))
        {
            *expression = *call.receiver.clone();
            return;
        }
        if call.args.is_empty()
            && matches!(call.method.to_string().as_str(), "to_owned" | "to_string")
            && self
                .ty(&call.receiver)
                .is_some_and(|ty| named(unreference(&ty), "String"))
        {
            call.method = syn::Ident::new("clone", call.method.span());
        }
    }

    fn visit_macro_mut(&mut self, macro_: &mut syn::Macro) {
        if let Ok(mut arguments) = macro_.parse_body_with(
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
        ) {
            for argument in &mut arguments {
                self.visit_expr_mut(argument);
            }
            if matches!(
                macro_.path.get_ident().map(ToString::to_string).as_deref(),
                Some("assert_eq" | "assert_ne")
            ) && arguments.len() >= 2
            {
                self.align_comparison_references(&mut arguments);
            }
            macro_.tokens = arguments.to_token_stream();
        }
    }
}

include!("typed_expression_tests.rs");

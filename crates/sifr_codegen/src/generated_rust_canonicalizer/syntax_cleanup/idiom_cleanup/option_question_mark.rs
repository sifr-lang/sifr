//! A let-else scrutinee supports match ergonomics; `?` requires an owned Option.
//! Rewrite only with lexical value-type evidence, never from a binding's name.

use std::collections::{HashMap, HashSet};
use syn::visit::{self, Visit};
use syn::visit_mut::{self, VisitMut};

mod types;
use types::{Types, Value, qualify};

pub(super) fn rewrite(file: &mut syn::File) {
    Rewriter {
        types: Types::collect(file),
        bindings: HashMap::new(),
        declarations: HashMap::new(),
        movable: HashSet::new(),
        pending_owned: HashSet::new(),
        clone_ambiguous: false,
        module: String::new(),
        owner: None,
        local_types: HashSet::new(),
        local_imports: false,
    }
    .visit_file_mut(file);
}

struct Rewriter {
    types: Types,
    declarations: HashMap<String, usize>,
    movable: HashSet<String>,
    pending_owned: HashSet<String>,
    clone_ambiguous: bool,
    bindings: HashMap<String, Value>,
    module: String,
    owner: Option<String>,
    local_types: HashSet<String>,
    local_imports: bool,
}

impl Rewriter {
    fn bind(&mut self, pattern: &syn::Pat, kind: &Value) {
        if let syn::Pat::TupleStruct(tuple) = pattern
            && tuple.path.is_ident("Some")
            && tuple.elems.len() == 1
            && let Value::Option(inner) = kind.dereferenced()
        {
            let inner = if matches!(kind, Value::Reference(_)) {
                Value::Reference(inner.clone())
            } else {
                *inner.clone()
            };
            self.bind(&tuple.elems[0], &inner);
            return;
        }

        struct Names(Vec<String>);
        impl<'ast> Visit<'ast> for Names {
            fn visit_pat_ident(&mut self, binding: &'ast syn::PatIdent) {
                self.0.push(binding.ident.to_string());
                visit::visit_pat_ident(self, binding);
            }
        }
        let mut names = Names(Vec::new());
        names.visit_pat(pattern);
        let direct = match pattern {
            syn::Pat::Type(typed) => typed.pat.as_ref(),
            pattern => pattern,
        };
        let direct = matches!(direct, syn::Pat::Ident(binding) if binding.by_ref.is_none() && binding.subpat.is_none());
        for name in names.0 {
            let count = self.declarations.entry(name.clone()).or_default();
            *count += 1;
            if *count == 1
                && direct
                && matches!(
                    kind,
                    Value::Option(_) | Value::Map | Value::Sequence | Value::Scalar
                )
            {
                self.movable.insert(name.clone());
            } else {
                self.movable.remove(&name);
            }
            self.bindings
                .insert(name, if direct { kind.clone() } else { Value::Unknown });
        }
    }

    fn type_kind(&self, ty: &syn::Type) -> Value {
        match ty {
            syn::Type::Reference(reference) => {
                return Value::Reference(Box::new(self.type_kind(&reference.elem)));
            }
            syn::Type::Paren(paren) => return self.type_kind(&paren.elem),
            syn::Type::Group(group) => return self.type_kind(&group.elem),
            syn::Type::Path(path)
                if path.path.leading_colon.is_none()
                    && (self.local_imports
                        || path
                            .path
                            .segments
                            .first()
                            .is_some_and(|s| self.local_types.contains(&s.ident.to_string()))) =>
            {
                return Value::Unknown;
            }
            _ => {}
        }
        self.types.ty(&self.module, ty, self.owner.as_deref())
    }

    fn expression_kind(&self, expression: &syn::Expr) -> Value {
        match expression {
            syn::Expr::Path(path) if path.qself.is_none() && path.path.segments.len() == 1 => self
                .bindings
                .get(&path.path.segments[0].ident.to_string())
                .cloned()
                .unwrap_or_default(),
            syn::Expr::Paren(paren) => self.expression_kind(&paren.expr),
            syn::Expr::Group(group) => self.expression_kind(&group.expr),
            syn::Expr::Reference(reference) => {
                Value::Reference(Box::new(self.expression_kind(&reference.expr)))
            }
            syn::Expr::Field(field) => self
                .types
                .field(&self.expression_kind(&field.base), &field.member),
            // Standard Option views return owned Option<&T>/Option<&mut T>,
            // unlike borrowing the Option itself. Reject competing method names.
            syn::Expr::MethodCall(call) => {
                let method = call.method.to_string();
                if self.local_imports || self.types.ambiguous_methods.contains(&method) {
                    return Value::Unknown;
                }
                let receiver = self.expression_kind(&call.receiver);
                match method.as_str() {
                    "as_ref" | "as_mut"
                        if call.args.is_empty()
                            && matches!(receiver.dereferenced(), Value::Option(_)) =>
                    {
                        let Value::Option(inner) = receiver.dereferenced() else {
                            unreachable!()
                        };
                        Value::Option(Box::new(Value::Reference(inner.clone())))
                    }
                    // An owned Option's valid standard Clone call returns Self.
                    // A borrowed Option may instead clone the reference when T
                    // is not Clone, so it cannot supply the same ownership fact.
                    "clone" if call.args.is_empty() && matches!(receiver, Value::Option(_)) => {
                        receiver
                    }
                    "get" | "get_mut"
                        if call.args.len() == 1
                            && matches!(receiver.dereferenced(), Value::Map | Value::Sequence) =>
                    {
                        Value::Option(Box::new(Value::Unknown))
                    }
                    _ => Value::Unknown,
                }
            }
            _ => Value::Unknown,
        }
    }

    fn owned_condition_bindings(&self, condition: &syn::Expr) -> HashSet<String> {
        fn collect(condition: &syn::Expr, names: &mut HashSet<String>) {
            match condition {
                syn::Expr::Let(expression) => {
                    names.extend(super::super::identifier_names_in_pattern(&expression.pat))
                }
                syn::Expr::Binary(binary) if matches!(binary.op, syn::BinOp::And(_)) => {
                    collect(&binary.left, names);
                    collect(&binary.right, names);
                }
                syn::Expr::Paren(paren) => collect(&paren.expr, names),
                _ => {}
            }
        }
        let mut names = HashSet::new();
        collect(condition, &mut names);
        names.retain(|name| self.bindings.get(name).is_some_and(is_owned_clone_value));
        names
    }

    fn clean_owned_suffix(&self, statements: &mut [syn::Stmt], candidates: &HashSet<String>) {
        if self.local_imports || self.types.ambiguous_methods.contains("clone") {
            return;
        }
        let mut owned = candidates.clone();
        for statement in statements.iter() {
            if let syn::Stmt::Local(local) = statement {
                for name in super::super::identifier_names_in_pattern(&local.pat) {
                    owned.remove(&name);
                }
            }
        }
        super::clippy_cleanup::remove_proven_owned_clone_statements(statements, &owned);
    }

    fn function(&mut self, signature: &syn::Signature, body: &mut syn::Block) {
        let previous = std::mem::take(&mut self.bindings);
        let declarations = std::mem::take(&mut self.declarations);
        let movable = std::mem::take(&mut self.movable);
        let clone_ambiguous = std::mem::replace(&mut self.clone_ambiguous, false);
        for argument in &signature.inputs {
            if let syn::FnArg::Typed(argument) = argument {
                self.bind(&argument.pat, &self.type_kind(&argument.ty));
            } else if let syn::FnArg::Receiver(receiver) = argument {
                let value = self
                    .owner
                    .as_ref()
                    .map_or(Value::Unknown, |owner| Value::Nominal(owner.clone()));
                let value = if matches!(receiver.kind, syn::ReceiverKind::Reference(..)) {
                    Value::Reference(Box::new(value))
                } else {
                    value
                };
                self.bindings.insert("self".to_owned(), value);
            }
        }
        let pending = std::mem::replace(
            &mut self.pending_owned,
            self.bindings
                .iter()
                .filter(|(_, kind)| is_owned_clone_value(kind))
                .map(|(name, _)| name.clone())
                .collect(),
        );
        self.visit_block_mut(body);
        self.pending_owned = pending;
        if !self.types.ambiguous_methods.contains("clone") && !self.clone_ambiguous {
            super::clippy_cleanup::remove_proven_owned_clones(body, &self.movable);
        }
        self.declarations = declarations;
        self.movable = movable;
        self.clone_ambiguous = clone_ambiguous;
        self.bindings = previous;
    }
}

impl VisitMut for Rewriter {
    fn visit_item_mod_mut(&mut self, module: &mut syn::ItemMod) {
        let previous = self.module.clone();
        self.module = qualify(&previous, &module.ident.to_string());
        visit_mut::visit_item_mod_mut(self, module);
        self.module = previous;
    }

    fn visit_item_impl_mut(&mut self, item: &mut syn::ItemImpl) {
        let previous = self.owner.take();
        if let Value::Nominal(owner) = self.type_kind(&item.self_ty) {
            self.owner = Some(owner);
        }
        visit_mut::visit_item_impl_mut(self, item);
        self.owner = previous;
    }

    fn visit_item_fn_mut(&mut self, function: &mut syn::ItemFn) {
        self.function(&function.sig, &mut function.block);
    }

    fn visit_impl_item_fn_mut(&mut self, function: &mut syn::ImplItemFn) {
        self.function(&function.sig, &mut function.block);
    }

    fn visit_trait_item_fn_mut(&mut self, function: &mut syn::TraitItemFn) {
        if let Some(body) = &mut function.default {
            self.function(&function.sig, body);
        }
    }

    fn visit_block_mut(&mut self, block: &mut syn::Block) {
        let previous = self.bindings.clone();
        let previous_types = self.local_types.clone();
        let previous_imports = self.local_imports;
        for statement in &block.stmts {
            let syn::Stmt::Item(item) = statement else {
                continue;
            };
            let name = match item {
                syn::Item::Struct(item) => Some(&item.ident),
                syn::Item::Enum(item) => Some(&item.ident),
                syn::Item::Union(item) => Some(&item.ident),
                syn::Item::Type(item) => Some(&item.ident),
                syn::Item::Trait(item) => Some(&item.ident),
                syn::Item::Mod(item) => Some(&item.ident),
                _ => None,
            };
            if let Some(name) = name {
                self.local_types.insert(name.to_string());
            }
            if let syn::Item::Use(item) = item {
                struct Imports<'a>(&'a mut HashSet<String>);
                impl<'ast> Visit<'ast> for Imports<'_> {
                    fn visit_use_name(&mut self, name: &'ast syn::UseName) {
                        self.0.insert(name.ident.to_string());
                    }
                    fn visit_use_rename(&mut self, name: &'ast syn::UseRename) {
                        self.0.insert(name.rename.to_string());
                    }
                }
                self.local_imports = true;
                self.clone_ambiguous = true;
                Imports(&mut self.local_types).visit_use_tree(&item.tree);
                // Unknown block globs can introduce any type spelling.
                if matches!(&item.tree, syn::UseTree::Glob(_)) {
                    self.local_types.extend(
                        ["Option", "Vec", "String", "HashMap", "BTreeMap"].map(str::to_owned),
                    );
                }
            }
        }
        // Only inputs born in this block may move without enclosing liveness.
        let incoming_owned = std::mem::take(&mut self.pending_owned);
        self.clean_owned_suffix(&mut block.stmts, &incoming_owned);
        for index in 0..block.stmts.len() {
            self.visit_stmt_mut(&mut block.stmts[index]);
            if let syn::Stmt::Local(local) = &block.stmts[index] {
                let owned = super::super::identifier_names_in_pattern(&local.pat)
                    .into_iter()
                    .filter(|name| self.bindings.get(name).is_some_and(is_owned_clone_value))
                    .collect::<HashSet<_>>();
                self.clean_owned_suffix(&mut block.stmts[index + 1..], &owned);
            }
        }
        self.bindings = previous;
        self.local_types = previous_types;
        self.local_imports = previous_imports;
    }

    fn visit_local_mut(&mut self, local: &mut syn::Local) {
        visit_mut::visit_local_mut(self, local);
        if local
            .init
            .as_ref()
            .is_some_and(|init| matches!(self.expression_kind(&init.expr), Value::Option(_)))
        {
            super::rewrite_option_let_else_with_question_mark(local);
        }
        let owned = match &local.pat {
            syn::Pat::Type(typed) => self.type_kind(&typed.ty),
            _ => local
                .init
                .as_ref()
                .map_or(Value::Unknown, |init| self.expression_kind(&init.expr)),
        };
        self.bind(&local.pat, &owned);
    }

    fn visit_expr_closure_mut(&mut self, closure: &mut syn::ExprClosure) {
        let previous = std::mem::take(&mut self.bindings);
        for pattern in &closure.inputs {
            let owned = match pattern {
                syn::Pat::Type(typed) => self.type_kind(&typed.ty),
                _ => Value::Unknown,
            };
            self.bind(pattern, &owned);
        }
        self.visit_expr_mut(&mut closure.body);
        self.bindings = previous;
    }

    fn visit_arm_mut(&mut self, arm: &mut syn::Arm) {
        let previous = self.bindings.clone();
        self.bind(&arm.pat, &Value::Unknown);
        visit_mut::visit_arm_mut(self, arm);
        self.bindings = previous;
    }

    fn visit_expr_for_loop_mut(&mut self, loop_: &mut syn::ExprForLoop) {
        self.visit_expr_mut(&mut loop_.expr);
        let previous = self.bindings.clone();
        self.bind(&loop_.pat, &Value::Unknown);
        self.visit_block_mut(&mut loop_.body);
        self.bindings = previous;
    }

    fn visit_expr_let_mut(&mut self, let_: &mut syn::ExprLet) {
        self.visit_expr_mut(&mut let_.expr);
        self.bind(&let_.pat, &self.expression_kind(&let_.expr));
    }

    fn visit_expr_if_mut(&mut self, if_: &mut syn::ExprIf) {
        let previous = self.bindings.clone();
        self.visit_expr_mut(&mut if_.cond);
        self.pending_owned = self.owned_condition_bindings(&if_.cond);
        self.visit_block_mut(&mut if_.then_branch);
        self.bindings = previous.clone();
        if let Some((_, branch)) = &mut if_.else_branch {
            self.visit_expr_mut(branch);
        }
        self.bindings = previous;
    }

    fn visit_expr_while_mut(&mut self, while_: &mut syn::ExprWhile) {
        let previous = self.bindings.clone();
        self.visit_expr_mut(&mut while_.cond);
        self.visit_block_mut(&mut while_.body);
        self.bindings = previous;
    }
}

fn is_owned_clone_value(kind: &Value) -> bool {
    matches!(
        kind,
        Value::Option(_) | Value::Map | Value::Sequence | Value::Scalar
    )
}

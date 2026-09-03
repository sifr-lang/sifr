use std::collections::{HashMap, HashSet};
use syn::visit::{self, Visit};

pub(super) fn format_capture_names(rust_macro: &syn::Macro) -> HashSet<String> {
    let Some(name) = rust_macro.path.segments.last() else {
        return HashSet::new();
    };
    let format_index = match name.ident.to_string().as_str() {
        "format" | "print" | "println" | "eprint" | "eprintln" => 0,
        "assert" | "write" | "writeln" => 1,
        "assert_eq" | "assert_ne" => 2,
        _ => return HashSet::new(),
    };
    let Ok(arguments) = rust_macro.parse_body_with(
        syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
    ) else {
        return HashSet::new();
    };
    let Some(syn::Expr::Lit(format_expression)) = arguments.iter().nth(format_index) else {
        return HashSet::new();
    };
    let syn::Lit::Str(format_literal) = &format_expression.lit else {
        return HashSet::new();
    };
    let format = format_literal.value();
    let bytes = format.as_bytes();
    let mut names = HashSet::new();
    let mut offset = 0;
    while offset < bytes.len() {
        let Some(relative_start) = format[offset..].find('{') else {
            break;
        };
        let start = offset + relative_start;
        if bytes.get(start + 1) == Some(&b'{') {
            offset = start + 2;
            continue;
        }
        let Some(relative_end) = format[start + 1..].find('}') else {
            break;
        };
        let end = start + 1 + relative_end;
        let name = format[start + 1..end].split(':').next().unwrap_or_default();
        if !name.is_empty()
            && name
                .chars()
                .all(|character| character == '_' || character.is_ascii_alphanumeric())
        {
            names.insert(name.to_string());
        }
        offset = end + 1;
    }
    names
}

pub(super) fn concrete_trait_impl_roots(items: &[syn::Item]) -> HashSet<String> {
    let implementations = items
        .iter()
        .filter_map(|item| {
            let syn::Item::Impl(implementation) = item else {
                return None;
            };
            Some((
                implementation
                    .trait_
                    .as_ref()?
                    .0
                    .segments
                    .last()?
                    .ident
                    .to_string(),
                type_owner_name(&implementation.self_ty)?,
            ))
        })
        .collect::<HashSet<_>>();
    if implementations.is_empty() {
        return HashSet::new();
    }

    let implemented_traits = implementations
        .iter()
        .map(|(trait_name, _)| trait_name.clone())
        .collect::<HashSet<_>>();
    let requirements = items
        .iter()
        .filter_map(|item| {
            let syn::Item::Fn(function) = item else {
                return None;
            };
            let bounds = function
                .sig
                .generics
                .type_params()
                .map(|parameter| {
                    let traits = parameter
                        .bounds
                        .iter()
                        .filter_map(|bound| {
                            let syn::TypeParamBound::Trait(bound) = bound else {
                                return None;
                            };
                            let name = bound.path.segments.last()?.ident.to_string();
                            implemented_traits.contains(&name).then_some(name)
                        })
                        .collect::<Vec<_>>();
                    (parameter.ident.to_string(), traits)
                })
                .collect::<HashMap<_, _>>();
            let parameters = function
                .sig
                .inputs
                .iter()
                .enumerate()
                .filter_map(|(index, argument)| {
                    let syn::FnArg::Typed(argument) = argument else {
                        return None;
                    };
                    let type_parameter = direct_type_parameter(&argument.ty)?;
                    let traits = bounds.get(&type_parameter)?;
                    (!traits.is_empty()).then_some(
                        traits
                            .iter()
                            .cloned()
                            .map(|trait_name| (index, trait_name))
                            .collect::<Vec<_>>(),
                    )
                })
                .flatten()
                .collect::<Vec<_>>();
            (!parameters.is_empty()).then_some((function.sig.ident.to_string(), parameters))
        })
        .collect::<HashMap<_, _>>();

    let return_owners = items
        .iter()
        .filter_map(|item| {
            let syn::Item::Fn(function) = item else {
                return None;
            };
            let syn::ReturnType::Type(_, ty) = &function.sig.output else {
                return None;
            };
            Some((function.sig.ident.to_string(), type_owner_name(ty)?))
        })
        .collect::<HashMap<_, _>>();
    let mut collector = GenericCallDemandCollector {
        requirements: &requirements,
        return_owners: &return_owners,
        implementations: &implementations,
        bindings: HashMap::new(),
        roots: HashSet::new(),
    };
    for item in items {
        collector.visit_item(item);
    }
    collector.roots
}

fn direct_type_parameter(ty: &syn::Type) -> Option<String> {
    match ty {
        syn::Type::Reference(reference) => direct_type_parameter(&reference.elem),
        syn::Type::Group(group) => direct_type_parameter(&group.elem),
        syn::Type::Paren(paren) => direct_type_parameter(&paren.elem),
        syn::Type::Path(path) if path.qself.is_none() && path.path.segments.len() == 1 => {
            Some(path.path.segments[0].ident.to_string())
        }
        _ => None,
    }
}

fn type_owner_name(ty: &syn::Type) -> Option<String> {
    match ty {
        syn::Type::Reference(reference) => type_owner_name(&reference.elem),
        syn::Type::Group(group) => type_owner_name(&group.elem),
        syn::Type::Paren(paren) => type_owner_name(&paren.elem),
        syn::Type::Path(path) => path
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string()),
        _ => None,
    }
}

struct GenericCallDemandCollector<'scope> {
    requirements: &'scope HashMap<String, Vec<(usize, String)>>,
    return_owners: &'scope HashMap<String, String>,
    implementations: &'scope HashSet<(String, String)>,
    bindings: HashMap<String, String>,
    roots: HashSet<String>,
}

impl GenericCallDemandCollector<'_> {
    fn expression_owner(&self, expression: &syn::Expr) -> Option<String> {
        match expression {
            syn::Expr::Reference(reference) => self.expression_owner(&reference.expr),
            syn::Expr::Group(group) => self.expression_owner(&group.expr),
            syn::Expr::Paren(paren) => self.expression_owner(&paren.expr),
            syn::Expr::Path(path) if path.qself.is_none() && path.path.segments.len() == 1 => self
                .bindings
                .get(&path.path.segments[0].ident.to_string())
                .cloned(),
            syn::Expr::MethodCall(call) if call.method == "to_string" => Some("String".to_string()),
            syn::Expr::MethodCall(call) if call.method == "clone" => {
                self.expression_owner(&call.receiver)
            }
            syn::Expr::Call(call) => {
                let syn::Expr::Path(path) = call.func.as_ref() else {
                    return None;
                };
                let segments = path.path.segments.iter().collect::<Vec<_>>();
                if segments.len() > 1 {
                    return Some(segments[segments.len() - 2].ident.to_string());
                }
                self.return_owners
                    .get(&segments.first()?.ident.to_string())
                    .cloned()
            }
            syn::Expr::Struct(struct_) => struct_
                .path
                .segments
                .last()
                .map(|segment| segment.ident.to_string()),
            syn::Expr::Cast(cast) => type_owner_name(&cast.ty),
            syn::Expr::Lit(literal) if matches!(literal.lit, syn::Lit::Float(_)) => {
                Some("f64".to_string())
            }
            syn::Expr::Macro(expression_macro)
                if expression_macro
                    .mac
                    .path
                    .segments
                    .last()
                    .is_some_and(|segment| segment.ident == "format") =>
            {
                Some("String".to_string())
            }
            _ => None,
        }
    }
}

impl<'ast> Visit<'ast> for GenericCallDemandCollector<'_> {
    fn visit_macro(&mut self, rust_macro: &'ast syn::Macro) {
        let Ok(arguments) = rust_macro.parse_body_with(
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
        ) else {
            return;
        };
        for argument in &arguments {
            self.visit_expr(argument);
        }
    }

    fn visit_item_fn(&mut self, function: &'ast syn::ItemFn) {
        let saved = std::mem::take(&mut self.bindings);
        for argument in &function.sig.inputs {
            if let syn::FnArg::Typed(argument) = argument
                && let syn::Pat::Ident(pattern) = argument.pat.as_ref()
                && let Some(owner) = type_owner_name(&argument.ty)
            {
                self.bindings.insert(pattern.ident.to_string(), owner);
            }
        }
        visit::visit_item_fn(self, function);
        self.bindings = saved;
    }

    fn visit_local(&mut self, local: &'ast syn::Local) {
        visit::visit_local(self, local);
        let (name, explicit_owner) = match &local.pat {
            syn::Pat::Ident(pattern) => (pattern.ident.to_string(), None),
            syn::Pat::Type(typed) => {
                let syn::Pat::Ident(pattern) = typed.pat.as_ref() else {
                    return;
                };
                (pattern.ident.to_string(), type_owner_name(&typed.ty))
            }
            _ => return,
        };
        let inferred = local
            .init
            .as_ref()
            .and_then(|initializer| self.expression_owner(&initializer.expr));
        if let Some(owner) = explicit_owner.or(inferred) {
            self.bindings.insert(name, owner);
        }
    }

    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        if let syn::Expr::Path(path) = call.func.as_ref()
            && let Some(function) = path.path.segments.last()
            && let Some(requirements) = self.requirements.get(&function.ident.to_string())
        {
            for (index, trait_name) in requirements {
                let Some(owner) = call
                    .args
                    .get(*index)
                    .and_then(|argument| self.expression_owner(argument))
                else {
                    continue;
                };
                if self
                    .implementations
                    .contains(&(trait_name.clone(), owner.clone()))
                {
                    self.roots.insert(owner);
                }
            }
        }
        visit::visit_expr_call(self, call);
    }
}

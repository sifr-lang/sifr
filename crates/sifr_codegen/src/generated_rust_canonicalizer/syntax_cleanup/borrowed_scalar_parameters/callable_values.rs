#[derive(Clone, Debug)]
struct CallableValueUse {
    modules: Vec<String>,
    functions: Vec<String>,
    owner: Option<String>,
    path: Vec<String>,
}

fn collect_callable_value_uses(files: &[syn::File]) -> Vec<CallableValueUse> {
    let mut collector = CallableValueCollector {
        uses: Vec::new(),
        modules: Vec::new(),
        functions: Vec::new(),
        owner: None,
    };
    for file in files {
        collector.visit_file(file);
    }
    collector.uses
}

struct CallableValueCollector {
    uses: Vec<CallableValueUse>,
    modules: Vec<String>,
    functions: Vec<String>,
    owner: Option<String>,
}

impl Visit<'_> for CallableValueCollector {
    fn visit_item_mod(&mut self, module: &syn::ItemMod) {
        let Some((_, items)) = &module.content else {
            return;
        };
        self.modules.push(module.ident.to_string());
        for item in items {
            self.visit_item(item);
        }
        self.modules.pop();
    }

    fn visit_item_impl(&mut self, implementation: &syn::ItemImpl) {
        let previous = self.owner.replace(type_owner_name(&implementation.self_ty));
        visit::visit_item_impl(self, implementation);
        self.owner = previous;
    }

    fn visit_item_fn(&mut self, function: &syn::ItemFn) {
        self.functions.push(function.sig.ident.to_string());
        self.visit_block(&function.block);
        self.functions.pop();
    }

    fn visit_impl_item_fn(&mut self, function: &syn::ImplItemFn) {
        self.functions.push(function.sig.ident.to_string());
        self.visit_block(&function.block);
        self.functions.pop();
    }

    fn visit_expr_call(&mut self, call: &syn::ExprCall) {
        if !matches!(call.func.as_ref(), syn::Expr::Path(_)) {
            self.visit_expr(&call.func);
        }
        for argument in &call.args {
            self.visit_expr(argument);
        }
    }

    fn visit_expr_path(&mut self, path: &syn::ExprPath) {
        if path.qself.is_some() {
            return;
        }
        self.uses.push(CallableValueUse {
            modules: self.modules.clone(),
            functions: self.functions.clone(),
            owner: self.owner.clone(),
            path: path
                .path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect(),
        });
    }

    fn visit_macro(&mut self, rust_macro: &syn::Macro) {
        if let Ok(arguments) = rust_macro.parse_body_with(
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
        ) {
            for argument in &arguments {
                self.visit_expr(argument);
            }
        }
    }
}

fn callable_value_plan_keys(
    plans: &HashMap<String, ScalarBorrowPlan>,
    uses: &[CallableValueUse],
) -> HashSet<String> {
    let mut preserved = HashSet::new();
    for callable in uses {
        let exact = callable_value_candidates(callable);
        let exact_matches = exact
            .iter()
            .flat_map(|candidate| matching_plan_keys(plans, candidate))
            .collect::<HashSet<_>>();
        preserved.extend(exact_matches);
    }
    preserved
}

fn callable_value_candidates(callable: &CallableValueUse) -> Vec<String> {
    let mut candidates = Vec::new();
    if callable.path.len() == 1 {
        for depth in (0..=callable.functions.len()).rev() {
            let mut path = callable.modules.clone();
            path.extend(callable.functions[..depth].iter().cloned());
            path.extend(callable.path.iter().cloned());
            candidates.push(format!("function:{}", path.join("::")));
        }
    } else {
        let mut relative = callable.path.clone();
        let mut base = callable.modules.clone();
        match relative.first().map(String::as_str) {
            Some("crate") => {
                relative.remove(0);
                base.clear();
            }
            Some("self") => {
                relative.remove(0);
            }
            Some("super") => {
                while relative.first().is_some_and(|part| part == "super") {
                    relative.remove(0);
                    base.pop();
                }
            }
            Some("Self") => {
                relative.remove(0);
                if let Some(owner) = &callable.owner {
                    base.push(owner.clone());
                    base.extend(relative);
                    candidates.push(format!("method:{}", base.join("::")));
                    return candidates;
                }
            }
            _ => {}
        }
        let mut local = base;
        local.extend(relative.iter().cloned());
        candidates.push(format!("function:{}", local.join("::")));
        candidates.push(format!("method:{}", local.join("::")));
    }
    candidates
}

fn matching_plan_keys(plans: &HashMap<String, ScalarBorrowPlan>, identity: &str) -> Vec<String> {
    let prefix = format!("{identity}#");
    plans
        .keys()
        .filter(|key| key.starts_with(&prefix))
        .filter_map(|key| plans.get(key).map(|plan| plan.identity.clone()))
        .collect()
}

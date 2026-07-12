use sifr_python_ast::visitor::{walk_expr, walk_stmt, Visitor};
use sifr_python_ast::{Expr, Stmt};
use std::collections::BTreeSet;

#[derive(Clone, Debug)]
pub(super) enum RawImport {
    Absolute(String),
    From {
        level: u32,
        module: Option<String>,
        names: Vec<String>,
    },
}

impl RawImport {
    pub(super) fn display(&self) -> String {
        match self {
            Self::Absolute(module) => format!("import {module}"),
            Self::From {
                level,
                module,
                names,
            } => format!(
                "from {}{} import {}",
                ".".repeat(*level as usize),
                module.as_deref().unwrap_or_default(),
                names.join(", ")
            ),
        }
    }
}

pub(super) struct CollectedImports {
    pub(super) raw_imports: Vec<RawImport>,
    pub(super) dynamic_calls: BTreeSet<String>,
    pub(super) reserved_imports: BTreeSet<String>,
}

pub(super) fn collect_imports(suite: &[Stmt]) -> CollectedImports {
    let mut collector = ImportCollector::default();
    collector.visit_body(suite);
    let mut dynamic = DynamicImportVisitor::new(&collector);
    dynamic.visit_body(suite);
    CollectedImports {
        raw_imports: collector.imports,
        dynamic_calls: dynamic.calls,
        reserved_imports: collector.reserved_imports,
    }
}

#[derive(Default)]
struct ImportCollector {
    imports: Vec<RawImport>,
    importlib_aliases: BTreeSet<String>,
    builtins_aliases: BTreeSet<String>,
    dynamic_function_aliases: BTreeSet<String>,
    reserved_imports: BTreeSet<String>,
}

impl<'a> Visitor<'a> for ImportCollector {
    fn visit_stmt(&mut self, stmt: &'a Stmt) {
        match stmt {
            Stmt::Import(import) => {
                for alias in &import.names {
                    let name = alias.name.as_str().to_string();
                    let bound = alias.asname.as_ref().map_or_else(
                        || name.split('.').next().unwrap_or_default().to_string(),
                        |value| value.as_str().to_string(),
                    );
                    if name == "importlib" {
                        self.importlib_aliases.insert(bound.clone());
                    }
                    if name == "builtins" {
                        self.builtins_aliases.insert(bound);
                    }
                    if name == "__sifr_bridge__" || name.starts_with("__sifr_bridge__.") {
                        self.reserved_imports.insert(name.clone());
                    }
                    self.imports.push(RawImport::Absolute(name));
                }
            }
            Stmt::ImportFrom(import) => {
                let module = import.module.as_ref().map(ToString::to_string);
                self.record_imported_dynamic_aliases(import, module.as_deref());
                if module.as_deref().is_some_and(|name| {
                    name == "__sifr_bridge__" || name.starts_with("__sifr_bridge__.")
                }) {
                    self.reserved_imports
                        .insert(module.clone().unwrap_or_default());
                }
                self.imports.push(RawImport::From {
                    level: import.level,
                    module,
                    names: import
                        .names
                        .iter()
                        .map(|alias| alias.name.as_str().to_string())
                        .collect(),
                });
            }
            Stmt::Assign(assign) => {
                for target in &assign.targets {
                    self.record_assignment(target, &assign.value);
                }
            }
            Stmt::AnnAssign(assign) => {
                if let Some(value) = &assign.value {
                    self.record_assignment(&assign.target, value);
                }
            }
            _ => {}
        }
        walk_stmt(self, stmt);
    }

    fn visit_expr(&mut self, expr: &'a Expr) {
        if let Expr::Named(named) = expr {
            self.record_assignment(&named.target, &named.value);
        }
        walk_expr(self, expr);
    }
}

impl ImportCollector {
    fn record_imported_dynamic_aliases(
        &mut self,
        import: &sifr_python_ast::StmtImportFrom,
        module: Option<&str>,
    ) {
        let expected = match module {
            Some("importlib") => "import_module",
            Some("builtins") => "__import__",
            _ => return,
        };
        for alias in &import.names {
            if alias.name.as_str() == expected {
                self.dynamic_function_aliases.insert(
                    alias
                        .asname
                        .as_ref()
                        .map_or(alias.name.as_str(), |value| value.as_str())
                        .to_string(),
                );
            }
        }
    }

    fn record_assignment(&mut self, target: &Expr, value: &Expr) {
        if let Some(targets) = sequence_elements(target) {
            if let Some(values) = sequence_elements(value) {
                for (target, value) in targets.iter().zip(values) {
                    self.record_assignment(target, value);
                }
            }
            return;
        }
        if let Expr::Name(target) = target {
            let Some(value_name) = qualified_name(value) else {
                return;
            };
            let target = target.id.as_str().to_string();
            if self.importlib_aliases.contains(&value_name) {
                self.importlib_aliases.insert(target.clone());
            }
            if self.builtins_aliases.contains(&value_name) {
                self.builtins_aliases.insert(target.clone());
            }
            if self.dynamic_function_aliases.contains(&value_name)
                || value_name.rsplit_once('.').is_some_and(|(prefix, member)| {
                    (member == "import_module" && self.importlib_aliases.contains(prefix))
                        || (member == "__import__" && self.builtins_aliases.contains(prefix))
                })
            {
                self.dynamic_function_aliases.insert(target);
            }
        }
    }
}

fn sequence_elements(expr: &Expr) -> Option<&[Expr]> {
    match expr {
        Expr::Tuple(tuple) => Some(&tuple.elts),
        Expr::List(list) => Some(&list.elts),
        _ => None,
    }
}

struct DynamicImportVisitor {
    importlib_aliases: BTreeSet<String>,
    builtins_aliases: BTreeSet<String>,
    dynamic_function_aliases: BTreeSet<String>,
    calls: BTreeSet<String>,
}

impl DynamicImportVisitor {
    fn new(imports: &ImportCollector) -> Self {
        let mut importlib_aliases = imports.importlib_aliases.clone();
        importlib_aliases.insert("importlib".to_string());
        let mut builtins_aliases = imports.builtins_aliases.clone();
        builtins_aliases.insert("builtins".to_string());
        builtins_aliases.insert("__builtins__".to_string());
        let mut dynamic_function_aliases = imports.dynamic_function_aliases.clone();
        dynamic_function_aliases.insert("__import__".to_string());
        Self {
            importlib_aliases,
            builtins_aliases,
            dynamic_function_aliases,
            calls: BTreeSet::new(),
        }
    }

    fn dynamic_callable_name(&self, expr: &Expr) -> Option<String> {
        if let Some(name) = qualified_name(expr) {
            let dynamic = self.dynamic_function_aliases.contains(&name)
                || name.rsplit_once('.').is_some_and(|(prefix, member)| {
                    (member == "import_module" && self.importlib_aliases.contains(prefix))
                        || (member == "__import__" && self.builtins_aliases.contains(prefix))
                });
            return dynamic.then_some(name);
        }
        let Expr::Call(call) = expr else {
            return None;
        };
        if qualified_name(&call.func).as_deref() != Some("getattr") || call.arguments.args.len() < 2
        {
            return None;
        }
        let owner = qualified_name(&call.arguments.args[0])?;
        let Expr::StringLiteral(member) = &call.arguments.args[1] else {
            return None;
        };
        let member = member.value.to_str();
        ((member == "import_module" && self.importlib_aliases.contains(&owner))
            || (member == "__import__" && self.builtins_aliases.contains(&owner)))
        .then(|| format!("getattr({owner}, {member})"))
    }
}

impl<'a> Visitor<'a> for DynamicImportVisitor {
    fn visit_expr(&mut self, expr: &'a Expr) {
        if let Expr::Call(call) = expr {
            if let Some(name) = self.dynamic_callable_name(&call.func) {
                self.calls.insert(name);
            }
        }
        walk_expr(self, expr);
    }
}

fn qualified_name(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Name(name) => Some(name.id.as_str().to_string()),
        Expr::Attribute(attribute) => qualified_name(&attribute.value)
            .map(|prefix| format!("{prefix}.{}", attribute.attr.as_str())),
        _ => None,
    }
}

//! Application demand over checked declarations, independent of emitted Rust.
//!
//! Classes are emission units: all methods, operators and lifecycle declarations
//! of a required class participate, including methods called only by generated
//! support. Imports inside a stdlib module are edges, not roots. The inventory
//! remains complete for bootstrap and whole-sysroot qualification.
mod types;

use crate::hir_analysis::traversal::walk_expr;
use crate::{RustInteropPlan, StdlibCode};
use sifr_ir::{
    CompilerIntrinsicId, HirClass, HirExpr, HirFunction, HirModule, RustInteropDeclaration,
    RustInteropValue,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Owner {
    Function(String),
    Class(String),
    Constant(String),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Declaration {
    module: String,
    owner: Owner,
}

pub(crate) fn select(stdlib: &StdlibCode, applications: &[&HirModule]) -> RustInteropPlan {
    let mut demand = Demand {
        stdlib,
        pending: BTreeSet::new(),
        selected: BTreeSet::new(),
    };
    for module in applications {
        // These imports are roots of the generated application support owner.
        // Unlike module/feature selection, each edge names a checked declaration.
        for import in &module.imports {
            for name in &import.names {
                demand.symbol(&import.module, name);
            }
        }
        demand.module_contents("", module);
    }
    while let Some(declaration) = demand.pending.pop_first() {
        if !demand.selected.insert(declaration.clone()) {
            continue;
        }
        let module = &stdlib.hir_modules[&declaration.module];
        match &declaration.owner {
            Owner::Function(name) => {
                if let Some(function) = module.functions.iter().find(|f| &f.name == name) {
                    demand.function(&declaration.module, module, function);
                }
            }
            Owner::Class(name) => {
                if let Some(class) = module.classes.iter().find(|c| &c.name == name) {
                    demand.class(&declaration.module, module, class);
                }
            }
            Owner::Constant(name) => {
                if let Some((_, ty, value)) = module.constants.iter().find(|(n, _, _)| n == name) {
                    demand.ty(&declaration.module, module, ty);
                    demand.expression(&declaration.module, module, value);
                }
            }
        }
    }

    // Rebuild contracts through the existing compiler authority, with all
    // selected nominal definitions available together. This includes generated
    // bridge layouts and structural identities; no signature is hand-filtered.
    let mut modules = BTreeMap::new();
    for declaration in &demand.selected {
        modules
            .entry(declaration.module.clone())
            .or_insert_with(|| {
                let mut module = (*stdlib.hir_modules[&declaration.module]).clone();
                let selected = |owner| {
                    demand.selected.contains(&Declaration {
                        module: declaration.module.clone(),
                        owner,
                    })
                };
                module
                    .functions
                    .retain(|f| selected(Owner::Function(f.name.clone())));
                module
                    .classes
                    .retain(|c| selected(Owner::Class(c.name.clone())));
                module
                    .constants
                    .retain(|(name, _, _)| selected(Owner::Constant(name.clone())));
                module
            });
    }
    crate::interop_build_plan_for_named_modules(
        modules
            .iter()
            .map(|(name, module)| (Some(name.as_str()), module)),
    )
    .rust
}

struct Demand<'a> {
    stdlib: &'a StdlibCode,
    pending: BTreeSet<Declaration>,
    selected: BTreeSet<Declaration>,
}

impl Demand<'_> {
    fn symbol(&mut self, module: &str, name: &str) {
        if let Some(declaration) = self.resolve(module, name, &mut BTreeSet::new()) {
            if !self.selected.contains(&declaration) {
                self.pending.insert(declaration);
            }
        }
    }

    fn resolve(
        &self,
        module: &str,
        name: &str,
        seen: &mut BTreeSet<(String, String)>,
    ) -> Option<Declaration> {
        if !seen.insert((module.to_string(), name.to_string())) {
            return None;
        }
        // Canonical type/callable identities can name a different declaring
        // module from the local alias. Prefer that exact identity.
        if let Some((owner_module, symbol)) = name.rsplit_once('.') {
            if self.stdlib.hir_modules.contains_key(owner_module) {
                return self.resolve(owner_module, symbol, seen);
            }
        }
        let hir = self.stdlib.hir_modules.get(module)?;
        let owner = if hir.functions.iter().any(|f| f.name == name) {
            Some(Owner::Function(name.to_string()))
        } else if hir.classes.iter().any(|c| c.name == name) {
            Some(Owner::Class(name.to_string()))
        } else if hir.constants.iter().any(|(n, _, _)| n == name) {
            Some(Owner::Constant(name.to_string()))
        } else {
            None
        };
        if let Some(owner) = owner {
            return Some(Declaration {
                module: module.to_string(),
                owner,
            });
        }
        for import in &hir.imports {
            for original in &import.names {
                let local = import
                    .aliases
                    .iter()
                    .find(|(n, _)| n == original)
                    .map_or(original, |(_, alias)| alias);
                // Private declaration calls retain the source declaration name
                // in typed HIR (private_import_function_sources), not its alias.
                if local == name || (import.module.starts_with("_sifr.") && original == name) {
                    return self.resolve(&import.module, original, seen);
                }
            }
        }
        None
    }

    fn local_symbol(&mut self, module_name: &str, module: &HirModule, name: &str) {
        // Application locals never resolve by a same-named stdlib declaration.
        if module_name.is_empty() {
            if let Some((owner, symbol)) = name.rsplit_once('.') {
                self.symbol(owner, symbol);
            }
            for import in &module.imports {
                for original in &import.names {
                    let local = import
                        .aliases
                        .iter()
                        .find(|(n, _)| n == original)
                        .map_or(original, |(_, alias)| alias);
                    if local == name {
                        self.symbol(&import.module, original);
                    }
                }
            }
        } else {
            self.symbol(module_name, name);
        }
    }

    fn module_contents(&mut self, name: &str, module: &HirModule) {
        for function in &module.functions {
            self.function(name, module, function);
        }
        for class in &module.classes {
            self.class(name, module, class);
        }
        for (_, ty, value) in &module.constants {
            self.ty(name, module, ty);
            self.expression(name, module, value);
        }
    }

    fn function(&mut self, name: &str, module: &HirModule, function: &HirFunction) {
        // Reuse the exhaustive IR visitor for statement-only type metadata as
        // well as expression types, defaults and nested callable bodies.
        let mut function = function.clone();
        sifr_ir::transform_hir_function_types(&mut function, &mut |ty| self.ty(name, module, ty));
        sifr_ir::visit_hir_function_exprs_mut(&mut function, &mut |expr| {
            self.expr_node(name, module, expr)
        });
        self.lifecycle(name, module, &function.rust_interop);
    }

    fn class(&mut self, name: &str, module: &HirModule, class: &HirClass) {
        if let Some(identity) = &class.identity {
            self.local_symbol(name, module, identity);
        }
        for (_, ty) in &class.fields {
            self.ty(name, module, ty);
        }
        if let Some(ty) = &class.parent_type {
            self.ty(name, module, ty);
        }
        if let Some(ty) = &class.newtype_inner {
            self.ty(name, module, ty);
        }
        for (_, value) in &class.field_defaults {
            self.expression(name, module, value);
        }
        for (_, identity) in &class.field_default_identities {
            self.local_symbol(name, module, identity);
        }
        for method in class
            .methods
            .iter()
            .chain(class.operator_impls.iter().map(|(_, method)| method))
        {
            self.function(name, module, method);
        }
        self.lifecycle(name, module, &class.rust_interop);
    }

    fn lifecycle(
        &mut self,
        name: &str,
        module: &HirModule,
        declarations: &[RustInteropDeclaration],
    ) {
        for declaration in declarations {
            for argument in &declaration.arguments {
                if argument.name.as_deref() == Some("close") {
                    if let RustInteropValue::Symbol(target) = &argument.value {
                        self.local_symbol(name, module, target);
                    }
                }
            }
        }
    }

    fn expression(&mut self, name: &str, module: &HirModule, expr: &HirExpr) {
        walk_expr(expr, &mut |expr| {
            self.ty(name, module, &expr.ty());
            self.expr_node(name, module, expr);
        });
    }

    fn expr_node(&mut self, name: &str, module: &HirModule, expr: &HirExpr) {
        match expr {
            HirExpr::Call { func, .. }
            | HirExpr::GenericCall { func, .. }
            | HirExpr::PythonCall { func, .. } => self.local_symbol(name, module, func),
            HirExpr::Name {
                name: symbol,
                binding_id: None,
                ..
            } => self.local_symbol(name, module, symbol),
            HirExpr::IntrinsicCall { intrinsic, .. } => {
                // Builtin open has no imported HIR declaration. Its generated
                // constructor and Drop support have explicit declaration owners.
                match intrinsic {
                    CompilerIntrinsicId::OpenBinary | CompilerIntrinsicId::OpenText => {
                        self.symbol("_sifr.fs", "_open_file");
                        self.symbol("_sifr.fs", "_file_close");
                        self.symbol("sifr.io", "FileHandle");
                        self.symbol("sifr.io", "BinaryFileHandle");
                        if *intrinsic == CompilerIntrinsicId::OpenText {
                            self.symbol("sifr.io", "TextFileHandle");
                        }
                    }
                    _ => {}
                }
                for (owner, hir) in &self.stdlib.hir_modules {
                    for function in &hir.functions {
                        if function.compiler_intrinsic == Some(*intrinsic) {
                            self.symbol(owner, &function.name);
                        }
                    }
                }
            }
            HirExpr::MethodCall {
                object,
                method,
                args,
                ..
            } if object.ty().is_python_object_contract() => {
                // These raw Object adapters are emitted directly by the
                // compiler, unlike ordinary class methods with checked bodies.
                match (method.as_str(), args.len()) {
                    ("get_attr", 1) => self.symbol("_sifr.python", "py_get_attr"),
                    ("get_item", 1) => self.symbol("_sifr.python", "py_get_item_str"),
                    _ => {}
                }
            }
            // Receiver/constructor/operator dependencies are selected from the
            // canonical class type, including its complete generated impl unit.
            _ => {}
        }
    }
}

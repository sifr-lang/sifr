//! Application demand over checked declarations, independent of emitted Rust.
//!
//! Classes are emission units: all methods, operators and lifecycle declarations
//! of a required class participate, including methods called only by generated
//! support. Imports inside a stdlib module are edges, not roots. The inventory
//! remains complete for bootstrap and whole-sysroot qualification.
mod observation;
mod types;
pub use observation::observe_stdlib_interop_selection;

use crate::{RustInteropPlan, StdlibCode};
use sifr_ir::{
    CompilerIntrinsicId, HirClass, HirExpr, HirFunction, HirModule, RustInteropDeclaration,
    RustInteropValue,
};
use std::collections::{BTreeMap, BTreeSet, HashMap};

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

pub(crate) fn application_plan(
    stdlib: &StdlibCode,
    modules: &[(Option<&str>, &HirModule)],
) -> crate::InteropBuildPlan {
    let mut plan = crate::interop_build_plan_for_named_modules(modules.iter().copied());
    plan.stdlib_demand = select(
        stdlib,
        &modules
            .iter()
            .map(|(_, module)| *module)
            .collect::<Vec<_>>(),
    );
    plan
}

pub(crate) fn select(stdlib: &StdlibCode, applications: &[&HirModule]) -> RustInteropPlan {
    observation::selected();
    let mut intrinsics: HashMap<CompilerIntrinsicId, Vec<(&str, &str)>> = HashMap::new();
    for (name, module) in stdlib.hir_modules.iter() {
        for function in &module.functions {
            if let Some(intrinsic) = function.compiler_intrinsic {
                intrinsics
                    .entry(intrinsic)
                    .or_default()
                    .push((name, &function.name));
            }
        }
    }
    let mut demand = Demand {
        stdlib,
        intrinsics,
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
    let mut selections: BTreeMap<&str, SelectedDeclarations<'_>> = BTreeMap::new();
    for declaration in &demand.selected {
        let selected = selections.entry(&declaration.module).or_default();
        match &declaration.owner {
            Owner::Function(name) => {
                selected.functions.insert(name);
            }
            Owner::Class(name) => {
                selected.classes.insert(name);
            }
            Owner::Constant(name) => {
                selected.constants.insert(name);
            }
        }
    }
    let modules: BTreeMap<&str, HirModule> = selections
        .into_iter()
        .map(|(name, selected)| {
            let source = &stdlib.hir_modules[name];
            // Source vector order is part of diagnostic and cache-key identity.
            // Membership selects declarations; it must not determine their order.
            let module = HirModule {
                functions: source
                    .functions
                    .iter()
                    .filter(|f| selected.functions.contains(f.name.as_str()))
                    .cloned()
                    .collect(),
                classes: source
                    .classes
                    .iter()
                    .filter(|c| selected.classes.contains(c.name.as_str()))
                    .cloned()
                    .collect(),
                constants: source
                    .constants
                    .iter()
                    .filter(|(n, _, _)| selected.constants.contains(n.as_str()))
                    .cloned()
                    .collect(),
                imports: source.imports.clone(),
                generic_functions: selected
                    .functions
                    .iter()
                    .filter_map(|name| {
                        source
                            .generic_functions
                            .get(*name)
                            .map(|params| ((*name).to_string(), params.clone()))
                    })
                    .collect(),
                type_param_bounds: selected
                    .functions
                    .iter()
                    .chain(&selected.classes)
                    .chain(&selected.constants)
                    .filter_map(|name| {
                        source
                            .type_param_bounds
                            .get(*name)
                            .map(|bounds| ((*name).to_string(), bounds.clone()))
                    })
                    .collect(),
            };
            (name, module)
        })
        .collect();
    crate::interop_build_plan_for_named_modules(
        modules.iter().map(|(name, module)| (Some(*name), module)),
    )
    .rust
}

#[derive(Default)]
struct SelectedDeclarations<'a> {
    functions: BTreeSet<&'a str>,
    classes: BTreeSet<&'a str>,
    constants: BTreeSet<&'a str>,
}

struct Demand<'a> {
    stdlib: &'a StdlibCode,
    intrinsics: HashMap<CompilerIntrinsicId, Vec<(&'a str, &'a str)>>,
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
        sifr_ir::visit_hir_function(function, &mut |node| self.node(name, module, node));
    }

    fn node(&mut self, name: &str, module: &HirModule, node: sifr_ir::HirNode<'_>) {
        match node {
            sifr_ir::HirNode::Type(ty) => self.ty(name, module, ty),
            sifr_ir::HirNode::Expr(expr) => self.expr_node(name, module, expr),
            sifr_ir::HirNode::Function(function) => {
                self.lifecycle(name, module, &function.rust_interop)
            }
        }
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
        sifr_ir::visit_hir_expr(expr, &mut |node| self.node(name, module, node));
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
                if let Some(owners) = self.intrinsics.get(intrinsic).cloned() {
                    for (owner, name) in owners {
                        self.symbol(owner, name);
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

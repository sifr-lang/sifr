use sifr_python_ast::{Expr, Stmt};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ModuleSignature {
    pub(super) imports: ImportSignature,
    pub(super) exports: ExportSignature,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ImportSignature {
    pub(super) entries: Vec<ImportSignatureEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct ImportSignatureEntry {
    module: String,
    level: u32,
    names: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ExportSignature {
    pub(super) entries: Vec<ExportSignatureEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct ExportSignatureEntry {
    kind: &'static str,
    name: String,
    shape: String,
}

pub(super) fn module_signature(stmts: &[Stmt]) -> ModuleSignature {
    ModuleSignature {
        imports: import_signature(stmts),
        exports: export_signature(stmts),
    }
}

pub(super) fn import_signature(stmts: &[Stmt]) -> ImportSignature {
    let mut entries = Vec::new();
    for stmt in stmts {
        let Stmt::ImportFrom(import_from) = stmt else {
            continue;
        };
        let Some(module) = &import_from.module else {
            continue;
        };
        let mut names = import_from
            .names
            .iter()
            .map(|alias| {
                let alias_name = alias
                    .asname
                    .as_ref()
                    .map_or_else(String::new, ToString::to_string);
                format!("{} as {alias_name}", alias.name)
            })
            .collect::<Vec<_>>();
        names.sort();
        entries.push(ImportSignatureEntry {
            module: module.to_string(),
            level: import_from.level,
            names,
        });
    }
    entries.sort();
    entries.dedup();
    ImportSignature { entries }
}

pub(super) fn export_signature(stmts: &[Stmt]) -> ExportSignature {
    let mut entries = Vec::new();
    for stmt in stmts {
        match stmt {
            Stmt::FunctionDef(function) if is_public(&function.name) => {
                entries.push(ExportSignatureEntry {
                    kind: "function",
                    name: function.name.to_string(),
                    shape: format!(
                        "{:?}|{:?}|{:?}|{:?}",
                        function.parameters,
                        function.returns,
                        function.decorator_list,
                        function.type_params
                    ),
                });
            }
            Stmt::ClassDef(class) if is_public(&class.name) => {
                let mut member_shapes = Vec::new();
                for member in &class.body {
                    match member {
                        Stmt::FunctionDef(function) if is_public(&function.name) => {
                            member_shapes.push(format!(
                                "method:{}:{:?}|{:?}|{:?}|{:?}",
                                function.name,
                                function.parameters,
                                function.returns,
                                function.decorator_list,
                                function.type_params
                            ));
                        }
                        Stmt::AnnAssign(assign) => {
                            if let Some(name) = public_target_name(assign.target.as_ref()) {
                                member_shapes.push(format!("field:{name}:{:?}", assign.annotation));
                            }
                        }
                        Stmt::Assign(assign) if assign.targets.len() == 1 => {
                            if let Some(name) = public_target_name(&assign.targets[0]) {
                                member_shapes.push(format!("field:{name}"));
                            }
                        }
                        _ => {}
                    }
                }
                member_shapes.sort();
                entries.push(ExportSignatureEntry {
                    kind: "class",
                    name: class.name.to_string(),
                    shape: format!(
                        "{:?}|{:?}|{}",
                        class.arguments,
                        class.type_params,
                        member_shapes.join(";")
                    ),
                });
            }
            Stmt::AnnAssign(assign) => {
                if let Some(name) = public_target_name(assign.target.as_ref()) {
                    entries.push(ExportSignatureEntry {
                        kind: "constant",
                        name,
                        shape: format!("{:?}", assign.annotation),
                    });
                }
            }
            Stmt::Assign(assign) if assign.targets.len() == 1 => {
                if let Some(name) = public_target_name(&assign.targets[0]) {
                    entries.push(ExportSignatureEntry {
                        kind: "constant",
                        name,
                        shape: String::new(),
                    });
                }
            }
            _ => {}
        }
    }
    entries.sort();
    entries.dedup();
    ExportSignature { entries }
}

fn public_target_name(target: &Expr) -> Option<String> {
    let Expr::Name(name) = target else {
        return None;
    };
    let name = name.id.to_string();
    is_public(&name).then_some(name)
}

fn is_public(name: &str) -> bool {
    !name.starts_with('_')
}

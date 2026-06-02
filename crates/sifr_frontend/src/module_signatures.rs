use sifr_python_ast::comparable::{
    ComparableArguments, ComparableDecorator, ComparableExpr, ComparableParameters,
    ComparableTypeParams,
};
use sifr_python_ast::{Decorator, Expr, Parameters, Stmt, TypeParams};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ModuleSignature {
    pub(super) imports: ImportSignature,
    pub(super) exports: ExportSignature,
}

impl ModuleSignature {
    pub(super) fn cache_key_input(&self) -> String {
        format!(
            "imports=[{}]|exports=[{}]",
            self.imports.cache_key_input(),
            self.exports.cache_key_input()
        )
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ImportSignature {
    pub(super) entries: Vec<ImportSignatureEntry>,
}

impl ImportSignature {
    fn cache_key_input(&self) -> String {
        self.entries
            .iter()
            .map(ImportSignatureEntry::cache_key_input)
            .collect::<Vec<_>>()
            .join(";")
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct ImportSignatureEntry {
    module: String,
    level: u32,
    names: Vec<String>,
}

impl ImportSignatureEntry {
    fn cache_key_input(&self) -> String {
        format!(
            "module={}|level={}|names={}",
            self.module,
            self.level,
            self.names.join(",")
        )
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ExportSignature {
    pub(super) entries: Vec<ExportSignatureEntry>,
}

impl ExportSignature {
    fn cache_key_input(&self) -> String {
        self.entries
            .iter()
            .map(ExportSignatureEntry::cache_key_input)
            .collect::<Vec<_>>()
            .join(";")
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct ExportSignatureEntry {
    kind: &'static str,
    name: String,
    shape: String,
}

impl ExportSignatureEntry {
    fn cache_key_input(&self) -> String {
        format!("kind={}|name={}|shape={}", self.kind, self.name, self.shape)
    }
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
                        comparable_parameters(&function.parameters),
                        comparable_optional_expr(function.returns.as_deref()),
                        comparable_decorators(&function.decorator_list),
                        comparable_optional_type_params(function.type_params.as_deref())
                    ),
                });
            }
            Stmt::ClassDef(class) if is_public(&class.name) => {
                let mut member_shapes = Vec::new();
                for member in &class.body {
                    match member {
                        Stmt::FunctionDef(function) => {
                            member_shapes.push(format!(
                                "method:{}:{:?}|{:?}|{:?}|{:?}",
                                function.name,
                                comparable_parameters(&function.parameters),
                                comparable_optional_expr(function.returns.as_deref()),
                                comparable_decorators(&function.decorator_list),
                                comparable_optional_type_params(function.type_params.as_deref())
                            ));
                        }
                        Stmt::AnnAssign(assign) => {
                            if let Some(name) = public_target_name(assign.target.as_ref()) {
                                member_shapes.push(format!(
                                    "field:{name}:{:?}",
                                    comparable_expr(&assign.annotation)
                                ));
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
                        "{:?}|{:?}|{:?}|{}",
                        comparable_optional_arguments(class.arguments.as_deref()),
                        comparable_decorators(&class.decorator_list),
                        comparable_optional_type_params(class.type_params.as_deref()),
                        member_shapes.join(";")
                    ),
                });
            }
            Stmt::AnnAssign(assign) => {
                if let Some(name) = public_target_name(assign.target.as_ref()) {
                    entries.push(ExportSignatureEntry {
                        kind: "constant",
                        name,
                        shape: format!(
                            "{:?}|{:?}",
                            comparable_expr(&assign.annotation),
                            comparable_optional_expr(assign.value.as_deref())
                        ),
                    });
                }
            }
            Stmt::Assign(assign) if assign.targets.len() == 1 => {
                if let Some(name) = public_target_name(&assign.targets[0]) {
                    entries.push(ExportSignatureEntry {
                        kind: "constant",
                        name,
                        shape: format!("{:?}", comparable_expr(&assign.value)),
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

fn comparable_parameters(parameters: &Parameters) -> ComparableParameters<'_> {
    parameters.into()
}

fn comparable_expr(expr: &Expr) -> ComparableExpr<'_> {
    expr.into()
}

fn comparable_optional_expr(expr: Option<&Expr>) -> Option<ComparableExpr<'_>> {
    expr.map(Into::into)
}

fn comparable_decorators(decorators: &[Decorator]) -> Vec<ComparableDecorator<'_>> {
    decorators.iter().map(Into::into).collect()
}

fn comparable_optional_arguments(
    arguments: Option<&sifr_python_ast::Arguments>,
) -> Option<ComparableArguments<'_>> {
    arguments.map(Into::into)
}

fn comparable_optional_type_params(
    type_params: Option<&TypeParams>,
) -> Option<ComparableTypeParams<'_>> {
    type_params.map(Into::into)
}

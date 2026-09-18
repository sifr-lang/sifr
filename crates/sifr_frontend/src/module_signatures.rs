use sifr_python_ast::comparable::{
    ComparableArguments, ComparableDecorator, ComparableExpr, ComparableParameters,
    ComparableTypeParams,
};
use sifr_python_ast::{Decorator, Expr, Parameters, Stmt, TypeParams};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ModuleSignature {
    pub(super) imports: ImportSignature,
    pub(super) exports: ExportSignature,
    pub(super) semantic_body: String,
}

impl ModuleSignature {
    pub(super) fn cache_key_input(&self) -> String {
        format!(
            "imports=[{}]|exports=[{}]|semantic=[{}]",
            self.imports.cache_key_input(),
            self.exports.cache_key_input(),
            self.semantic_body
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
        semantic_body: interface_projection(stmts),
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

/// Only this deliberately small, effect-free body class is erased. Everything
/// else, including private helpers, generic/const bodies, class field defaults,
/// decorators and unknown constructs, is part of the consumed interface proof.
/// Callers must also establish successful checking before retaining importers.
pub(crate) fn interface_projection(stmts: &[Stmt]) -> String {
    use sifr_python_ast::comparable::ComparableStmt;
    use std::hash::Hash;
    let mut hash = InterfaceHasher(sifr_identity::IdentityEncoder::new(
        "consumed-module-interface-v1",
    ));
    stmts.len().hash(&mut hash);
    for stmt in stmts {
        if let Stmt::FunctionDef(function) = stmt {
            if function.type_params.is_none()
                && function.decorator_list.is_empty()
                && !function.is_async
                && function.parameters.is_empty()
                && matches!(function.returns.as_deref(), Some(Expr::Name(name))
                if matches!(name.id.as_str(), "int" | "float" | "bool" | "str"))
                && matches!(function.body.as_slice(), [Stmt::Return(ret)]
                if ret.value.as_deref().is_some_and(pure_return))
            {
                (
                    0_u8,
                    function.name.as_str(),
                    comparable_parameters(&function.parameters),
                    comparable_optional_expr(function.returns.as_deref()),
                )
                    .hash(&mut hash);
                continue;
            }
        }
        (1_u8, ComparableStmt::from(stmt)).hash(&mut hash);
    }
    // Keep the full SHA-256 identity. No AST rendering enters resident state or
    // graph cache keys, whose size must not grow with implementation bodies.
    hash.0.finish()
}

/// Adapt the comparable AST's structural Hash stream to the existing identity
/// encoder. The compiler identity pins Rust/AST hash encoding for disk records.
struct InterfaceHasher(sifr_identity::IdentityEncoder);
impl std::hash::Hasher for InterfaceHasher {
    fn write(&mut self, bytes: &[u8]) {
        self.0.field("structural-hash-chunk", bytes);
    }

    fn finish(&self) -> u64 {
        // Required by Hasher, but interface_projection consumes the full digest.
        // Do not use this shortened value as persisted semantic authority.
        self.0
            .clone()
            .finish()
            .bytes()
            .take(16)
            .fold(0, |value, byte| {
                (value << 4)
                    | u64::from(if byte <= b'9' {
                        byte - b'0'
                    } else {
                        byte - b'a' + 10
                    })
            })
    }
}

fn pure_return(expr: &Expr) -> bool {
    match expr {
        Expr::NumberLiteral(_)
        | Expr::BooleanLiteral(_)
        | Expr::StringLiteral(_)
        | Expr::Name(_) => true,
        Expr::BinOp(value) => pure_return(&value.left) && pure_return(&value.right),
        Expr::UnaryOp(value) => pure_return(&value.operand),
        _ => false,
    }
}

#[cfg(test)]
mod dx14_tests {
    use super::*;

    #[test]
    fn dx14_semantic_interface_keys_have_bounded_size() {
        fn signature(body: &str) -> ModuleSignature {
            let source = format!("def value(x: int) -> int:\n{body}    return x\n");
            let parsed = sifr_syntax::parse_module(&source, Some("fixture")).unwrap();
            module_signature(parsed.suite())
        }
        let small = signature("    x = 1\n");
        let large = signature(&"    x = x + 1\n".repeat(2000));
        let changed = signature(&"    x = x + 2\n".repeat(2000));
        assert_eq!(small.semantic_body.len(), 64);
        assert_eq!(large.semantic_body.len(), 64);
        assert_eq!(small.cache_key_input().len(), large.cache_key_input().len());
        assert_ne!(large.semantic_body, changed.semantic_body);
        assert_eq!(large, signature(&"    x = x + 1\n".repeat(2000)));
    }
}

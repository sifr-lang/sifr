use super::{Result, wire};
use ruff_text_size::Ranged;
use sifr_python_ast::{Expr, Stmt};
use std::collections::BTreeMap;
pub(super) struct SourceDeclarations {
    pub(super) locations: BTreeMap<String, (u32, u32)>,
    pub(super) parameters: Vec<String>,
}
pub(super) fn declarations(source: &str) -> Result<SourceDeclarations> {
    fn visit(statements: &[Stmt], prefix: &str, out: &mut BTreeMap<String, (u32, u32)>) {
        fn target_names(target: &Expr, out: &mut Vec<String>) {
            match target {
                Expr::Name(name) => out.push(name.id.to_string()),
                Expr::Tuple(tuple) => {
                    for target in &tuple.elts {
                        target_names(target, out);
                    }
                }
                _ => {}
            }
        }
        for statement in statements {
            let mut names = Vec::new();
            match statement {
                Stmt::Assign(value) => {
                    for target in &value.targets {
                        target_names(target, &mut names);
                    }
                }
                Stmt::AnnAssign(value) => target_names(&value.target, &mut names),
                Stmt::TypeAlias(value) => target_names(&value.name, &mut names),
                _ => {}
            }
            for name in names {
                let name = if prefix.is_empty() {
                    name
                } else {
                    format!("{prefix}.{name}")
                };
                out.insert(
                    name,
                    (
                        statement.range().start().to_u32(),
                        statement.range().end().to_u32(),
                    ),
                );
            }
            let (name, range, body) = match statement {
                Stmt::FunctionDef(value) => (
                    value.name.to_string(),
                    value.range,
                    Some(value.body.as_slice()),
                ),
                Stmt::ClassDef(value) => (
                    value.name.to_string(),
                    value.range,
                    Some(value.body.as_slice()),
                ),
                _ => continue,
            };
            let name = if prefix.is_empty() {
                name
            } else {
                format!("{prefix}.{name}")
            };
            out.insert(name.clone(), (range.start().to_u32(), range.end().to_u32()));
            if let Some(class) = name.strip_suffix(".__init__") {
                out.insert(
                    format!("{class}.new"),
                    (range.start().to_u32(), range.end().to_u32()),
                );
            }
            if let Some(body) = body {
                visit(body, &name, out);
            }
        }
    }
    let parsed = sifr_syntax::parse_module_raw(source, None).map_err(|_| {
        wire::MetadataError("cannot recover checked declaration source ranges".into())
    })?;
    let mut locations = BTreeMap::new();
    visit(parsed.suite(), "", &mut locations);
    let mut parameters = Vec::new();
    for statement in parsed.suite() {
        if let Stmt::Assign(assign) = statement {
            if let Expr::Call(call) = assign.value.as_ref() {
                if matches!(call.func.as_ref(),Expr::Name(name) if name.id.as_str()=="TypeVar") {
                    for target in &assign.targets {
                        if let Expr::Name(name) = target {
                            parameters.push(name.id.to_string());
                        }
                    }
                }
            }
        }
    }
    Ok(SourceDeclarations {
        locations,
        parameters,
    })
}

use crate::{
    ExternalDefs, HirDiagnostic, HirExpr, HirModule, HirStmt, lower_module,
    lower_module_with_externals,
};
use ruff_text_size::{TextRange, TextSize};
use sifr_python_parser::parse_module;

pub(crate) fn lower_source(source: &str) -> Result<HirModule, Vec<HirDiagnostic>> {
    let parsed = parse_module(source).expect("parse failed");
    lower_module(parsed.suite()).map(|r| r.module)
}

pub(crate) fn lower_source_with_externals(
    source: &str,
    externals: &ExternalDefs,
) -> Result<HirModule, Vec<HirDiagnostic>> {
    let parsed = parse_module(source).expect("parse failed");
    lower_module_with_externals(parsed.suite(), externals).map(|r| r.module)
}

pub(crate) fn lower_source_with_stdlib_collections(
    source: &str,
) -> Result<HirModule, Vec<HirDiagnostic>> {
    let parsed = parse_module(source).expect("parse failed");
    let mut externals = ExternalDefs::default();
    externals
        .functions
        .entry("sifr.collections".to_string())
        .or_default();
    lower_module_with_externals(parsed.suite(), &externals).map(|r| r.module)
}

pub(crate) fn range_for(source: &str, needle: &str) -> TextRange {
    let start = source.find(needle).expect("needle should exist") as u32;
    TextRange::new(
        TextSize::new(start),
        TextSize::new(start + needle.len() as u32),
    )
}

pub(crate) fn range_for_after(source: &str, after: &str, needle: &str) -> TextRange {
    let after_start = source.find(after).expect("anchor should exist");
    let relative_start = source[after_start..]
        .find(needle)
        .expect("needle should exist after anchor");
    let start = (after_start + relative_start) as u32;
    TextRange::new(
        TextSize::new(start),
        TextSize::new(start + needle.len() as u32),
    )
}

pub(crate) fn range_for_after_anchor(source: &str, after: &str, needle: &str) -> TextRange {
    let search_start = source.find(after).expect("anchor should exist") + after.len();
    let relative_start = source[search_start..]
        .find(needle)
        .expect("needle should exist after anchor");
    let start = (search_start + relative_start) as u32;
    TextRange::new(
        TextSize::new(start),
        TextSize::new(start + needle.len() as u32),
    )
}

pub(crate) fn function_let_value<'a>(module: &'a HirModule, name: &str) -> &'a HirExpr {
    module
        .functions
        .iter()
        .flat_map(|function| &function.body)
        .find_map(|stmt| match stmt {
            HirStmt::Let {
                name: local_name,
                value,
                ..
            } if local_name == name => Some(value),
            _ => None,
        })
        .expect("expected local binding")
}

fn let_value_in_stmts<'a>(stmts: &'a [HirStmt], name: &str) -> Option<&'a HirExpr> {
    for stmt in stmts {
        match stmt {
            HirStmt::Let {
                name: local_name,
                value,
                ..
            } if local_name == name => return Some(value),
            HirStmt::If {
                then_body,
                elif_clauses,
                else_body,
                ..
            } => {
                if let Some(value) = let_value_in_stmts(then_body, name) {
                    return Some(value);
                }
                for (_, body) in elif_clauses {
                    if let Some(value) = let_value_in_stmts(body, name) {
                        return Some(value);
                    }
                }
                if let Some(else_body) = else_body {
                    if let Some(value) = let_value_in_stmts(else_body, name) {
                        return Some(value);
                    }
                }
            }
            _ => {}
        }
    }
    None
}

pub(crate) fn function_nested_let_value<'a>(module: &'a HirModule, name: &str) -> &'a HirExpr {
    module
        .functions
        .iter()
        .find_map(|function| let_value_in_stmts(&function.body, name))
        .expect("expected nested local binding")
}

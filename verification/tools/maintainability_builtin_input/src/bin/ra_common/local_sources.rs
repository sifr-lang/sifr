//! Resolve body-local declaration scopes and owners through public semantics.
use hir::{Adt, Crate, EditionedFileId, Module, Semantics, db::HirDatabase};
use syntax::{AstNode, ast, ast::HasName};

pub fn modules<DB: HirDatabase>(
    semantics: &Semantics<'_, DB>,
    db: &DB,
    file: EditionedFileId,
    krate: Crate,
) -> Vec<Module> {
    let parsed = semantics.parse(file);
    let mut modules = krate.modules(db);
    for node in parsed.syntax().descendants().filter_map(ast::Adt::cast) {
        let Some(token) = node.name().and_then(|n| n.syntax().first_token()) else {
            continue;
        };
        semantics.descend_into_macros_cb(token, |mapped, _| {
            for node in mapped.value.parent_ancestors().filter_map(ast::Adt::cast) {
                if let Some(adt) = semantics.to_def(&node) {
                    let module = adt.module(db);
                    if module.krate(db) == krate && !modules.contains(&module) {
                        modules.push(module);
                    }
                }
            }
        });
    }
    modules
}

pub fn canonical_adt(db: &dyn HirDatabase, adt: Adt) -> anyhow::Result<String> {
    let module = adt.module(db);
    if module == module.nearest_non_block_module(db) {
        return Ok(format!(
            "{}::{}",
            super::ra_types::module(db, module),
            adt.name(db).as_str()
        ));
    }
    let semantics = Semantics::new_dyn(db);
    let source = semantics
        .source(adt)
        .ok_or_else(|| anyhow::anyhow!("missing body-local ADT source authority"))?;
    let body = source
        .value
        .syntax()
        .ancestors()
        .find_map(ast::BlockExpr::cast)
        .ok_or_else(|| anyhow::anyhow!("missing original body-local declaration block source"))?;
    let scope = semantics
        .scope(body.syntax())
        .ok_or_else(|| anyhow::anyhow!("missing body-local ADT semantic scope"))?;
    let function = scope
        .containing_function()
        .ok_or_else(|| anyhow::anyhow!("unsupported body-local declaration owner authority"))?;
    anyhow::ensure!(
        function.module(db).krate(db) == module.krate(db),
        "body-local semantic owner crate conflict"
    );
    let owner_source = semantics.source(function).ok_or_else(|| {
        anyhow::anyhow!("missing body-local containing function source authority")
    })?;
    let parents = source
        .value
        .syntax()
        .ancestors()
        .filter_map(ast::Fn::cast)
        .collect::<Vec<_>>();
    anyhow::ensure!(
        parents.len() == 1 && semantics.to_def(&parents[0]) == Some(function),
        "unsupported/ambiguous body-local containing-function source correspondence"
    );
    anyhow::ensure!(
        owner_source.file_id == source.file_id
            && owner_source.value.syntax().text_range() == parents[0].syntax().text_range()
            && super::declarations::source_tokens(owner_source.value.syntax())
                == super::declarations::source_tokens(parents[0].syntax()),
        "body-local original semantic function source correspondence conflict"
    );
    Ok(format!(
        "{}::{}::{}",
        super::ra_types::module(db, function.module(db)),
        function.name(db).as_str(),
        adt.name(db).as_str()
    ))
}

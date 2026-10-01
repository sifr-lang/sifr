//! Included physical nodes are admitted only by public semantic token descent.
use hir::{Adt, EditionedFileId, HirFileId, Semantics, db::HirDatabase};
use syntax::{AstNode, ast, ast::HasName};

pub fn physical_adt<DB: HirDatabase>(
    semantics: &Semantics<'_, DB>,
    expected: Adt,
    expanded_file: HirFileId,
    included_file: EditionedFileId,
) -> anyhow::Result<ast::Adt> {
    let parsed = semantics.parse(included_file);
    let mut matched = vec![];
    for node in parsed.syntax().descendants().filter_map(ast::Adt::cast) {
        let name = node
            .name()
            .ok_or_else(|| anyhow::anyhow!("missing included ADT name node"))?;
        let token = name
            .syntax()
            .first_token()
            .ok_or_else(|| anyhow::anyhow!("missing included ADT name token"))?;
        let mut resolved = false;
        semantics.descend_into_macros_cb(token, |mapped, _| {
            if mapped.file_id != expanded_file {
                return;
            }
            for expanded in mapped.value.parent_ancestors().filter_map(ast::Adt::cast) {
                if semantics.to_def(&expanded) == Some(expected) {
                    resolved = true;
                }
            }
        });
        if resolved {
            matched.push(node);
        }
    }
    anyhow::ensure!(
        matched.len() == 1,
        "unsupported/ambiguous exact RA include ADT source authority: {} matches",
        matched.len()
    );
    let node = matched.remove(0);
    anyhow::ensure!(
        semantics.hir_file_for(node.syntax()) == HirFileId::from(included_file),
        "included physical source file conflict"
    );
    Ok(node)
}

//! Public semantic subnode observations; unavailable targets remain explicit.
use hir::{Semantics, db::HirDatabase};
use serde_json::{Value, json};
use syntax::{AstNode, ast};

pub fn capture<DB: HirDatabase>(
    inventory: &mut super::include_inventory::Inventory,
    sem: &Semantics<'_, DB>,
    db: &DB,
    files: &vfs::Vfs,
    file: hir::HirFileId,
    node: &syntax::SyntaxNode,
) -> anyhow::Result<Value> {
    let signature_end = if let Some(f) = ast::Fn::cast(node.clone()) {
        f.body().map(|b| b.syntax().text_range().start())
    } else if let Some(i) = ast::Impl::cast(node.clone()) {
        i.assoc_item_list().map(|b| b.syntax().text_range().start())
    } else if let Some(t) = ast::Trait::cast(node.clone()) {
        t.assoc_item_list().map(|b| b.syntax().text_range().start())
    } else {
        None
    }
    .unwrap_or(node.text_range().end());
    let mut lifetimes = vec![];
    let mut traits = vec![];
    for n in node
        .descendants()
        .filter(|n| n.text_range().start() < signature_end)
    {
        if let Some(l) = ast::Lifetime::cast(n.clone()) {
            let resolved = if let Some(p) = sem.resolve_lifetime_param(&l) {
                let parent = p.parent(db);
                let src = sem.source(p);
                let target = if let Some(src) = src {
                    Some(
                        json!({"syntax":inventory.reference(sem,db,files,src.file_id,src.value.syntax())?,"roundtrip":sem.to_def(&src.value)==Some(p)}),
                    )
                } else {
                    None
                };
                Some(
                    json!({"owner":format!("{parent:?}"),"parameter":format!("{p:?}"),"index":parent.params(db).iter().position(|v|*v==hir::GenericParam::LifetimeParam(p)),"source":target}),
                )
            } else {
                None
            };
            lifetimes.push(json!({"syntax":inventory.reference(sem,db,files,file,&n)?,"resolved":resolved,"spelling_observation":n.text().to_string()}));
        }
        if let Some(p) = ast::Path::cast(n.clone()) {
            if let Some(t) = sem.resolve_trait(&p) {
                let src = sem.source(t);
                let target = if let Some(src) = src {
                    Some(
                        json!({"syntax":inventory.reference(sem,db,files,src.file_id,src.value.syntax())?,"roundtrip":sem.to_def(&src.value)==Some(t)}),
                    )
                } else {
                    None
                };
                traits.push(json!({"syntax":inventory.reference(sem,db,files,file,&n)?,"owner":format!("{t:?}"),"source":target}));
            }
        }
    }
    let generics = super::include_generics::capture(inventory, sem, db, files, node)?;
    Ok(json!({"lifetimes":lifetimes,"traits":traits,"generics":generics}))
}

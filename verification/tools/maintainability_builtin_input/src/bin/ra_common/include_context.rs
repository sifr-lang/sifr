//! Actual semantic parents and include ancestors, kept distinct from physical ranges.
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
    module: hir::Module,
) -> anyhow::Result<Value> {
    let mut chain = vec![];
    let mut current = file;
    let mut seen = std::collections::HashSet::new();
    while let Some(parent) = sem.find_parent_file(current) {
        anyhow::ensure!(
            seen.insert(current),
            "cyclic actual native include ancestry"
        );
        chain.push(json!({"child_file":format!("{current:?}"),"parent_file":format!("{:?}",parent.file_id),"syntax":inventory.reference(sem,db,files,parent.file_id,&parent.value)?}));
        current = parent.file_id;
    }
    let declaration=module.declaration_source(db).map(|s|{
        Ok::<_,anyhow::Error>(json!({"syntax":inventory.reference(sem,db,files,s.file_id,s.value.syntax())?,"roundtrip":sem.to_def(&s.value)==Some(module)}))
    }).transpose()?;
    let mut module_ancestors = vec![];
    let mut ancestor = module.parent(db);
    let mut seen_modules = std::collections::HashSet::new();
    while let Some(m) = ancestor {
        anyhow::ensure!(
            seen_modules.insert(m),
            "cyclic actual native module ancestry"
        );
        let src = sem.module_definition_node(m);
        let roundtrip = if let Some(n) = ast::Module::cast(src.value.clone()) {
            sem.to_def(&n) == Some(m)
        } else if let Some(n) = ast::SourceFile::cast(src.value.clone()) {
            sem.to_def(&n) == Some(m)
        } else {
            false
        };
        module_ancestors.push(json!({"owner":format!("{m:?}"),"syntax":inventory.reference(sem,db,files,src.file_id,&src.value)?,"kind":format!("{:?}",src.value.kind()),"roundtrip":roundtrip}));
        ancestor = m.parent(db);
    }
    let mut parents = vec![];
    for n in node.ancestors().skip(1) {
        if let Some(f) = ast::Fn::cast(n.clone()) {
            if let Some(d) = sem.to_def(&f) {
                let src = sem
                    .source(d)
                    .ok_or_else(|| anyhow::anyhow!("missing actual enclosing function source"))?;
                parents.push(json!({"kind":"Function","owner":format!("{d:?}"),"syntax":inventory.reference(sem,db,files,src.file_id,src.value.syntax())?,"roundtrip":sem.to_def(&src.value)==Some(d)}));
            }
        } else if let Some(i) = ast::Impl::cast(n.clone()) {
            if let Some(d) = sem.to_def(&i) {
                let s = sem
                    .source(d)
                    .ok_or_else(|| anyhow::anyhow!("missing actual semantic parent impl source"))?;
                let tr=d.trait_(db).map(|t|{
                    let src=sem.source(t).ok_or_else(||anyhow::anyhow!("missing actual impl trait source"))?;
                    Ok::<_,anyhow::Error>(json!({"owner":format!("{t:?}"),"syntax":inventory.reference(sem,db,files,src.file_id,src.value.syntax())?,"roundtrip":sem.to_def(&src.value)==Some(t)}))
                }).transpose()?;
                parents.push(json!({"kind":"Impl","owner":format!("{d:?}"),"syntax":inventory.reference(sem,db,files,s.file_id,s.value.syntax())?,"roundtrip":sem.to_def(&s.value)==Some(d),"trait":tr}));
            }
        } else if let Some(t) = ast::Trait::cast(n.clone()) {
            if let Some(d) = sem.to_def(&t) {
                let s = sem.source(d).ok_or_else(|| {
                    anyhow::anyhow!("missing actual semantic parent trait source")
                })?;
                parents.push(json!({"kind":"Trait","owner":format!("{d:?}"),"syntax":inventory.reference(sem,db,files,s.file_id,s.value.syntax())?,"roundtrip":sem.to_def(&s.value)==Some(d)}));
            }
        }
    }
    let function=sem.scope(node).and_then(|s|s.containing_function()).map(|f|{
        let s=sem.source(f).ok_or_else(||anyhow::anyhow!("missing actual enclosing function source"))?;
        Ok::<_,anyhow::Error>(json!({"owner":format!("{f:?}"),"syntax":inventory.reference(sem,db,files,s.file_id,s.value.syntax())?,"roundtrip":sem.to_def(&s.value)==Some(f)}))
    }).transpose()?;
    Ok(
        json!({"include_ancestors":chain,"module_declaration":declaration,"module_ancestors":module_ancestors,"semantic_parents":parents,"enclosing_function":function}),
    )
}

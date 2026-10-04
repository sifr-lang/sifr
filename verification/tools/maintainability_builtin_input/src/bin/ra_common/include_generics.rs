//! Actual generic owner/parameter identities; source absence stays explicit.
use hir::{GenericDef, GenericParam, Semantics, db::HirDatabase};
use serde_json::{Value, json};
use syntax::{AstNode, ast};

pub fn capture<DB: HirDatabase>(
    inventory: &mut super::include_inventory::Inventory,
    sem: &Semantics<'_, DB>,
    db: &DB,
    files: &vfs::Vfs,
    node: &syntax::SyntaxNode,
) -> anyhow::Result<Value> {
    let owner: Option<GenericDef> = if let Some(n) = ast::Fn::cast(node.clone()) {
        sem.to_def(&n).map(Into::into)
    } else if let Some(n) = ast::Impl::cast(node.clone()) {
        sem.to_def(&n).map(Into::into)
    } else if let Some(n) = ast::Adt::cast(node.clone()) {
        sem.to_def(&n).map(Into::into)
    } else if let Some(n) = ast::Trait::cast(node.clone()) {
        sem.to_def(&n).map(Into::into)
    } else if let Some(n) = ast::TypeAlias::cast(node.clone()) {
        sem.to_def(&n).map(Into::into)
    } else if let Some(n) = ast::Const::cast(node.clone()) {
        sem.to_def(&n).map(Into::into)
    } else if let Some(n) = ast::Static::cast(node.clone()) {
        sem.to_def(&n).map(Into::into)
    } else {
        None
    };
    let Some(owner) = owner else {
        return Ok(Value::Null);
    };
    let mut parameters = vec![];
    for (index, p) in owner.params(db).into_iter().enumerate() {
        let (kind, implicit, source) = match p {
            GenericParam::LifetimeParam(d) => {
                let source=sem.source(d).map(|src|Ok::<_,anyhow::Error>(json!({"syntax":inventory.reference(sem,db,files,src.file_id,src.value.syntax())?,"roundtrip":sem.to_def(&src.value)==Some(d),"disposition":"explicit-parameter-source"}))).transpose()?;
                ("lifetime", false, source)
            }
            GenericParam::TypeParam(d) => {
                let source=sem.source(d.merge()).map(|src|{
                    let value=if src.value.as_ref().left().is_some(){
                        let roundtrip=ast::GenericParam::cast(src.value.syntax().clone()).is_some_and(|n|sem.to_def(&n)==Some(p));
                        json!({"syntax":inventory.reference(sem,db,files,src.file_id,src.value.syntax())?,"roundtrip":roundtrip,"disposition":"explicit-parameter-source"})
                    }else if let Some(n)=src.value.as_ref().right(){
                        let roundtrip=match p.parent(){GenericDef::Trait(t)=>sem.to_def(n)==Some(t),_=>false};
                        json!({"syntax":inventory.reference(sem,db,files,src.file_id,n.syntax())?,"roundtrip":roundtrip,"disposition":"implicit-trait-self-owner-source"})
                    }else{anyhow::bail!("unknown actual generic parameter source variant")}
                    ;Ok::<_,anyhow::Error>(value)
                }).transpose()?;
                ("type", d.is_implicit(db), source)
            }
            GenericParam::ConstParam(d) => {
                let source=sem.source(d.merge()).map(|src|{
                    let roundtrip=src.value.as_ref().left().and_then(|n|ast::GenericParam::cast(n.syntax().clone())).is_some_and(|n|sem.to_def(&n)==Some(p));
                    Ok::<_,anyhow::Error>(json!({"syntax":inventory.reference(sem,db,files,src.file_id,src.value.syntax())?,"roundtrip":roundtrip,"disposition":"explicit-parameter-source"}))
                }).transpose()?;
                ("const", false, source)
            }
        };
        anyhow::ensure!(p.parent() == owner, "actual native generic owner mismatch");
        parameters.push(json!({"index":index,"parameter":format!("{p:?}"),"owner":format!("{owner:?}"),"kind":kind,"implicit":implicit,"source":source}));
    }
    Ok(json!({"owner":format!("{owner:?}"),"parameters":parameters}))
}

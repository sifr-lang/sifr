//! Authenticated external owner/source roundtrip for the bounded diagnostic.
use hir::{HirDisplay, Semantics, db::HirDatabase};
use serde_json::{Value, json};
use syntax::{AstNode, ast, ast::HasName};
fn interval(node: &syntax::SyntaxNode) -> Value {
    json!([
        u32::from(node.text_range().start()),
        u32::from(node.text_range().end())
    ])
}
pub fn capture<DB: HirDatabase>(
    semantics: &Semantics<'_, DB>,
    db: &DB,
    files: &vfs::Vfs,
    krate: hir::Crate,
    suffix: &str,
) -> anyhow::Result<Value> {
    let mut calls = vec![];
    for (file, path) in files.iter() {
        let Some(path) = path.as_path() else { continue };
        if !path.as_str().ends_with(suffix) {
            continue;
        }
        let parsed = semantics.parse(hir::EditionedFileId::new(db, file, krate.edition(db)));
        for call in parsed
            .syntax()
            .descendants()
            .filter_map(ast::MethodCallExpr::cast)
        {
            let Some(function) = semantics.resolve_method_call(&call) else {
                continue;
            };
            if function.name(db).as_str() != "step" {
                continue;
            }
            let receiver = call
                .receiver()
                .ok_or_else(|| anyhow::anyhow!("missing receiver"))?;
            let receiver_ty = semantics
                .type_of_expr(&receiver)
                .ok_or_else(|| anyhow::anyhow!("missing receiver type"))?
                .adjusted();
            anyhow::ensure!(!receiver_ty.contains_unknown(), "unknown original receiver");
            // Register the external original tree through source before queries.
            let source = semantics
                .source(function)
                .ok_or_else(|| anyhow::anyhow!("missing external function source"))?;
            anyhow::ensure!(
                semantics.to_def(&source.value) == Some(function),
                "external original callable roundtrip conflict"
            );
            let original = semantics
                .original_range_opt(source.value.syntax())
                .ok_or_else(|| anyhow::anyhow!("missing external original file range"))?;
            let original_path = files.file_path(original.file_id.file_id(db));
            let original_path = original_path
                .as_path()
                .ok_or_else(|| anyhow::anyhow!("nonphysical external source"))?;
            let implementation = source
                .value
                .syntax()
                .ancestors()
                .find_map(ast::Impl::cast)
                .ok_or_else(|| anyhow::anyhow!("missing external parent impl"))?;
            let resolved_impl = semantics
                .to_def(&implementation)
                .ok_or_else(|| anyhow::anyhow!("unresolved external parent impl"))?;
            let impl_source = semantics
                .source(resolved_impl)
                .ok_or_else(|| anyhow::anyhow!("missing external impl source"))?;
            anyhow::ensure!(
                impl_source.file_id == source.file_id
                    && semantics.to_def(&impl_source.value) == Some(resolved_impl),
                "external parent impl source roundtrip conflict"
            );
            let signature_end = source
                .value
                .body()
                .map(|b| u32::from(b.syntax().text_range().start()))
                .unwrap_or(u32::from(source.value.syntax().text_range().end()));
            let params = hir::GenericDef::Function(function).params(db);
            let inherited = hir::GenericDef::Impl(resolved_impl).params(db);
            let lifetimes=source.value.syntax().descendants().filter_map(ast::Lifetime::cast).filter(|l|u32::from(l.syntax().text_range().start())<signature_end).enumerate().map(|(ordinal,l)|{
                let resolved=semantics.resolve_lifetime_param(&l).map(|p|{
                    let parent=p.parent(db);
                    json!({"kind":"GenericDefParameter","relation":if parent==hir::GenericDef::Function(function){"own"}else if parent==hir::GenericDef::Impl(resolved_impl){"inherited"}else{"other"},"ordinal":parent.params(db).iter().position(|v|*v==hir::GenericParam::LifetimeParam(p)),"name":p.name(db).as_str()})
                });
                json!({"ordinal":ordinal,"kind":format!("{:?}",l.syntax().kind()),"range":interval(l.syntax()),"token":l.syntax().text().to_string(),"resolved":resolved})
            }).collect::<Vec<_>>();
            let mut traits = vec![];
            for p in source
                .value
                .syntax()
                .descendants()
                .filter_map(ast::Path::cast)
                .filter(|p| u32::from(p.syntax().text_range().start()) < signature_end)
            {
                if let Some(t) = semantics.resolve_trait(&p) {
                    let s = semantics
                        .source(t)
                        .ok_or_else(|| anyhow::anyhow!("missing resolved trait source"))?;
                    anyhow::ensure!(
                        semantics.to_def(&s.value) == Some(t),
                        "trait source roundtrip conflict"
                    );
                    traits.push(json!({"range":interval(p.syntax()),"token":p.syntax().text().to_string(),"identity":format!("{}::{}",super::ra_types::module(db,t.module(db)),t.name(db).as_str()),"source_range":interval(s.value.syntax()),"name_range":s.value.name().map(|n|interval(n.syntax())),"name_token":s.value.name().map(|n|n.syntax().text().to_string()),"source_file":files.file_path(semantics.original_range(s.value.syntax()).file_id.file_id(db)).as_path().map(|p|p.as_str()),"roundtrip":true}));
                }
            }
            let mut callee_cfg = function
                .module(db)
                .krate(db)
                .cfg(db)
                .into_iter()
                .map(|atom| match atom {
                    cfg::CfgAtom::Flag(key) => json!({"key":key.as_str(),"value":null}),
                    cfg::CfgAtom::KeyValue { key, value } => {
                        json!({"key":key.as_str(),"value":value.as_str()})
                    }
                })
                .collect::<Vec<_>>();
            callee_cfg.sort_by_cached_key(|a| a.to_string());
            calls.push(json!({"callee_cfg":callee_cfg,"caller_file":path.as_str(),"caller_range":interval(call.syntax()),"receiver":receiver_ty.display(db,krate.to_display_target(db)).to_string(),"contains_unknown":receiver_ty.contains_unknown(),"owner_file":original_path.as_str(),"owner_range":interval(source.value.syntax()),"parent_range":interval(impl_source.value.syntax()),"owner_tokens":super::declarations::source_tokens(source.value.syntax()),"owner_name":function.name(db).as_str(),"owner_module":super::ra_types::module(db,function.module(db)),"owner_roundtrip":true,"parent_roundtrip":true,"impl_trait":resolved_impl.trait_(db).map(|t|format!("{}::{}",super::ra_types::module(db,t.module(db)),t.name(db).as_str())),"parameters":params.iter().enumerate().map(|(ordinal,p)|generic(db,ordinal,*p)).collect::<Vec<_>>(),"inherited_parameters":inherited.iter().enumerate().map(|(ordinal,p)|generic(db,ordinal,*p)).collect::<Vec<_>>(),"lifetime_occurrences":lifetimes,"trait_constraints":traits}));
        }
    }
    anyhow::ensure!(
        calls.len() == 1,
        "missing/ambiguous required original step call"
    );
    Ok(
        json!({"schema":"sifr-maintainability-source-binder-ra-v1","calls":calls,"semantic_export":false}),
    )
}

pub fn syntax_fixture(path: &str) -> anyhow::Result<Value> {
    let text = std::fs::read_to_string(path)?;
    let parsed = ast::SourceFile::parse(&text, syntax::Edition::Edition2024);
    anyhow::ensure!(
        parsed.errors().is_empty(),
        "invalid bounded RA fixture syntax"
    );
    let functions=parsed.tree().syntax().descendants().filter_map(ast::Fn::cast).map(|f| {
        let end=f.body().map(|b|b.syntax().text_range().start()).unwrap_or(f.syntax().text_range().end());
        json!({"range":interval(f.syntax()),"lifetimes":f.syntax().descendants().filter_map(ast::Lifetime::cast).filter(|l|l.syntax().text_range().start()<end).enumerate().map(|(ordinal,l)|json!({"ordinal":ordinal,"kind":format!("{:?}",l.syntax().kind()),"range":interval(l.syntax()),"token":l.syntax().text().to_string()})).collect::<Vec<_>>()})
    }).collect::<Vec<_>>();
    Ok(json!({"authority":"pinned-ra-original-syntax-only","file":path,"functions":functions}))
}

fn generic(db: &dyn HirDatabase, ordinal: usize, param: hir::GenericParam) -> Value {
    match param {
        hir::GenericParam::LifetimeParam(p) => {
            json!({"ordinal":ordinal,"kind":"lifetime","name":p.name(db).as_str()})
        }
        hir::GenericParam::TypeParam(p) => {
            json!({"ordinal":ordinal,"kind":"type","name":p.name(db).as_str()})
        }
        hir::GenericParam::ConstParam(_) => json!({"ordinal":ordinal,"kind":"unsupported-const"}),
    }
}

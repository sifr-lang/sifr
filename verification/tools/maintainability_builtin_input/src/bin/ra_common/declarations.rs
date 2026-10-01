//! RA semantic declaration/source correspondence; no generated region inference.
use hir::{GenericDef, GenericParam, Semantics, db::HirDatabase};
use serde_json::{Value, json};
use syntax::{AstNode, SyntaxKind, SyntaxNode, ast};

pub fn source_tokens(node: &SyntaxNode) -> Vec<String> {
    node.descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .filter(|token| token.kind() != SyntaxKind::WHITESPACE)
        .flat_map(|token| {
            let text = token.text();
            if matches!(
                text,
                "->" | "::"
                    | ".."
                    | "..="
                    | "=>"
                    | "&&"
                    | "||"
                    | "=="
                    | "!="
                    | "<="
                    | ">="
                    | "<<"
                    | ">>"
                    | "+="
                    | "-="
                    | "*="
                    | "/="
                    | "%="
                    | "&="
                    | "|="
                    | "^="
            ) {
                text.chars().map(|c| c.to_string()).collect::<Vec<_>>()
            } else {
                vec![text.to_string()]
            }
        })
        .collect()
}
pub fn facts<DB: HirDatabase>(
    db: &dyn HirDatabase,
    semantics: &Semantics<'_, DB>,
    owner: GenericDef,
    node: &SyntaxNode,
    identity: &str,
) -> anyhow::Result<Value> {
    let params = owner.params(db);
    let parameters = params
        .iter()
        .enumerate()
        .map(|(index, p)| match p {
            GenericParam::LifetimeParam(p) => {
                json!({"index":index,"name":p.name(db).as_str(),"kind":"lifetime"})
            }
            GenericParam::TypeParam(p) => {
                json!({"index":index,"name":p.name(db).as_str(),"kind":"type"})
            }
            GenericParam::ConstParam(_) => json!({"unsupported":"const declaration parameter"}),
        })
        .collect::<Vec<_>>();
    let mut lifetimes = vec![];
    for lifetime in node.descendants().filter_map(ast::Lifetime::cast) {
        let text = lifetime.syntax().text().to_string();
        let disposition = if text == "'static" {
            json!({"kind":"static"})
        } else if text == "'_" {
            json!({"kind":"placeholder"})
        } else if let Some(resolved) = semantics.resolve_lifetime_param(&lifetime) {
            let parent = resolved.parent(db);
            let parameter = parent
                .params(db)
                .iter()
                .position(|p| *p == GenericParam::LifetimeParam(resolved))
                .ok_or_else(|| anyhow::anyhow!("unowned resolved lifetime"))?;
            json!({"kind":"parameter","owner_relation":if parent==owner{"own"}else{"binder-or-parent"},"index":parameter,"name":resolved.name(db).as_str()})
        } else {
            anyhow::bail!("unresolved declaration lifetime {text} in {identity}")
        };
        lifetimes.push(json!({"syntax":text,"disposition":disposition,"range":format!("{:?}",lifetime.syntax().text_range())}));
    }
    let bounds=node.descendants().filter_map(ast::TypeBound::cast).filter_map(|bound|bound.syntax().descendants().filter_map(ast::Path::cast).find_map(|path|semantics.resolve_trait(&path).map(|t|json!({"trait":format!("{}::{}",super::ra_types::module(db,t.module(db)),t.name(db).as_str()),"tokens":source_tokens(bound.syntax())})))).collect::<Vec<_>>();
    Ok(
        json!({"identity":identity,"parameters":parameters,"lifetimes":lifetimes,"bounds":bounds,"source_tokens":source_tokens(node),"range":format!("{:?}",semantics.original_range(node).range),"authority":"ra-semantic-declaration-and-source-correspondence"}),
    )
}

pub fn invocation_site(
    meta: &ast::Meta,
    ordinal: usize,
    file_text: &str,
    file: &str,
) -> anyhow::Result<Value> {
    let tokens = meta
        .syntax()
        .descendants_with_tokens()
        .filter_map(|t| t.into_token())
        .filter(|t| t.kind() != SyntaxKind::WHITESPACE)
        .collect::<Vec<_>>();
    let opening = tokens
        .iter()
        .position(|t| t.text() == "(")
        .ok_or_else(|| anyhow::anyhow!("missing derive arguments"))?;
    let closing = tokens
        .iter()
        .rposition(|t| t.text() == ")")
        .ok_or_else(|| anyhow::anyhow!("missing derive argument terminator"))?;
    let paths = tokens[opening + 1..closing]
        .split(|t| t.text() == ",")
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>();
    let path = paths
        .get(ordinal)
        .ok_or_else(|| anyhow::anyhow!("derive ordinal/source mismatch"))?;
    let location = |offset: u32| -> anyhow::Result<Value> {
        let prefix = file_text
            .get(..offset as usize)
            .ok_or_else(|| anyhow::anyhow!("derive range is not an original source range"))?;
        let line = prefix.bytes().filter(|b| *b == b'\n').count() + 1;
        let column = prefix.rsplit('\n').next().unwrap_or("").chars().count();
        Ok(json!([line, column]))
    };
    let start = u32::from(
        path.first()
            .expect("nonempty derive path")
            .text_range()
            .start(),
    );
    let end = u32::from(
        path.last()
            .expect("nonempty derive path")
            .text_range()
            .end(),
    );
    Ok(
        json!({"file":file,"start":location(start)?,"end":location(end)?,"quality":"exact-source","attribute_start":location(u32::from(meta.syntax().parent().ok_or_else(||anyhow::anyhow!("derive attribute owner missing"))?.text_range().start()))?,"attribute_end":location(u32::from(meta.syntax().parent().ok_or_else(||anyhow::anyhow!("derive attribute owner missing"))?.text_range().end()))?}),
    )
}

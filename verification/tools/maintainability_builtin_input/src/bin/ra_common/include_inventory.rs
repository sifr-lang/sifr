//! Complete native/physical syntax roots are captured once before any selection.
use hir::{Semantics, db::HirDatabase};
use serde_json::{Value, json};
use std::{collections::HashMap, fs::File, io::BufWriter, path::PathBuf};
use syntax::{AstNode, SyntaxNode, ast::HasAttrs};

fn range(r: syntax::TextRange) -> Value {
    json!([u32::from(r.start()), u32::from(r.end())])
}

pub struct Inventory {
    directory: PathBuf,
    roots: Vec<Value>,
    registered: HashMap<hir::HirFileId, usize>,
    source_roots: Vec<(hir::HirFileId, SyntaxNode)>,
    physical_files: std::collections::HashSet<hir::EditionedFileId>,
}
impl Inventory {
    pub fn new() -> anyhow::Result<Self> {
        let directory = PathBuf::from(
            std::env::var_os("SIFR_BUILTIN_NATIVE_INVENTORY_DIR").ok_or_else(|| {
                anyhow::anyhow!("explicit owned original native inventory destination required")
            })?,
        );
        anyhow::ensure!(
            !directory.exists(),
            "original native inventory destination already exists"
        );
        std::fs::create_dir_all(&directory)?;
        Ok(Self {
            directory,
            roots: vec![],
            registered: HashMap::new(),
            source_roots: vec![],
            physical_files: std::collections::HashSet::new(),
        })
    }
    pub fn reference<DB: HirDatabase>(
        &mut self,
        sem: &Semantics<'_, DB>,
        db: &DB,
        files: &vfs::Vfs,
        file: hir::HirFileId,
        node: &SyntaxNode,
    ) -> anyhow::Result<Value> {
        let index = if let Some(index) = self.registered.get(&file) {
            *index
        } else {
            let root = node.tree_top();
            let index = self.roots.len();
            let name = format!("root-{index}.json");
            let path = self.directory.join(&name);
            let mut writer = BufWriter::new(File::create(path.with_extension("tmp"))?);
            let (data, origins) = root_inventory(sem, db, files, file, &root);
            self.physical_files.extend(origins);
            serde_json::to_writer(&mut writer, &data)?;
            std::io::Write::flush(&mut writer)?;
            std::fs::rename(path.with_extension("tmp"), &path)?;
            self.roots.push(json!({"is_include":is_include(file,db),"file":format!("{file:?}"),"path":path,"kind":format!("{:?}",root.kind()),"range":range(root.text_range()),"node_count":data["nodes"].as_array().map(Vec::len),"token_count":data["tokens"].as_array().map(Vec::len)}));
            self.registered.insert(file, index);
            self.source_roots.push((file, root));
            index
        };
        // Use the actual native syntax node, not a range/hull-selected substitute.
        let root = node.tree_top();
        let position = root.descendants().position(|n| n == *node).ok_or_else(|| {
            anyhow::anyhow!("actual owner missing from complete native syntax root")
        })?;
        Ok(json!({"root":index,"node":position}))
    }
    pub fn physical_files(&self) -> Vec<hir::EditionedFileId> {
        self.physical_files.iter().copied().collect()
    }
    pub fn source_roots(&self) -> Vec<(hir::HirFileId, SyntaxNode)> {
        self.source_roots.clone()
    }
    pub fn finish(self) -> Value {
        json!({"roots":self.roots,"directory":self.directory})
    }
}

fn root_inventory<DB: HirDatabase>(
    sem: &Semantics<'_, DB>,
    db: &DB,
    files: &vfs::Vfs,
    file: hir::HirFileId,
    root: &SyntaxNode,
) -> (Value, Vec<hir::EditionedFileId>) {
    let nodes = root.descendants().collect::<Vec<_>>();
    let tokens = root
        .descendants_with_tokens()
        .filter_map(|e| e.into_token())
        .collect::<Vec<_>>();
    let node_ids = nodes
        .iter()
        .cloned()
        .enumerate()
        .map(|(i, n)| (n, i))
        .collect::<HashMap<_, _>>();
    let mut origin_files = vec![];
    let mut origin_ids = HashMap::new();
    let mut origin = |file: hir::EditionedFileId, r: syntax::TextRange| {
        let id = *origin_ids.entry(file).or_insert_with(|| {
            let id = origin_files.len();
            origin_files.push(json!({"file":format!("{file:?}"),"path":files.file_path(file.file_id(db)).as_path().map(|p|p.as_str())}));
            id
        });
        json!({"file":id,"range":range(r)})
    };
    let token_rows = tokens.iter().enumerate().map(|(ordinal,t)| {
        let mapped = hir::InFile::new(file,t.clone()).original_file_range_opt(db)
            .map(|r|origin(r.file_id,r.range));
        json!({"ordinal":ordinal,"kind":format!("{:?}",t.kind()),"text":t.text(),"range":range(t.text_range()),"trivia":t.kind().is_trivia(),"parent":t.parent().and_then(|p|node_ids.get(&p).copied()),"original":mapped})
    }).collect::<Vec<_>>();
    let node_rows = nodes.iter().enumerate().map(|(ordinal,n)| {
        let aggregate = sem.original_range_opt(n).map(|r|origin(r.file_id,r.range));
        let contextual = hir::InFile::new(file,n).original_file_range_opt(db)
            .map(|(r,ctx)|json!({"original":origin(r.file_id,r.range),"syntax_context":format!("{ctx:?}"),"root_context":ctx.is_root()}));
        let children = n.children().map(|child|node_ids[&child]).collect::<Vec<_>>();
        let attributes = syntax::ast::AnyHasAttrs::cast(n.clone()).map(|n|n.attrs_with_doc().enumerate().map(|(i,a)|{
            let style=match &a {syntax::ast::AnyAttr::Attr(a)=>if a.excl_token().is_some(){"inner"}else{"outer"},syntax::ast::AnyAttr::DocComment(d)=>if d.inner_doc_comment_token().is_some(){"inner"}else{"outer"}};
            json!({"ordinal":i,"node":node_ids[a.syntax()],"kind":format!("{:?}",a.syntax().kind()),"style":style})
        }).collect::<Vec<_>>()).unwrap_or_default();
        json!({"ordinal":ordinal,"kind":format!("{:?}",n.kind()),"range":range(n.text_range()),"parent":n.parent().and_then(|p|node_ids.get(&p).copied()),"children":children,"direct_attributes":attributes,"aggregate_original":aggregate,"contextual_original":contextual})
    }).collect::<Vec<_>>();
    let ids = origin_ids.keys().copied().collect::<Vec<_>>();
    (
        json!({"schema":"development-complete-syntax-root-inventory-v2","file":format!("{file:?}"),"text":root.text().to_string(),"origin_files":origin_files,"tokens":token_rows,"nodes":node_rows,"semantic_export":false,"accepted_proof":false}),
        ids,
    )
}

/// Direct official include expander identity; ancestry alone is not enough.
pub fn is_include<DB: HirDatabase>(file: hir::HirFileId, db: &DB) -> bool {
    matches!(file,hir::HirFileId::MacroFile(call) if call.is_include_macro(db))
}

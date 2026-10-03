use rustc_ast::{
    self as ast,
    visit::{self, Visitor},
};
use rustc_ast_pretty::pprust::{self, PrintState};
use rustc_hir::def_id::LocalDefId;
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;
use std::collections::HashMap;

pub struct Site {
    pub kind: String,
    pub tokens: String,
    pub span: Span,
    pub ancestor: Option<usize>,
}
pub struct Declaration {
    pub def: LocalDefId,
    pub node_id: ast::NodeId,
    pub ast_kind: String,
    pub attrs: ast::AttrVec,
    pub tokens_available: bool,
    pub span: Span,
    pub tokens: String,
    pub body: bool,
    pub impl_member_count: Option<usize>,
    pub body_tokens: Option<String>,
    pub sites: Vec<Site>,
}
struct Expanded {
    owners: HashMap<ast::NodeId, LocalDefId>,
    declarations: Vec<Declaration>,
    original_text: storage::OriginalText,
}
impl<'ast> Visitor<'ast> for Expanded {
    fn visit_item(&mut self, item: &'ast ast::Item) {
        if let Some(def) = self.owners.get(&item.id).copied() {
            let mut declaration = Declaration {
                def,
                node_id: item.id,
                ast_kind: item.kind.descr().into(),
                attrs: item.attrs.clone(),
                tokens_available: item.tokens.is_some(),
                span: item.span,
                tokens: pprust::item_to_string(item),
                body: false,
                body_tokens: None,
                impl_member_count: if let ast::ItemKind::Impl(implementation) = &item.kind {
                    Some(implementation.items.len())
                } else {
                    None
                },
                sites: vec![],
            };
            self.original_text.store(&mut declaration);
            self.declarations.push(declaration);
        }
        visit::walk_item(self, item);
    }
    fn visit_assoc_item(&mut self, item: &'ast ast::AssocItem, ctxt: visit::AssocCtxt) {
        if let Some(def) = self.owners.get(&item.id).copied() {
            let mut sites = Sites {
                nodes: vec![],
                parent: None,
            };
            let body = if let ast::AssocItemKind::Fn(f) = &item.kind {
                if let Some(block) = &f.body {
                    sites.visit_block(block);
                    true
                } else {
                    false
                }
            } else {
                false
            };
            let mut declaration = Declaration {
                def,
                node_id: item.id,
                ast_kind: match &item.kind {
                    ast::AssocItemKind::Fn(_) => "associated function",
                    ast::AssocItemKind::Const(_) => "associated const",
                    ast::AssocItemKind::Type(_) => "associated type",
                    _ => "other associated item",
                }
                .into(),
                attrs: item.attrs.clone(),
                tokens_available: item.tokens.is_some(),
                span: item.span,
                tokens: pprust::assoc_item_to_string(item),
                body,
                body_tokens: if let ast::AssocItemKind::Fn(f) = &item.kind {
                    f.body
                        .as_ref()
                        .map(|b| pprust::State::new().block_to_string(b))
                } else {
                    None
                },
                impl_member_count: None,
                sites: sites.nodes,
            };
            self.original_text.store(&mut declaration);
            self.declarations.push(declaration);
        }
        visit::walk_assoc_item(self, item, ctxt);
    }
}
struct Sites {
    nodes: Vec<Site>,
    parent: Option<usize>,
}
impl<'ast> Visitor<'ast> for Sites {
    fn visit_expr(&mut self, expr: &'ast ast::Expr) {
        let kind = match &expr.kind {
            ast::ExprKind::Call(..) => Some("call".into()),
            ast::ExprKind::MethodCall(..) => Some("method".into()),
            ast::ExprKind::Binary(op, ..) => Some(format!("binary:{:?}", op.node)),
            ast::ExprKind::Unary(op, ..) => Some(format!("unary:{op:?}")),
            _ => None,
        };
        let parent = self.parent;
        if let Some(kind) = kind {
            self.parent = Some(self.nodes.len());
            self.nodes.push(Site {
                kind,
                tokens: pprust::expr_to_string(expr),
                span: expr.span,
                ancestor: parent,
            });
        }
        visit::walk_expr(self, expr);
        self.parent = parent;
    }
}
pub fn capture(tcx: TyCtxt<'_>) -> (Vec<Declaration>, storage::OriginalText) {
    let resolver = tcx.resolver_for_lowering();
    let mapping = resolver.0.borrow();
    let mut owners = HashMap::new();
    for (_, owner) in mapping
        .owners
        .items()
        .map(|(node, owner)| (node.as_u32(), owner))
        .collect_stable_ord_by_key::<_, Vec<_>, _>(|(key, _)| key)
    {
        owners.insert(owner.id, owner.def_id);
        for (_, (node, def)) in owner
            .node_id_to_def_id
            .items()
            .map(|(node, def)| (node.as_u32(), (node, def)))
            .collect_stable_ord_by_key::<_, Vec<_>, _>(|(key, _)| key)
        {
            owners.insert(*node, *def);
        }
    }
    let mut visitor = Expanded {
        owners,
        declarations: vec![],
        original_text: storage::OriginalText::create(),
    };
    visitor.visit_crate(&resolver.1.borrow());
    visitor.original_text.seal();
    (visitor.declarations, visitor.original_text)
}

/// Complete actual compiler resolver associations, captured before lowering consumes them.
pub fn associations(tcx: TyCtxt<'_>) -> Vec<(ast::NodeId, LocalDefId)> {
    let resolver = tcx.resolver_for_lowering();
    let data = resolver.0.borrow();
    let mut nodes = HashMap::new();
    for (_, owner) in data
        .owners
        .items()
        .map(|(node, owner)| (node.as_u32(), owner))
        .collect_stable_ord_by_key::<_, Vec<_>, _>(|(node, _)| node)
    {
        nodes.insert(owner.id, owner.def_id);
        for (_, (node, def)) in owner
            .node_id_to_def_id
            .items()
            .map(|(node, def)| (node.as_u32(), (node, def)))
            .collect_stable_ord_by_key::<_, Vec<_>, _>(|(node, _)| node)
        {
            nodes.insert(*node, *def);
        }
    }
    let mut result = nodes.into_iter().collect::<Vec<_>>();
    result.sort_by_key(|(node, _)| node.as_u32());
    result
}

/// Exhaustive expanded AST attribute observations, including nested/body attrs.
/// Declaration ancestry is actual visitor context, never original attachment.
pub struct StageAttribute {
    pub attribute: ast::Attribute,
    pub declaration_ancestry: Vec<ast::NodeId>,
    pub direct_declaration_attribute: bool,
}
struct Attributes {
    rows: Vec<StageAttribute>,
    ancestry: Vec<ast::NodeId>,
    direct: Vec<*const ast::Attribute>,
}
impl<'ast> Visitor<'ast> for Attributes {
    fn visit_attribute(&mut self, a: &'ast ast::Attribute) {
        self.rows.push(StageAttribute {
            attribute: a.clone(),
            declaration_ancestry: self.ancestry.clone(),
            direct_declaration_attribute: self.direct.contains(&(a as *const ast::Attribute)),
        });
        visit::walk_attribute(self, a);
    }
    fn visit_item(&mut self, item: &'ast ast::Item) {
        let prior = std::mem::replace(
            &mut self.direct,
            item.attrs
                .iter()
                .map(|a| a as *const ast::Attribute)
                .collect(),
        );
        self.ancestry.push(item.id);
        visit::walk_item(self, item);
        self.ancestry.pop();
        self.direct = prior;
    }
    fn visit_assoc_item(&mut self, item: &'ast ast::AssocItem, context: visit::AssocCtxt) {
        let prior = std::mem::replace(
            &mut self.direct,
            item.attrs
                .iter()
                .map(|a| a as *const ast::Attribute)
                .collect(),
        );
        self.ancestry.push(item.id);
        visit::walk_assoc_item(self, item, context);
        self.ancestry.pop();
        self.direct = prior;
    }
    fn visit_foreign_item(&mut self, item: &'ast ast::ForeignItem) {
        let prior = std::mem::replace(
            &mut self.direct,
            item.attrs
                .iter()
                .map(|a| a as *const ast::Attribute)
                .collect(),
        );
        self.ancestry.push(item.id);
        visit::walk_item(self, item);
        self.ancestry.pop();
        self.direct = prior;
    }
}
pub fn stage_attributes(tcx: TyCtxt<'_>) -> Vec<StageAttribute> {
    let resolver = tcx.resolver_for_lowering();
    let krate = resolver.1.borrow();
    let mut visitor = Attributes {
        rows: vec![],
        ancestry: vec![ast::CRATE_NODE_ID],
        direct: krate
            .attrs
            .iter()
            .map(|a| a as *const ast::Attribute)
            .collect(),
    };
    visitor.visit_crate(&krate);
    visitor.rows
}

/// Original text capture serialization, independent of semantic selection.
pub mod storage {
    //! Lossless original expanded text storage between compiler callbacks.
    //! The actual owner/span/site metadata stays in the caller-held declarations.
    //! Every record binds that metadata and all text bytes before any selection.
    use crate::expanded::Declaration;
    use rustc_span::{SourceFileHash, SourceFileHashAlgorithm, Span};
    use serde_json::json;
    use std::fs::{File, OpenOptions};
    use std::io::{Read, Seek, SeekFrom, Write};
    use std::os::unix::fs::OpenOptionsExt;
    use std::path::PathBuf;

    struct Record {
        start: u64,
        length: u64,
        hash: SourceFileHash,
    }
    pub struct OriginalText {
        file: File,
        path: PathBuf,
        records: Vec<Record>,
        sealed_length: Option<u64>,
    }
    fn original_span(span: Span) -> serde_json::Value {
        let data = span.data_untracked();
        json!({"lo":data.lo.0,"hi":data.hi.0,"syntax_context":format!("{:?}",data.ctxt),
        "parent_local_def_id":data.parent.map(|parent|parent.local_def_index.as_u32())})
    }
    fn identity(decl: &Declaration) -> Vec<u8> {
        serde_json::to_vec(&json!({
        "node_id":decl.node_id.as_u32(),"local_def_id":decl.def.local_def_index.as_u32(),
        "span":original_span(decl.span),"ast_kind":decl.ast_kind,
        "tokens_available":decl.tokens_available,"body":decl.body,
        "impl_member_count":decl.impl_member_count,"body_tokens_present":decl.body_tokens.is_some(),
        "sites":decl.sites.iter().map(|site|json!({"kind":site.kind,
            "span":original_span(site.span),"ancestor":site.ancestor})).collect::<Vec<_>>()
    }))
    .expect("serialize actual expanded owner/span/site identity")
    }
    fn write_bytes(file: &mut File, bytes: &[u8]) {
        file.write_all(&(bytes.len() as u64).to_le_bytes())
            .expect("write original length");
        file.write_all(bytes)
            .expect("write complete original bytes");
    }
    fn read_bytes(file: &mut impl Read, remaining: &mut u64) -> Vec<u8> {
        assert!(*remaining >= 8, "original text length is truncated");
        let mut length = [0; 8];
        file.read_exact(&mut length).expect("read original length");
        *remaining -= 8;
        let length = u64::from_le_bytes(length);
        assert!(
            length <= *remaining,
            "original text exceeds its authenticated record"
        );
        let mut bytes = vec![0; usize::try_from(length).expect("original text length fits host")];
        file.read_exact(&mut bytes)
            .expect("read complete original bytes");
        *remaining -= length;
        bytes
    }
    impl OriginalText {
        pub fn create() -> Self {
            let destination = [
                "SIFR_BUILTIN_SOURCE_ENVELOPE",
                "SIFR_BUILTIN_CAPTURE",
                "SIFR_BUILTIN_SOURCE_BINDER",
            ]
            .iter()
            .find_map(std::env::var_os)
            .expect("explicit original capture destination");
            let mut path = PathBuf::from(destination);
            let name = path
                .file_name()
                .expect("capture filename")
                .to_string_lossy();
            path.set_file_name(format!(
                "{name}.expanded-original-{}.tmp",
                std::process::id()
            ));
            Self::open(path)
        }
        fn open(path: PathBuf) -> Self {
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&path)
                .expect("create exclusively owned original text storage");
            Self {
                file,
                path,
                records: vec![],
                sealed_length: None,
            }
        }
        pub fn store(&mut self, decl: &mut Declaration) {
            assert!(self.sealed_length.is_none(), "original text is sealed");
            let start = self.file.stream_position().expect("original record start");
            write_bytes(&mut self.file, &identity(decl));
            write_bytes(&mut self.file, decl.tokens.as_bytes());
            if let Some(body) = &decl.body_tokens {
                write_bytes(&mut self.file, body.as_bytes());
            }
            for site in &decl.sites {
                write_bytes(&mut self.file, site.tokens.as_bytes());
            }
            let end = self.file.stream_position().expect("original record end");
            self.file
                .seek(SeekFrom::Start(start))
                .expect("hash original record");
            let hash = SourceFileHash::new(
                SourceFileHashAlgorithm::Sha256,
                (&mut self.file).take(end - start),
            )
            .expect("hash all original bytes");
            self.file
                .seek(SeekFrom::Start(end))
                .expect("resume original serialization");
            self.records.push(Record {
                start,
                length: end - start,
                hash,
            });
            // Drop capacity as well as content while the compiler performs analysis.
            decl.tokens = String::new();
            if decl.body_tokens.is_some() {
                decl.body_tokens = Some(String::new());
            }
            for site in &mut decl.sites {
                site.tokens = String::new();
            }
        }
        pub fn seal(&mut self) {
            self.file
                .sync_all()
                .expect("durable complete original serialization");
            self.sealed_length = Some(self.file.metadata().expect("original metadata").len());
            let mut final_path = self.path.clone();
            final_path.set_extension("bin");
            assert!(
                !final_path.exists(),
                "original capture destination already exists"
            );
            std::fs::rename(&self.path, &final_path)
                .expect("publish complete original text serialization");
            self.file = File::open(&final_path).expect("read-only original text storage");
            self.path = final_path;
        }
        pub fn restore(mut self, declarations: &mut [Declaration]) {
            assert_eq!(
                self.records.len(),
                declarations.len(),
                "complete original owner inventory"
            );
            assert_eq!(
                self.sealed_length,
                Some(self.file.metadata().expect("original metadata").len()),
                "complete original byte length"
            );
            for (record, decl) in self.records.iter().zip(declarations) {
                self.file
                    .seek(SeekFrom::Start(record.start))
                    .expect("authenticate original record");
                let mut bytes =
                    vec![0; usize::try_from(record.length).expect("original record fits host")];
                self.file
                    .read_exact(&mut bytes)
                    .expect("read complete original record");
                let hash = SourceFileHash::new_in_memory(SourceFileHashAlgorithm::Sha256, &bytes);
                assert_eq!(hash, record.hash, "original owner/text record changed");
                // Decode the same authenticated bytes, rather than rereading the file.
                let mut original = std::io::Cursor::new(bytes);
                let mut remaining = record.length;
                assert_eq!(
                    read_bytes(&mut original, &mut remaining),
                    identity(decl),
                    "actual original owner/span/site identity changed"
                );
                decl.tokens = String::from_utf8(read_bytes(&mut original, &mut remaining))
                    .expect("original UTF-8");
                if decl.body_tokens.is_some() {
                    decl.body_tokens = Some(
                        String::from_utf8(read_bytes(&mut original, &mut remaining))
                            .expect("original body UTF-8"),
                    );
                }
                for site in &mut decl.sites {
                    site.tokens = String::from_utf8(read_bytes(&mut original, &mut remaining))
                        .expect("original site UTF-8");
                }
                assert_eq!(remaining, 0, "all original bytes restored exhaustively");
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::expanded::Site;
        use rustc_ast::NodeId;
        use rustc_hir::def_id::{DefIndex, LocalDefId};
        use rustc_span::DUMMY_SP;
        fn original(label: &str) -> (OriginalText, Declaration) {
            let path = std::env::temp_dir().join(format!(
                "sifr-expanded-original-{}-{label}.tmp",
                std::process::id()
            ));
            let declaration = Declaration {
                def: LocalDefId {
                    local_def_index: DefIndex::from_u32(7),
                },
                node_id: NodeId::from_u32(19),
                ast_kind: "associated function".into(),
                attrs: Default::default(),
                tokens_available: false,
                span: DUMMY_SP,
                tokens: "fn café() {\r\n println!(\"雪\"); }".into(),
                body: true,
                impl_member_count: None,
                body_tokens: Some("{\n café();\n}".into()),
                sites: vec![
                    Site {
                        kind: "call".into(),
                        tokens: "café()".into(),
                        span: DUMMY_SP,
                        ancestor: None,
                    },
                    Site {
                        kind: "unary:Not".into(),
                        tokens: "!false".into(),
                        span: DUMMY_SP,
                        ancestor: Some(0),
                    },
                ],
            };
            (OriginalText::open(path), declaration)
        }
        #[test]
        fn complete_original_text_and_identity_round_trip() {
            let (mut storage, mut decl) = original("round-trip");
            let expected = (
                identity(&decl),
                decl.tokens.clone(),
                decl.body_tokens.clone(),
                decl.sites
                    .iter()
                    .map(|site| site.tokens.clone())
                    .collect::<Vec<_>>(),
            );
            storage.store(&mut decl);
            assert!(decl.tokens.is_empty() && decl.sites.iter().all(|site| site.tokens.is_empty()));
            assert_eq!(identity(&decl), expected.0);
            storage.seal();
            storage.restore(std::slice::from_mut(&mut decl));
            assert_eq!(identity(&decl), expected.0);
            assert_eq!(decl.tokens, expected.1);
            assert_eq!(decl.body_tokens, expected.2);
            assert_eq!(
                decl.sites
                    .iter()
                    .map(|site| site.tokens.clone())
                    .collect::<Vec<_>>(),
                expected.3
            );
        }
        #[test]
        #[should_panic(expected = "original owner/text record changed")]
        fn changed_original_bytes_reject_before_restore() {
            let (mut storage, mut decl) = original("corrupt");
            storage.store(&mut decl);
            storage.seal();
            let mut writer = OpenOptions::new()
                .write(true)
                .open(&storage.path)
                .expect("test owned file");
            writer
                .seek(SeekFrom::Start(storage.records[0].length - 1))
                .expect("test byte");
            writer.write_all(&[0]).expect("mutate actual original byte");
            writer.sync_all().expect("durable mutation");
            storage.restore(std::slice::from_mut(&mut decl));
        }
        #[test]
        #[should_panic(expected = "actual original owner/span/site identity changed")]
        fn changed_site_ancestry_rejects_authenticated_text() {
            let (mut storage, mut decl) = original("ancestry");
            storage.store(&mut decl);
            storage.seal();
            decl.sites[1].ancestor = None;
            storage.restore(std::slice::from_mut(&mut decl));
        }
        #[test]
        #[should_panic(expected = "actual original owner/span/site identity changed")]
        fn changed_original_span_parent_rejects_authenticated_text() {
            let (mut storage, mut decl) = original("span-parent");
            storage.store(&mut decl);
            storage.seal();
            decl.span = decl.span.with_parent(Some(decl.def));
            storage.restore(std::slice::from_mut(&mut decl));
        }
    }
}

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
    original_text: crate::expanded_storage::OriginalText,
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
pub fn capture(tcx: TyCtxt<'_>) -> (Vec<Declaration>, crate::expanded_storage::OriginalText) {
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
        original_text: crate::expanded_storage::OriginalText::create(),
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

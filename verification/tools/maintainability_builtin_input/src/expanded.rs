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
}
impl<'ast> Visitor<'ast> for Expanded {
    fn visit_item(&mut self, item: &'ast ast::Item) {
        if let Some(def) = self.owners.get(&item.id).copied() {
            self.declarations.push(Declaration {
                def,
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
            });
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
            self.declarations.push(Declaration {
                def,
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
            });
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
pub fn capture(tcx: TyCtxt<'_>) -> Vec<Declaration> {
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
    };
    visitor.visit_crate(&resolver.1.borrow());
    visitor.declarations
}

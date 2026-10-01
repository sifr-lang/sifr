use crate::expanded::Declaration;
use rustc_hir::{
    self as hir,
    intravisit::{self, Visitor},
};
use rustc_middle::ty::{self, TyCtxt};
use rustc_span::{
    Span,
    hygiene::{ExpnId, ExpnKind, MacroKind},
};
use serde_json::{Value, json};
fn span(tcx: TyCtxt<'_>, s: Span) -> Value {
    let sm = tcx.sess.source_map();
    let lo = sm.lookup_char_pos(s.lo());
    let hi = sm.lookup_char_pos(s.hi());
    json!({"file":lo.file.name.prefer_local_unconditionally().to_string(),"start":[lo.line,lo.col.0],"end":[hi.line,hi.col.0],"quality":if s.from_expansion(){"coarse-generated-anchor"}else{"exact-source"}})
}
pub fn chain(tcx: TyCtxt<'_>, s: Span) -> Vec<Value> {
    expansion_chain(tcx, s.ctxt().outer_expn())
}
pub fn expansion_chain(tcx: TyCtxt<'_>, mut exp: ExpnId) -> Vec<Value> {
    let mut result = vec![];
    while exp != ExpnId::root() {
        let data = exp.expn_data();
        let builtin = data
            .macro_def_id
            .is_some_and(|def| rustc_hir::find_attr!(tcx, def, RustcBuiltinMacro { .. }));
        result.push(json!({"kind":format!("{:?}",data.kind),"macro_identity":data.macro_def_id.map(|def|crate::identity::path(tcx,def)),"macro":data.macro_def_id.map(|def|tcx.def_path_str(def)),"builtin":builtin,"call_site":span(tcx,data.call_site),"definition_site":span(tcx,data.def_site)}));
        exp = data.parent;
    }
    result
}
fn callable_trait(tcx: TyCtxt<'_>, def: hir::def_id::DefId) -> Option<hir::def_id::DefId> {
    tcx.trait_of_assoc(def).or_else(|| {
        tcx.impl_of_assoc(def).and_then(|implementation| {
            tcx.impl_is_of_trait(implementation).then(|| {
                tcx.impl_trait_ref(implementation)
                    .instantiate_identity()
                    .skip_norm_wip()
                    .def_id
            })
        })
    })
}
fn callable_descriptor(tcx: TyCtxt<'_>, def: hir::def_id::DefId) -> Value {
    let trait_origin = callable_trait(tcx, def);
    let kind = if tcx.trait_of_assoc(def).is_some() {
        "trait-member"
    } else if trait_origin.is_some() {
        "trait-implementation"
    } else if tcx.impl_of_assoc(def).is_some() {
        "inherent-member"
    } else {
        "function"
    };
    json!({"path":tcx.def_path_str(def),"kind":kind,"trait":trait_origin.map(|def|tcx.def_path_str(def)),"signature":tcx.fn_sig(def).instantiate_identity().skip_norm_wip().to_string(),"origin_paths":if def.is_local(){None}else{Some(tcx.used_crate_source(def.krate).paths().map(|path|path.to_string_lossy().to_string()).collect::<Vec<_>>())}})
}
struct Typed<'tcx> {
    tcx: TyCtxt<'tcx>,
    typeck: &'tcx ty::TypeckResults<'tcx>,
    nodes: Vec<Value>,
    parent: Option<usize>,
    catalog: std::collections::BTreeMap<String, Value>,
}
impl<'tcx> Visitor<'tcx> for Typed<'tcx> {
    fn visit_expr(&mut self, expr: &'tcx hir::Expr<'tcx>) {
        let mut target = None;
        let mut receiver = None;
        let mut args = None;
        let mut generic_args = None;
        let kind = match expr.kind {
            hir::ExprKind::Call(callee, arguments) => {
                let callable = self.typeck.expr_ty(callee);
                if let ty::FnDef(def, generics) = callable.kind() {
                    target = Some(*def);
                    generic_args = Some(*generics);
                    args = Some(format!("{generics:?}"));
                    if self.tcx.trait_of_assoc(*def).is_some() {
                        receiver = arguments.first();
                    }
                }
                Some("call".into())
            }
            hir::ExprKind::MethodCall(_, recv, _, _) => {
                target = self.typeck.type_dependent_def_id(expr.hir_id);
                receiver = Some(recv);
                Some("method".into())
            }
            hir::ExprKind::Binary(op, left, _) => {
                target = self.typeck.type_dependent_def_id(expr.hir_id);
                receiver = Some(left);
                Some(format!("binary:{:?}", op.node))
            }
            hir::ExprKind::Unary(op, recv) => {
                target = self.typeck.type_dependent_def_id(expr.hir_id);
                receiver = Some(recv);
                Some(format!("unary:{op:?}"))
            }
            _ => None,
        };
        if generic_args.is_none() {
            generic_args = self.typeck.node_args_opt(expr.hir_id);
            args = generic_args.map(|a| format!("{a:?}"));
        }
        let parent = self.parent;
        if let Some(kind) = kind {
            let index = self.nodes.len();
            self.parent = Some(index);
            let scalar =
                target.is_none() && (kind.starts_with("binary:") || kind.starts_with("unary:"));
            let trait_origin = target.and_then(|def| callable_trait(self.tcx, def));
            let implementation = target.zip(generic_args).and_then(|(def, args)| {
                match ty::Instance::try_resolve(
                    self.tcx,
                    ty::TypingEnv::post_analysis(self.tcx, expr.hir_id.owner.def_id),
                    def,
                    args,
                ) {
                    Ok(instance) => instance,
                    Err(_) => self
                        .tcx
                        .sess
                        .dcx()
                        .fatal("builtin capability instance resolution failed"),
                }
            });
            let signature = target.map(|def| {
                self.tcx
                    .fn_sig(def)
                    .instantiate_identity()
                    .skip_norm_wip()
                    .to_string()
            });
            let implementation_def = implementation.and_then(|instance| {
                if let ty::InstanceKind::Item(def) = instance.def {
                    Some(def)
                } else {
                    None
                }
            });
            for def in target.into_iter().chain(implementation_def) {
                let descriptor = callable_descriptor(self.tcx, def);
                let key = self.tcx.def_path_str(def);
                if let Some(previous) = self.catalog.insert(key, descriptor.clone()) {
                    if previous != descriptor {
                        self.tcx
                            .sess
                            .dcx()
                            .fatal("ambiguous callable declaration identity");
                    }
                }
            }
            self.nodes.push(json!({"token_sequence":crate::identity::tokens(&rustc_hir_pretty::expr_to_string(&(&self.tcx as &dyn hir::intravisit::HirTyCtxt),expr)),"tokens":rustc_hir_pretty::expr_to_string(&(&self.tcx as &dyn hir::intravisit::HirTyCtxt),expr),"kind":kind,"ancestor":parent,"implementation":implementation.and_then(|instance|if let ty::InstanceKind::Item(def)=instance.def{Some(self.tcx.def_path_str(def))}else{None}),"instance_kind":implementation.map(|instance|match instance.def{ty::InstanceKind::Item(_)=>"static",ty::InstanceKind::Virtual(..)=>"dynamic",ty::InstanceKind::Intrinsic(..)=>"intrinsic",_=>"compiler-shim"}),"target":target.map(|def|self.tcx.def_path_str(def)),"target_origin_paths":target.filter(|def|!def.is_local()).map(|def|self.tcx.used_crate_source(def.krate).paths().map(|path|path.to_string_lossy().to_string()).collect::<Vec<_>>()),"implementation_origin_paths":implementation.and_then(|instance|if let ty::InstanceKind::Item(def)=instance.def{if !def.is_local(){Some(self.tcx.used_crate_source(def.krate).paths().map(|path|path.to_string_lossy().to_string()).collect::<Vec<_>>())}else{None}}else{None}),"signature":signature,"trait":trait_origin.map(|def|self.tcx.def_path_str(def)),"generics":args,"result_type":self.typeck.expr_ty(expr).to_string(),"receiver":receiver.map(|recv|self.typeck.expr_ty(recv).to_string()),"adjustments":receiver.map(|recv|self.typeck.expr_adjustments(recv).iter().map(|a|json!({"kind":format!("{:?}",a.kind),"target":a.target.to_string()})).collect::<Vec<_>>()),"span":span(self.tcx,expr.span),"expansion_chain":chain(self.tcx,expr.span),"disposition":if scalar {"scalar-operation"}else if matches!(implementation.map(|instance|instance.def),Some(ty::InstanceKind::Virtual(..))){"dynamic-trait-call"}else if target.is_some() && trait_origin.is_some() && implementation.is_none(){"generic-trait-call"}else if target.is_some(){"resolved-call"}else{"unsupported-call"}}));
        }
        intravisit::walk_expr(self, expr);
        self.parent = parent;
    }
}
pub fn selected(tcx: TyCtxt<'_>, decl: &Declaration) -> bool {
    let expansion = decl.span.ctxt().outer_expn().expn_data();
    if !matches!(expansion.kind, ExpnKind::Macro(MacroKind::Derive, _)) {
        return false;
    }
    std::env::var("SIFR_BUILTIN_SOURCE_SUFFIX").is_ok_and(|source| {
        tcx.sess
            .source_map()
            .lookup_char_pos(decl.span.lo())
            .file
            .name
            .prefer_local_unconditionally()
            .to_string()
            .ends_with(&source)
    })
}
pub fn capture(
    tcx: TyCtxt<'_>,
    ast: &[Declaration],
) -> (Vec<Value>, std::collections::BTreeMap<String, Value>) {
    let mut records = vec![];
    let mut catalog = std::collections::BTreeMap::new();
    for decl in ast.iter().filter(|decl| selected(tcx, decl)) {
        let did = decl.def.to_def_id();
        let mut sites = vec![];
        let mut body_tokens = None;
        if let Some(bodyid) = tcx.hir_maybe_body_owned_by(decl.def) {
            let body = bodyid;
            body_tokens = Some(crate::identity::tokens(&rustc_hir_pretty::expr_to_string(
                &(&tcx as &dyn hir::intravisit::HirTyCtxt),
                body.value,
            )));
            let mut visitor = Typed {
                tcx,
                typeck: tcx.typeck(decl.def),
                nodes: vec![],
                parent: None,
                catalog: Default::default(),
            };
            visitor.visit_body(body);
            sites = visitor.nodes;
            catalog.extend(visitor.catalog);
        }
        let ast_sites=decl.sites.iter().map(|site|json!({"kind":site.kind,"token_sequence":crate::identity::tokens(&site.tokens),"tokens":site.tokens,"ancestor":site.ancestor,"span":span(tcx,site.span),"expansion_chain":chain(tcx,site.span)})).collect::<Vec<_>>();
        let parent = tcx.parent(did);
        let implementation_owner = if matches!(tcx.def_kind(did), hir::def::DefKind::Impl { .. }) {
            did
        } else {
            parent
        };
        let receiver = tcx
            .type_of(implementation_owner)
            .instantiate_identity()
            .skip_norm_wip();
        let trait_identity = Some(crate::identity::path(
            tcx,
            tcx.impl_trait_ref(implementation_owner)
                .instantiate_identity()
                .skip_norm_wip()
                .def_id,
        ));
        records.push(json!({"structural_identity":tcx.def_path(did).to_string_no_crate_verbose(),"generic_bounds":crate::identity::bounds(tcx,implementation_owner),"generic_parameters":tcx.generics_of(implementation_owner).own_params.iter().map(|param|json!({"name":param.name.to_string(),"kind":format!("{:?}",param.kind)})).collect::<Vec<_>>(),"generic_predicates":tcx.predicates_of(implementation_owner).predicates.iter().map(|(predicate,_)|predicate.to_string()).collect::<Vec<_>>(),"receiver_identity":crate::identity::ty(tcx,receiver),"trait_identity":trait_identity,"visibility":match tcx.visibility(did){ty::Visibility::Public=>json!("Public"),ty::Visibility::Restricted(module)=>json!({"restricted_to":crate::identity::path(tcx,module)})},"canonical_signature":if matches!(tcx.def_kind(did),hir::def::DefKind::AssocFn){Some(crate::identity::signature(tcx,did))}else{None},"token_sequence":crate::identity::tokens(&decl.tokens),"owner":tcx.def_path_str(did),"owner_kind":format!("{:?}",tcx.def_kind(did)),"parent":tcx.def_path_str(parent),"trait":tcx.trait_of_assoc(did).map(|def|tcx.def_path_str(def)),"signature":if matches!(tcx.def_kind(did),hir::def::DefKind::Fn|hir::def::DefKind::AssocFn){Some(tcx.fn_sig(did).instantiate_identity().skip_norm_wip().to_string())}else{None},"tokens":decl.tokens,"ast_impl_member_count":decl.impl_member_count,"hir_members":if matches!(tcx.def_kind(did),hir::def::DefKind::Impl{..}){Some(tcx.associated_items(did).in_definition_order().map(|item|tcx.def_path_str(item.def_id)).collect::<Vec<_>>())}else{None},"hir_impl_member_count":if matches!(tcx.def_kind(did),hir::def::DefKind::Impl{..}){Some(tcx.associated_items(did).in_definition_order().count())}else{None},"ast_body_tokens":decl.body_tokens.as_ref().map(|text|crate::identity::tokens(text)),"hir_body_tokens":body_tokens,"ast_body":decl.body,"hir_body":tcx.hir_maybe_body_owned_by(decl.def).is_some(),"span":span(tcx,decl.span),"expansion_chain":chain(tcx,decl.span),"ast_sites":ast_sites,"typed_sites":sites}));
    }
    (records, catalog)
}

pub fn other_macros(tcx: TyCtxt<'_>, ast: &[Declaration]) -> Vec<Value> {
    ast.iter().filter(|d|matches!(d.span.ctxt().outer_expn().expn_data().kind,ExpnKind::Macro(MacroKind::Bang,_))).map(|d|json!({"owner":tcx.def_path_str(d.def.to_def_id()),"expansion_chain":chain(tcx,d.span),"tokens":d.tokens})).collect()
}

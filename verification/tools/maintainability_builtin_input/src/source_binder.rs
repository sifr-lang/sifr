//! Original local declaration HIR diagnostic. Never a semantic adapter export.
use rustc_hir::{
    self as hir,
    def_id::DefId,
    intravisit::{self, Visitor},
};
use rustc_middle::{middle::resolve_bound_vars::ResolvedArg, ty::TyCtxt};
use rustc_span::Span;
use serde_json::{Value, json};

fn definition(tcx: TyCtxt<'_>, def: DefId) -> Value {
    json!({"crate":format!("{:?}",tcx.stable_crate_id(def.krate)),"hash":format!("{:?}",tcx.def_path_hash(def)),"raw":format!("{def:?}"),"path":tcx.def_path_str(def),"kind":format!("{:?}",tcx.def_kind(def))})
}
fn source(tcx: TyCtxt<'_>, span: Span) -> Value {
    if span.is_dummy() {
        return json!({"kind":"synthetic","hygiene":format!("{:?}",span.ctxt())});
    }
    let map = tcx.sess.source_map();
    let first = map.lookup_source_file(span.lo());
    let last = map.lookup_source_file(span.hi());
    if first.start_pos != last.start_pos {
        return json!({"kind":"cross-file","hygiene":format!("{:?}",span.ctxt())});
    }
    json!({"kind":if span.from_expansion(){"generated"}else{"original"},"file":first.name.prefer_local_unconditionally().to_string(),"start":first.original_relative_byte_pos(span.lo()).0,"end":first.original_relative_byte_pos(span.hi()).0,"hygiene":format!("{:?}",span.ctxt()),"snippet":map.span_to_snippet(span).ok()})
}
fn resolved(tcx: TyCtxt<'_>, id: hir::HirId) -> Value {
    match tcx.named_bound_var(id) {
        Some(ResolvedArg::EarlyBound(d)) => {
            json!({"kind":"EarlyBound","target":definition(tcx,d.to_def_id())})
        }
        Some(ResolvedArg::LateBound(depth, index, d)) => {
            json!({"kind":"LateBound","depth":depth.as_u32(),"index":index,"target":definition(tcx,d.to_def_id())})
        }
        Some(ResolvedArg::Free(scope, d)) => {
            json!({"kind":"Free","scope":definition(tcx,scope.to_def_id()),"target":definition(tcx,d.to_def_id())})
        }
        Some(ResolvedArg::StaticLifetime) => json!({"kind":"StaticLifetime"}),
        Some(ResolvedArg::Error(_)) => json!({"kind":"Error"}),
        None => json!({"kind":"Absent"}),
    }
}
struct Signature<'tcx> {
    tcx: TyCtxt<'tcx>,
    parameters: Vec<Value>,
    lifetimes: Vec<Value>,
    binders: Vec<Value>,
    traits: Vec<Value>,
}
impl<'tcx> Signature<'tcx> {
    fn new(tcx: TyCtxt<'tcx>) -> Self {
        Self {
            tcx,
            parameters: vec![],
            lifetimes: vec![],
            binders: vec![],
            traits: vec![],
        }
    }
    fn binder(
        &mut self,
        id: hir::HirId,
        kind: &str,
        span: Span,
        params: &[hir::GenericParam<'tcx>],
    ) {
        // Query only nodes present in the compiler's supported binder map.
        let vars = self
            .tcx
            .late_bound_vars_map(id.owner)
            .get(&id.local_id)
            .map(|_| self.tcx.late_bound_vars(id));
        self.binders.push(json!({"id":format!("{id:?}"),"kind":kind,"source":source(self.tcx,span),"parameters":params.iter().map(|p|definition(self.tcx,p.def_id.to_def_id())).collect::<Vec<_>>(),"variables":vars.map(|v|variables(self.tcx,v)),"compiler_map_disposition":if vars.is_some(){"present"}else{"not-supported-node"}}));
    }
}
impl<'tcx> Visitor<'tcx> for Signature<'tcx> {
    fn visit_generic_param(&mut self, p: &'tcx hir::GenericParam<'tcx>) {
        self.parameters.push(json!({"identity":definition(self.tcx,p.def_id.to_def_id()),"parent":definition(self.tcx,self.tcx.parent(p.def_id.to_def_id())),"hir":format!("{:?}",p.hir_id),"source":source(self.tcx,p.span),"name":format!("{:?}",p.name),"kind":format!("{:?}",p.kind),"origin":format!("{:?}",p.source),"resolved":resolved(self.tcx,p.hir_id)}));
        intravisit::walk_generic_param(self, p);
    }
    fn visit_lifetime(&mut self, l: &'tcx hir::Lifetime) {
        self.lifetimes.push(json!({"hir":format!("{:?}",l.hir_id),"source":source(self.tcx,l.ident.span),"syntax":format!("{:?}",l.syntax),"origin":format!("{:?}",l.source),"kind":format!("{:?}",l.kind),"target":match l.kind{hir::LifetimeKind::Param(d)=>Some(definition(self.tcx,d.to_def_id())),_=>None},"resolved":resolved(self.tcx,l.hir_id)}));
    }
    fn visit_poly_trait_ref(&mut self, p: &'tcx hir::PolyTraitRef<'tcx>) {
        let id = p.trait_ref.hir_ref_id;
        self.binder(id, "PolyTraitRef", p.span, p.bound_generic_params);
        self.traits.push(json!({"hir":format!("{id:?}"),"source":source(self.tcx,p.trait_ref.path.span),"resolved":match p.trait_ref.path.res{hir::def::Res::Def(_,d)=>Some(definition(self.tcx,d)),_=>None},"declaration_source":match p.trait_ref.path.res{hir::def::Res::Def(_,d)=>Some(source(self.tcx,self.tcx.def_span(d))),_=>None},"declaration_name_source":match p.trait_ref.path.res{hir::def::Res::Def(_,d)=>self.tcx.def_ident_span(d).map(|span|source(self.tcx,span)),_=>None},"canonical_path":match p.trait_ref.path.res{hir::def::Res::Def(_,d)=>Some(crate::identity::path(self.tcx,d)),_=>None},"modifiers":format!("{:?}",p.modifiers)}));
        intravisit::walk_poly_trait_ref(self, p);
    }
    fn visit_ty(&mut self, t: &'tcx hir::Ty<'tcx, hir::AmbigArg>) {
        if let hir::TyKind::FnPtr(f) = t.kind {
            self.binder(t.hir_id, "FnPtr", t.span, f.generic_params);
        }
        intravisit::walk_ty(self, t);
    }
    fn visit_where_predicate(&mut self, p: &'tcx hir::WherePredicate<'tcx>) {
        if let hir::WherePredicateKind::BoundPredicate(b) = p.kind {
            self.binder(
                p.hir_id,
                "WhereBoundPredicate",
                p.span,
                b.bound_generic_params,
            );
        }
        intravisit::walk_where_predicate(self, p);
    }
}
pub fn capture(tcx: TyCtxt<'_>) -> Value {
    let mut owners = vec![];
    // Independent local owner enumeration happens before any range selection.
    for id in tcx.hir_crate_items(()).owners() {
        let node = tcx.hir_node_by_def_id(id.def_id);
        let mut signature = Signature::new(tcx);
        let span = match node {
            hir::Node::Item(i) => {
                signature.visit_item(i);
                i.span
            }
            hir::Node::ImplItem(i) => {
                signature.visit_impl_item(i);
                i.span
            }
            hir::Node::TraitItem(i) => {
                signature.visit_trait_item(i);
                i.span
            }
            hir::Node::ForeignItem(i) => {
                signature.visit_foreign_item(i);
                i.span
            }
            _ => continue,
        };
        let did = id.def_id.to_def_id();
        if matches!(
            tcx.def_kind(did),
            hir::def::DefKind::AssocFn | hir::def::DefKind::Fn
        ) {
            signature.binder(
                tcx.local_def_id_to_hir_id(id.def_id),
                "Function",
                span,
                node.generics().map_or(&[], |g| g.params),
            );
        }
        owners.push(json!({"identity":definition(tcx,did),"parent":definition(tcx,tcx.parent(did)),"source":source(tcx,span),"parameters":signature.parameters,"binders":signature.binders,"lifetime_occurrences":signature.lifetimes,"trait_constraints":signature.traits,"raw_hir":format!("{node:?}"),"generics":if matches!(tcx.def_kind(did),hir::def::DefKind::AssocFn|hir::def::DefKind::Fn|hir::def::DefKind::Impl{..}){Some(crate::semantic::generics(tcx,did))}else{None},"signature":if matches!(tcx.def_kind(did),hir::def::DefKind::AssocFn|hir::def::DefKind::Fn){Some(crate::semantic::signature(tcx,did))}else{None},"predicates":if matches!(tcx.def_kind(did),hir::def::DefKind::AssocFn|hir::def::DefKind::Fn|hir::def::DefKind::Impl{..}){Some(predicates(tcx,did))}else{None}}));
    }
    let mut calls = vec![];
    if let Ok(suffix) = std::env::var("SIFR_BUILTIN_SOURCE_BINDER_CALL_SUFFIX") {
        for owner in tcx.hir_body_owners() {
            let sp = tcx.def_span(owner);
            if source(tcx, sp)["file"]
                .as_str()
                .is_some_and(|f| f.ends_with(&suffix))
            {
                let mut visitor = Calls {
                    tcx,
                    owner,
                    records: vec![],
                };
                visitor.visit_body(tcx.hir_body_owned_by(owner));
                calls.extend(visitor.records);
            }
        }
    }
    let mut cfg = tcx
        .sess
        .config
        .iter()
        .map(|(k, v)| json!({"key":k.to_string(),"value":v.map(|v|v.to_string())}))
        .collect::<Vec<_>>();
    cfg.sort_by_cached_key(|a| a.to_string());
    let sources=tcx.sess.source_map().files().iter().filter_map(|f|f.src.as_ref().map(|text|json!({"file":f.name.prefer_local_unconditionally().to_string(),"normalized_text":text.as_str(),"normalization":format!("{:?}",f.normalized_pos),"start_pos":f.start_pos.0,"source_hash":format!("{:?}",f.src_hash)}))).collect::<Vec<_>>();
    json!({"schema":"sifr-maintainability-source-binder-capture-v1","fragment":"sifr-maintainability-source-binder-v1","crate":format!("{:?}",tcx.stable_crate_id(hir::def_id::LOCAL_CRATE)),"cfg":cfg,"calls":calls,"declaration_owners":owners,"source_files":sources,"stage":"rustc-after-analysis","semantic_export":false})
}

struct Calls<'tcx> {
    tcx: TyCtxt<'tcx>,
    owner: hir::def_id::LocalDefId,
    records: Vec<Value>,
}
impl<'tcx> Visitor<'tcx> for Calls<'tcx> {
    fn visit_expr(&mut self, e: &'tcx hir::Expr<'tcx>) {
        if let hir::ExprKind::MethodCall(..) = e.kind {
            if let Some(d) = self.tcx.typeck(self.owner).type_dependent_def_id(e.hir_id) {
                self.records.push(json!({"source":source(self.tcx,e.span),"target":definition(self.tcx,d),"parent":definition(self.tcx,self.tcx.parent(d)),"owner":definition(self.tcx,self.owner.to_def_id())}));
            }
        }
        intravisit::walk_expr(self, e);
    }
}

fn predicates(tcx: TyCtxt<'_>, def: DefId) -> Value {
    let p = tcx.predicates_of(def);
    let own=p.predicates.iter().map(|(clause,span)|{
        let binder=clause.kind();
        let mut fact=crate::semantic::clause_fact(tcx,*clause);
        if let rustc_middle::ty::ClauseKind::Projection(projection)=binder.skip_binder() {
            fact=json!({"binders":crate::semantic::binders(tcx,binder.bound_vars()),"fact":{"projection":definition(tcx,projection.def_id()),"arguments":crate::semantic::args(tcx,projection.projection_term.args),"term":projection.term.as_type().map(|t|crate::semantic::ty(tcx,t)),"raw_term":format!("{:?}",projection.term)}});
        }
        json!({"source":source(tcx,*span),"fact":fact,"raw":format!("{clause:?}")})
    }).collect::<Vec<_>>();
    json!({"parent":p.parent.map(|d|predicates(tcx,d)),"own":own})
}

fn variables(
    tcx: TyCtxt<'_>,
    vars: &rustc_middle::ty::List<rustc_middle::ty::BoundVariableKind>,
) -> Vec<Value> {
    use rustc_middle::ty::{BoundRegionKind, BoundVariableKind};
    vars.iter()
        .enumerate()
        .map(|(ordinal, var)| match var {
            BoundVariableKind::Region(BoundRegionKind::Named(d)) => {
                json!({"kind":"NamedRegion","ordinal":ordinal,"declaration":definition(tcx,d)})
            }
            BoundVariableKind::Region(BoundRegionKind::Anon) => {
                json!({"kind":"AnonymousRegion","ordinal":ordinal})
            }
            other => json!({"kind":"Unsupported","ordinal":ordinal,"raw":format!("{other:?}")}),
        })
        .collect()
}

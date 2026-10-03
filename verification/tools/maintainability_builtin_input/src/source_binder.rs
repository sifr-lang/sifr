//! Original local declaration HIR diagnostic. Never a semantic adapter export.
use rustc_hir::{
    self as hir,
    def_id::DefId,
    intravisit::{self, Visitor},
};
use rustc_middle::{middle::resolve_bound_vars::ResolvedArg, ty::TyCtxt};
use rustc_span::Span;
use serde_json::{Value, json};

pub(crate) fn definition(tcx: TyCtxt<'_>, def: DefId) -> Value {
    json!({"crate":format!("{:?}",tcx.stable_crate_id(def.krate)),"hash":format!("{:?}",tcx.def_path_hash(def)),"raw":format!("{def:?}"),"path":tcx.def_path_str(def),"kind":format!("{:?}",tcx.def_kind(def))})
}
pub(crate) fn source(tcx: TyCtxt<'_>, span: Span) -> Value {
    if span.is_dummy() {
        return json!({"kind":"synthetic","hygiene":format!("{:?}",span.ctxt())});
    }
    let map = tcx.sess.source_map();
    let first = map.lookup_source_file(span.lo());
    let last = map.lookup_source_file(span.hi());
    if first.start_pos != last.start_pos {
        return json!({"kind":"cross-file","hygiene":format!("{:?}",span.ctxt())});
    }
    json!({"kind":if span.from_expansion(){"generated"}else{"original"},"file":first.name.prefer_local_unconditionally().to_string(),"start":first.original_relative_byte_pos(span.lo()).0,"end":first.original_relative_byte_pos(span.hi()).0,"hygiene":format!("{:?}",span.ctxt()),"normalized_interval":[first.relative_position(span.lo()).0,first.relative_position(span.hi()).0],"source_hash":format!("{:?}",first.src_hash),"original_source_hash":first.src_hash.to_string(),"source_file_start_pos":first.start_pos.0,"snippet":map.span_to_snippet(span).ok()})
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
        self.parameters.push(json!({"identity":definition(self.tcx,p.def_id.to_def_id()),"parent":definition(self.tcx,self.tcx.parent(p.def_id.to_def_id())),"hir":format!("{:?}",p.hir_id),"source":source(self.tcx,p.span),"name_source":self.tcx.def_ident_span(p.def_id).map(|span|source(self.tcx,span)),"name":format!("{:?}",p.name),"kind":format!("{:?}",p.kind),"origin":format!("{:?}",p.source),"source_parameter_disposition":match p.kind {hir::GenericParamKind::Lifetime{kind:hir::LifetimeParamKind::Explicit}=>"explicit-source-parameter",hir::GenericParamKind::Lifetime{..}=>"compiler-elided-lifetime-parameter",hir::GenericParamKind::Type{synthetic:true,..}=>"compiler-implicit-impl-trait-parameter",_=>"explicit-source-parameter"},"resolved":resolved(self.tcx,p.hir_id)}));
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
        let generic_parameter_order = generic_order(tcx, did);
        let has_generic_semantics = !generic_parameter_order.is_null();
        owners.push(json!({"macro_rules":match node{hir::Node::Item(i)=>match i.kind{hir::ItemKind::Macro(_,d,_)=>Some(d.macro_rules),_=>None},_=>None},"source_owner_kind":source_owner_kind(tcx.def_kind(did)),"identity":definition(tcx,did),"local_def_id":format!("{:?}",id.def_id),"hir_id":format!("{:?}",tcx.local_def_id_to_hir_id(id.def_id)),"name_source":tcx.def_ident_span(did).map(|span|source(tcx,span)),"implemented_trait":if matches!(tcx.def_kind(did),hir::def::DefKind::Impl{of_trait:true}){Some(definition(tcx,tcx.impl_trait_ref(did).instantiate_identity().skip_norm_wip().def_id))}else{None},"parent":definition(tcx,tcx.parent(did)),"source":source(tcx,span),"generic_parameter_order":generic_parameter_order,"implemented_trait_source":if matches!(tcx.def_kind(did),hir::def::DefKind::Impl{of_trait:true}){Some(source(tcx,tcx.def_span(tcx.impl_trait_ref(did).instantiate_identity().skip_norm_wip().def_id)))}else{None},"implemented_trait_name_source":if matches!(tcx.def_kind(did),hir::def::DefKind::Impl{of_trait:true}){tcx.def_ident_span(tcx.impl_trait_ref(did).instantiate_identity().skip_norm_wip().def_id).map(|span|source(tcx,span))}else{None},"parameters":signature.parameters,"binders":signature.binders,"lifetime_occurrences":signature.lifetimes,"trait_constraints":signature.traits,"raw_hir":format!("{node:?}"),"generics":if has_generic_semantics{Some(crate::semantic::generics(tcx,did))}else{None},"signature":if matches!(tcx.def_kind(did),hir::def::DefKind::AssocFn|hir::def::DefKind::Fn){Some(crate::semantic::signature(tcx,did))}else{None},"predicates":if has_generic_semantics{Some(predicates(tcx,did))}else{None}}));
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
    let sources=tcx.sess.source_map().files().iter().map(|f|json!({"file":f.name.prefer_local_unconditionally().to_string(),"normalized_text":f.src.as_ref().map(|text|text.as_str()),"normalization":format!("{:?}",f.normalized_pos),"official_normalization":f.normalized_pos.iter().map(|p|json!({"position":p.pos.0,"difference":p.diff,"original_position":f.original_relative_byte_pos(f.start_pos+rustc_span::BytePos(p.pos.0)).0})).collect::<Vec<_>>(),"normalized_source_len":f.normalized_source_len.0,"original_source_len":f.unnormalized_source_len,"official_endpoints":[f.original_relative_byte_pos(f.start_pos).0,f.original_relative_byte_pos(f.end_position()).0],"start_pos":f.start_pos.0,"source_hash":format!("{:?}",f.src_hash),"original_source_hash":f.src_hash.to_string(),"source_disposition":if f.src.is_some(){"local-source-text"}else{"compiler-imported-source-metadata"}})).collect::<Vec<_>>();
    json!({"schema":"sifr-maintainability-source-binder-capture-v1","fragment":"sifr-maintainability-source-binder-v1","crate":format!("{:?}",tcx.stable_crate_id(hir::def_id::LOCAL_CRATE)),"cfg":cfg,"calls":calls,"declaration_owners":owners,"source_files":sources,"stage":"rustc-after-analysis","semantic_export":false})
}

fn source_owner_kind(kind: hir::def::DefKind) -> &'static str {
    use hir::def::DefKind;
    match kind {
        DefKind::Fn | DefKind::AssocFn => "Function",
        DefKind::Struct => "Struct",
        DefKind::Enum => "Enum",
        DefKind::Union => "Union",
        DefKind::Trait => "Trait",
        DefKind::TraitAlias => "TraitAlias",
        DefKind::Impl { .. } => "Impl",
        DefKind::Const { .. } | DefKind::AssocConst { .. } => "Const",
        DefKind::Static { .. } => "Static",
        DefKind::TyAlias | DefKind::AssocTy => "TypeAlias",
        DefKind::Mod => "Module",
        DefKind::Use => "Use",
        DefKind::Macro(_) => "Macro",
        _ => "other-compiler-owner-kind",
    }
}

fn generic_order(tcx: TyCtxt<'_>, def: DefId) -> Value {
    if !matches!(
        tcx.def_kind(def),
        hir::def::DefKind::Fn
            | hir::def::DefKind::AssocFn
            | hir::def::DefKind::Impl { .. }
            | hir::def::DefKind::Struct
            | hir::def::DefKind::Enum
            | hir::def::DefKind::Union
            | hir::def::DefKind::Trait
            | hir::def::DefKind::TraitAlias
            | hir::def::DefKind::TyAlias
            | hir::def::DefKind::AssocTy
            | hir::def::DefKind::Const { .. }
            | hir::def::DefKind::AssocConst { .. }
    ) {
        return Value::Null;
    }
    let g = tcx.generics_of(def);
    let parameters=g.own_params.iter().map(|p|json!({"identity":definition(tcx,p.def_id),"index":p.index,"source":source(tcx,tcx.def_span(p.def_id)),"name_source":tcx.def_ident_span(p.def_id).map(|span|source(tcx,span)),"kind":match p.kind{rustc_middle::ty::GenericParamDefKind::Lifetime=>json!({"kind":"lifetime"}),rustc_middle::ty::GenericParamDefKind::Type{synthetic,..}=>json!({"kind":"type","synthetic":synthetic}),rustc_middle::ty::GenericParamDefKind::Const{..}=>json!({"kind":"const"})}})).collect::<Vec<_>>();
    json!({"parameters":parameters,"parent":g.parent.map(|d|definition(tcx,d)),"parent_count":g.parent_count,"has_self":g.has_self})
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

fn region_authority(tcx: TyCtxt<'_>, owner: DefId, region: rustc_middle::ty::Region<'_>) -> Value {
    use rustc_middle::ty;
    match region.kind() {
        ty::ReEarlyParam(p) => {
            json!({"kind":"EarlyBound","index":p.index,"target":definition(tcx,tcx.generics_of(owner).param_at(p.index as usize,tcx).def_id)})
        }
        ty::ReBound(depth, p) => {
            json!({"kind":"LateBound","depth":match depth{ty::BoundVarIndexKind::Bound(d)=>json!({"debruijn":d.as_u32()}),ty::BoundVarIndexKind::Canonical=>json!({"canonical":true})},"index":p.var.as_u32(),"target":match p.kind {ty::BoundRegionKind::Named(d)=>Some(definition(tcx,d)),_=>None},"variable_kind":format!("{:?}",p.kind)})
        }
        ty::ReLateParam(p) => {
            json!({"kind":"LateParameter","scope":definition(tcx,p.scope),"target":match p.kind{ty::LateParamRegionKind::Named(d)=>Some(definition(tcx,d)),_=>None},"parameter_kind":format!("{:?}",p.kind)})
        }
        ty::ReStatic => json!({"kind":"StaticLifetime"}),
        ty::ReErased => json!({"kind":"Erased"}),
        other => json!({"kind":"Unsupported","raw":format!("{other:?}")}),
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
        json!({"source":source(tcx,*span),"fact":fact,"bound_variables":variables(tcx,binder.bound_vars()),"trait_identity":match binder.skip_binder(){rustc_middle::ty::ClauseKind::Trait(p)=>Some(definition(tcx,p.trait_ref.def_id)),_=>None},"outlives_region_authority":match binder.skip_binder(){rustc_middle::ty::ClauseKind::RegionOutlives(p)=>Some(vec![region_authority(tcx,def,p.0),region_authority(tcx,def,p.1)]),rustc_middle::ty::ClauseKind::TypeOutlives(p)=>Some(vec![region_authority(tcx,def,p.1)]),_=>None},"raw":format!("{clause:?}")})
    }).collect::<Vec<_>>();
    json!({"parent_identity":p.parent.map(|d|definition(tcx,d)),"parent":p.parent.map(|d|predicates(tcx,d)),"own":own})
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

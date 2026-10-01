//! Published compiler declaration and body facts. Regions are never synthesized.
use rustc_hir::{def::DefKind, def_id::DefId};
use rustc_middle::ty::{self, TyCtxt};
use rustc_type_ir::inherent::IntoKind;
use serde_json::{Value, json};

pub fn bound_region(tcx: TyCtxt<'_>, kind: ty::BoundRegionKind<'_>) -> Value {
    match kind {
        ty::BoundRegionKind::Anon => json!({"anonymous":true}),
        ty::BoundRegionKind::NamedForPrinting(n) => json!({"printing_name":n.to_string()}),
        ty::BoundRegionKind::Named(d) => json!({"declaration":tcx.def_path_str(d)}),
        ty::BoundRegionKind::ClosureEnv => json!({"closure_environment":true}),
    }
}
pub fn region(tcx: TyCtxt<'_>, region: ty::Region<'_>) -> Value {
    match region.kind() {
        ty::ReEarlyParam(p) => {
            json!({"kind":"ReEarlyParam","index":p.index,"name":p.name.to_string()})
        }
        ty::ReBound(depth, p) => {
            json!({"kind":"ReBound","depth":match depth {ty::BoundVarIndexKind::Bound(d)=>json!({"debruijn":d.as_u32()}),ty::BoundVarIndexKind::Canonical=>json!({"canonical":true})},"variable":p.var.as_u32(),"origin":bound_region(tcx,p.kind)})
        }
        ty::ReLateParam(p) => {
            json!({"kind":"ReLateParam","scope":tcx.def_path_str(p.scope),"origin":match p.kind {ty::LateParamRegionKind::Anon(i)=>json!({"anonymous":i}),ty::LateParamRegionKind::NamedAnon(i,n)=>json!({"anonymous":i,"name":n.to_string()}),ty::LateParamRegionKind::Named(d)=>json!({"declaration":tcx.def_path_str(d)}),ty::LateParamRegionKind::ClosureEnv=>json!({"closure_environment":true})}})
        }
        ty::ReStatic => json!({"kind":"ReStatic"}),
        ty::ReErased => json!({"kind":"ReErased"}),
        other => json!({"unsupported_region":format!("{other:?}")}),
    }
}
pub fn constant<'tcx>(tcx: TyCtxt<'tcx>, c: ty::Const<'tcx>) -> Value {
    match c.kind() {
        ty::ConstKind::Value(v) => {
            json!({"kind":"Value","type":self::ty(tcx,v.ty),"value":match v.valtree.kind(){ty::ValTreeKind::Leaf(s)=>json!({"bits":format!("{:x}",s.to_bits(s.size())),"bytes":s.size().bytes()}),ty::ValTreeKind::Branch(children)=>json!({"children":children.iter().map(|c|constant(tcx,c)).collect::<Vec<_>>()})}})
        }
        ty::ConstKind::Alias(rigid, a) => {
            json!({"kind":"Alias","rigid":format!("{rigid:?}"),"definition":tcx.def_path_str(match a.kind {ty::AliasConstKind::Projection{def_id}|ty::AliasConstKind::Inherent{def_id}|ty::AliasConstKind::Free{def_id}|ty::AliasConstKind::Anon{def_id}=>def_id}),"arguments":args(tcx,a.args),"type":self::ty(tcx,a.type_of(tcx).skip_norm_wip())})
        }
        other => json!({"unsupported_const":format!("{other:?}")}),
    }
}
pub fn args<'tcx>(tcx: TyCtxt<'tcx>, args: ty::GenericArgsRef<'tcx>) -> Vec<Value> {
    args.iter()
        .map(|arg| match arg.kind() {
            ty::GenericArgKind::Lifetime(r) => json!({"lifetime":region(tcx,r)}),
            ty::GenericArgKind::Type(t) => json!({"type":self::ty(tcx,t)}),
            ty::GenericArgKind::Const(c) => json!({"const":constant(tcx,c)}),
        })
        .collect()
}
pub fn ty<'tcx>(tcx: TyCtxt<'tcx>, t: ty::Ty<'tcx>) -> Value {
    match t.kind() {
        ty::Ref(r, inner, mutable) => {
            json!({"reference":self::ty(tcx,*inner),"mutable":mutable.is_mut(),"region":region(tcx,*r)})
        }
        ty::Adt(def, a) => {
            json!({"adt":crate::identity::path(tcx,def.did()),"arguments":args(tcx,a)})
        }
        ty::Tuple(ts) => json!({"tuple":ts.iter().map(|t|self::ty(tcx,t)).collect::<Vec<_>>()}),
        ty::Param(p) => json!({"parameter":p.name.to_string(),"index":p.index}),
        ty::Bool => json!({"builtin":"bool"}),
        ty::Char => json!({"builtin":"char"}),
        ty::Str => json!({"builtin":"str"}),
        ty::Int(k) => json!({"builtin":format!("{k:?}").to_lowercase()}),
        ty::Uint(k) => json!({"builtin":format!("{k:?}").to_lowercase()}),
        ty::Float(k) => json!({"builtin":format!("{k:?}").to_lowercase()}),
        ty::Never => json!({"builtin":"never"}),
        ty::Slice(t) => json!({"slice":self::ty(tcx,*t)}),
        ty::Array(t, c) => json!({"array":self::ty(tcx,*t),"length":constant(tcx,*c)}),
        ty::RawPtr(t, m) => json!({"pointer":self::ty(tcx,*t),"mutable":m.is_mut()}),
        ty::Alias(k, a) => {
            json!({"alias":crate::identity::path(tcx,match a.kind { ty::AliasTyKind::Projection{def_id} | ty::AliasTyKind::Inherent{def_id} | ty::AliasTyKind::Opaque{def_id} | ty::AliasTyKind::Free{def_id} => def_id }),"kind":format!("{k:?}"),"arguments":args(tcx,a.args)})
        }
        ty::Dynamic(predicates, r) => crate::dynamic::capture(tcx, predicates, *r),
        ty::FnDef(d, a) => {
            json!({"function":crate::identity::path(tcx,*d),"arguments":args(tcx,a)})
        }
        other => json!({"unsupported_type":format!("{other:?}")}),
    }
}
pub fn binders(tcx: TyCtxt<'_>, variables: &ty::List<ty::BoundVariableKind>) -> Vec<Value> {
    variables
        .iter()
        .map(|v| match v {
            ty::BoundVariableKind::Region(k) => {
                json!({"kind":"region","origin":bound_region(tcx,k)})
            }
            other => json!({"unsupported_binder":format!("{other:?}")}),
        })
        .collect()
}
pub fn bound_signature<'tcx>(tcx: TyCtxt<'tcx>, binder: ty::PolyFnSig<'tcx>) -> Value {
    let sig = binder.skip_binder();
    json!({"binders":binders(tcx,binder.bound_vars()),"parameters":sig.inputs().iter().map(|t|ty(tcx,*t)).collect::<Vec<_>>(),"return":ty(tcx,sig.output()),"variadic":sig.c_variadic(),"unsafe":sig.safety().is_unsafe()})
}
pub fn signature(tcx: TyCtxt<'_>, def: DefId) -> Value {
    let binder = tcx.fn_sig(def).instantiate_identity().skip_norm_wip();
    bound_signature(tcx, binder)
}
pub fn generics(tcx: TyCtxt<'_>, def: DefId) -> Value {
    let g = tcx.generics_of(def);
    json!({"owner":tcx.def_path_str(def),"parent":g.parent.map(|d|generics(tcx,d)),"parent_count":g.parent_count,"own":g.own_params.iter().map(|p|json!({"owner":tcx.def_path_str(tcx.parent(p.def_id)),"index":p.index,"name":p.name.to_string(),"kind":p.kind.descr()})).collect::<Vec<_>>()})
}
pub fn clause_fact<'tcx>(tcx: TyCtxt<'tcx>, clause: ty::Clause<'tcx>) -> Value {
    let binder = clause.kind();
    let fact = match binder.skip_binder() {
        ty::ClauseKind::Trait(t) => {
            json!({"trait":crate::identity::path(tcx,t.trait_ref.def_id),"arguments":args(tcx,t.trait_ref.args),"polarity":format!("{:?}",t.polarity)})
        }
        ty::ClauseKind::RegionOutlives(p) => {
            json!({"region_outlives":[region(tcx,p.0),region(tcx,p.1)]})
        }
        ty::ClauseKind::TypeOutlives(p) => json!({"type_outlives":[ty(tcx,p.0),region(tcx,p.1)]}),
        other => json!({"unsupported_clause":format!("{other:?}")}),
    };
    json!({"binders":binders(tcx,binder.bound_vars()),"fact":fact})
}
pub fn predicates(tcx: TyCtxt<'_>, def: DefId) -> Value {
    let p = tcx.predicates_of(def);
    json!({"parent":p.parent.map(|d|predicates(tcx,d)),"own":p.predicates.iter().map(|(clause,_)| {
        clause_fact(tcx,*clause)
    }).collect::<Vec<_>>()})
}
pub fn trait_bridge(tcx: TyCtxt<'_>, def: DefId, implementation: DefId) -> Value {
    if !matches!(tcx.def_kind(def), DefKind::AssocFn) {
        return Value::Null;
    }
    let original = tcx
        .associated_item(def)
        .trait_item_def_id()
        .unwrap_or_else(|| {
            tcx.sess
                .dcx()
                .fatal("missing compiler trait-method relation")
        });
    let generated = tcx.generics_of(def);
    let inherited = tcx
        .impl_trait_ref(implementation)
        .instantiate_identity()
        .skip_norm_wip();
    let trait_generics = tcx.generics_of(original);
    if generated.own_params.len() != trait_generics.own_params.len() {
        tcx.sess
            .dcx()
            .fatal("trait/generated method parameter arity conflict");
    }
    let substitution = inherited.args.extend_to(tcx, original, |param, _| {
        let own = &generated.own_params[(param.index as usize) - trait_generics.parent_count];
        if own.kind.descr() != param.kind.descr() {
            tcx.sess
                .dcx()
                .fatal("trait/generated generic kind conflict");
        }
        tcx.mk_param_from_def(own)
    });
    let span = tcx.def_span(original);
    let source = tcx
        .sess
        .source_map()
        .span_to_snippet(span)
        .unwrap_or_else(|_| {
            tcx.sess
                .dcx()
                .fatal("unreadable resolved original trait method")
        });
    json!({"trait_method":crate::identity::path(tcx,original),"trait":crate::identity::path(tcx,inherited.def_id),"original":{"generics":generics(tcx,original),"predicates":predicates(tcx,original),"signature":signature(tcx,original),"source_tokens":crate::identity::tokens(&source),"span":crate::typed::span(tcx,span)},"substitution":args(tcx,substitution),"substituted_own_predicates":tcx.predicates_of(original).instantiate_own(tcx,substitution).map(|(clause,_)|clause_fact(tcx,clause.skip_norm_wip())).collect::<Vec<_>>(),"substituted_signature":bound_signature(tcx,tcx.fn_sig(original).instantiate(tcx,substitution).skip_norm_wip()),"alpha_parameters":trait_generics.own_params.iter().zip(&generated.own_params).map(|(from,to)|json!({"trait_index":from.index,"generated_index":to.index,"trait_name":from.name.to_string(),"generated_name":to.name.to_string(),"kind":to.kind.descr()})).collect::<Vec<_>>()})
}
pub fn declaration(tcx: TyCtxt<'_>, def: DefId, implementation: DefId) -> Value {
    let receiver = tcx
        .type_of(implementation)
        .instantiate_identity()
        .skip_norm_wip();
    let original = match receiver.kind() {
        ty::Adt(adt, _) => adt.did(),
        _ => tcx.sess.dcx().fatal("unsupported builtin receiver"),
    };
    let span = tcx.hir_expect_item(original.expect_local()).span;
    let source = tcx
        .sess
        .source_map()
        .span_to_snippet(span)
        .unwrap_or_else(|_| tcx.sess.dcx().fatal("missing original declaration source"));
    json!({"trait_bridge":trait_bridge(tcx,def,implementation),"generics":generics(tcx,def),"predicates":predicates(tcx,def),"receiver":ty(tcx,receiver),"signature":if matches!(tcx.def_kind(def),DefKind::AssocFn){Some(signature(tcx,def))}else{None},"original":{"identity":crate::identity::path(tcx,original),"generics":generics(tcx,original),"predicates":predicates(tcx,original),"source_tokens":crate::identity::tokens(&source),"span":crate::typed::span(tcx,span),"field_types":match receiver.kind(){ty::Adt(adt,arguments)=>adt.variants().iter().flat_map(|variant|variant.fields.iter().map(|field|json!({"identity":crate::identity::path(tcx,field.did),"type":ty(tcx,field.ty(tcx,arguments).skip_norm_wip())}))).collect::<Vec<_>>(),_=>vec![]}}})
}

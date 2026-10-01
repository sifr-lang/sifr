//! Exhaustive pinned existential facts; Self remains omitted as published.
use rustc_hir::def_id::DefId;
use rustc_middle::ty::{self, TyCtxt};
use serde_json::{Value, json};

fn definition(tcx: TyCtxt<'_>, def: DefId) -> Value {
    json!({"canonical":crate::identity::path(tcx,def),
        "structural":tcx.def_path(def).to_string_no_crate_verbose(),
        "crate":tcx.crate_name(def.krate).to_string(),
        "kind":format!("{:?}",tcx.def_kind(def)),
        "origin_paths":if def.is_local(){None}else{Some(tcx.used_crate_source(def.krate).paths().map(|p|p.to_string_lossy().to_string()).collect::<Vec<_>>())}})
}

pub fn capture<'tcx>(
    tcx: TyCtxt<'tcx>,
    predicates: &'tcx ty::List<ty::PolyExistentialPredicate<'tcx>>,
    region: ty::Region<'tcx>,
) -> Value {
    let mut principal = None;
    let mut projections = vec![];
    let mut auto_traits = vec![];
    let ordered = predicates.iter().enumerate().map(|(index,binder)| {
        let fact = match binder.skip_binder() {
            ty::ExistentialPredicate::Trait(t) => {
                principal = Some(index);
                json!({"kind":"Trait","definition":definition(tcx,t.def_id),"arguments":crate::semantic::args(tcx,t.args),"existential_self":"omitted"})
            }
            ty::ExistentialPredicate::Projection(p) => {
                projections.push(index);
                let owner = tcx.trait_of_assoc(p.def_id).unwrap_or_else(||tcx.sess.dcx().fatal("existential projection has no declaring trait"));
                let term = match p.term.kind() {
                    ty::TermKind::Ty(t)=>json!({"kind":"type","payload":crate::semantic::ty(tcx,t)}),
                    ty::TermKind::Const(c)=>json!({"kind":"const","payload":crate::semantic::constant(tcx,c)}),
                };
                json!({"kind":"Projection","definition":definition(tcx,p.def_id),"trait_owner":definition(tcx,owner),"arguments":crate::semantic::args(tcx,p.args),"existential_self":"omitted","term":term})
            }
            ty::ExistentialPredicate::AutoTrait(d) => {
                auto_traits.push(index);
                json!({"kind":"AutoTrait","definition":definition(tcx,d)})
            }
        };
        json!({"binders":crate::semantic::binders(tcx,binder.bound_vars()),"fact":fact})
    }).collect::<Vec<_>>();
    json!({"dynamic":{"predicates":ordered,"principal":principal,"projections":projections,"auto_traits":auto_traits,"object_region":crate::semantic::region(tcx,region)}})
}

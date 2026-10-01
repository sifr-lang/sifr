//! Compiler owner authority, enumerated before consulting expanded AST projections.
use crate::{expanded::Declaration, identity, typed};
use rustc_hir::{def::DefKind, def_id::DefId};
use rustc_middle::ty::TyCtxt;
use rustc_span::hygiene::{ExpnId, ExpnKind, MacroKind};
use serde_json::{Value, json};
use std::collections::HashSet;

pub fn owner(tcx: TyCtxt<'_>, did: DefId, implementation: DefId, chain: &[Value]) -> Value {
    let kind = tcx.def_kind(did);
    let body = tcx.hir_maybe_body_owned_by(did.expect_local()).is_some();
    let receiver = tcx
        .type_of(implementation)
        .instantiate_identity()
        .skip_norm_wip();
    let trait_identity = tcx.impl_is_of_trait(implementation).then(|| {
        identity::path(
            tcx,
            tcx.impl_trait_ref(implementation)
                .instantiate_identity()
                .skip_norm_wip()
                .def_id,
        )
    });
    let primary = match chain
        .first()
        .and_then(|origin| origin["macro_identity"].as_str())
    {
        Some("core::fmt::macros::Debug") => Some("core::fmt::Debug"),
        Some("core::clone::Clone") => Some("core::clone::Clone"),
        Some("core::cmp::PartialEq") => Some("core::cmp::PartialEq"),
        Some("core::marker::Copy") => Some("core::marker::Copy"),
        _ => None,
    };
    let role = if primary.is_some() && primary == trait_identity.as_deref() {
        "common-primary"
    } else {
        "compiler-only-auxiliary"
    };
    json!({
        "output_role":role,"structural_identity":tcx.def_path(did).to_string_no_crate_verbose(), "owner": tcx.def_path_str(did), "owner_kind": format!("{kind:?}"),
        "parent": tcx.def_path_str(tcx.parent(did)),
        "receiver_identity": identity::ty(tcx, receiver),
        "trait_identity": if tcx.impl_is_of_trait(implementation) {Some(identity::path(tcx, tcx.impl_trait_ref(implementation).instantiate_identity().skip_norm_wip().def_id))} else {None},
        "generic_bounds": identity::bounds(tcx, implementation),
        "canonical_signature": if matches!(kind, DefKind::AssocFn) {Some(identity::signature(tcx, did))} else {None},
        "visibility": match tcx.visibility(did) {rustc_middle::ty::Visibility::Public => json!("Public"), rustc_middle::ty::Visibility::Restricted(module) => json!({"restricted_to":identity::path(tcx,module)})},
        "hir_body": body, "expansion_chain": chain,
        "hir_members": if matches!(kind, DefKind::Impl {..}) {Some(tcx.associated_items(did).in_definition_order().map(|item| tcx.def_path_str(item.def_id)).collect::<Vec<_>>())} else {None},
        "member_disposition": if matches!(kind, DefKind::Impl {..}) {"impl-owner"} else if body {"actual-body"} else {"actual-bodyless-member"}
    })
}

pub fn capture(tcx: TyCtxt<'_>) -> Value {
    // The universe comes only from compiler HIR; even bodyless/nested impls enter it.
    let implementations = tcx
        .hir_crate_items(())
        .free_items()
        .map(|item| item.owner_id.def_id)
        .filter(|def| matches!(tcx.def_kind(*def), DefKind::Impl { .. }))
        .collect::<Vec<_>>();
    let hir_trait = implementations
        .iter()
        .filter(|def| tcx.impl_is_of_trait(**def))
        .copied()
        .collect::<HashSet<_>>();
    let ty_trait = tcx
        .all_local_trait_impls(())
        .values()
        .flatten()
        .copied()
        .collect::<HashSet<_>>();
    if hir_trait != ty_trait {
        tcx.sess
            .dcx()
            .fatal("independent inventory HIR/type trait-impl universe discrepancy");
    }
    let suffix = std::env::var("SIFR_BUILTIN_SOURCE_SUFFIX").expect("selected source contract");
    let mut universe = vec![];
    let mut owners = vec![];
    for local in implementations {
        let did = local.to_def_id();
        let expansion = tcx.expn_that_defined(did);
        let chain = typed::expansion_chain(tcx, expansion);
        let source = tcx
            .sess
            .source_map()
            .lookup_char_pos(tcx.def_span(local).lo())
            .file
            .name
            .prefer_local_unconditionally()
            .to_string();
        let derive = chain.iter().find(|entry| {
            entry["kind"]
                .as_str()
                .is_some_and(|kind| kind.starts_with("Macro(Derive,"))
        });
        let (disposition, reason) = if source.is_empty()
            || chain.iter().any(|entry| {
                entry["kind"]
                    .as_str()
                    .is_some_and(|kind| kind.starts_with("Macro("))
                    && entry["macro_identity"].is_null()
            }) {
            ("unsupported", "unknown source or resolved macro owner")
        } else if !source.ends_with(&suffix) {
            ("nonselected", "outside selected Cargo source")
        } else if expansion == ExpnId::root() {
            ("nonselected", "source implementation")
        } else if let Some(origin) = derive {
            if origin["macro_identity"].is_null() {
                ("unsupported", "unresolved defining derive")
            } else if origin["builtin"] != true {
                ("nonselected", "resolved nonbuiltin derive")
            } else if !matches!(
                expansion.expn_data().kind,
                ExpnKind::Macro(MacroKind::Derive, _)
            ) {
                ("unsupported", "unsupported nested defining expansion")
            } else {
                ("selected", "resolved builtin derive in selected source")
            }
        } else if chain.iter().any(|entry| entry["macro_identity"].is_null()) {
            ("unsupported", "unknown defining expansion owner")
        } else {
            ("nonselected", "resolved nonderive expansion")
        };
        let implementation = owner(tcx, did, did, &chain);
        universe.push(json!({"structural_identity":implementation["structural_identity"],"owner":implementation["owner"], "module":identity::path(tcx,tcx.parent_module_from_def_id(local).to_def_id()),"receiver_identity":implementation["receiver_identity"],"trait_identity":implementation["trait_identity"],"source":source,"expansion_chain":chain,"disposition":disposition,"reason":reason,"trait_impl_cross_checked":tcx.impl_is_of_trait(did)}));
        if disposition == "unsupported" {
            tcx.sess.dcx().fatal(format!(
                "independent inventory unsupported owner {}: {reason}",
                tcx.def_path_str(did)
            ));
        }
        if disposition != "selected" {
            continue;
        }
        owners.push(implementation);
        for member in tcx.associated_items(did).in_definition_order() {
            if tcx.parent(member.def_id) != did {
                tcx.sess
                    .dcx()
                    .fatal("independent inventory associated-item parent discrepancy");
            }
            let member_chain = typed::expansion_chain(tcx, tcx.expn_that_defined(member.def_id));
            owners.push(owner(tcx, member.def_id, did, &member_chain));
        }
    }
    universe.sort_by_cached_key(|entry| entry["owner"].to_string());
    owners.sort_by_cached_key(|entry| entry["owner"].to_string());
    json!({"schema":"sifr-builtin-owner-inventory-v1", "crate":tcx.crate_name(rustc_hir::def_id::LOCAL_CRATE).to_string(), "enumeration":"hir-crate-free-items-and-associated-items", "trait_impl_cross_check":true,"universe":universe,"owners":owners})
}

pub fn reconcile(tcx: TyCtxt<'_>, inventory: &Value, ast: &[Declaration]) {
    let mut actual = ast
        .iter()
        .filter(|decl| typed::selected(tcx, decl))
        .map(|decl| tcx.def_path_str(decl.def.to_def_id()))
        .collect::<Vec<_>>();
    let expected = inventory["owners"]
        .as_array()
        .expect("compiler inventory")
        .iter()
        .map(|entry| entry["owner"].as_str().expect("compiler owner").to_owned())
        .collect::<Vec<_>>();
    actual.sort();
    if actual != expected {
        tcx.sess
            .dcx()
            .fatal("independent inventory/expanded AST owner mismatch before export");
    }
}

//! Complete transformed stage observations, never original attachment/lineage.
use rustc_ast::AttrKind;
use rustc_middle::ty::TyCtxt;
use serde_json::{Value, json};

pub fn capture(
    tcx: TyCtxt<'_>,
    ast: &[crate::expanded::Declaration],
    associations: &[(rustc_ast::NodeId, rustc_hir::def_id::LocalDefId)],
    stage_attributes: &[crate::expanded::StageAttribute],
) -> Value {
    let owners = ast.iter().map(|d| {
        let def = d.def.to_def_id();
        let attrs = d.attrs.iter().enumerate().map(|(ordinal,a)| {
            let (kind,path,args,tokens)=match &a.kind {
                AttrKind::Normal(n) => ("Normal",Some(format!("{:?}",n.item.path)),Some(format!("{:?}",n.item.args)),n.tokens.as_ref().map(|t|format!("{:?}",t.to_attr_token_stream()))),
                AttrKind::DocComment(..) => ("DocComment",None,None,None),
            };
            json!({"ordinal":ordinal,"stage_attr_id":format!("{:?}",a.id),"kind":kind,"style":format!("{:?}",a.style),"source":crate::source_binder::source(tcx,a.span),"raw_kind":format!("{:?}",a.kind),"path":path,"args":args,"tokens":tokens})
        }).collect::<Vec<_>>();
        json!({"node_id":d.node_id.as_u32(),"identity":crate::source_binder::definition(tcx,def),"local_def_id":format!("{:?}",d.def),"parent":crate::source_binder::definition(tcx,tcx.parent(def)),"ast_kind":d.ast_kind,"declaration_source":crate::source_binder::source(tcx,d.span),"declaration_tokens_available":d.tokens_available,"attributes":attrs,"lowered_attributes":format!("{:?}",tcx.hir_attrs(tcx.local_def_id_to_hir_id(d.def)))})
    }).collect::<Vec<_>>();
    let expanded_attributes=stage_attributes.iter().enumerate().map(|(ordinal,row)|json!({"ordinal":ordinal,"stage":"rustc-after-expansion-resolver-for-lowering","status":"transformed-observation","declaration_visitor_ancestry":row.declaration_ancestry.iter().map(|n|n.as_u32()).collect::<Vec<_>>(),"direct_stage_declaration_attribute":row.direct_declaration_attribute,"attribute":attribute(tcx,&row.attribute)})).collect::<Vec<_>>();
    let mut lowered_owners = vec![];
    // Enumerate actual HIR owners, not every allocated compiler definition.
    // Non-owner nodes (including opaque/generic nodes) live in these complete
    // owner attribute maps; some allocated generated defs have no HIR mapping.
    for owner in tcx.hir_crate_items(()).owners() {
        let def = owner.def_id;
        {
            let nodes = tcx.hir_owner_nodes(owner);
            let hir = tcx.local_def_id_to_hir_id(def);
            let map = tcx.hir_attr_map(hir.owner);
            let entries=map.map.iter().map(|(local,attrs)|json!({"local_id":local.as_u32(),"attributes_debug":format!("{attrs:?}")})).collect::<Vec<_>>();
            lowered_owners.push(json!({"stage":"rustc-after-analysis-hir-attrs","status":"transformed-observation","identity":crate::source_binder::definition(tcx,def.to_def_id()),"local_def_id":format!("{def:?}"),"owner_hir_id":format!("{hir:?}"),"node_count":nodes.nodes.len(),"attribute_map_entries":entries,"unlisted_local_node_disposition":"empty-transformed-observation-from-hir-attr-map-get","define_opaque_observation":format!("{:?}",map.define_opaque)}));
        }
    }
    let mut cfg = tcx
        .sess
        .config
        .iter()
        .map(|(k, v)| json!({"key":k.to_string(),"value":v.map(|v|v.to_string())}))
        .collect::<Vec<_>>();
    cfg.sort_by_cached_key(|v| v.to_string());
    json!({"context":{"crate":format!("{:?}",tcx.stable_crate_id(rustc_hir::def_id::LOCAL_CRATE)),"cfg":cfg,"target":tcx.sess.opts.target_triple.to_string(),"test":tcx.sess.opts.test},"lowered_owner_enumeration":"actual-hir-crate-items-owners-including-crate-root","schema":"development-transformed-attribute-inventory-v2","original_rustc_attr_id":"not_claimed_by_authorized_contract","compiler_original_attribute_attachment":"not_claimed_by_authorized_contract","exhaustive_consumed_item_lineage":"not_claimed_by_authorized_contract","preconsumption_to_final_survival":"not_claimed_by_authorized_contract","capture_stage":"rustc-after-expansion-resolver-for-lowering","serialization_stage":"rustc-after-analysis","accepted_proof":false,"semantic_export":false,"expanded_attributes":expanded_attributes,"lowered_attribute_owners":lowered_owners,"owners":owners,"resolver_associations":associations.iter().map(|(node,def)|json!({"node_id":node.as_u32(),"local_def_id":format!("{def:?}"),"identity":crate::source_binder::definition(tcx,def.to_def_id()),"hir_id":format!("{:?}",tcx.local_def_id_to_hir_id(*def)),"parent":tcx.opt_parent(def.to_def_id()).map(|d|crate::source_binder::definition(tcx,d))})).collect::<Vec<_>>()})
}

fn attribute(tcx: TyCtxt<'_>, a: &rustc_ast::Attribute) -> Value {
    let (kind, path, args, tokens) = match &a.kind {
        AttrKind::Normal(n) => (
            "Normal",
            Some(format!("{:?}", n.item.path)),
            Some(format!("{:?}", n.item.args)),
            n.tokens
                .as_ref()
                .map(|t| format!("{:?}", t.to_attr_token_stream())),
        ),
        AttrKind::DocComment(..) => ("DocComment", None, None, None),
    };
    json!({"stage_attr_id":format!("{:?}",a.id),"kind":kind,"style":format!("{:?}",a.style),"source":crate::source_binder::source(tcx,a.span),"raw_kind":format!("{:?}",a.kind),"path":path,"args":args,"tokens":tokens})
}

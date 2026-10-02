use rustc_hir::def_id::DefId;
use rustc_middle::ty::{self, TyCtxt};
use serde_json::{Value, json};
pub fn path(tcx: TyCtxt<'_>, def: DefId) -> String {
    let mut names = vec![tcx.crate_name(def.krate).to_string()];
    names.extend(
        tcx.def_path(def)
            .data
            .iter()
            .filter_map(|part| part.data.get_opt_name().map(|name| name.to_string())),
    );
    names.join("::")
}
pub fn ty(tcx: TyCtxt<'_>, ty: ty::Ty<'_>) -> Value {
    match ty.kind() {
        ty::Ref(_, inner, mutable) => {
            json!({"reference":self::ty(tcx,*inner),"mutable":mutable.is_mut()})
        }
        ty::Adt(def, args) => {
            json!({"adt":path(tcx,def.did()),"arguments":args.types().map(|t|self::ty(tcx,t)).collect::<Vec<_>>()})
        }
        ty::Tuple(types) => {
            json!({"tuple":types.iter().map(|t|self::ty(tcx,t)).collect::<Vec<_>>()})
        }
        ty::Param(parameter) => json!({"parameter":parameter.name.to_string()}),
        ty::Bool => json!({"builtin":"bool"}),
        ty::Int(k) => json!({"builtin":format!("{k:?}").to_lowercase()}),
        ty::Uint(k) => json!({"builtin":format!("{k:?}").to_lowercase()}),
        ty::Str => json!({"builtin":"str"}),
        _ => json!({"unsupported":ty.to_string()}),
    }
}
pub fn signature(tcx: TyCtxt<'_>, def: DefId) -> Value {
    let sig = tcx
        .fn_sig(def)
        .instantiate_identity()
        .skip_norm_wip()
        .skip_binder();
    json!({"parameters":sig.inputs().iter().map(|t|ty(tcx,*t)).collect::<Vec<_>>(),"return":ty(tcx,sig.output()),"variadic":sig.c_variadic(),"unsafe":sig.safety().is_unsafe()})
}
pub fn tokens(text: &str) -> Vec<String> {
    let mut offset = 0;
    let mut result = vec![];
    for token in rustc_lexer::tokenize(text, rustc_lexer::FrontmatterAllowed::No) {
        let end = offset + token.len as usize;
        if !matches!(token.kind, rustc_lexer::TokenKind::Whitespace) {
            result.push(text[offset..end].to_owned());
        }
        offset = end;
    }
    result
}

pub fn bounds(tcx: TyCtxt<'_>, def: DefId) -> Value {
    let mut parameters =
        std::collections::BTreeMap::<String, std::collections::BTreeSet<String>>::new();
    for parameter in &tcx.generics_of(def).own_params {
        if matches!(parameter.kind, ty::GenericParamDefKind::Type { .. }) {
            parameters.insert(parameter.name.to_string(), Default::default());
        } else if matches!(parameter.kind, ty::GenericParamDefKind::Const { .. }) {
            return json!({"unsupported_generic":format!("{parameter:?}")});
        }
    }
    let clauses = tcx
        .predicates_of(def)
        .predicates
        .iter()
        .map(|(clause, _)| *clause);
    for clause in ty::elaborate::elaborate(tcx, clauses).elaborate_sized() {
        if let ty::ClauseKind::Trait(predicate) = clause.kind().skip_binder() {
            if let ty::Param(parameter) = predicate.trait_ref.self_ty().kind() {
                if let Some(traits) = parameters.get_mut(parameter.name.as_str()) {
                    traits.insert(json!({"trait":path(tcx,predicate.trait_ref.def_id),"arguments":predicate.trait_ref.args.types().map(|arg|ty(tcx,arg)).collect::<Vec<_>>(),"polarity":format!("{:?}",predicate.polarity)}).to_string());
                }
            }
        }
    }
    json!(parameters.into_iter().map(|(parameter,traits)|json!({"parameter":{"parameter":parameter},"traits":traits.into_iter().map(|text|serde_json::from_str::<Value>(&text).expect("serialized predicate")).collect::<Vec<_>>()})).collect::<Vec<_>>())
}

use crate::hir_analysis::traversal;
use crate::{HirStmt, ModuleFuncSignatures};
use sifr_type_system::{OwnershipKind, ParamConvention};
use std::collections::HashMap;

pub(super) type CallParamConventions = HashMap<String, Vec<ParamConvention>>;

pub(super) fn collect_call_param_conventions(
    stmts: &[HirStmt],
    func_signatures: &ModuleFuncSignatures,
) -> CallParamConventions {
    let mut conventions = func_signatures
        .iter()
        .map(|(name, (params, _))| {
            (
                name.clone(),
                params.iter().map(|(_, convention)| *convention).collect(),
            )
        })
        .collect::<CallParamConventions>();
    collect_local_call_param_conventions(stmts, func_signatures, &mut conventions);
    conventions
}

fn collect_local_call_param_conventions(
    stmts: &[HirStmt],
    func_signatures: &ModuleFuncSignatures,
    conventions: &mut CallParamConventions,
) {
    traversal::walk_stmts(
        stmts,
        traversal::TraversalConfig::LOCAL_SCOPE_ONLY,
        &mut |stmt| {
            let HirStmt::NestedFunction { func, .. } = stmt else {
                return;
            };
            let mutated =
                crate::helpers::collect_mutated_vars_with_sigs(&func.body, func_signatures);
            let params = func
                .params
                .iter()
                .map(|param| {
                    if !mutated.contains(&param.name) {
                        return param.convention;
                    }
                    if param.ty.ownership() == OwnershipKind::Copy {
                        if param.convention.is_owned() {
                            ParamConvention::own_mut()
                        } else {
                            param.convention
                        }
                    } else if param.convention.is_borrowed() {
                        ParamConvention::mut_borrow()
                    } else {
                        ParamConvention::own_mut()
                    }
                })
                .collect();
            conventions.insert(func.name.clone(), params);
            collect_local_call_param_conventions(&func.body, func_signatures, conventions);
        },
        &mut |_| {},
    );
}

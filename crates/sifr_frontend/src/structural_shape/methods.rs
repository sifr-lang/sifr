use super::{describe_node, metadata_for, ShapeMetadata, ShapeNode};
use sifr_lowering::{
    substitute_type_vars, DeclarationMetadataTargetKind, HirClass, HirFunction, LoweringResult,
    MethodKind,
};
use sifr_type_system::{
    ParamConvention, ParamMutability, ParamOwnership, ReceiverConvention, Type,
};
use std::collections::{BTreeSet, HashMap};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShapeMethod {
    pub name: String,
    pub kind: String,
    pub receiver: Option<String>,
    pub is_async: bool,
    pub params: Vec<ShapeParameter>,
    pub result: Box<ShapeNode>,
    pub metadata: Vec<ShapeMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShapeParameter {
    pub name: String,
    pub declared_type: ShapeNode,
    pub convention: String,
    pub keyword_only: bool,
    pub metadata: Vec<ShapeMetadata>,
}

pub(super) fn described_methods(
    module_name: &str,
    class_name: &str,
    class: &HirClass,
    type_args: &[Type],
    lowering: &LoweringResult,
    visiting: &mut BTreeSet<String>,
) -> Vec<ShapeMethod> {
    let mut seen = BTreeSet::new();
    let owner_prefix = format!("{class_name}.");
    let bindings = class
        .type_params
        .iter()
        .cloned()
        .zip(type_args.iter().cloned())
        .collect::<HashMap<_, _>>();
    lowering
        .declaration_metadata
        .iter()
        .filter(|entry| {
            entry.target_kind == DeclarationMetadataTargetKind::Method
                && entry.owner.starts_with(&owner_prefix)
                && seen.insert(entry.owner.clone())
        })
        .map(|entry| {
            let declared_name = &entry.owner[owner_prefix.len()..];
            let hir_name = if declared_name == "__init__" {
                "new"
            } else {
                declared_name
            };
            let Some(method) = class
                .methods
                .iter()
                .chain(class.operator_impls.iter().map(|(_, method)| method))
                .find(|method| method.name == hir_name)
            else {
                panic!("validated method metadata must resolve to a lowered method");
            };
            described_method(
                module_name,
                &entry.owner,
                declared_name,
                method,
                &bindings,
                metadata_for(
                    &lowering.declaration_metadata,
                    &entry.owner,
                    DeclarationMetadataTargetKind::Method,
                    None,
                ),
                lowering,
                visiting,
            )
        })
        .collect()
}

fn described_method(
    module_name: &str,
    owner: &str,
    declared_name: &str,
    method: &HirFunction,
    bindings: &HashMap<String, Type>,
    metadata: Vec<ShapeMetadata>,
    lowering: &LoweringResult,
    visiting: &mut BTreeSet<String>,
) -> ShapeMethod {
    ShapeMethod {
        name: declared_name.to_string(),
        kind: method_kind_name(method.method_kind).to_string(),
        receiver: method
            .receiver
            .map(receiver_convention_name)
            .map(str::to_string),
        is_async: method.is_async,
        params: method
            .params
            .iter()
            .map(|param| ShapeParameter {
                name: param.name.clone(),
                declared_type: describe_node(
                    module_name,
                    &substitute_type_vars(&param.ty, bindings),
                    lowering,
                    visiting,
                ),
                convention: param_convention_name(param.convention).to_string(),
                keyword_only: param.keyword_only,
                metadata: metadata_for(
                    &lowering.declaration_metadata,
                    owner,
                    DeclarationMetadataTargetKind::Parameter,
                    Some(&param.name),
                ),
            })
            .collect(),
        result: Box::new(describe_node(
            module_name,
            &substitute_type_vars(&method.return_type, bindings),
            lowering,
            visiting,
        )),
        metadata,
    }
}

const fn method_kind_name(kind: MethodKind) -> &'static str {
    match kind {
        MethodKind::Regular => "regular",
        MethodKind::ClassMethod => "class",
        MethodKind::StaticMethod => "static",
    }
}

const fn receiver_convention_name(convention: ReceiverConvention) -> &'static str {
    match convention {
        ReceiverConvention::SharedBorrow => "borrow",
        ReceiverConvention::MutableBorrow => "mut_borrow",
        ReceiverConvention::Owned => "own",
    }
}

const fn param_convention_name(convention: ParamConvention) -> &'static str {
    match (convention.ownership(), convention.mutability()) {
        (ParamOwnership::Borrow, ParamMutability::Immutable) => "borrow",
        (ParamOwnership::Borrow, ParamMutability::Mutable) => "mut_borrow",
        (ParamOwnership::Own, ParamMutability::Immutable) => "own",
        (ParamOwnership::Own, ParamMutability::Mutable) => "own_mut",
    }
}

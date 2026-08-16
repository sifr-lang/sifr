use super::{describe_node, metadata_for, ShapeMetadata, ShapeNode};
use sifr_lowering::{
    substitute_type_vars, DeclarationMetadataTargetKind, HirClass, HirParam, LoweringResult,
    MethodKind, StructuralMethodExport, TypedDeclarationMetadata,
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
    external_defs: &sifr_lowering::ExternalDefs,
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
        .filter_map(|entry| {
            let declared_name = &entry.owner[owner_prefix.len()..];
            let hir_name = if declared_name == "__init__" {
                "new"
            } else {
                declared_name
            };
            let method = class
                .methods
                .iter()
                .chain(class.operator_impls.iter().map(|(_, method)| method))
                .find(|method| method.name == hir_name)?;
            Some(described_method(
                module_name,
                &entry.owner,
                declared_name,
                &method.params,
                &method.return_type,
                method.method_kind,
                method.receiver,
                method.is_async,
                &bindings,
                metadata_for(
                    &lowering.declaration_metadata,
                    &entry.owner,
                    DeclarationMetadataTargetKind::Method,
                    None,
                ),
                &lowering.declaration_metadata,
                lowering,
                external_defs,
                visiting,
            ))
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
pub(super) fn described_exported_methods(
    module_name: &str,
    class_name: &str,
    methods: &[StructuralMethodExport],
    type_params: &[String],
    type_args: &[Type],
    declaration_metadata: &[TypedDeclarationMetadata],
    lowering: &LoweringResult,
    external_defs: &sifr_lowering::ExternalDefs,
    visiting: &mut BTreeSet<String>,
) -> Vec<ShapeMethod> {
    let bindings = type_params
        .iter()
        .cloned()
        .zip(type_args.iter().cloned())
        .collect::<HashMap<_, _>>();
    methods
        .iter()
        .map(|method| {
            let owner = format!("{class_name}.{}", method.name);
            described_method(
                module_name,
                &owner,
                &method.name,
                &method.params,
                &method.return_type,
                method.method_kind,
                method.receiver,
                method.is_async,
                &bindings,
                metadata_for(
                    declaration_metadata,
                    &owner,
                    DeclarationMetadataTargetKind::Method,
                    None,
                ),
                declaration_metadata,
                lowering,
                external_defs,
                visiting,
            )
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn described_method(
    module_name: &str,
    owner: &str,
    declared_name: &str,
    params: &[HirParam],
    return_type: &Type,
    method_kind: MethodKind,
    receiver: Option<ReceiverConvention>,
    is_async: bool,
    bindings: &HashMap<String, Type>,
    metadata: Vec<ShapeMetadata>,
    declaration_metadata: &[TypedDeclarationMetadata],
    lowering: &LoweringResult,
    external_defs: &sifr_lowering::ExternalDefs,
    visiting: &mut BTreeSet<String>,
) -> ShapeMethod {
    ShapeMethod {
        name: declared_name.to_string(),
        kind: method_kind_name(method_kind).to_string(),
        receiver: receiver.map(receiver_convention_name).map(str::to_string),
        is_async,
        params: params
            .iter()
            .map(|param| ShapeParameter {
                name: param.name.clone(),
                declared_type: describe_node(
                    module_name,
                    &substitute_type_vars(&param.ty, bindings),
                    lowering,
                    external_defs,
                    visiting,
                ),
                convention: param_convention_name(param.convention).to_string(),
                keyword_only: param.keyword_only,
                metadata: metadata_for(
                    declaration_metadata,
                    owner,
                    DeclarationMetadataTargetKind::Parameter,
                    Some(&param.name),
                ),
            })
            .collect(),
        result: Box::new(describe_node(
            module_name,
            &substitute_type_vars(return_type, bindings),
            lowering,
            external_defs,
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

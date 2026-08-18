use super::{
    describe_node, metadata_const_value, metadata_for, node_const_value, validated_adapter_value,
    ShapeMetadata, ShapeNode,
};
use crate::ConstValue;
use num_bigint::BigInt;
use sifr_lowering::{
    substitute_type_vars, AdapterHandlerPlan, CallableIdentity, DeclarationMetadataTargetKind,
    HirClass, HirParam, LoweringResult, MethodKind, SourceOriginId, StructuralMethodExport,
    TypedDeclarationMetadata,
};
use sifr_type_system::{
    ParamConvention, ParamMutability, ParamOwnership, ReceiverConvention, Type,
};
use std::collections::{BTreeMap, BTreeSet, HashMap};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShapeMethod {
    /// Compiler-sealed checked method target for package-selected handlers.
    pub target: Option<CallableIdentity>,
    /// Package-owned descriptor value paired with `target`.
    pub descriptor: Option<ConstValue>,
    /// Diagnostic-only source location token for the descriptor application.
    pub origin: Option<SourceOriginId>,
    /// Package declaration/inheritance order for deterministic selection.
    pub declaration_order: Option<usize>,
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

pub(super) fn const_value(method: &ShapeMethod) -> ConstValue {
    ConstValue::Record(BTreeMap::from([
        (
            "target".to_string(),
            method
                .target
                .clone()
                .map(ConstValue::CallableIdentity)
                .unwrap_or(ConstValue::None),
        ),
        (
            "descriptor".to_string(),
            method.descriptor.clone().unwrap_or(ConstValue::None),
        ),
        (
            "origin".to_string(),
            method
                .origin
                .map(ConstValue::SourceOrigin)
                .unwrap_or(ConstValue::None),
        ),
        (
            "declaration_order".to_string(),
            method
                .declaration_order
                .map(BigInt::from)
                .map(ConstValue::Integer)
                .unwrap_or(ConstValue::None),
        ),
        ("name".to_string(), ConstValue::String(method.name.clone())),
        ("kind".to_string(), ConstValue::String(method.kind.clone())),
        (
            "receiver".to_string(),
            method
                .receiver
                .clone()
                .map(ConstValue::String)
                .unwrap_or(ConstValue::None),
        ),
        ("is_async".to_string(), ConstValue::Bool(method.is_async)),
        (
            "params".to_string(),
            ConstValue::List(
                method
                    .params
                    .iter()
                    .map(|param| {
                        ConstValue::Record(BTreeMap::from([
                            ("name".to_string(), ConstValue::String(param.name.clone())),
                            ("type".to_string(), node_const_value(&param.declared_type)),
                            (
                                "convention".to_string(),
                                ConstValue::String(param.convention.clone()),
                            ),
                            (
                                "keyword_only".to_string(),
                                ConstValue::Bool(param.keyword_only),
                            ),
                            (
                                "metadata".to_string(),
                                metadata_const_value(&param.metadata),
                            ),
                        ]))
                    })
                    .collect(),
            ),
        ),
        ("result".to_string(), node_const_value(&method.result)),
        (
            "metadata".to_string(),
            metadata_const_value(&method.metadata),
        ),
    ]))
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
    let handlers = lowering
        .class_adapter_selections
        .iter()
        .find(|selection| selection.owner == class.name)
        .map(|selection| selection.handler_plans.as_slice())
        .unwrap_or_default();
    let handler_targets = handlers
        .iter()
        .filter_map(|handler| {
            Some((
                handler.callable.owner.clone()?,
                handler.callable.symbol.clone(),
            ))
        })
        .collect::<BTreeSet<_>>();
    let mut methods = lowering
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
            let local_owner = class
                .identity
                .clone()
                .unwrap_or_else(|| format!("{module_name}.{}", class.name));
            if handler_targets.contains(&(local_owner, declared_name.to_string())) {
                return None;
            }
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
        .collect::<Vec<_>>();
    let mut ordered_handlers = handlers.iter().collect::<Vec<_>>();
    ordered_handlers.sort_by_key(|handler| handler.declaration_order);
    for handler in ordered_handlers {
        methods.push(described_handler(
            module_name,
            class_name,
            class,
            handler,
            &bindings,
            lowering,
            external_defs,
            visiting,
        ));
    }
    methods
}

#[allow(clippy::too_many_arguments)]
fn described_handler(
    module_name: &str,
    class_name: &str,
    class: &HirClass,
    handler: &AdapterHandlerPlan,
    bindings: &HashMap<String, Type>,
    lowering: &LoweringResult,
    external_defs: &sifr_lowering::ExternalDefs,
    visiting: &mut BTreeSet<String>,
) -> ShapeMethod {
    let declared_name = handler.callable.symbol.as_str();
    let hir_name = if declared_name == "__init__" {
        "new"
    } else {
        declared_name
    };
    let local_owner = class
        .identity
        .clone()
        .unwrap_or_else(|| format!("{module_name}.{}", class.name));
    let detailed = (handler.callable.owner.as_deref() == Some(local_owner.as_str()))
        .then(|| {
            class
                .methods
                .iter()
                .chain(class.operator_impls.iter().map(|(_, method)| method))
                .find(|method| method.name == hir_name)
        })
        .flatten();
    let mut method = if let Some(method) = detailed {
        let owner = format!("{class_name}.{declared_name}");
        described_method(
            module_name,
            &owner,
            declared_name,
            &method.params,
            &method.return_type,
            method.method_kind,
            method.receiver,
            method.is_async,
            bindings,
            metadata_for(
                &lowering.declaration_metadata,
                &owner,
                DeclarationMetadataTargetKind::Method,
                None,
            ),
            &lowering.declaration_metadata,
            lowering,
            external_defs,
            visiting,
        )
    } else {
        ShapeMethod {
            target: None,
            descriptor: None,
            origin: None,
            declaration_order: None,
            name: declared_name.to_string(),
            kind: "inherited".to_string(),
            receiver: None,
            is_async: false,
            params: Vec::new(),
            result: Box::new(ShapeNode::Other("checked_handler".to_string())),
            metadata: Vec::new(),
        }
    };
    method.target = Some(handler.callable.clone());
    method.descriptor = Some(validated_adapter_value(&handler.descriptor_value));
    method.origin = Some(handler.descriptor_origin);
    method.declaration_order = Some(handler.declaration_order);
    method
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
        target: None,
        descriptor: None,
        origin: None,
        declaration_order: None,
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

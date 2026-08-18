//! Deterministic, package-neutral structural type descriptions for const specialization.

use crate::ConstValue;
use num_bigint::BigInt;
use sifr_lowering::{
    DeclarationMetadataTargetKind, ExternalDefs, HirClassKind, HirExpr, LoweringResult,
    TypedDeclarationMetadata,
};
use sifr_type_system::Type;
use std::collections::{BTreeMap, BTreeSet};

mod methods;
use methods::{described_exported_methods, described_methods};
pub use methods::{ShapeMethod, ShapeParameter};
mod nominal_types;
use nominal_types::{describe_enum, describe_newtype};
mod canonical_helpers;
use crate::const_canonical::canonical_value;
use canonical_helpers::canonical_sequence;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuralShape {
    pub root: ShapeNode,
    pub canonical_identity: String,
}

impl StructuralShape {
    /// Closed const representation consumed by package-owned `@const_eval` functions.
    #[must_use]
    pub fn to_const_value(&self) -> ConstValue {
        ConstValue::Record(BTreeMap::from([
            (
                "canonical_identity".to_string(),
                ConstValue::String(self.canonical_identity.clone()),
            ),
            ("root".to_string(), node_const_value(&self.root)),
        ]))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShapeField {
    pub name: String,
    pub declared_type: ShapeNode,
    pub default: Option<ConstValue>,
    pub metadata: Vec<ShapeMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShapeMetadata {
    pub key: String,
    pub value_type: String,
    pub value: ConstValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShapeEnumVariant {
    pub name: String,
    pub value: Option<i64>,
    pub metadata: Vec<ShapeMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShapeNode {
    Primitive(String),
    FixedInteger(String),
    List(Box<Self>),
    Dictionary(Box<Self>, Box<Self>),
    Set(Box<Self>),
    Tuple(Vec<Self>),
    Optional(Box<Self>),
    Union(Vec<Self>),
    Nominal {
        identity: String,
        type_arguments: Vec<Self>,
        fields: Vec<ShapeField>,
        methods: Vec<ShapeMethod>,
        metadata: Vec<ShapeMetadata>,
    },
    Enum {
        identity: String,
        variants: Vec<ShapeEnumVariant>,
        metadata: Vec<ShapeMetadata>,
    },
    Newtype {
        identity: String,
        inner: Box<Self>,
        metadata: Vec<ShapeMetadata>,
    },
    RecursiveReference(String),
    TypeParameter(String),
    Other(String),
}

/// Describe a type with project export metadata available for imported nominals.
#[must_use]
pub fn describe_type_with_externals(
    module_name: &str,
    ty: &Type,
    lowering: &LoweringResult,
    external_defs: &ExternalDefs,
) -> StructuralShape {
    describe_type_inner(module_name, ty, lowering, external_defs)
}

fn describe_type_inner(
    module_name: &str,
    ty: &Type,
    lowering: &LoweringResult,
    external_defs: &ExternalDefs,
) -> StructuralShape {
    let mut visiting = BTreeSet::new();
    let root = describe_node(
        module_name,
        ty.resolve_alias(),
        lowering,
        external_defs,
        &mut visiting,
    );
    let canonical_identity = canonical_node(&root);
    StructuralShape {
        root,
        canonical_identity,
    }
}

fn describe_node(
    module_name: &str,
    ty: &Type,
    lowering: &LoweringResult,
    external_defs: &ExternalDefs,
    visiting: &mut BTreeSet<String>,
) -> ShapeNode {
    match ty {
        Type::Int => primitive("int"),
        Type::FixedInt(kind) => ShapeNode::FixedInteger(kind.source_name().to_string()),
        Type::Float => primitive("float"),
        Type::Bool => primitive("bool"),
        Type::Str => primitive("str"),
        Type::Bytes => primitive("bytes"),
        Type::None => primitive("None"),
        Type::Decimal => primitive("decimal"),
        Type::BigDecimal => primitive("bigdecimal"),
        Type::List(element) => ShapeNode::List(Box::new(describe_node(
            module_name,
            element,
            lowering,
            external_defs,
            visiting,
        ))),
        Type::Dict(key, value) => ShapeNode::Dictionary(
            Box::new(describe_node(
                module_name,
                key,
                lowering,
                external_defs,
                visiting,
            )),
            Box::new(describe_node(
                module_name,
                value,
                lowering,
                external_defs,
                visiting,
            )),
        ),
        Type::Set(element) => ShapeNode::Set(Box::new(describe_node(
            module_name,
            element,
            lowering,
            external_defs,
            visiting,
        ))),
        Type::Tuple(elements) => ShapeNode::Tuple(
            elements
                .iter()
                .map(|element| {
                    describe_node(module_name, element, lowering, external_defs, visiting)
                })
                .collect(),
        ),
        Type::Union(members) => {
            describe_union(module_name, members, lowering, external_defs, visiting)
        }
        Type::TypeVar(name) => ShapeNode::TypeParameter(name.clone()),
        Type::Enum {
            identity,
            name,
            variants,
        } => describe_enum(
            module_name,
            identity.as_deref(),
            name,
            variants,
            lowering,
            external_defs,
        ),
        Type::Newtype {
            identity,
            name,
            inner,
        } => describe_newtype(
            module_name,
            identity.as_deref(),
            name,
            inner,
            lowering,
            external_defs,
            visiting,
        ),
        Type::Class {
            identity,
            type_args,
            name,
            fields,
            ..
        } => describe_class(
            module_name,
            identity.as_deref().unwrap_or(name),
            name,
            type_args,
            fields,
            lowering,
            external_defs,
            visiting,
        ),
        other => ShapeNode::Other(other.display_name()),
    }
}

fn primitive(name: &str) -> ShapeNode {
    ShapeNode::Primitive(name.to_string())
}

fn describe_union(
    module_name: &str,
    members: &[Type],
    lowering: &LoweringResult,
    external_defs: &ExternalDefs,
    visiting: &mut BTreeSet<String>,
) -> ShapeNode {
    let non_none = members
        .iter()
        .filter(|member| !matches!(member.resolve_alias(), Type::None))
        .collect::<Vec<_>>();
    if members.len() == 2 && non_none.len() == 1 {
        return ShapeNode::Optional(Box::new(describe_node(
            module_name,
            non_none[0],
            lowering,
            external_defs,
            visiting,
        )));
    }
    ShapeNode::Union(
        members
            .iter()
            .map(|member| describe_node(module_name, member, lowering, external_defs, visiting))
            .collect(),
    )
}

#[allow(clippy::too_many_arguments)]
fn describe_class(
    module_name: &str,
    declared_identity: &str,
    local_name: &str,
    type_args: &[Type],
    fields: &[(String, Type)],
    lowering: &LoweringResult,
    external_defs: &ExternalDefs,
    visiting: &mut BTreeSet<String>,
) -> ShapeNode {
    let (source_module, source_name) = declared_identity
        .rsplit_once('.')
        .unwrap_or((module_name, declared_identity));
    let identity = format!("{source_module}.{source_name}");
    if !visiting.insert(identity.clone()) {
        return ShapeNode::RecursiveReference(identity);
    }

    let local_identity = format!("{module_name}.{local_name}");
    let local_class = (identity == local_identity)
        .then(|| {
            lowering
                .module
                .classes
                .iter()
                .find(|class| class.name == local_name)
        })
        .flatten();
    if let Some(class) = local_class {
        if matches!(class.kind, HirClassKind::Enum) {
            let described = describe_enum(
                module_name,
                Some(&identity),
                local_name,
                &class.enum_variants,
                lowering,
                external_defs,
            );
            visiting.remove(&identity);
            return described;
        }
        if let Some(inner) = &class.newtype_inner {
            let described = describe_newtype(
                module_name,
                Some(&identity),
                local_name,
                inner,
                lowering,
                external_defs,
                visiting,
            );
            visiting.remove(&identity);
            return described;
        }
    }

    let (declaration_metadata, adapter_metadata, metadata_owner, defaults) =
        if local_class.is_some() {
            (
                lowering.declaration_metadata.as_slice(),
                lowering.applied_adapter_metadata.as_slice(),
                local_name,
                lowering.class_field_defaults.get(local_name),
            )
        } else {
            (
                external_defs
                    .declaration_metadata
                    .get(source_module)
                    .map_or(&[][..], Vec::as_slice),
                external_defs
                    .applied_adapter_metadata
                    .get(source_module)
                    .map_or(&[][..], Vec::as_slice),
                source_name,
                external_defs
                    .class_field_defaults
                    .get(source_module)
                    .and_then(|classes| classes.get(source_name)),
            )
        };
    let described_fields = fields
        .iter()
        .enumerate()
        .map(|(index, (name, ty))| ShapeField {
            name: name.clone(),
            declared_type: describe_node(module_name, ty, lowering, external_defs, visiting),
            default: defaults
                .and_then(|values| values.iter().find(|(field, _)| *field == index))
                .and_then(|(_, value)| const_value_from_hir(value)),
            metadata: combined_metadata_for(
                declaration_metadata,
                adapter_metadata,
                metadata_owner,
                DeclarationMetadataTargetKind::Field,
                Some(name),
            ),
        })
        .collect();
    let described_methods = if let Some(class) = local_class {
        described_methods(
            module_name,
            local_name,
            class,
            type_args,
            lowering,
            external_defs,
            visiting,
        )
    } else {
        let methods = external_defs
            .structural_methods_for(source_module)
            .and_then(|classes| classes.get(source_name))
            .map(Vec::as_slice)
            .unwrap_or_default();
        let type_params = external_defs
            .class_type_params
            .get(source_module)
            .and_then(|classes| classes.get(source_name))
            .map(Vec::as_slice)
            .unwrap_or_default();
        described_exported_methods(
            module_name,
            source_name,
            methods,
            type_params,
            type_args,
            declaration_metadata,
            lowering,
            external_defs,
            visiting,
        )
    };
    let type_arguments = type_args
        .iter()
        .map(|argument| describe_node(module_name, argument, lowering, external_defs, visiting))
        .collect();
    visiting.remove(&identity);
    ShapeNode::Nominal {
        identity: identity.clone(),
        type_arguments,
        fields: described_fields,
        methods: described_methods,
        metadata: combined_metadata_for(
            declaration_metadata,
            adapter_metadata,
            metadata_owner,
            DeclarationMetadataTargetKind::Type,
            None,
        ),
    }
}

fn combined_metadata_for(
    metadata: &[TypedDeclarationMetadata],
    adapter_metadata: &[sifr_lowering::AppliedAdapterMetadata],
    owner: &str,
    target_kind: DeclarationMetadataTargetKind,
    target_name: Option<&str>,
) -> Vec<ShapeMetadata> {
    let mut values = metadata_for(metadata, owner, target_kind, target_name);
    values.extend(
        adapter_metadata
            .iter()
            .filter(|item| {
                item.owner == owner
                    && item.target_kind == target_kind
                    && item.target_name.as_deref() == target_name
            })
            .map(|item| ShapeMetadata {
                key: item.key.clone(),
                value_type: item.value_type.display_name(),
                value: validated_adapter_value(&item.value),
            }),
    );
    values
}

#[allow(clippy::expect_used)]
fn validated_adapter_value(value: &sifr_lowering::StaticProgramValue) -> ConstValue {
    // Adapter metadata reaches this point only through static_program_value(),
    // which canonicalizes integer text while validating the plan.
    crate::specialization_support::const_value(value)
        .expect("validated adapter metadata has canonical static values")
}

fn metadata_for(
    metadata: &[TypedDeclarationMetadata],
    owner: &str,
    target_kind: DeclarationMetadataTargetKind,
    target_name: Option<&str>,
) -> Vec<ShapeMetadata> {
    metadata
        .iter()
        .filter(|item| {
            item.owner == owner
                && item.target_kind == target_kind
                && item.target_name.as_deref() == target_name
        })
        .filter_map(|item| {
            const_value_from_hir(&item.value).map(|value| ShapeMetadata {
                key: item.key.clone(),
                value_type: item.value_type.display_name(),
                value,
            })
        })
        .collect()
}

pub(crate) fn const_value_from_hir(expr: &HirExpr) -> Option<ConstValue> {
    match expr {
        HirExpr::IntLiteral(value) => Some(ConstValue::Integer(BigInt::from(*value))),
        HirExpr::LargeIntLiteral(value) => value.parse().ok().map(ConstValue::Integer),
        HirExpr::FloatLiteral(value) => Some(ConstValue::FloatBits(value.to_bits())),
        HirExpr::StringLiteral(value) => Some(ConstValue::String(value.clone())),
        HirExpr::BoolLiteral(value) => Some(ConstValue::Bool(*value)),
        HirExpr::NoneLiteral => Some(ConstValue::None),
        HirExpr::ListLiteral { elements, ty } if matches!(ty.resolve_alias(), Type::Bytes) => {
            elements
                .iter()
                .map(|element| match element {
                    HirExpr::IntLiteral(value) => u8::try_from(*value).ok(),
                    _ => None,
                })
                .collect::<Option<Vec<_>>>()
                .map(ConstValue::Bytes)
        }
        HirExpr::ListLiteral { elements, .. } => elements
            .iter()
            .map(const_value_from_hir)
            .collect::<Option<Vec<_>>>()
            .map(ConstValue::List),
        HirExpr::TupleLiteral { elements, .. } => elements
            .iter()
            .map(const_value_from_hir)
            .collect::<Option<Vec<_>>>()
            .map(ConstValue::Tuple),
        _ => None,
    }
}

fn canonical_node(node: &ShapeNode) -> String {
    match node {
        ShapeNode::Primitive(name) => name.clone(),
        ShapeNode::FixedInteger(name) => format!("fixed:{name}"),
        ShapeNode::List(element) => format!("list[{}]", canonical_node(element)),
        ShapeNode::Dictionary(key, value) => {
            format!("dict[{},{}]", canonical_node(key), canonical_node(value))
        }
        ShapeNode::Set(element) => format!("set[{}]", canonical_node(element)),
        ShapeNode::Tuple(elements) => canonical_sequence("tuple", elements),
        ShapeNode::Optional(element) => format!("optional[{}]", canonical_node(element)),
        ShapeNode::Union(elements) => canonical_sequence("union", elements),
        ShapeNode::Nominal {
            identity,
            type_arguments,
            fields,
            methods,
            metadata,
        } => {
            let args = canonical_sequence("args", type_arguments);
            let fields = fields
                .iter()
                .map(|field| {
                    let default = field.default.as_ref().map_or_else(
                        || "required".to_string(),
                        |value| format!("default={}", canonical_value(value)),
                    );
                    format!(
                        "{}:{}:{default}:{}",
                        field.name,
                        canonical_node(&field.declared_type),
                        canonical_metadata(&field.metadata),
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            let methods = methods
                .iter()
                .map(canonical_method)
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "class:{identity}:{args}:meta[{}]:fields[{fields}]:methods[{methods}]",
                canonical_metadata(metadata)
            )
        }
        ShapeNode::Enum {
            identity,
            variants,
            metadata,
        } => {
            let variants = variants
                .iter()
                .map(|variant| {
                    let value = variant
                        .value
                        .map_or_else(|| "none".to_string(), |value| format!("i64:{value}"));
                    format!(
                        "{}={}:{}",
                        variant.name,
                        value,
                        canonical_metadata(&variant.metadata)
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "enum:{identity}:meta[{}]:variants[{variants}]",
                canonical_metadata(metadata)
            )
        }
        ShapeNode::Newtype {
            identity,
            inner,
            metadata,
        } => {
            format!(
                "newtype:{identity}:meta[{}][{}]",
                canonical_metadata(metadata),
                canonical_node(inner)
            )
        }
        ShapeNode::RecursiveReference(identity) => format!("ref:{identity}"),
        ShapeNode::TypeParameter(name) => format!("param:{name}"),
        ShapeNode::Other(name) => format!("other:{name}"),
    }
}

fn canonical_method(method: &ShapeMethod) -> String {
    let params = method
        .params
        .iter()
        .map(|param| {
            format!(
                "{}:{}:{}:{}:{}:{}",
                param.name.len(),
                param.name,
                param.convention,
                param.keyword_only,
                canonical_node(&param.declared_type),
                canonical_metadata(&param.metadata)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let target = method.target.as_ref().map_or_else(
        || "none".to_string(),
        |target| canonical_value(&ConstValue::CallableIdentity(target.clone())),
    );
    let descriptor = method
        .descriptor
        .as_ref()
        .map_or_else(|| "none".to_string(), canonical_value);
    format!(
        "{}:{}:{}:{}:{}:target[{target}]:descriptor[{descriptor}]:params[{params}]:result[{}]:meta[{}]",
        method.name.len(),
        method.name,
        method.kind,
        method.receiver.as_deref().unwrap_or("none"),
        method.is_async,
        canonical_node(&method.result),
        canonical_metadata(&method.metadata)
    )
}

pub(super) fn node_const_value(node: &ShapeNode) -> ConstValue {
    let mut record = BTreeMap::new();
    match node {
        ShapeNode::Primitive(name) => {
            record.insert(
                "kind".to_string(),
                ConstValue::String("primitive".to_string()),
            );
            record.insert("name".to_string(), ConstValue::String(name.clone()));
        }
        ShapeNode::FixedInteger(name) => {
            record.insert(
                "kind".to_string(),
                ConstValue::String("fixed_integer".to_string()),
            );
            record.insert("name".to_string(), ConstValue::String(name.clone()));
        }
        ShapeNode::List(element) | ShapeNode::Set(element) | ShapeNode::Optional(element) => {
            let kind = match node {
                ShapeNode::List(_) => "list",
                ShapeNode::Set(_) => "set",
                _ => "optional",
            };
            record.insert("kind".to_string(), ConstValue::String(kind.to_string()));
            record.insert("element".to_string(), node_const_value(element));
        }
        ShapeNode::Dictionary(key, value) => {
            record.insert(
                "kind".to_string(),
                ConstValue::String("dictionary".to_string()),
            );
            record.insert("key".to_string(), node_const_value(key));
            record.insert("value".to_string(), node_const_value(value));
        }
        ShapeNode::Tuple(elements) | ShapeNode::Union(elements) => {
            let kind = if matches!(node, ShapeNode::Tuple(_)) {
                "tuple"
            } else {
                "union"
            };
            record.insert("kind".to_string(), ConstValue::String(kind.to_string()));
            record.insert(
                "members".to_string(),
                ConstValue::List(elements.iter().map(node_const_value).collect()),
            );
        }
        ShapeNode::Nominal {
            identity,
            type_arguments,
            fields,
            methods,
            metadata,
        } => {
            record.insert(
                "kind".to_string(),
                ConstValue::String("nominal".to_string()),
            );
            record.insert("identity".to_string(), ConstValue::String(identity.clone()));
            record.insert(
                "type_arguments".to_string(),
                ConstValue::List(type_arguments.iter().map(node_const_value).collect()),
            );
            record.insert(
                "fields".to_string(),
                ConstValue::List(
                    fields
                        .iter()
                        .map(|field| {
                            ConstValue::Record(BTreeMap::from([
                                ("name".to_string(), ConstValue::String(field.name.clone())),
                                ("type".to_string(), node_const_value(&field.declared_type)),
                                (
                                    "required".to_string(),
                                    ConstValue::Bool(field.default.is_none()),
                                ),
                                (
                                    "default".to_string(),
                                    field.default.clone().unwrap_or(ConstValue::None),
                                ),
                                (
                                    "metadata".to_string(),
                                    metadata_const_value(&field.metadata),
                                ),
                            ]))
                        })
                        .collect(),
                ),
            );
            record.insert(
                "methods".to_string(),
                ConstValue::List(methods.iter().map(methods::const_value).collect()),
            );
            record.insert("metadata".to_string(), metadata_const_value(metadata));
        }
        ShapeNode::Enum {
            identity,
            variants,
            metadata,
        } => {
            record.insert("kind".to_string(), ConstValue::String("enum".to_string()));
            record.insert("identity".to_string(), ConstValue::String(identity.clone()));
            record.insert(
                "variants".to_string(),
                ConstValue::List(
                    variants
                        .iter()
                        .map(|variant| {
                            ConstValue::Record(BTreeMap::from([
                                ("name".to_string(), ConstValue::String(variant.name.clone())),
                                (
                                    "value".to_string(),
                                    variant
                                        .value
                                        .map(BigInt::from)
                                        .map(ConstValue::Integer)
                                        .unwrap_or(ConstValue::None),
                                ),
                                (
                                    "metadata".to_string(),
                                    metadata_const_value(&variant.metadata),
                                ),
                            ]))
                        })
                        .collect(),
                ),
            );
            record.insert("metadata".to_string(), metadata_const_value(metadata));
        }
        ShapeNode::Newtype {
            identity,
            inner,
            metadata,
        } => {
            record.insert(
                "kind".to_string(),
                ConstValue::String("newtype".to_string()),
            );
            record.insert("identity".to_string(), ConstValue::String(identity.clone()));
            record.insert("inner".to_string(), node_const_value(inner));
            record.insert("metadata".to_string(), metadata_const_value(metadata));
        }
        ShapeNode::RecursiveReference(identity) => {
            record.insert(
                "kind".to_string(),
                ConstValue::String("recursive_reference".to_string()),
            );
            record.insert("identity".to_string(), ConstValue::String(identity.clone()));
        }
        ShapeNode::TypeParameter(name) => {
            record.insert(
                "kind".to_string(),
                ConstValue::String("type_parameter".to_string()),
            );
            record.insert("name".to_string(), ConstValue::String(name.clone()));
        }
        ShapeNode::Other(name) => {
            record.insert("kind".to_string(), ConstValue::String("other".to_string()));
            record.insert("name".to_string(), ConstValue::String(name.clone()));
        }
    }
    ConstValue::Record(record)
}

pub(super) fn metadata_const_value(metadata: &[ShapeMetadata]) -> ConstValue {
    ConstValue::List(
        metadata
            .iter()
            .map(|item| {
                ConstValue::Record(BTreeMap::from([
                    ("key".to_string(), ConstValue::String(item.key.clone())),
                    (
                        "value_type".to_string(),
                        ConstValue::String(item.value_type.clone()),
                    ),
                    ("value".to_string(), item.value.clone()),
                ]))
            })
            .collect(),
    )
}

fn canonical_metadata(metadata: &[ShapeMetadata]) -> String {
    metadata
        .iter()
        .map(|item| {
            format!(
                "{}:{}={}",
                item.key,
                item.value_type,
                canonical_value(&item.value)
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

#[cfg(test)]
mod tests;

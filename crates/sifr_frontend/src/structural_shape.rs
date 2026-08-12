//! Deterministic, package-neutral structural type descriptions for const specialization.

use crate::ConstValue;
use num_bigint::BigInt;
use sifr_lowering::{
    DeclarationMetadataTargetKind, HirClassKind, HirExpr, LoweringResult, TypedDeclarationMetadata,
};
use sifr_type_system::Type;
use std::collections::{BTreeMap, BTreeSet};

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

/// Describe a concrete type as seen from one lowered module.
///
/// The output uses only declaration identity and ordered vectors. It therefore remains stable
/// across hash-map seeds and process boundaries and can be used directly as an incremental query
/// key or serialized static description.
#[must_use]
pub fn describe_type(module_name: &str, ty: &Type, lowering: &LoweringResult) -> StructuralShape {
    let mut visiting = BTreeSet::new();
    let root = describe_node(module_name, ty.resolve_alias(), lowering, &mut visiting);
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
    visiting: &mut BTreeSet<String>,
) -> ShapeNode {
    match ty {
        Type::Int => primitive("int"),
        Type::FixedInt(kind) => ShapeNode::FixedInteger(kind.source_name().to_string()),
        Type::BigInt => primitive("bigint"),
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
            visiting,
        ))),
        Type::Dict(key, value) => ShapeNode::Dictionary(
            Box::new(describe_node(module_name, key, lowering, visiting)),
            Box::new(describe_node(module_name, value, lowering, visiting)),
        ),
        Type::Set(element) => ShapeNode::Set(Box::new(describe_node(
            module_name,
            element,
            lowering,
            visiting,
        ))),
        Type::Tuple(elements) => ShapeNode::Tuple(
            elements
                .iter()
                .map(|element| describe_node(module_name, element, lowering, visiting))
                .collect(),
        ),
        Type::Union(members) => describe_union(module_name, members, lowering, visiting),
        Type::TypeVar(name) => ShapeNode::TypeParameter(name.clone()),
        Type::Enum { name, variants, .. } => ShapeNode::Enum {
            identity: format!("{module_name}.{name}"),
            variants: variants
                .iter()
                .map(|(name, value)| ShapeEnumVariant {
                    name: name.clone(),
                    value: *value,
                    metadata: Vec::new(),
                })
                .collect(),
            metadata: Vec::new(),
        },
        Type::Newtype { name, inner, .. } => ShapeNode::Newtype {
            identity: format!("{module_name}.{name}"),
            inner: Box::new(describe_node(module_name, inner, lowering, visiting)),
            metadata: Vec::new(),
        },
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
            visiting,
        )));
    }
    ShapeNode::Union(
        members
            .iter()
            .map(|member| describe_node(module_name, member, lowering, visiting))
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
    visiting: &mut BTreeSet<String>,
) -> ShapeNode {
    let identity = if declared_identity.contains('.') {
        declared_identity.to_string()
    } else {
        format!("{module_name}.{declared_identity}")
    };
    if !visiting.insert(identity.clone()) {
        return ShapeNode::RecursiveReference(identity);
    }

    let local_class = lowering
        .module
        .classes
        .iter()
        .find(|class| class.name == local_name);
    if let Some(class) = local_class {
        if matches!(class.kind, HirClassKind::Enum) {
            visiting.remove(&identity);
            return ShapeNode::Enum {
                identity,
                variants: class
                    .enum_variants
                    .iter()
                    .map(|(name, value)| ShapeEnumVariant {
                        name: name.clone(),
                        value: *value,
                        metadata: metadata_for(
                            &lowering.declaration_metadata,
                            local_name,
                            DeclarationMetadataTargetKind::EnumVariant,
                            Some(name),
                        ),
                    })
                    .collect(),
                metadata: metadata_for(
                    &lowering.declaration_metadata,
                    local_name,
                    DeclarationMetadataTargetKind::Type,
                    None,
                ),
            };
        }
        if let Some(inner) = &class.newtype_inner {
            let inner = describe_node(module_name, inner, lowering, visiting);
            visiting.remove(&identity);
            return ShapeNode::Newtype {
                identity,
                inner: Box::new(inner),
                metadata: metadata_for(
                    &lowering.declaration_metadata,
                    local_name,
                    DeclarationMetadataTargetKind::Type,
                    None,
                ),
            };
        }
    }

    let defaults = lowering.class_field_defaults.get(local_name);
    let described_fields = fields
        .iter()
        .enumerate()
        .map(|(index, (name, ty))| ShapeField {
            name: name.clone(),
            declared_type: describe_node(module_name, ty, lowering, visiting),
            default: defaults
                .and_then(|values| values.iter().find(|(field, _)| *field == index))
                .and_then(|(_, value)| const_value_from_hir(value)),
            metadata: metadata_for(
                &lowering.declaration_metadata,
                local_name,
                DeclarationMetadataTargetKind::Field,
                Some(name),
            ),
        })
        .collect();
    let type_arguments = type_args
        .iter()
        .map(|argument| describe_node(module_name, argument, lowering, visiting))
        .collect();
    visiting.remove(&identity);
    ShapeNode::Nominal {
        identity,
        type_arguments,
        fields: described_fields,
        metadata: metadata_for(
            &lowering.declaration_metadata,
            local_name,
            DeclarationMetadataTargetKind::Type,
            None,
        ),
    }
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
            format!(
                "class:{identity}:{args}:meta[{}]:fields[{fields}]",
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
                    format!(
                        "{}={:?}:{}",
                        variant.name,
                        variant.value,
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

fn node_const_value(node: &ShapeNode) -> ConstValue {
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

fn metadata_const_value(metadata: &[ShapeMetadata]) -> ConstValue {
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

pub(crate) fn canonical_value(value: &ConstValue) -> String {
    match value {
        ConstValue::None => "none".to_string(),
        ConstValue::Bool(value) => format!("bool:{value}"),
        ConstValue::Integer(value) => format!("int:{value}"),
        ConstValue::FloatBits(value) => format!("float:{value:016x}"),
        ConstValue::String(value) => format!("str:{}:{value}", value.len()),
        ConstValue::Bytes(value) => format!("bytes:{}:{}", value.len(), canonical_bytes(value)),
        ConstValue::Tuple(values) => canonical_values("tuple", values),
        ConstValue::List(values) => canonical_values("list", values),
        ConstValue::Record(values) => format!(
            "record[{}]",
            values
                .iter()
                .map(|(key, value)| { format!("{}:{key}={}", key.len(), canonical_value(value)) })
                .collect::<Vec<_>>()
                .join(",")
        ),
    }
}

fn canonical_bytes(value: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(value.len() * 2);
    for byte in value {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}

fn canonical_values(kind: &str, values: &[ConstValue]) -> String {
    format!(
        "{kind}[{}]",
        values
            .iter()
            .map(canonical_value)
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn canonical_sequence(kind: &str, elements: &[ShapeNode]) -> String {
    format!(
        "{kind}[{}]",
        elements
            .iter()
            .map(canonical_node)
            .collect::<Vec<_>>()
            .join(",")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use sifr_lowering::lower_module;
    use sifr_syntax::parse_module_suite;

    #[test]
    fn class_shape_preserves_defaults_generics_and_recursive_identity() {
        let source = "class Node[T]:\n    value: T\n    label: str = \"root\"\n    next: Node[T] | None = None\n";
        let parsed = parse_module_suite(source, None).expect("fixture parses");
        let lowered = lower_module(&parsed).expect("fixture lowers");
        let class = lowered.module.classes.first().expect("class exists");
        let ty = Type::Class {
            identity: None,
            type_args: vec![Type::Int],
            name: class.name.clone(),
            fields: class.fields.clone(),
            methods: Vec::new(),
            parent_class: None,
        };
        let shape = describe_type("fixture.shapes", &ty, &lowered);
        assert!(shape.canonical_identity.contains("fixture.shapes.Node"));
        assert!(shape.canonical_identity.contains("label:str:default"));
        assert!(shape.canonical_identity.contains("ref:fixture.shapes.Node"));
    }

    #[test]
    fn union_description_is_deterministic() {
        let parsed = parse_module_suite("", None).expect("fixture parses");
        let lowered = lower_module(&parsed).expect("fixture lowers");
        let ty = Type::Union(vec![Type::Int, Type::Str]);
        assert_eq!(
            describe_type("fixture", &ty, &lowered),
            describe_type("fixture", &ty, &lowered)
        );
    }

    #[test]
    fn explicit_initializer_does_not_erase_field_default_metadata() {
        let source = "class Config:\n    retries: int = 3\n\n    def __init__(self, retries: int) -> None:\n        self.retries = retries\n";
        let parsed = parse_module_suite(source, None).expect("fixture parses");
        let lowered = lower_module(&parsed).expect("fixture lowers");
        let class = lowered.module.classes.first().expect("class exists");
        let ty = Type::Class {
            identity: None,
            type_args: Vec::new(),
            name: class.name.clone(),
            fields: class.fields.clone(),
            methods: Vec::new(),
            parent_class: None,
        };
        let shape = describe_type("fixture.defaults", &ty, &lowered);
        assert!(shape
            .canonical_identity
            .contains("retries:int:default=int:3"));
    }

    #[test]
    fn enum_and_newtype_shapes_preserve_metadata_and_nominal_identity() {
        let source = r#"
from enum import Enum

@metadata("fixture.kind", "color")
@metadata("enum_variant", "RED", "fixture.label", "red")
class Color(Enum):
    RED = 1
    BLUE = 2

@metadata("fixture.kind", "port")
class Port(int):
    pass
"#;
        let parsed = parse_module_suite(source, None).expect("fixture parses");
        let lowered = lower_module(&parsed).expect("fixture lowers");
        let color = &lowered.module.classes[0];
        let color_ty = Type::Class {
            identity: None,
            type_args: Vec::new(),
            name: color.name.clone(),
            fields: color.fields.clone(),
            methods: Vec::new(),
            parent_class: None,
        };
        let color_shape = describe_type("fixture.nominal", &color_ty, &lowered);
        assert!(color_shape
            .canonical_identity
            .contains("enum:fixture.nominal.Color"));
        assert!(color_shape.canonical_identity.contains("fixture.label"));

        let port = &lowered.module.classes[1];
        let port_ty = Type::Class {
            identity: None,
            type_args: Vec::new(),
            name: port.name.clone(),
            fields: port.fields.clone(),
            methods: Vec::new(),
            parent_class: None,
        };
        let port_shape = describe_type("fixture.nominal", &port_ty, &lowered);
        assert!(port_shape
            .canonical_identity
            .contains("newtype:fixture.nominal.Port"));
        assert!(port_shape.canonical_identity.contains("fixture.kind"));
    }

    #[test]
    fn canonical_values_bind_record_keys_and_bytes_without_collisions() {
        let ambiguous_key = ConstValue::Record(BTreeMap::from([(
            "a=str:1:x,b".to_string(),
            ConstValue::None,
        )]));
        let two_fields = ConstValue::Record(BTreeMap::from([
            ("a".to_string(), ConstValue::String("x".to_string())),
            ("b".to_string(), ConstValue::None),
        ]));
        assert_ne!(
            canonical_value(&ambiguous_key),
            canonical_value(&two_fields)
        );
        assert_eq!(
            canonical_value(&ConstValue::Bytes(vec![0, 255])),
            "bytes:2:00ff"
        );
        assert_ne!(
            canonical_value(&ConstValue::Bytes(Vec::new())),
            canonical_value(&ConstValue::String(String::new()))
        );
    }
}

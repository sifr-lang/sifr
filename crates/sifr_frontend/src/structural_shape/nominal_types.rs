use super::{describe_node, metadata_for, ShapeEnumVariant, ShapeNode};
use sifr_lowering::{
    DeclarationMetadataTargetKind, ExternalDefs, LoweringResult, TypedDeclarationMetadata,
};
use sifr_type_system::Type;
use std::collections::BTreeSet;

pub(super) fn describe_enum(
    module_name: &str,
    declared_identity: Option<&str>,
    local_name: &str,
    variants: &[(String, Option<i64>)],
    lowering: &LoweringResult,
    external_defs: Option<&ExternalDefs>,
) -> ShapeNode {
    let identity = qualified_identity(module_name, declared_identity.unwrap_or(local_name));
    let (metadata, owner) =
        declaration_metadata_for(&identity, local_name, lowering, external_defs);
    ShapeNode::Enum {
        identity,
        variants: variants
            .iter()
            .map(|(name, value)| ShapeEnumVariant {
                name: name.clone(),
                value: *value,
                metadata: metadata_for(
                    metadata,
                    &owner,
                    DeclarationMetadataTargetKind::EnumVariant,
                    Some(name),
                ),
            })
            .collect(),
        metadata: metadata_for(metadata, &owner, DeclarationMetadataTargetKind::Type, None),
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn describe_newtype(
    module_name: &str,
    declared_identity: Option<&str>,
    local_name: &str,
    inner: &Type,
    lowering: &LoweringResult,
    external_defs: Option<&ExternalDefs>,
    visiting: &mut BTreeSet<String>,
) -> ShapeNode {
    let identity = qualified_identity(module_name, declared_identity.unwrap_or(local_name));
    let (metadata, owner) =
        declaration_metadata_for(&identity, local_name, lowering, external_defs);
    ShapeNode::Newtype {
        identity,
        inner: Box::new(describe_node(
            module_name,
            inner,
            lowering,
            external_defs,
            visiting,
        )),
        metadata: metadata_for(metadata, &owner, DeclarationMetadataTargetKind::Type, None),
    }
}

pub(super) fn qualified_identity(module_name: &str, declared_identity: &str) -> String {
    if declared_identity.contains('.') {
        declared_identity.to_string()
    } else {
        format!("{module_name}.{declared_identity}")
    }
}

fn declaration_metadata_for<'a>(
    identity: &str,
    local_name: &str,
    lowering: &'a LoweringResult,
    external_defs: Option<&'a ExternalDefs>,
) -> (&'a [TypedDeclarationMetadata], String) {
    if lowering
        .module
        .classes
        .iter()
        .any(|class| class.name == local_name)
    {
        return (&lowering.declaration_metadata, local_name.to_string());
    }
    let Some((source_module, source_name)) = identity.rsplit_once('.') else {
        return (&[], local_name.to_string());
    };
    let metadata = external_defs
        .and_then(|defs| defs.declaration_metadata.get(source_module))
        .map_or(&[][..], Vec::as_slice);
    (metadata, source_name.to_string())
}

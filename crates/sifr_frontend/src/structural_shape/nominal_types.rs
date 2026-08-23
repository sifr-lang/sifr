use super::{ShapeEnumVariant, ShapeNode, describe_node, metadata_for};
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
    external_defs: &ExternalDefs,
) -> ShapeNode {
    let (source_module, source_name) =
        identity_parts(module_name, declared_identity.unwrap_or(local_name));
    let identity = format!("{source_module}.{source_name}");
    let (metadata, owner) = declaration_metadata_for(
        module_name,
        source_module,
        source_name,
        local_name,
        lowering,
        external_defs,
    );
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
    external_defs: &ExternalDefs,
    visiting: &mut BTreeSet<String>,
) -> ShapeNode {
    let (source_module, source_name) =
        identity_parts(module_name, declared_identity.unwrap_or(local_name));
    let identity = format!("{source_module}.{source_name}");
    let (metadata, owner) = declaration_metadata_for(
        module_name,
        source_module,
        source_name,
        local_name,
        lowering,
        external_defs,
    );
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

fn identity_parts<'a>(module_name: &'a str, declared_identity: &'a str) -> (&'a str, &'a str) {
    declared_identity
        .rsplit_once('.')
        .unwrap_or((module_name, declared_identity))
}

fn declaration_metadata_for<'a>(
    module_name: &str,
    source_module: &str,
    source_name: &str,
    local_name: &str,
    lowering: &'a LoweringResult,
    external_defs: &'a ExternalDefs,
) -> (&'a [TypedDeclarationMetadata], String) {
    if source_module == module_name
        && source_name == local_name
        && lowering
            .module
            .classes
            .iter()
            .any(|class| class.name == local_name)
    {
        return (&lowering.declaration_metadata, local_name.to_string());
    }
    let metadata = external_defs
        .declaration_metadata
        .get(source_module)
        .map_or(&[][..], Vec::as_slice);
    (metadata, source_name.to_string())
}

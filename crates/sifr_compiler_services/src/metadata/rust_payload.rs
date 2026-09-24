use super::{Encode, Encoder, Result, wire};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use syn::spanned::Spanned;

type RustProjection = (
    Option<wire::Ref<wire::RustPayload>>,
    BTreeSet<wire::Ref<wire::Text>>,
);

pub(super) fn project(
    compiled: &crate::stdlib::SourceStdlibCompiled,
    module_name: &str,
    module: wire::Ref<wire::Module>,
    declarations: &BTreeMap<wire::Ref<wire::Text>, wire::Ref<wire::Declaration>>,
    cx: &mut Encoder,
) -> Result<RustProjection> {
    let Some(rust) = compiled.code.module_rust_code.get(module_name) else {
        return Ok((None, BTreeSet::new()));
    };
    // Name/range extraction uses the same Rust grammar as codegen's completed
    // fragment validation. The captured bytes remain the emitted authority.
    let parsed = syn::parse_file(&rust.rust)
        .map_err(|e| wire::MetadataError(format!("emitted stdlib Rust for {module_name}: {e}")))?;
    let constants = compiled
        .code
        .module_constants
        .get(module_name)
        .cloned()
        .unwrap_or_default();
    let mut names = BTreeMap::new();
    let mut support = BTreeSet::new();
    let mut mappings = Vec::new();
    for item in &parsed.items {
        let ident = match item {
            syn::Item::Fn(v) => &v.sig.ident,
            syn::Item::Struct(v) => &v.ident,
            syn::Item::Enum(v) => &v.ident,
            syn::Item::Trait(v) => &v.ident,
            syn::Item::Type(v) => &v.ident,
            syn::Item::Const(v) => &v.ident,
            syn::Item::Static(v) => &v.ident,
            syn::Item::Mod(v) => &v.ident,
            syn::Item::Union(v) => &v.ident,
            _ => continue,
        };
        let rendered = ident.to_string();
        let source_name = constants
            .iter()
            .filter(|(_, (_, name))| name == &rendered)
            .min_by_key(|(name, _)| *name)
            .map(|(name, _)| name.as_str())
            .unwrap_or_else(|| rendered.strip_prefix("r#").unwrap_or(&rendered));
        let name = source_name.encode(cx)?;
        if let Some(declaration) = declarations.get(&name) {
            names.insert(*declaration, rendered.encode(cx)?);
            if let Some((start, end)) = cx.scopes[module_name].locations.get(source_name).copied() {
                let location = cx.records.intern(&wire::SourceLocation {
                    source: cx.source,
                    start,
                    end,
                    documentation: None,
                })?;
                let range = item.span().byte_range();
                mappings.push((
                    u32::try_from(range.start).map_err(|_| {
                        wire::MetadataError("Rust mapping exceeds wire range".into())
                    })?,
                    u32::try_from(range.end).map_err(|_| {
                        wire::MetadataError("Rust mapping exceeds wire range".into())
                    })?,
                    location,
                ));
            }
        } else {
            support.insert(rendered.encode(cx)?);
        }
    }
    for (name, (_, rendered)) in &constants {
        let key = name.encode(cx)?;
        let declaration = if let Some(declaration) = declarations.get(&key) {
            *declaration
        } else {
            super::exports::bind_reference(
                compiled,
                module_name,
                name,
                wire::DeclarationKind::Constant,
                cx,
            )?
        };
        let rendered = rendered.encode(cx)?;
        names.insert(declaration, rendered);
        support.remove(&rendered);
    }
    let boundary = cx.records.intern(&wire::FragmentBoundary::StdlibModule)?;
    let identity = cx.compatibility;
    let validation = wire::Ref::anchor(&[&module.id(), b"rust-validation"]);
    cx.records.insert(
        validation,
        &wire::FragmentValidation {
            bytes_digest: Sha256::digest(rust.rust.as_bytes()).into(),
            compiler_identity: identity.compiler,
            target_identity: identity.semantic_target,
            grammar_identity: Sha256::digest(b"syn-stdlib-module-v1").into(),
            boundary,
        },
    )?;
    let source = rust.rust.encode(cx)?;
    let generators = compiled
        .code
        .generator_functions
        .get(module_name)
        .cloned()
        .unwrap_or_default()
        .encode(cx)?;
    let payload = wire::Ref::anchor(&[&module.id(), b"rust-payload"]);
    cx.records.insert(
        payload,
        &wire::RustPayload {
            module,
            source,
            names,
            mappings,
            validation,
            generators,
        },
    )?;
    Ok((Some(payload), support))
}

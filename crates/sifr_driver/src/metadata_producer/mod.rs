//! Canonical projection of checked source values into indexed metadata.
use encode::Encode;
use sifr_sysroot::metadata as wire;
use wire::{MetadataEncoder, Result};
mod encode;
mod hir_nodes;
mod python_interop;
mod rust_interop;
mod specialization_metadata;
mod sql_migrations;
mod sql_queries;
mod template_strings;
mod type_records;
mod types;
use std::collections::BTreeMap;

mod locations;
#[derive(Clone)]
struct SourceScope {
    parameters: Vec<String>,
    module: wire::Ref<wire::Module>,
    source: wire::Ref<wire::SourceFile>,
    locations: BTreeMap<String, (u32, u32)>,
    kinds: BTreeMap<String, wire::DeclarationKind>,
}
struct Encoder {
    compatibility: wire::Compatibility,
    scopes: std::sync::Arc<BTreeMap<String, SourceScope>>,
    module_name: String,
    records: MetadataEncoder,
    package: wire::Ref<wire::Package>,
    source: wire::Ref<wire::SourceFile>,
    owner: String,
    binder: wire::Ref<wire::Binder>,
    parameters: Vec<(String, wire::Ref<wire::Binder>, u32)>,
    origins: BTreeMap<sifr_ir::SourceOriginId, u32>,
}
impl Encoder {
    fn declaration(
        &mut self,
        name: &str,
        kind: wire::DeclarationKind,
    ) -> Result<wire::Ref<wire::Declaration>> {
        let (scope, symbol) = self
            .scopes
            .iter()
            .filter_map(|(module, scope)| {
                name.strip_prefix(&format!("{module}."))
                    .map(|symbol| (module.len(), scope, symbol))
            })
            .max_by_key(|(len, _, _)| *len)
            .map(|(_, scope, symbol)| (scope, symbol))
            .unwrap_or((&self.scopes[&self.module_name], name));
        let kind = scope.kinds.get(symbol).cloned().unwrap_or(kind);
        let module = scope.module;
        let source = scope.source;
        let (start, end) = scope.locations.get(symbol).copied().unwrap_or_default();
        let reference = wire::Ref::anchor(&[&self.package.id(), &module.id(), symbol.as_bytes()]);
        let symbol = symbol.to_owned().encode(self)?;
        let kind = self.records.intern(&kind)?;
        let location = self.records.intern(&wire::SourceLocation {
            source,
            start,
            end,
            documentation: None,
        })?;
        self.records
            .insert(
                reference,
                &wire::Declaration {
                    module,
                    symbol,
                    kind,
                    binder: None,
                    full_view: None,
                    location,
                },
            )
            .map_err(|error| {
                wire::MetadataError(format!("declaration {}::{name}: {error}", self.module_name))
            })?;
        Ok(reference)
    }
    fn with_owner<T>(
        &mut self,
        name: &str,
        parameters: &[String],
        kind: wire::DeclarationKind,
        f: impl FnOnce(&mut Self) -> Result<T>,
    ) -> Result<T> {
        let owner = if self.owner.is_empty() {
            name.to_owned()
        } else {
            format!("{}.{}", self.owner, name)
        };
        let declaration = self.declaration(&owner, kind)?;
        let binder = wire::Ref::anchor(&[&declaration.id(), b"binder"]);
        let values = parameters
            .iter()
            .map(|p| Ok((p.encode(self)?, Vec::new())))
            .collect::<Result<_>>()?;
        self.records.insert(
            binder,
            &wire::Binder {
                declaration,
                nested_path: Vec::new(),
                parameters: values,
            },
        )?;
        let previous = (
            std::mem::replace(&mut self.owner, owner),
            std::mem::replace(&mut self.binder, binder),
            self.parameters.len(),
        );
        for (slot, name) in parameters.iter().enumerate() {
            self.parameters.push((
                name.clone(),
                binder,
                u32::try_from(slot)
                    .map_err(|_| wire::MetadataError("too many generic parameters".into()))?,
            ));
        }
        let value = f(self);
        (self.owner, self.binder) = (previous.0, previous.1);
        self.parameters.truncate(previous.2);
        value
    }
}
impl Encode<wire::Ref<wire::BindingId>> for sifr_ir::BindingId {
    fn encode(&self, cx: &mut Encoder) -> Result<wire::Ref<wire::BindingId>> {
        cx.records.intern(&wire::BindingId {
            binder: cx.binder,
            slot: self.0,
        })
    }
}
impl Encode<wire::Ref<wire::TextRange>> for ruff_text_size::TextRange {
    fn encode(&self, cx: &mut Encoder) -> Result<wire::Ref<wire::TextRange>> {
        cx.records.intern(&wire::TextRange {
            source: cx.source,
            start: self.start().to_u32(),
            end: self.end().to_u32(),
        })
    }
}
impl Encode<wire::Ref<wire::SourceOriginId>> for sifr_ir::SourceOriginId {
    fn encode(&self, cx: &mut Encoder) -> Result<wire::Ref<wire::SourceOriginId>> {
        let next = u32::try_from(cx.origins.len()).map_err(|_| {
            wire::MetadataError("source origin inventory exceeds wire range".into())
        })?;
        let ordinal = *cx.origins.entry(*self).or_insert(next);
        let (start, end) = cx.scopes[&cx.module_name]
            .locations
            .get(&cx.owner)
            .copied()
            .unwrap_or_default();
        let location = cx.records.intern(&wire::SourceLocation {
            source: cx.source,
            start,
            end,
            documentation: None,
        })?;
        cx.records.intern(&wire::SourceOriginId {
            location,
            local_ordinal: ordinal,
        })
    }
}

mod ensure;
mod production;
mod project;
mod semantic;
pub use ensure::{PreparedMetadata, ensure_development_metadata, validate_development_metadata};
#[cfg(test)]
mod tests;

fn class_kind(class: &sifr_ir::HirClass) -> wire::DeclarationKind {
    if class.newtype_inner.is_some() {
        wire::DeclarationKind::Newtype
    } else if class.is_enum() {
        wire::DeclarationKind::Enum
    } else if class.is_protocol() {
        wire::DeclarationKind::Protocol
    } else {
        wire::DeclarationKind::Class
    }
}

fn type_kind(value: &sifr_type_system::Type) -> wire::DeclarationKind {
    match value {
        sifr_type_system::Type::Enum { .. } => wire::DeclarationKind::Enum,
        sifr_type_system::Type::Protocol { .. } => wire::DeclarationKind::Protocol,
        sifr_type_system::Type::Newtype { .. } => wire::DeclarationKind::Newtype,
        sifr_type_system::Type::Alias { .. } => wire::DeclarationKind::Alias,
        _ => wire::DeclarationKind::Class,
    }
}

mod rust_payload;

mod exports;

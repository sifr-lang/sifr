use super::{
    BTreeMap, Binder, BindingId, Declaration, DeclarationKind, Digest, MetadataStore, Module,
    NominalView, Record, RecordId, Ref, Result, RustPayload, Sha256, SourceFile, SourceLocation,
    TemplatePayload, TemplateRole, TextRange, Type, err,
};
use std::any::Any;

impl MetadataStore {
    fn referenced<T: Record>(&self, reference: Ref<T>) -> Result<T> {
        let entry = self
            .directory
            .get(&reference.id)
            .ok_or_else(|| err("missing referenced record"))?;
        if entry.kind != T::KIND {
            return Err(err("wrong-kind referenced record"));
        }
        serde_json::from_slice(&self.read_payload(entry)?)
            .map_err(|e| err(format!("referenced record: {e}")))
    }
    pub(super) fn validate_fields<T: Record>(&self, value: &T) -> Result<()> {
        let any = value as &dyn Any;
        if let Some(source) = any.downcast_ref::<SourceFile>() {
            let path = &source.relative_path;
            if path.is_empty()
                || path.starts_with('/')
                || path.contains('\\')
                || path.contains(':')
                || path
                    .split('/')
                    .any(|p| p.is_empty() || p == "." || p == "..")
            {
                return Err(err(
                    "source path must be normalized and packaged-source relative",
                ));
            }
        }
        let range = any
            .downcast_ref::<SourceLocation>()
            .map(|v| (v.source, v.start, v.end))
            .or_else(|| {
                any.downcast_ref::<TextRange>()
                    .map(|v| (v.source, v.start, v.end))
            });
        if let Some((source, start, end)) = range {
            let source = self.referenced(source)?;
            self.validate_fields(&source)?;
            if start > end || end > source.byte_length {
                return Err(err("source range is outside its captured source"));
            }
        }
        if let Some(Type::TypeVar { binder, slot }) = any.downcast_ref::<Type>() {
            let binder = self.referenced(*binder)?;
            if *slot as usize >= binder.parameters.len() {
                return Err(err("type parameter slot is outside its declaration binder"));
            }
        }
        if let Some(ty) = any.downcast_ref::<Type>() {
            let nominal = match ty {
                Type::Class { declaration, .. } => Some((*declaration, DeclarationKind::Class)),
                Type::Protocol { declaration, .. } => {
                    Some((*declaration, DeclarationKind::Protocol))
                }
                Type::Enum { declaration, .. } => Some((*declaration, DeclarationKind::Enum)),
                Type::Newtype { declaration, .. } => Some((*declaration, DeclarationKind::Newtype)),
                Type::Alias { declaration, .. } => Some((*declaration, DeclarationKind::Alias)),
                _ => None,
            };
            if let Some((decl, expected)) = nominal {
                let declaration = self.referenced(decl)?;
                if self.referenced(declaration.kind)? != expected {
                    return Err(err("nominal type has the wrong declaration kind"));
                }
            }
        }
        if let Some(binding) = any.downcast_ref::<BindingId>() {
            // Binding slots name local source bindings, rather than generic parameters.
            let _ = self.referenced(binding.binder)?;
        }
        if let Some(payload) = any.downcast_ref::<RustPayload>() {
            let source = self.referenced(payload.source)?;
            let validation = self.referenced(payload.validation)?;
            let digest: RecordId = Sha256::digest(source.value.as_bytes()).into();
            let identity = self.compatibility();
            if validation.bytes_digest != digest
                || validation.compiler_identity != identity.compiler
                || validation.target_identity != identity.semantic_target
            {
                return Err(err(
                    "emitted fragment validation does not bind these bytes and compatibility identities",
                ));
            }
            for (start, end, location) in &payload.mappings {
                if start > end || *end as usize > source.value.len() {
                    return Err(err("Rust source mapping is outside emitted bytes"));
                }
                self.validate_fields(&self.referenced(*location)?)?;
            }
        }
        if let Some(template) = any.downcast_ref::<TemplatePayload>() {
            match self.referenced(template.role)? {
                TemplateRole::GenericFunction
                    if template.function.is_some() && template.class.is_none() => {}
                TemplateRole::GenericClass | TemplateRole::ProjectPolicyClass
                    if template.class.is_some() && template.function.is_none() => {}
                _ => return Err(err("template payload is incomplete for its declared role")),
            }
        }
        Ok(())
    }
    /// Validate a demanded structural graph iteratively. Nominal/module/binder
    /// anchors are graph boundaries, not recursively expanded values. A class
    /// occurrence never forces its field/method view into memory. Requesting the
    /// view itself validates its own demanded structural graph. Cyclic bodies,
    /// structural type cycles and excessive depth/work are rejected before a
    /// provider may project them into owned compiler values.
    pub(super) fn validate_graph(&self, root: RecordId, root_bytes: &[u8]) -> Result<()> {
        let root_kind = self
            .directory
            .get(&root)
            .ok_or_else(|| err("missing graph root"))?
            .kind;
        if anchor_kind(root_kind) {
            return Ok(());
        }
        let mut colors = BTreeMap::new();
        let mut stack = vec![(root, false, 0_usize)];
        let mut work = 0_u64;
        while let Some((id, exit, depth)) = stack.pop() {
            if exit {
                colors.insert(id, 2);
                continue;
            }
            match colors.get(&id) {
                Some(1) => return Err(err("cycle in structural type or semantic body records")),
                Some(2) => continue,
                _ => {}
            }
            if depth > self.limits.graph_depth {
                return Err(err("record graph exceeds depth limit"));
            }
            let entry = self
                .directory
                .get(&id)
                .ok_or_else(|| err("missing graph reference"))?;
            work = work
                .checked_add(entry.decoded_bound)
                .ok_or_else(|| err("graph work overflow"))?;
            if work > self.limits.retained_bytes {
                return Err(err("demanded graph exceeds decode work budget"));
            }
            let owned;
            let bytes = if id == root {
                root_bytes
            } else {
                owned = self.read_payload(entry)?;
                &owned
            };
            let refs = super::decode::references(self, entry.kind, bytes)?;
            colors.insert(id, 1);
            stack.push((id, true, depth));
            for (child, kind) in refs.into_iter().rev() {
                if self.directory.get(&child).is_none_or(|e| e.kind != kind) {
                    return Err(err("missing or wrong-kind graph reference"));
                }
                if !anchor_kind(kind) && kind != NominalView::KIND {
                    stack.push((child, false, depth + 1));
                }
            }
        }
        Ok(())
    }
}
fn anchor_kind(kind: u16) -> bool {
    [Module::KIND, Declaration::KIND, Binder::KIND].contains(&kind)
}

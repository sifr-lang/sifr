//! Metadata navigation indexes names and paths; source text is read only on demand.
use super::{Provider, Result, wire};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    io::Read,
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
};
#[derive(Debug)]
struct Source {
    path: PathBuf,
    digest: [u8; 32],
    length: u32,
    text: OnceLock<String>,
}
#[derive(Clone, Debug)]
pub struct StdlibNavigationSymbol {
    pub module: String,
    pub name: String,
    pub kind: String,
    pub file: u32,
    pub start: u32,
    pub end: u32,
    pub private: bool,
}
#[derive(Debug)]
pub struct StdlibNavigation {
    sources: BTreeMap<u32, Source>,
    pub symbols: Vec<StdlibNavigationSymbol>,
}
impl StdlibNavigation {
    pub fn path(&self, file: u32) -> Option<&Path> {
        self.sources.get(&file).map(|s| s.path.as_path())
    }
    pub fn source(&self, file: u32) -> std::result::Result<Option<&str>, String> {
        let Some(source) = self.sources.get(&file) else {
            return Ok(None);
        };
        if source.text.get().is_none() {
            let file = std::fs::File::open(&source.path).map_err(|e| {
                format!(
                    "required metadata navigation source {}: {e}",
                    source.path.display()
                )
            })?;
            let mut bytes = Vec::new();
            file.take(u64::from(source.length) + 1)
                .read_to_end(&mut bytes)
                .map_err(|e| e.to_string())?;
            if bytes.len() != source.length as usize
                || <[u8; 32]>::from(Sha256::digest(&bytes)) != source.digest
            {
                return Err(format!(
                    "metadata navigation source identity differs: {}",
                    source.path.display()
                ));
            }
            let text = String::from_utf8(bytes).map_err(|e| e.to_string())?;
            let _ = source.text.set(text);
        }
        Ok(source.text.get().map(String::as_str))
    }
    pub fn loaded_sources(&self) -> usize {
        self.sources
            .values()
            .filter(|source| source.text.get().is_some())
            .count()
    }
}
impl Provider {
    pub(crate) fn navigation(&self, root: &Path) -> Result<Arc<StdlibNavigation>> {
        let mut cache = self
            .navigation_cache
            .lock()
            .map_err(|_| wire::MetadataError("metadata navigation owner poisoned".into()))?;
        if let Some(index) = cache.get(root).and_then(std::sync::Weak::upgrade) {
            return Ok(index);
        }
        cache.retain(|_, value| value.strong_count() != 0);
        let mut sources = BTreeMap::new();
        let mut source_ids = BTreeMap::new();
        for (ordinal, reference) in self
            .metadata
            .store
            .record_refs::<wire::SourceFile>()
            .enumerate()
        {
            let source = self.metadata.store.get(reference)?;
            let ordinal = u32::try_from(ordinal)
                .map_err(|_| wire::MetadataError("too many navigation source identities".into()))?;
            let id = 0x8000_0000_u32
                .checked_add(ordinal)
                .ok_or_else(|| wire::MetadataError("navigation source identity overflow".into()))?;
            sources.insert(
                id,
                Source {
                    path: root.join(&source.relative_path),
                    digest: source.content_digest,
                    length: source.byte_length,
                    text: OnceLock::new(),
                },
            );
            source_ids.insert(reference, id);
        }
        let mut symbols = Vec::new();
        for (module_name, reference) in &self.modules {
            let module = self.metadata.store.get(*reference)?;
            for (name, declaration) in &module.exports {
                let declaration = self.metadata.store.get(*declaration)?;
                let location = self.metadata.store.get(declaration.location)?;
                let file = *source_ids
                    .get(&location.source)
                    .ok_or_else(|| wire::MetadataError("navigation source not indexed".into()))?;
                let kind = match self.metadata.store.get(declaration.kind)?.as_ref() {
                    wire::DeclarationKind::Function => "function",
                    wire::DeclarationKind::Class
                    | wire::DeclarationKind::Protocol
                    | wire::DeclarationKind::Enum
                    | wire::DeclarationKind::Newtype => "class",
                    _ => "constant",
                };
                symbols.push(StdlibNavigationSymbol {
                    module: module_name.clone(),
                    name: self.metadata.store.get(*name)?.value.clone(),
                    kind: kind.into(),
                    file,
                    start: location.start,
                    end: location.end,
                    private: module.private,
                });
            }
        }
        let index = Arc::new(StdlibNavigation { sources, symbols });
        cache.insert(root.to_owned(), Arc::downgrade(&index));
        Ok(index)
    }
}

use super::implementation::AnalysisHost;
use crate::editor::EditorToken;
use crate::queries::Location;
use crate::snapshot::AnalysisError;
use crate::symbols::StdlibSymbolInput;
use ruff_text_size::TextRange;
use sifr_frontend::{FileId, SourceOrigin};

impl AnalysisHost {
    pub(super) fn refresh_stdlib_symbol_bucket(&mut self) {
        let revision = self.current_revision;
        let symbols = self.stdlib_symbols_from_source_map();
        if let Some(index) = self.symbol_index.as_mut() {
            index.replace_stdlib_symbols(revision, symbols);
        }
    }

    pub(super) fn stdlib_import_location_for_token(
        &mut self,
        file: FileId,
        token: &EditorToken,
    ) -> Result<Option<Location>, AnalysisError> {
        let allow_private = self.stdlib_navigation.path(file.as_u32()).is_some()
            || self
                .context()?
                .source_file_for_file(file)
                .is_some_and(|source_file| {
                    matches!(
                        source_file.origin,
                        SourceOrigin::SysrootPublicStdlib | SourceOrigin::SysrootPrivateDeclaration
                    )
                });
        let source = self.source_text_for_file(file)?;
        let Some((module_name, imported_name)) = stdlib_import_target(source, token, allow_private)
        else {
            return Ok(None);
        };
        if module_name.starts_with("sifr.") {
            if let Some(location) = self
                .symbol_index()?
                .stdlib_symbol_location(&module_name, &imported_name)
            {
                return Ok(Some(location));
            }
        }
        if allow_private && module_name.starts_with("_sifr.") {
            return Ok(self.stdlib_symbol_location_from_source_map(
                &module_name,
                &imported_name,
                SourceOrigin::SysrootPrivateDeclaration,
            ));
        }
        Ok(None)
    }

    fn stdlib_symbols_from_source_map(&self) -> Vec<StdlibSymbolInput> {
        self.stdlib_navigation
            .symbols
            .iter()
            .filter(|symbol| !symbol.private)
            .enumerate()
            .map(|(ordinal, symbol)| StdlibSymbolInput {
                module_name: symbol.module.clone(),
                name: symbol.name.clone(),
                kind: symbol.kind.clone(),
                file: FileId::new(symbol.file),
                range: Some(TextRange::new(symbol.start.into(), symbol.end.into())),
                ordinal,
            })
            .collect()
    }

    fn stdlib_symbol_location_from_source_map(
        &self,
        module_name: &str,
        name: &str,
        origin: SourceOrigin,
    ) -> Option<Location> {
        self.stdlib_navigation
            .symbols
            .iter()
            .find(|symbol| {
                symbol.module == module_name
                    && symbol.name == name
                    && symbol.private == (origin == SourceOrigin::SysrootPrivateDeclaration)
            })
            .map(|symbol| Location {
                file: FileId::new(symbol.file),
                range: Some(TextRange::new(symbol.start.into(), symbol.end.into())),
            })
    }
}

fn stdlib_import_target(
    source: &str,
    token: &EditorToken,
    allow_private: bool,
) -> Option<(String, String)> {
    source.lines().find_map(|line| {
        let trimmed = line.trim_start();
        let import_start = trimmed.strip_prefix("from ")?;
        let (module_name, imported) = import_start.split_once(" import ")?;
        if !(module_name.starts_with("sifr.") || allow_private && module_name.starts_with("_sifr."))
        {
            return None;
        }
        import_target_from_names(module_name, imported, &token.text)
    })
}

fn import_target_from_names(
    module_name: &str,
    imported: &str,
    token_text: &str,
) -> Option<(String, String)> {
    imported.split(',').find_map(|part| {
        let part = part.trim();
        let (imported_name, visible_name) = part
            .split_once(" as ")
            .map_or((part, part), |(imported_name, alias)| {
                (imported_name.trim(), alias.trim())
            });
        (visible_name == token_text || imported_name == token_text)
            .then(|| (module_name.to_string(), imported_name.to_string()))
    })
}

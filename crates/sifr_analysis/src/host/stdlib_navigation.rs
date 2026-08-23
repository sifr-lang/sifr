use super::implementation::AnalysisHost;
use crate::editor::EditorToken;
use crate::queries::Location;
use crate::snapshot::AnalysisError;
use crate::symbols::StdlibSymbolInput;
use ruff_text_size::{Ranged as _, TextRange};
use sifr_frontend::{FileId, SourceFileView, SourceOrigin, parse_source};
use sifr_python_ast::{Expr, Stmt};

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
        let context = self.context()?;
        let Some(source_file) = context.source_file_for_file(file) else {
            return Ok(None);
        };
        let allow_private = matches!(
            source_file.origin,
            SourceOrigin::SysrootPublicStdlib | SourceOrigin::SysrootPrivateDeclaration
        );
        let Some(source) = context.source_text_for_file(file) else {
            return Ok(None);
        };
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
        self.session
            .context()
            .map(|context| {
                context
                    .source_map()
                    .files
                    .into_iter()
                    .filter(|file| file.origin == SourceOrigin::SysrootPublicStdlib)
                    .flat_map(|file| stdlib_symbols_from_file(&file))
                    .collect()
            })
            .unwrap_or_default()
    }

    fn stdlib_symbol_location_from_source_map(
        &self,
        module_name: &str,
        name: &str,
        origin: SourceOrigin,
    ) -> Option<Location> {
        self.session
            .context()?
            .source_map()
            .files
            .into_iter()
            .filter(|file| {
                file.origin == origin && file.module_name.as_deref() == Some(module_name)
            })
            .find_map(|file| {
                stdlib_symbols_from_file(&file)
                    .into_iter()
                    .find(|symbol| symbol.name == name)
                    .map_or_else(
                        || {
                            Some(Location {
                                file: file.id,
                                range: None,
                            })
                        },
                        |symbol| {
                            Some(Location {
                                file: symbol.file,
                                range: symbol.range,
                            })
                        },
                    )
            })
    }
}

fn stdlib_symbols_from_file(file: &SourceFileView) -> Vec<StdlibSymbolInput> {
    let Some(module_name) = file.module_name.as_ref() else {
        return Vec::new();
    };
    let Ok(stmts) = parse_source(file.source.as_str(), Some(module_name)) else {
        return Vec::new();
    };
    let mut symbols = stmts
        .iter()
        .enumerate()
        .filter_map(|(ordinal, stmt)| symbol_from_stmt(module_name, file.id, stmt, ordinal))
        .collect::<Vec<_>>();
    symbols.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then(left.ordinal.cmp(&right.ordinal))
    });
    symbols
}

fn symbol_from_stmt(
    module_name: &str,
    file: FileId,
    stmt: &Stmt,
    ordinal: usize,
) -> Option<StdlibSymbolInput> {
    match stmt {
        Stmt::FunctionDef(function) if public_name(&function.name) => Some(stdlib_symbol(
            module_name,
            function.name.to_string(),
            "function",
            file,
            Some(function.range()),
            ordinal,
        )),
        Stmt::ClassDef(class) if public_name(&class.name) => Some(stdlib_symbol(
            module_name,
            class.name.to_string(),
            "class",
            file,
            Some(class.range()),
            ordinal,
        )),
        Stmt::AnnAssign(assign) => public_target_name(assign.target.as_ref()).map(|name| {
            stdlib_symbol(
                module_name,
                name,
                "constant",
                file,
                Some(assign.range()),
                ordinal,
            )
        }),
        Stmt::Assign(assign) if assign.targets.len() == 1 => public_target_name(&assign.targets[0])
            .map(|name| {
                stdlib_symbol(
                    module_name,
                    name,
                    "constant",
                    file,
                    Some(assign.range()),
                    ordinal,
                )
            }),
        _ => None,
    }
}

fn stdlib_symbol(
    module_name: &str,
    name: String,
    kind: &str,
    file: FileId,
    range: Option<TextRange>,
    ordinal: usize,
) -> StdlibSymbolInput {
    StdlibSymbolInput {
        module_name: module_name.to_string(),
        name,
        kind: kind.to_string(),
        file,
        range,
        ordinal,
    }
}

fn public_name(name: &str) -> bool {
    !name.starts_with('_')
}

fn public_target_name(target: &Expr) -> Option<String> {
    let Expr::Name(name) = target else {
        return None;
    };
    let name = name.id.to_string();
    public_name(&name).then_some(name)
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

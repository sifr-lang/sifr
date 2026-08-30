use crate::TemplateDocumentView;
use ruff_text_size::{TextRange, TextSize};
use sifr_ir::{
    HirExpr, HirModule, HirStmt, HirTemplateString, visit_hir_function_exprs_mut,
    visit_hir_stmts_exprs_mut,
};
use sifr_sql_contract::{
    Nullability, ProviderAnalysis, SchemaIr, decode_generated_path, encode_generated_path,
};
use std::collections::{BTreeMap, BTreeSet};

mod support;
use support::{
    SQL_KEYWORDS, cardinality_label, contains, infer_cardinality, inferred_symbol, lex_sql,
    qualifier_before, ranges_overlap, relation_aliases, schema_kind_label, semantic_bool,
    semantic_display, semantic_text, symbol_allowed_in_fragment, symbol_matches_relation,
    virtual_text,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SqlEditorTokenKind {
    Keyword,
    Identifier,
    String,
    Number,
    Operator,
    Hole { index: usize },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SqlEditorToken {
    pub text: String,
    pub virtual_range: TextRange,
    pub kind: SqlEditorTokenKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SqlEditorSymbol {
    pub name: String,
    pub kind: String,
    pub database_type: Option<String>,
    pub sifr_type: Option<String>,
    pub nullable: Option<bool>,
    pub definition_document: Option<String>,
    pub definition_range: Option<TextRange>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SqlEditorFixKind {
    Alias,
    Cast,
    MissingColumn,
    UnsafeCollection,
    MigrationImpact,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SqlEditorFix {
    pub title: String,
    pub kind: SqlEditorFixKind,
    pub virtual_range: TextRange,
    pub replacement: Option<String>,
    pub detail: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SqlEditorCatalog {
    pub symbols: BTreeMap<String, SqlEditorSymbol>,
    pub fragment_relations: BTreeMap<String, BTreeSet<String>>,
    pub generated_names: BTreeMap<String, String>,
}

impl SqlEditorCatalog {
    #[must_use]
    pub fn from_schema(schema: &SchemaIr) -> Self {
        let mut catalog = Self::default();
        for object in schema.objects.values() {
            let identity = object.identity.as_str();
            let definition_range = object.source.as_ref().map(|source| {
                TextRange::new(TextSize::new(source.start), TextSize::new(source.end))
            });
            let symbol = SqlEditorSymbol {
                name: identity.to_string(),
                kind: schema_kind_label(object.kind).to_string(),
                database_type: semantic_display(&object.semantic, "database-type"),
                sifr_type: semantic_text(&object.semantic, "sifr_type"),
                nullable: semantic_bool(&object.semantic, "nullable"),
                definition_document: object.source.as_ref().map(|source| source.document.clone()),
                definition_range,
            };
            catalog.symbols.insert(identity.to_string(), symbol.clone());
            if let Some(short) = identity.rsplit('.').next()
                && !catalog.symbols.contains_key(short)
            {
                let mut short_symbol = symbol;
                short_symbol.name = short.to_string();
                catalog.symbols.insert(short.to_string(), short_symbol);
            }
            let path = identity.split('.').map(str::to_string).collect::<Vec<_>>();
            if let Ok(encoded) = encode_generated_path(&path)
                && let Ok(decoded) = decode_generated_path(&encoded)
            {
                catalog.generated_names.insert(encoded, decoded.join("."));
            }
        }
        catalog
    }

    #[must_use]
    pub fn database_name_for_generated(&self, generated: &str) -> Option<&str> {
        self.generated_names.get(generated).map(String::as_str)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SqlEditorDocumentView {
    pub template: TemplateDocumentView,
    pub tokens: Vec<SqlEditorToken>,
    pub catalog: SqlEditorCatalog,
    pub fragment_identity: Option<String>,
    pub relation_aliases: BTreeMap<String, String>,
    pub parameter_types: Vec<String>,
    pub result_fields: Vec<SqlEditorSymbol>,
    pub cardinality: String,
    pub fixes: Vec<SqlEditorFix>,
}

impl SqlEditorDocumentView {
    #[must_use]
    pub fn from_template(template: TemplateDocumentView) -> Self {
        let tokens = lex_sql(&template);
        let cardinality = infer_cardinality(&tokens);
        let relation_aliases = relation_aliases(&tokens);
        Self {
            template,
            tokens,
            catalog: SqlEditorCatalog::default(),
            fragment_identity: None,
            relation_aliases,
            parameter_types: Vec::new(),
            result_fields: Vec::new(),
            cardinality,
            fixes: Vec::new(),
        }
    }

    #[must_use]
    pub fn from_hir(template: &HirTemplateString) -> Self {
        let mut document = Self::from_template(TemplateDocumentView::from_hir(template));
        document.parameter_types = template
            .interpolations
            .iter()
            .map(|interpolation| interpolation.value_type.to_string())
            .collect();
        document
    }

    #[must_use]
    pub fn with_semantics(
        mut self,
        catalog: SqlEditorCatalog,
        fragment_identity: Option<String>,
        parameter_types: Vec<String>,
        result_fields: Vec<SqlEditorSymbol>,
        cardinality: impl Into<String>,
    ) -> Self {
        self.catalog = catalog;
        self.fragment_identity = fragment_identity;
        self.parameter_types = parameter_types;
        self.result_fields = result_fields;
        self.cardinality = cardinality.into();
        self
    }

    #[must_use]
    pub fn with_provider_analysis(
        mut self,
        schema: &SchemaIr,
        analysis: &ProviderAnalysis,
    ) -> Self {
        self.catalog = SqlEditorCatalog::from_schema(schema);
        self.result_fields = analysis
            .result_fields
            .iter()
            .map(|field| SqlEditorSymbol {
                name: field.name.clone(),
                kind: "result-field".to_string(),
                database_type: Some(format!("{:?}", field.database_type)),
                sifr_type: Some(format!("{:?}", field.sifr_type)),
                nullable: Some(field.nullability == Nullability::Nullable),
                definition_document: field
                    .source_object
                    .as_ref()
                    .and_then(|identity| self.catalog.symbols.get(identity.as_str()))
                    .and_then(|symbol| symbol.definition_document.clone()),
                definition_range: field
                    .source_object
                    .as_ref()
                    .and_then(|identity| self.catalog.symbols.get(identity.as_str()))
                    .and_then(|symbol| symbol.definition_range),
            })
            .collect();
        self.cardinality = cardinality_label(analysis.cardinality);
        self
    }

    #[must_use]
    pub fn with_fixes(mut self, fixes: Vec<SqlEditorFix>) -> Self {
        self.fixes = fixes;
        self
    }

    #[must_use]
    pub fn fixes_for_diagnostic(&self, code: &str, source_range: TextRange) -> Vec<SqlEditorFix> {
        let Some(virtual_range) = self
            .template
            .virtual_range_for_source_range(source_range)
            .or_else(|| self.template.virtual_range_for_source(source_range.start()))
        else {
            return Vec::new();
        };
        let selected = virtual_text(&self.template.source, virtual_range).unwrap_or("value");
        let upper = code.to_ascii_uppercase();
        let mut fixes = self
            .fixes
            .iter()
            .filter(|fix| ranges_overlap(fix.virtual_range, virtual_range))
            .cloned()
            .collect::<Vec<_>>();
        let synthesized = if upper.ends_with("POSTGRESQL-0004") || upper.contains("ALIAS") {
            Some(SqlEditorFix {
                title: "Add an explicit SQL alias".to_string(),
                kind: SqlEditorFixKind::Alias,
                virtual_range,
                replacement: Some(format!("{selected} AS value")),
                detail: None,
            })
        } else if upper.ends_with("POSTGRESQL-0005") || upper.contains("CAST") {
            Some(SqlEditorFix {
                title: "Add an explicit SQL cast".to_string(),
                kind: SqlEditorFixKind::Cast,
                virtual_range,
                replacement: Some(format!("CAST({selected} AS text)")),
                detail: None,
            })
        } else if upper.ends_with("POSTGRESQL-0003") || upper.contains("COLUMN") {
            self.catalog
                .symbols
                .values()
                .find(|symbol| symbol.kind == "column")
                .map(|symbol| SqlEditorFix {
                    title: format!("Replace with SQL column `{}`", symbol.name),
                    kind: SqlEditorFixKind::MissingColumn,
                    virtual_range,
                    replacement: Some(symbol.name.clone()),
                    detail: symbol.sifr_type.clone(),
                })
        } else if upper.ends_with("POSTGRESQL-0010") && selected == "*" {
            self.catalog
                .symbols
                .values()
                .find(|symbol| symbol.kind == "column")
                .map(|symbol| SqlEditorFix {
                    title: "Replace the nested wildcard with an explicit column".to_string(),
                    kind: SqlEditorFixKind::MissingColumn,
                    virtual_range,
                    replacement: Some(symbol.name.clone()),
                    detail: Some(
                        "The wildcard is at this nested SQL expression, not the exported row."
                            .to_string(),
                    ),
                })
        } else if upper == "SIFR-SQL-0005"
            || upper.contains("COLLECT")
            || upper.contains("CARDINALITY")
        {
            Some(SqlEditorFix {
                title: "Bound the SQL collection to 100 rows".to_string(),
                kind: SqlEditorFixKind::UnsafeCollection,
                virtual_range: TextRange::empty(TextSize::of(&self.template.source)),
                replacement: Some(" LIMIT 100".to_string()),
                detail: Some("Review the bound for the application workload.".to_string()),
            })
        } else {
            None
        };
        if let Some(synthesized) = synthesized {
            fixes.push(synthesized);
        }
        fixes.sort_by(|left, right| {
            (&left.title, left.virtual_range.start())
                .cmp(&(&right.title, right.virtual_range.start()))
        });
        fixes.dedup_by(|left, right| {
            left.title == right.title && left.virtual_range == right.virtual_range
        });
        fixes
    }

    #[must_use]
    pub fn source_range_for_fix(&self, fix: &SqlEditorFix) -> Option<TextRange> {
        self.template
            .source_range_for_virtual_range(fix.virtual_range)
    }

    #[must_use]
    pub fn contains_source_offset(&self, offset: TextSize) -> bool {
        contains(self.template.source_range, offset)
    }

    #[must_use]
    pub fn token_at_source_offset(&self, offset: TextSize) -> Option<&SqlEditorToken> {
        let virtual_offset = self.template.virtual_offset_for_source(offset)?;
        self.tokens
            .iter()
            .find(|token| contains(token.virtual_range, virtual_offset))
    }

    #[must_use]
    pub fn completion_symbols(&self, source_offset: TextSize) -> Vec<SqlEditorSymbol> {
        let virtual_offset = self
            .template
            .virtual_offset_for_source(source_offset)
            .unwrap_or_default();
        let qualifier = qualifier_before(&self.tokens, virtual_offset);
        let qualified_relation = qualifier
            .as_ref()
            .and_then(|qualifier| self.relation_aliases.get(qualifier))
            .or(qualifier.as_ref());
        let allowed = self
            .fragment_identity
            .as_ref()
            .and_then(|fragment| self.catalog.fragment_relations.get(fragment));
        let mut symbols = self
            .catalog
            .symbols
            .values()
            .filter(|symbol| {
                allowed.is_none_or(|relations| symbol_allowed_in_fragment(symbol, relations))
            })
            .filter(|symbol| {
                qualified_relation.is_none_or(|relation| symbol_matches_relation(symbol, relation))
            })
            .cloned()
            .collect::<Vec<_>>();
        if qualifier.is_none() {
            symbols.extend(self.relation_aliases.keys().map(|alias| SqlEditorSymbol {
                name: alias.clone(),
                kind: "alias".to_string(),
                database_type: None,
                sifr_type: None,
                nullable: None,
                definition_document: None,
                definition_range: None,
            }));
        }
        symbols.extend(SQL_KEYWORDS.iter().map(|keyword| SqlEditorSymbol {
            name: (*keyword).to_string(),
            kind: "keyword".to_string(),
            database_type: None,
            sifr_type: None,
            nullable: None,
            definition_document: None,
            definition_range: None,
        }));
        symbols.sort_by(|left, right| (&left.name, &left.kind).cmp(&(&right.name, &right.kind)));
        symbols.dedup_by(|left, right| left.name == right.name && left.kind == right.kind);
        symbols
    }

    #[must_use]
    pub fn symbol_at_source_offset(&self, offset: TextSize) -> Option<SqlEditorSymbol> {
        let token = self.token_at_source_offset(offset)?;
        match token.kind {
            SqlEditorTokenKind::Hole { index } => Some(SqlEditorSymbol {
                name: format!("parameter ${}", index + 1),
                kind: "parameter".to_string(),
                database_type: None,
                sifr_type: self.parameter_types.get(index).cloned(),
                nullable: None,
                definition_document: None,
                definition_range: Some(
                    self.template
                        .source_range_for_virtual_range(token.virtual_range)?,
                ),
            }),
            SqlEditorTokenKind::Identifier => {
                let database_name = self
                    .catalog
                    .database_name_for_generated(&token.text)
                    .unwrap_or(&token.text);
                self.catalog
                    .symbols
                    .get(database_name)
                    .or_else(|| {
                        self.catalog
                            .symbols
                            .values()
                            .find(|symbol| symbol.name.rsplit('.').next() == Some(database_name))
                    })
                    .or_else(|| {
                        self.relation_aliases
                            .get(&token.text)
                            .and_then(|relation| self.catalog.symbols.get(relation))
                    })
                    .cloned()
                    .or_else(|| Some(inferred_symbol(token, &self.tokens)))
            }
            _ => None,
        }
    }

    #[must_use]
    pub fn source_ranges_for_symbol(&self, name: &str) -> Vec<TextRange> {
        self.tokens
            .iter()
            .filter(|token| {
                token.kind == SqlEditorTokenKind::Identifier
                    && (token.text == name || name.rsplit('.').next() == Some(token.text.as_str()))
            })
            .filter_map(|token| {
                self.template
                    .source_range_for_virtual_range(token.virtual_range)
            })
            .collect()
    }

    #[must_use]
    pub fn semantic_source_tokens(&self) -> Vec<(TextRange, &'static str)> {
        self.tokens
            .iter()
            .filter_map(|token| {
                let token_type = match token.kind {
                    SqlEditorTokenKind::Keyword => "keyword",
                    SqlEditorTokenKind::Identifier => "property",
                    SqlEditorTokenKind::String => "string",
                    SqlEditorTokenKind::Number => "number",
                    SqlEditorTokenKind::Operator => "operator",
                    SqlEditorTokenKind::Hole { .. } => "parameter",
                };
                Some((
                    self.template
                        .source_range_for_virtual_range(token.virtual_range)?,
                    token_type,
                ))
            })
            .collect()
    }

    #[must_use]
    pub fn parameter_source_ranges(&self) -> Vec<(usize, TextRange)> {
        self.tokens
            .iter()
            .filter_map(|token| {
                let SqlEditorTokenKind::Hole { index } = token.kind else {
                    return None;
                };
                Some((
                    index,
                    self.template
                        .source_range_for_virtual_range(token.virtual_range)?,
                ))
            })
            .collect()
    }
}

#[must_use]
pub fn sql_editor_documents(module: &HirModule) -> Vec<SqlEditorDocumentView> {
    let mut module = module.clone();
    let mut documents = Vec::new();
    let mut collect = |expression: &mut HirExpr| {
        if let HirExpr::TemplateString(template) = expression {
            documents.push(SqlEditorDocumentView::from_hir(template));
        }
    };
    for function in &mut module.functions {
        visit_hir_function_exprs_mut(function, &mut collect);
    }
    for class in &mut module.classes {
        for method in &mut class.methods {
            visit_hir_function_exprs_mut(method, &mut collect);
        }
        for (_, method) in &mut class.operator_impls {
            visit_hir_function_exprs_mut(method, &mut collect);
        }
        let mut defaults = class
            .field_defaults
            .iter()
            .map(|(_, value)| HirStmt::Expr {
                expr: value.clone(),
            })
            .collect::<Vec<_>>();
        visit_hir_stmts_exprs_mut(&mut defaults, &mut collect);
    }
    let mut constants = module
        .constants
        .iter()
        .map(|(_, _, value)| HirStmt::Expr {
            expr: value.clone(),
        })
        .collect::<Vec<_>>();
    visit_hir_stmts_exprs_mut(&mut constants, &mut collect);
    documents.sort_by_key(|document| document.template.source_range.start());
    documents.dedup_by_key(|document| document.template.source_range);
    documents
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FrontendDiagnosticStyle, FrontendSourceContext, compile_module_hir_with_source};
    use sifr_lowering::ExternalDefs;

    #[test]
    fn documents_cover_sql_tokens_holes_semantics_and_fragment_scope() {
        let source = "def query(user_id: int) -> Template:\n    return t\"SELECT u.name FROM users AS u WHERE u.id = {user_id} LIMIT 1\"\n";
        let parsed = crate::parse_source_module(source, Some("editor.sifr")).expect("parse");
        let lowered = compile_module_hir_with_source(
            "editor",
            parsed.suite(),
            &ExternalDefs::default(),
            FrontendDiagnosticStyle::Bare,
            Some(FrontendSourceContext {
                display_path: "editor.sifr",
                source,
            }),
        )
        .expect("lower");
        let mut documents = sql_editor_documents(&lowered.module);
        assert_eq!(documents.len(), 1);
        let mut catalog = SqlEditorCatalog::default();
        catalog.symbols.insert(
            "users".to_string(),
            SqlEditorSymbol {
                name: "users".to_string(),
                kind: "relation".to_string(),
                database_type: None,
                sifr_type: None,
                nullable: None,
                definition_document: Some("schema.sql".to_string()),
                definition_range: Some(TextRange::new(TextSize::new(0), TextSize::new(5))),
            },
        );
        catalog.symbols.insert(
            "users.name".to_string(),
            SqlEditorSymbol {
                name: "users.name".to_string(),
                kind: "column".to_string(),
                database_type: Some("text".to_string()),
                sifr_type: Some("str".to_string()),
                nullable: Some(false),
                definition_document: Some("schema.sql".to_string()),
                definition_range: Some(TextRange::new(TextSize::new(6), TextSize::new(10))),
            },
        );
        catalog.symbols.insert(
            "orders.total".to_string(),
            SqlEditorSymbol {
                name: "orders.total".to_string(),
                kind: "column".to_string(),
                database_type: None,
                sifr_type: None,
                nullable: None,
                definition_document: None,
                definition_range: None,
            },
        );
        catalog.fragment_relations.insert(
            "users-only".to_string(),
            BTreeSet::from(["users".to_string()]),
        );
        documents[0] = documents[0].clone().with_semantics(
            catalog,
            Some("users-only".to_string()),
            vec!["int".to_string()],
            Vec::new(),
            "zero-or-one",
        );
        let document = &documents[0];
        assert_eq!(document.cardinality, "zero-or-one");
        assert!(document.tokens.iter().any(|token| token.text == "SELECT"));
        assert_eq!(document.parameter_source_ranges().len(), 1);
        assert!(
            document
                .completion_symbols(document.template.source_range.start())
                .iter()
                .any(|symbol| symbol.name == "users")
        );
        assert!(
            document
                .completion_symbols(document.template.source_range.start())
                .iter()
                .any(|symbol| symbol.name == "users.name")
        );
        assert!(
            !document
                .completion_symbols(document.template.source_range.start())
                .iter()
                .any(|symbol| symbol.name == "orders.total")
        );

        let name = document
            .tokens
            .iter()
            .find(|token| token.text == "name")
            .expect("name token");
        let name_source = document
            .template
            .source_range_for_virtual_range(name.virtual_range)
            .expect("name source");
        let fixes = document.fixes_for_diagnostic("SIFR-SQL-POSTGRESQL-0005", name_source);
        assert_eq!(fixes[0].kind, SqlEditorFixKind::Cast);
        assert_eq!(document.source_range_for_fix(&fixes[0]), Some(name_source));
    }
}

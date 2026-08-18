use crate::capabilities::{SEMANTIC_TOKEN_MODIFIERS, SEMANTIC_TOKEN_TYPES};
use crate::errors::{LspError, LspResult};
use ruff_text_size::{TextRange, TextSize};
use serde_json::{json, Value};
use sifr_analysis::{
    CodeAction, CompletionItem, DeferredCodeAction, DiagnosticClass, DiagnosticId,
    DocumentHighlight, DocumentSymbol, FileId, FileTextEdits, FoldingRange, GeneratedRustPreview,
    HoverInfo, InlayHint, Location, SemanticToken, SignatureHelp, TextEdit, TypeHierarchyItem,
    WorkspaceEdit, WorkspaceSymbol,
};
use sifr_diagnostics::{DiagnosticArg, DiagnosticSpan, RenderedDiagnostic, Severity};
use sifr_source::{PositionEncoding, SourceText as SourceTextMap, TextPosition};
use std::path::PathBuf;
use url::Url;

pub(crate) fn uri_to_path(uri: &str) -> LspResult<PathBuf> {
    let parsed = Url::parse(uri).map_err(|error| {
        LspError::invalid_params(format!("invalid document URI {uri:?}: {error}"))
    })?;
    if parsed.scheme() != "file" {
        return Err(LspError::invalid_params(format!(
            "unsupported document URI scheme {:?}; only file URIs are supported",
            parsed.scheme()
        )));
    }
    parsed
        .to_file_path()
        .map_err(|()| LspError::invalid_params(format!("document URI is not a file path: {uri}")))
}

pub(crate) fn lsp_position(value: &Value) -> LspResult<TextPosition> {
    let line = value
        .get("line")
        .and_then(Value::as_u64)
        .and_then(|line| u32::try_from(line).ok())
        .ok_or_else(|| LspError::invalid_params("missing or invalid LSP position line"))?;
    let character = value
        .get("character")
        .and_then(Value::as_u64)
        .and_then(|character| u32::try_from(character).ok())
        .ok_or_else(|| LspError::invalid_params("missing or invalid LSP position character"))?;
    Ok(TextPosition { line, character })
}

pub(crate) fn lsp_position_to_utf8(
    value: &Value,
    source: &str,
    encoding: PositionEncoding,
) -> LspResult<TextPosition> {
    let position = lsp_position(value)?;
    if encoding == PositionEncoding::Utf8 {
        return Ok(position);
    }
    let source = SourceTextMap::new(source);
    let offset = source
        .byte_offset_with_encoding(&position, encoding)
        .ok_or_else(|| LspError::invalid_params("position is outside the document"))?;
    source
        .position_at(offset, PositionEncoding::Utf8)
        .ok_or_else(|| LspError::invalid_params("position is outside the document"))
}

pub(crate) fn text_position(
    position: &TextPosition,
    source: &str,
    encoding: PositionEncoding,
) -> LspResult<Value> {
    if encoding == PositionEncoding::Utf8 {
        return Ok(json!({ "line": position.line, "character": position.character }));
    }
    let source = SourceTextMap::new(source);
    let offset = source
        .byte_offset_with_encoding(position, PositionEncoding::Utf8)
        .ok_or_else(|| LspError::internal("analysis returned a position outside the document"))?;
    let encoded = source
        .position_at(offset, encoding)
        .ok_or_else(|| LspError::internal("analysis returned a position outside the document"))?;
    Ok(json!({ "line": encoded.line, "character": encoded.character }))
}

pub(crate) fn lsp_range(
    value: &Value,
    source: &str,
    encoding: PositionEncoding,
) -> LspResult<TextRange> {
    lsp_range_with_encoding(value, source, encoding)
}

fn lsp_range_with_encoding(
    value: &Value,
    source: &str,
    encoding: PositionEncoding,
) -> LspResult<TextRange> {
    let start = value
        .get("start")
        .ok_or_else(|| LspError::invalid_params("missing LSP range start"))
        .and_then(lsp_position)?;
    let end = value
        .get("end")
        .ok_or_else(|| LspError::invalid_params("missing LSP range end"))
        .and_then(lsp_position)?;
    let source = SourceTextMap::new(source);
    let start = source
        .byte_offset_with_encoding(&start, encoding)
        .ok_or_else(|| LspError::invalid_params("range start is outside the document"))?;
    let end = source
        .byte_offset_with_encoding(&end, encoding)
        .ok_or_else(|| LspError::invalid_params("range end is outside the document"))?;
    if start > end {
        return Err(LspError::invalid_params(
            "range start must not be after range end",
        ));
    }
    Ok(TextRange::new(start, end))
}

pub(crate) fn text_range(
    range: TextRange,
    source: &str,
    encoding: PositionEncoding,
) -> LspResult<Value> {
    text_range_with_encoding(range, source, encoding)
}

fn text_range_with_encoding(
    range: TextRange,
    source: &str,
    encoding: PositionEncoding,
) -> LspResult<Value> {
    let source = SourceTextMap::new(source);
    let Some(utf_range) = source.range_at(range, encoding) else {
        return Err(LspError::internal(
            "analysis returned a range outside the document",
        ));
    };
    Ok(json!({
        "start": {
            "line": utf_range.start.line,
            "character": utf_range.start.character
        },
        "end": {
            "line": utf_range.end.line,
            "character": utf_range.end.character
        }
    }))
}

pub(crate) fn location(
    location: &Location,
    uri_for_file: impl Fn(FileId) -> LspResult<String>,
    source: &str,
    encoding: PositionEncoding,
) -> LspResult<Value> {
    let uri = uri_for_file(location.file)?;
    let range = location.range.map_or_else(
        || Ok(json!({"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 0}})),
        |range| text_range(range, source, encoding),
    )?;
    Ok(json!({ "uri": uri, "range": range }))
}

pub(crate) fn workspace_symbol(symbol: WorkspaceSymbol, uri: String) -> Value {
    json!({
        "name": symbol.name,
        "kind": symbol_kind(&symbol.kind),
        "location": {
            "uri": uri,
            "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 0, "character": 0 } }
        },
        "containerName": symbol.container_name
    })
}

pub(crate) fn document_symbol(
    symbol: DocumentSymbol,
    source: &str,
    encoding: PositionEncoding,
) -> LspResult<Value> {
    let range = symbol.range.map_or_else(
        || Ok(json!({"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 0}})),
        |range| text_range(range, source, encoding),
    )?;
    Ok(json!({
        "name": symbol.name,
        "kind": symbol_kind(&symbol.kind),
        "range": range,
        "selectionRange": range
    }))
}

pub(crate) fn completion_item(item: CompletionItem) -> Value {
    let symbol_file = item.symbol_file.map(sifr_analysis::FileId::as_u32);
    json!({
        "label": item.label,
        "kind": completion_kind(&item.kind),
        "detail": item.detail,
        "data": { "sifrKind": item.kind, "sifrFile": symbol_file }
    })
}

pub(crate) fn hover(info: HoverInfo) -> Value {
    json!({
        "contents": {
            "kind": "markdown",
            "value": format!("```sifr\n{}\n```", info.contents)
        }
    })
}

pub(crate) fn signature_help(help: SignatureHelp) -> Value {
    let parameters = help
        .parameters
        .into_iter()
        .map(|label| json!({ "label": label }))
        .collect::<Vec<_>>();
    json!({
        "signatures": [{ "label": help.label, "parameters": parameters }],
        "activeSignature": 0,
        "activeParameter": help.active_parameter.unwrap_or(0)
    })
}

pub(crate) fn semantic_tokens(
    tokens: Vec<SemanticToken>,
    source: &str,
    encoding: PositionEncoding,
) -> LspResult<Value> {
    let source_map = SourceTextMap::new(source);
    let mut encoded = Vec::new();
    let mut previous_line = 0;
    let mut previous_start = 0;
    for token in tokens {
        let Some(start) = source_map.position_at(token.range.start(), encoding) else {
            return Err(LspError::internal(
                "semantic token start is outside the document",
            ));
        };
        let Some(end) = source_map.position_at(token.range.end(), encoding) else {
            return Err(LspError::internal(
                "semantic token end is outside the document",
            ));
        };
        let delta_line = start.line.saturating_sub(previous_line);
        let delta_start = if delta_line == 0 {
            start.character.saturating_sub(previous_start)
        } else {
            start.character
        };
        encoded.push(delta_line);
        encoded.push(delta_start);
        encoded.push(end.character.saturating_sub(start.character));
        encoded.push(token_type_index(&token.token_type));
        encoded.push(token_modifier_bits(&token.modifiers));
        previous_line = start.line;
        previous_start = start.character;
    }
    Ok(json!({ "data": encoded }))
}

pub(crate) fn inlay_hint(
    hint: InlayHint,
    source: &str,
    encoding: PositionEncoding,
) -> LspResult<Value> {
    Ok(json!({
        "position": text_position(&hint.position, source, encoding)?,
        "label": hint.label,
        "kind": 1
    }))
}

pub(crate) fn document_highlight(
    highlight: DocumentHighlight,
    source: &str,
    encoding: PositionEncoding,
) -> LspResult<Value> {
    Ok(json!({ "range": text_range(highlight.range, source, encoding)?, "kind": 1 }))
}

pub(crate) fn folding_range(
    range: FoldingRange,
    source: &str,
    encoding: PositionEncoding,
) -> LspResult<Value> {
    let value = text_range(range.range, source, encoding)?;
    Ok(json!({
        "startLine": value["start"]["line"],
        "startCharacter": value["start"]["character"],
        "endLine": value["end"]["line"],
        "endCharacter": value["end"]["character"]
    }))
}

pub(crate) fn selection_range(
    range: sifr_analysis::SelectionRange,
    source: &str,
    encoding: PositionEncoding,
) -> LspResult<Value> {
    let parent = range
        .parent
        .map(|parent| selection_range(*parent, source, encoding))
        .transpose()?;
    Ok(json!({
        "range": text_range(range.range, source, encoding)?,
        "parent": parent
    }))
}

pub(crate) fn type_hierarchy_item(
    item: TypeHierarchyItem,
    uri: String,
    source: &str,
    encoding: PositionEncoding,
) -> LspResult<Value> {
    let range = item.location.range.map_or_else(
        || Ok(json!({"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 0}})),
        |range| text_range(range, source, encoding),
    )?;
    Ok(json!({
        "name": item.name,
        "kind": symbol_kind(&item.kind),
        "uri": uri,
        "range": range,
        "selectionRange": range,
        "data": item.id.0
    }))
}

pub(crate) fn code_action(
    action: CodeAction,
    request_uri: &str,
    uri_for_file: impl Fn(FileId) -> LspResult<String>,
    source_for_file: impl Fn(FileId) -> LspResult<String>,
    encoding: PositionEncoding,
) -> LspResult<Value> {
    let edit = action
        .edit
        .map(|edit| workspace_edit(edit, uri_for_file, source_for_file, encoding))
        .transpose()?;
    Ok(json!({
        "title": action.title,
        "kind": action.kind,
        "edit": edit,
        "data": code_action_data(action.data, request_uri)
    }))
}

fn code_action_data(data: Option<sifr_analysis::CodeActionData>, request_uri: &str) -> Value {
    match data {
        Some(data) => json!({
            "sifrResolved": false,
            "action": match data.action {
                DeferredCodeAction::FixAllSafePolicy => "fixAllSafePolicy",
            },
            "uri": request_uri,
            "file": data.file.as_u32(),
            "expectedVersion": data.expected_version,
        }),
        None => json!({ "sifrResolved": true }),
    }
}

pub(crate) fn workspace_edit(
    edit: WorkspaceEdit,
    uri_for_file: impl Fn(FileId) -> LspResult<String>,
    source_for_file: impl Fn(FileId) -> LspResult<String>,
    encoding: PositionEncoding,
) -> LspResult<Value> {
    let mut changes = serde_json::Map::new();
    for file_edit in edit.edits {
        let uri = uri_for_file(file_edit.file)?;
        changes.insert(uri, file_text_edits(file_edit, &source_for_file, encoding)?);
    }
    Ok(json!({ "changes": changes }))
}

pub(crate) fn file_text_edits(
    file_edit: FileTextEdits,
    source_for_file: &impl Fn(FileId) -> LspResult<String>,
    encoding: PositionEncoding,
) -> LspResult<Value> {
    let source = source_for_file(file_edit.file)?;
    text_edits(file_edit.edits, &source, encoding)
}

pub(crate) fn text_edits(
    edits: Vec<TextEdit>,
    source: &str,
    encoding: PositionEncoding,
) -> LspResult<Value> {
    edits
        .into_iter()
        .map(|edit| {
            Ok(json!({
                "range": text_range(edit.range, source, encoding)?,
                "newText": edit.replacement
            }))
        })
        .collect::<LspResult<Vec<_>>>()
        .map(Value::Array)
}

pub(crate) fn diagnostic(
    diagnostic: RenderedDiagnostic,
    uri: &str,
    source: &str,
    encoding: PositionEncoding,
) -> LspResult<Value> {
    let primary = diagnostic
        .spans
        .iter()
        .find(|span| span.is_primary)
        .or_else(|| diagnostic.spans.first());
    let class = diagnostic_class(&diagnostic);
    let rule_id = diagnostic_rule_id(&diagnostic);
    let related_information = diagnostic
        .spans
        .iter()
        .filter(|span| !span.is_primary)
        .map(|span| {
            Ok(json!({
                "location": {
                    "uri": uri,
                    "range": diagnostic_span_range(span, source, encoding)?,
                },
                "message": span.label.as_deref().unwrap_or("related declaration"),
            }))
        })
        .collect::<LspResult<Vec<_>>>()?;
    Ok(json!({
        "range": primary.map_or_else(|| Ok(default_range()), |span| diagnostic_span_range(span, source, encoding))?,
        "severity": diagnostic_severity(diagnostic.severity),
        "code": diagnostic.code,
        "codeDescription": { "href": diagnostic.url },
        "source": "sifr",
        "message": diagnostic.message,
        "relatedInformation": related_information,
        "data": {
            "code": diagnostic.code,
            "diagnosticClass": class,
            "ruleId": rule_id,
            "help": diagnostic.help,
            "children": diagnostic.children,
            "suggestions": diagnostic.suggestions
        }
    }))
}

pub(crate) fn diagnostic_id(value: &Value) -> Option<DiagnosticId> {
    let code = value.get("code").and_then(Value::as_str)?;
    let data = value.get("data");
    let class = match data
        .and_then(|data| data.get("diagnosticClass"))
        .and_then(Value::as_str)
    {
        Some("policy") => DiagnosticClass::Policy,
        _ => DiagnosticClass::Hard,
    };
    let rule_id = data
        .and_then(|data| data.get("ruleId"))
        .and_then(Value::as_str)
        .map(str::to_string);
    Some(DiagnosticId {
        code: code.to_string(),
        class,
        rule_id,
    })
}

fn diagnostic_class(diagnostic: &RenderedDiagnostic) -> &'static str {
    if diagnostic_rule_id(diagnostic).is_some() {
        "policy"
    } else {
        "hard"
    }
}

fn diagnostic_rule_id(diagnostic: &RenderedDiagnostic) -> Option<&str> {
    match diagnostic.args.get("rule") {
        Some(DiagnosticArg::String(rule)) => Some(rule),
        _ => None,
    }
}

pub(crate) fn generated_rust_preview(preview: GeneratedRustPreview) -> Value {
    json!({
        "file": preview.file.as_u32(),
        "rust": preview.rust,
        "sourceMapFiles": preview.source_map_files.into_iter().map(|file| {
            json!({
                "path": file.path,
                "origin": source_origin_name(file.origin),
                "source": file.source,
            })
        }).collect::<Vec<_>>(),
        "unavailableReason": preview.unavailable_reason
    })
}

fn source_origin_name(origin: sifr_analysis::SourceOrigin) -> &'static str {
    match origin {
        sifr_analysis::SourceOrigin::UserSource => "UserSource",
        sifr_analysis::SourceOrigin::SysrootPublicStdlib => "SysrootPublicStdlib",
        sifr_analysis::SourceOrigin::SysrootPrivateDeclaration => "SysrootPrivateDeclaration",
        sifr_analysis::SourceOrigin::GeneratedSupport => "GeneratedSupport",
        sifr_analysis::SourceOrigin::CompilerSynthetic => "CompilerSynthetic",
    }
}

fn diagnostic_span_range(
    span: &DiagnosticSpan,
    source: &str,
    encoding: PositionEncoding,
) -> LspResult<Value> {
    if span.byte_start > span.byte_end {
        return Err(LspError::internal("diagnostic span start is after end"));
    }
    text_range(
        TextRange::new(TextSize::new(span.byte_start), TextSize::new(span.byte_end)),
        source,
        encoding,
    )
}

fn default_range() -> Value {
    json!({
        "start": { "line": 0, "character": 0 },
        "end": { "line": 0, "character": 0 }
    })
}

fn diagnostic_severity(severity: Severity) -> u32 {
    match severity {
        Severity::Error => 1,
        Severity::Warning => 2,
        Severity::Note => 3,
    }
}

fn symbol_kind(kind: &str) -> u32 {
    match kind {
        "function" => 12,
        "type" | "class" => 5,
        "module" => 2,
        "variable" => 13,
        _ => 13,
    }
}

fn completion_kind(kind: &str) -> u32 {
    match kind {
        "function" => 3,
        "type" | "class" => 7,
        "module" => 9,
        "property" => 10,
        "keyword" => 14,
        "decorator" => 15,
        _ => 6,
    }
}

fn token_type_index(kind: &str) -> u32 {
    SEMANTIC_TOKEN_TYPES
        .iter()
        .position(|candidate| *candidate == kind)
        .and_then(|index| u32::try_from(index).ok())
        .unwrap_or(4)
}

fn token_modifier_bits(modifiers: &[String]) -> u32 {
    modifiers.iter().fold(0, |bits, modifier| {
        let Some(index) = SEMANTIC_TOKEN_MODIFIERS
            .iter()
            .position(|candidate| *candidate == modifier)
        else {
            return bits;
        };
        let Ok(shift) = u32::try_from(index) else {
            return bits;
        };
        bits | (1_u32 << shift)
    })
}

#[cfg(test)]
mod tests {
    use super::{
        completion_item, diagnostic, lsp_range_with_encoding, signature_help,
        text_range_with_encoding,
    };
    use ruff_text_size::{TextRange, TextSize};
    use serde_json::json;
    use sifr_analysis::{CompletionItem, SignatureHelp};
    use sifr_diagnostics::{DiagnosticSpan, RenderedDiagnostic, Severity};
    use sifr_source::PositionEncoding;
    use std::collections::BTreeMap;

    #[test]
    fn utf16_ranges_round_trip_through_conversion_layer() {
        let source = "a🦀b\n";
        let range_json = json!({
            "start": { "line": 0, "character": 1 },
            "end": { "line": 0, "character": 3 }
        });
        let range = lsp_range_with_encoding(&range_json, source, PositionEncoding::Utf16).unwrap();
        assert_eq!(range, TextRange::new(TextSize::new(1), TextSize::new(5)));
        assert_eq!(
            text_range_with_encoding(range, source, PositionEncoding::Utf16).unwrap(),
            range_json
        );
    }

    #[test]
    fn utf16_ranges_reject_surrogate_pair_interiors() {
        let source = "a🦀b\n";
        let range_json = json!({
            "start": { "line": 0, "character": 2 },
            "end": { "line": 0, "character": 3 }
        });
        assert!(lsp_range_with_encoding(&range_json, source, PositionEncoding::Utf16).is_err());
    }

    #[test]
    fn completion_conversion_maps_rust_interop_kinds_to_lsp_kinds() {
        let decorator = completion_item(CompletionItem {
            label: "rust.callback".to_string(),
            kind: "decorator".to_string(),
            detail: Some("Rust interop decorator".to_string()),
            symbol_file: None,
        });
        assert_eq!(decorator["kind"], json!(15));

        let property = completion_item(CompletionItem {
            label: "panic".to_string(),
            kind: "property".to_string(),
            detail: Some("Rust interop policy key".to_string()),
            symbol_file: None,
        });
        assert_eq!(property["kind"], json!(10));
    }

    #[test]
    fn signature_help_conversion_includes_parameter_labels() {
        let help = signature_help(SignatureHelp {
            label: "combine(left: int, right: int) -> int".to_string(),
            parameters: vec!["left: int".to_string(), "right: int".to_string()],
            active_parameter: Some(1),
        });
        let signature = &help["signatures"][0];
        assert_eq!(signature["label"], "combine(left: int, right: int) -> int");
        assert_eq!(signature["parameters"][0]["label"], "left: int");
        assert_eq!(signature["parameters"][1]["label"], "right: int");
        assert_eq!(help["activeParameter"], 1);
    }

    #[test]
    fn diagnostic_conversion_preserves_related_source_locations() {
        let span = |byte_start, byte_end, is_primary, label| DiagnosticSpan {
            file: Some("model.sifr".to_string()),
            byte_start,
            byte_end,
            line: None,
            column: None,
            end_line: None,
            end_column: None,
            is_primary,
            label,
            lines: Vec::new(),
        };
        let rendered = RenderedDiagnostic {
            code: "SIFR-META-0001".to_string(),
            severity: Severity::Error,
            message: "package fixture.meta specialization failed: rejected".to_string(),
            message_template: "{message}".to_string(),
            args: BTreeMap::new(),
            url: "https://docs.sifr.sh/errors/SIFR-META-0001".to_string(),
            spans: vec![
                span(0, 5, true, None),
                span(6, 11, false, Some("declared here".to_string())),
            ],
            children: Vec::new(),
            help: Some("note: retained".to_string()),
            suggestions: Vec::new(),
        };
        let value = diagnostic(
            rendered,
            "file:///workspace/model.sifr",
            "class field\n",
            PositionEncoding::Utf16,
        )
        .unwrap();
        assert_eq!(value["code"], "SIFR-META-0001");
        assert_eq!(value["data"]["help"], "note: retained");
        assert_eq!(
            value["relatedInformation"][0]["location"]["uri"],
            "file:///workspace/model.sifr"
        );
        assert_eq!(value["relatedInformation"][0]["message"], "declared here");
    }
}

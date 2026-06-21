use lsp_types::PositionEncodingKind;
use serde_json::{json, Value};
use sifr_source::PositionEncoding;

pub(crate) const LANGUAGE_ID: &str = "sifr";

pub(crate) const SEMANTIC_TOKEN_TYPES: &[&str] = &[
    "keyword",
    "type",
    "function",
    "method",
    "variable",
    "parameter",
    "property",
    "module",
    "comment",
    "string",
    "number",
    "operator",
    "decorator",
    "mutableBinding",
    "ownershipSensitive",
];

pub(crate) const SEMANTIC_TOKEN_MODIFIERS: &[&str] = &["declaration", "readonly"];

pub(crate) fn negotiated_position_encoding(initialize_params: &Value) -> PositionEncoding {
    let Some(encodings) = initialize_params
        .pointer("/capabilities/general/positionEncodings")
        .and_then(Value::as_array)
    else {
        return PositionEncoding::Utf16;
    };
    for encoding in encodings {
        match encoding.as_str() {
            Some("utf-8") => return PositionEncoding::Utf8,
            Some("utf-32") => return PositionEncoding::Utf32,
            _ => {}
        }
    }
    PositionEncoding::Utf16
}

pub(crate) fn position_encoding_kind(encoding: PositionEncoding) -> PositionEncodingKind {
    match encoding {
        PositionEncoding::Utf8 => PositionEncodingKind::UTF8,
        PositionEncoding::Utf16 => PositionEncodingKind::UTF16,
        PositionEncoding::Utf32 => PositionEncodingKind::UTF32,
    }
}

pub(crate) fn server_capabilities(
    format_enable: bool,
    position_encoding: PositionEncoding,
) -> Value {
    let mut capabilities = json!({
        "positionEncoding": position_encoding_kind(position_encoding),
        "textDocumentSync": {
            "openClose": true,
            "change": 2,
            "save": { "includeText": true }
        },
        "diagnosticProvider": {
            "identifier": "sifr",
            "interFileDependencies": true,
            "workspaceDiagnostics": true
        },
        "completionProvider": {
            "resolveProvider": true,
            "triggerCharacters": [".", ":", "_"]
        },
        "hoverProvider": true,
        "signatureHelpProvider": {
            "triggerCharacters": ["(", ","]
        },
        "definitionProvider": true,
        "declarationProvider": true,
        "typeDefinitionProvider": true,
        "referencesProvider": true,
        "renameProvider": { "prepareProvider": true },
        "documentSymbolProvider": true,
        "workspaceSymbolProvider": true,
        "semanticTokensProvider": {
            "legend": {
                "tokenTypes": SEMANTIC_TOKEN_TYPES,
                "tokenModifiers": SEMANTIC_TOKEN_MODIFIERS
            },
            "full": true,
            "range": true
        },
        "inlayHintProvider": { "resolveProvider": false },
        "documentHighlightProvider": true,
        "foldingRangeProvider": true,
        "selectionRangeProvider": true,
        "typeHierarchyProvider": true,
        "codeActionProvider": {
            "resolveProvider": true,
            "codeActionKinds": [
                "quickfix",
                "refactor.rename",
                "source.organizeImports",
                "source.fixAll.sifr",
                "source.sifr.suppressPolicyRule"
            ]
        },
        "executeCommandProvider": {
            "commands": [
                "sifr.restartServer",
                "sifr.showServerLogs",
                "sifr.explainDiagnostic",
                "sifr.showGeneratedRust",
                "sifr.checkWorkspace",
                "sifr.runTests"
            ]
        },
        "workspace": {
            "workspaceFolders": { "supported": true, "changeNotifications": true },
            "fileOperations": {}
        }
    });
    if format_enable {
        capabilities["documentFormattingProvider"] = json!(true);
        capabilities["documentRangeFormattingProvider"] = json!(true);
    }
    capabilities
}

#[cfg(test)]
mod tests {
    use super::negotiated_position_encoding;
    use serde_json::json;
    use sifr_source::PositionEncoding;

    #[test]
    fn negotiation_defaults_to_utf16_for_legacy_clients() {
        assert_eq!(
            negotiated_position_encoding(&json!({ "capabilities": {} })),
            PositionEncoding::Utf16
        );
    }

    #[test]
    fn negotiation_prefers_utf8_when_client_offers_it() {
        assert_eq!(
            negotiated_position_encoding(&json!({
                "capabilities": {
                    "general": { "positionEncodings": ["utf-16", "utf-8"] }
                }
            })),
            PositionEncoding::Utf8
        );
    }

    #[test]
    fn negotiation_uses_utf16_for_vscode_style_clients() {
        assert_eq!(
            negotiated_position_encoding(&json!({
                "capabilities": {
                    "general": { "positionEncodings": ["utf-16"] }
                }
            })),
            PositionEncoding::Utf16
        );
    }
}

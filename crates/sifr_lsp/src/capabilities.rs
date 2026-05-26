use lsp_types::PositionEncodingKind;
use serde_json::{json, Value};

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

pub(crate) fn server_capabilities(format_enable: bool) -> Value {
    let mut capabilities = json!({
        "positionEncoding": PositionEncodingKind::UTF8,
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

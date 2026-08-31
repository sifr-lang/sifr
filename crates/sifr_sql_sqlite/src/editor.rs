use crate::lexer::{SpannedToken, Token, tokenize};
use crate::lower_hex;
use crate::parser::SqliteParser;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SqliteRecoveryDocument {
    pub tokens: Vec<RecoveryToken>,
    pub errors: Vec<RecoveryError>,
    pub compile_authority: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecoveryToken {
    pub start: u32,
    pub end: u32,
    pub normalized: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecoveryError {
    pub offset: u32,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SqliteEditorFacts {
    pub normalized: Option<String>,
    pub recovery: SqliteRecoveryDocument,
    pub completion_keywords: Vec<String>,
    pub documentation_base: String,
    pub semantic_settings_fingerprint: String,
}

#[must_use]
pub fn recover_document(source: &str) -> SqliteRecoveryDocument {
    let mut tokens = Vec::new();
    let mut errors = Vec::new();
    for token in tokenize(source) {
        match token {
            Ok((start, token, end)) => tokens.push(recovery_token(start, &token, end)),
            Err(error) => errors.push(RecoveryError {
                offset: u32::try_from(error.offset).unwrap_or(u32::MAX),
                message: error.message,
            }),
        }
    }
    SqliteRecoveryDocument {
        tokens,
        errors,
        compile_authority: false,
    }
}

impl SqliteEditorFacts {
    #[must_use]
    pub fn analyze(parser: &SqliteParser, source: &str) -> Self {
        let recovery = recover_document(source);
        let normalized = parser.normalize(source).ok();
        let mut digest = Sha256::new();
        digest.update(parser.series().profile().as_bytes());
        digest.update([0]);
        for mode in parser.compile_flags() {
            digest.update([0]);
            digest.update(mode.as_bytes());
        }
        Self {
            normalized,
            recovery,
            completion_keywords: [
                "SELECT",
                "FROM",
                "WHERE",
                "JOIN",
                "GROUP BY",
                "HAVING",
                "ORDER BY",
                "LIMIT",
                "INSERT INTO",
                "ON CONFLICT DO UPDATE",
                "RETURNING",
                "STRICT",
                "WITHOUT ROWID",
                "UPDATE",
                "DELETE FROM",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
            documentation_base: "https://sqlite.org/lang.html".to_string(),
            semantic_settings_fingerprint: lower_hex(&digest.finalize()),
        }
    }
}

fn recovery_token(start: usize, token: &Token, end: usize) -> RecoveryToken {
    RecoveryToken {
        start: u32::try_from(start).unwrap_or(u32::MAX),
        end: u32::try_from(end).unwrap_or(u32::MAX),
        normalized: token.normalized(),
    }
}

#[allow(dead_code)]
fn collect_tokens(tokens: Vec<SpannedToken>) -> Vec<RecoveryToken> {
    tokens
        .into_iter()
        .filter_map(Result::ok)
        .map(|(start, token, end)| recovery_token(start, &token, end))
        .collect()
}

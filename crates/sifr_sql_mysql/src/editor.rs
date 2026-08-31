use crate::lexer::{SpannedToken, Token, tokenize};
use crate::lower_hex;
use crate::parser::MysqlParser;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MysqlRecoveryDocument {
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
pub struct MysqlEditorFacts {
    pub normalized: Option<String>,
    pub recovery: MysqlRecoveryDocument,
    pub completion_keywords: Vec<String>,
    pub documentation_base: String,
    pub semantic_settings_fingerprint: String,
}

#[must_use]
pub fn recover_document(source: &str) -> MysqlRecoveryDocument {
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
    MysqlRecoveryDocument {
        tokens,
        errors,
        compile_authority: false,
    }
}

impl MysqlEditorFacts {
    #[must_use]
    pub fn analyze(parser: &MysqlParser, source: &str) -> Self {
        let recovery = recover_document(source);
        let normalized = parser.normalize(source).ok();
        let mut digest = Sha256::new();
        digest.update(parser.series().profile().as_bytes());
        digest.update([0]);
        digest.update(parser.default_collation().as_bytes());
        for mode in parser.sql_modes() {
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
                "ON DUPLICATE KEY UPDATE",
                "UPDATE",
                "DELETE FROM",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
            documentation_base: format!(
                "https://dev.mysql.com/doc/refman/{}.{}/en/",
                parser.series().major,
                parser.series().minor
            ),
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

//! Verification fixture ownership, consumed only by tests and documentation tools.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Component, Path};

use serde_json::Value;

pub(super) struct FixtureCatalog(BTreeMap<String, String>);

impl FixtureCatalog {
    pub(super) fn load<'a>(
        root: &Path,
        active_codes: impl Iterator<Item = &'a str>,
    ) -> io::Result<Self> {
        let path = root.join("verification/areas/diagnostics/data/code_catalog.json");
        let value = serde_json::from_slice(&fs::read(path)?)?;
        Self::parse(root, active_codes, &value)
    }

    pub(super) fn parse<'a>(
        root: &Path,
        active_codes: impl Iterator<Item = &'a str>,
        value: &Value,
    ) -> io::Result<Self> {
        let invalid = |message: &str| io::Error::new(io::ErrorKind::InvalidData, message);
        if value.get("schema_version").and_then(Value::as_u64) != Some(1) {
            return Err(invalid("diagnostic fixture catalog schema must be one"));
        }
        let rows = value
            .get("codes")
            .and_then(Value::as_array)
            .ok_or_else(|| invalid("diagnostic fixture catalog must contain codes"))?;
        let root = root.canonicalize()?;
        let mut fixtures = BTreeMap::new();
        for row in rows {
            let code = row
                .get("code")
                .and_then(Value::as_str)
                .ok_or_else(|| invalid("diagnostic fixture code must be a string"))?;
            let fixture = row
                .get("representative_fixture")
                .and_then(Value::as_str)
                .ok_or_else(|| invalid("diagnostic fixture reference must be a string"))?;
            if fixture.is_empty() || fixture.contains(['`', '\n', '\r']) {
                return Err(invalid(
                    "diagnostic fixture reference is empty or not markdown-safe",
                ));
            }
            let (relative, symbol) = fixture
                .split_once("::")
                .map_or((fixture, None), |(path, symbol)| (path, Some(symbol)));
            let path = Path::new(relative);
            if path.as_os_str().is_empty()
                || relative.contains('\\')
                || relative
                    .split('/')
                    .any(|part| matches!(part, "" | "." | ".."))
                || path
                    .components()
                    .any(|part| !matches!(part, Component::Normal(_)))
                || !root.join(path).exists()
                || !root.join(path).canonicalize()?.starts_with(&root)
            {
                return Err(invalid(
                    "diagnostic fixture must name an existing repository path",
                ));
            }
            if let Some(symbol) = symbol {
                if path.extension().and_then(|value| value.to_str()) != Some("rs")
                    || !root.join(path).is_file()
                    || !symbol.starts_with(|ch: char| ch.is_ascii_alphabetic() || ch == '_')
                    || !symbol
                        .chars()
                        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
                {
                    return Err(invalid(
                        "diagnostic fixture Rust symbol reference is invalid",
                    ));
                }
            }
            if fixtures
                .insert(code.to_owned(), fixture.to_owned())
                .is_some()
            {
                return Err(invalid(
                    "diagnostic fixture catalog contains duplicate codes",
                ));
            }
        }
        let expected: BTreeSet<_> = active_codes.collect();
        let actual: BTreeSet<_> = fixtures.keys().map(String::as_str).collect();
        if actual != expected {
            return Err(invalid(
                "diagnostic fixture catalog must cover exactly the active codes",
            ));
        }
        Ok(Self(fixtures))
    }

    pub(super) fn get(&self, code: &str) -> Option<&str> {
        self.0.get(code).map(String::as_str)
    }
}

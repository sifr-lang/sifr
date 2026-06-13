use serde::Deserialize;
use std::collections::BTreeSet;
use std::env;
use std::path::{Path, PathBuf};

const SUITE_FILTER_ENV: &str = "SIFR_VALIDATION_CONTRACT_SUITE_FILTER";
const MANIFEST_PATH_ENV: &str = "SIFR_VALIDATION_CONTRACT_MANIFEST";

#[derive(Debug, Deserialize)]
pub(crate) struct Manifest {
    pub(crate) suites: Vec<Suite>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Suite {
    pub(crate) name: String,
    pub(crate) label: String,
    pub(crate) rows: Vec<Row>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Row {
    pub(crate) id: String,
    pub(crate) commands: Vec<CommandSpec>,
    pub(crate) assertions: Vec<Assertion>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct CommandSpec {
    pub(crate) id: String,
    pub(crate) argv: Vec<String>,
    pub(crate) expected_exit: i32,
    pub(crate) parallel_group: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Stream {
    Stdout,
    Stderr,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum Assertion {
    Contains {
        command_id: String,
        stream: Stream,
        text: String,
    },
    EqualStreams {
        left_command_id: String,
        right_command_id: String,
        stream: Stream,
    },
}

pub(crate) fn load(repo_root: &Path) -> Result<Vec<Suite>, String> {
    let manifest_path = manifest_path(repo_root)?;
    let raw = std::fs::read_to_string(&manifest_path)
        .map_err(|err| format!("failed to read {}: {err}", manifest_path.display()))?;
    let manifest: Manifest = serde_json::from_str(&raw)
        .map_err(|err| format!("failed to parse {}: {err}", manifest_path.display()))?;
    if manifest.suites.is_empty() {
        return Err("validation contract manifest contains no suites".to_string());
    }

    let suite_filter = suite_filter_from_env();
    let suites = if let Some(filter) = suite_filter {
        let selected = manifest
            .suites
            .into_iter()
            .filter(|suite| filter.contains(&suite.name))
            .collect::<Vec<_>>();
        if selected.len() != filter.len() {
            let present = selected
                .iter()
                .map(|suite| suite.name.clone())
                .collect::<BTreeSet<_>>();
            let missing = filter
                .difference(&present)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ");
            return Err(format!(
                "validation contract suite filter referenced unknown suites: {missing}"
            ));
        }
        selected
    } else {
        manifest.suites
    };

    if suites.is_empty() {
        return Err("no validation contract suites selected".to_string());
    }
    Ok(suites)
}

fn manifest_path(_repo_root: &Path) -> Result<PathBuf, String> {
    let from_env = env::var(MANIFEST_PATH_ENV)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let path = match from_env {
        Some(raw) => PathBuf::from(raw),
        None => {
            return Err(format!(
                "{MANIFEST_PATH_ENV} must point at an area-owned validation contract manifest"
            ));
        }
    };
    if !path.is_file() {
        return Err(format!(
            "validation contract manifest not found: {}",
            path.display()
        ));
    }
    Ok(path)
}

fn suite_filter_from_env() -> Option<BTreeSet<String>> {
    env::var(SUITE_FILTER_ENV)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|suite| !suite.is_empty())
                .map(ToOwned::to_owned)
                .collect::<BTreeSet<_>>()
        })
}

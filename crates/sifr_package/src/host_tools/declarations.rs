//! Parse the closed host-tool namespace and capability declaration contract.

use super::{
    HOST_TOOL_CAPABILITIES, RESERVED_TOOL_NAMESPACES, host_tool_diagnostic,
    required_nonempty_string,
};
use crate::PackageDiagnostic;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ToolDeclaration {
    pub(super) package: String,
    pub(super) entrypoint: String,
    pub(super) capabilities: BTreeSet<String>,
}

pub(super) fn parse_tool_declarations(
    path: &Path,
    source: &str,
) -> Result<BTreeMap<String, ToolDeclaration>, Vec<PackageDiagnostic>> {
    let root = source.parse::<toml::Table>().map_err(|error| {
        vec![host_tool_diagnostic(format!(
            "cannot parse tools manifest '{}': {error}",
            path.display()
        ))]
    })?;
    let Some(tools) = root.get("tools").and_then(toml::Value::as_table) else {
        return Err(vec![host_tool_diagnostic(format!(
            "tools manifest '{}' requires a [tools] table",
            path.display()
        ))]);
    };
    let mut declarations = BTreeMap::new();
    let mut diagnostics = Vec::new();
    for (namespace, value) in tools {
        if !valid_namespace(namespace) {
            diagnostics.push(host_tool_diagnostic(format!(
                "tool namespace '{namespace}' must use lowercase ASCII letters, digits, and single hyphens"
            )));
            continue;
        }
        if RESERVED_TOOL_NAMESPACES.contains(&namespace.as_str()) {
            diagnostics.push(host_tool_diagnostic(format!(
                "tool namespace '{namespace}' is reserved by Sifr"
            )));
            continue;
        }
        let Some(table) = value.as_table() else {
            diagnostics.push(host_tool_diagnostic(format!(
                "tools.{namespace} must be a table"
            )));
            continue;
        };
        if let Some(key) = table
            .keys()
            .find(|key| !matches!(key.as_str(), "package" | "entrypoint" | "capabilities"))
        {
            diagnostics.push(host_tool_diagnostic(format!(
                "tools.{namespace} contains unsupported key '{key}'"
            )));
            continue;
        }
        let Some(package) = required_nonempty_string(table, "package") else {
            diagnostics.push(host_tool_diagnostic(format!(
                "tools.{namespace}.package must be a non-empty string"
            )));
            continue;
        };
        let Some(entrypoint) = required_nonempty_string(table, "entrypoint") else {
            diagnostics.push(host_tool_diagnostic(format!(
                "tools.{namespace}.entrypoint must be a non-empty string"
            )));
            continue;
        };
        let capabilities = match parse_capabilities(namespace, table.get("capabilities")) {
            Ok(capabilities) => capabilities,
            Err(error) => {
                diagnostics.push(error);
                continue;
            }
        };
        declarations.insert(
            namespace.clone(),
            ToolDeclaration {
                package,
                entrypoint,
                capabilities,
            },
        );
    }
    if declarations.is_empty() && diagnostics.is_empty() {
        diagnostics.push(host_tool_diagnostic("tools manifest exports no namespaces"));
    }
    if diagnostics.is_empty() {
        Ok(declarations)
    } else {
        Err(diagnostics)
    }
}

fn parse_capabilities(
    namespace: &str,
    value: Option<&toml::Value>,
) -> Result<BTreeSet<String>, PackageDiagnostic> {
    let Some(values) = value.and_then(toml::Value::as_array) else {
        return Err(host_tool_diagnostic(format!(
            "tools.{namespace}.capabilities must be an explicit array"
        )));
    };
    let mut capabilities = BTreeSet::new();
    for value in values {
        let Some(capability) = value.as_str() else {
            return Err(host_tool_diagnostic(format!(
                "tools.{namespace}.capabilities entries must be strings"
            )));
        };
        if !HOST_TOOL_CAPABILITIES.contains(&capability) {
            return Err(host_tool_diagnostic(format!(
                "tools.{namespace} requests unknown capability '{capability}'"
            )));
        }
        if !capabilities.insert(capability.to_string()) {
            return Err(host_tool_diagnostic(format!(
                "tools.{namespace} repeats capability '{capability}'"
            )));
        }
    }
    Ok(capabilities)
}

fn valid_namespace(namespace: &str) -> bool {
    !namespace.is_empty()
        && !namespace.starts_with('-')
        && !namespace.ends_with('-')
        && !namespace.contains("--")
        && namespace
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

use super::binding_authoring::{
    python_binding_generated_digest, python_binding_source_fingerprint, safe_package_path,
    PythonBinding, PythonBindingArtifact, PythonBindingSourceKind, PYTHON_BINDING_SCHEMA_VERSION,
};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub fn validate_python_bindings(
    package_root: &Path,
    environment_digest: &str,
    artifact: &PythonBindingArtifact,
) -> Result<(), String> {
    validate_python_bindings_inner(package_root, environment_digest, artifact, None)
}

pub fn validate_python_bindings_with_generated_source(
    package_root: &Path,
    environment_digest: &str,
    artifact: &PythonBindingArtifact,
    generated_module: &str,
    generated_output: &str,
    generated_source: &[u8],
) -> Result<(), String> {
    validate_python_bindings_inner(
        package_root,
        environment_digest,
        artifact,
        Some((generated_module, generated_output, generated_source)),
    )
}

fn validate_python_bindings_inner(
    package_root: &Path,
    environment_digest: &str,
    artifact: &PythonBindingArtifact,
    generated: Option<(&str, &str, &[u8])>,
) -> Result<(), String> {
    if artifact.schema_version != PYTHON_BINDING_SCHEMA_VERSION {
        return Err(format!(
            "unsupported Python binding schema version {}; expected {}",
            artifact.schema_version, PYTHON_BINDING_SCHEMA_VERSION
        ));
    }
    if artifact.environment_digest != environment_digest {
        return Err(
            "Python binding environment digest does not match the selected environment".to_string(),
        );
    }
    if artifact.bindings.is_empty() {
        return Err("Python binding artifact must contain at least one binding".to_string());
    }
    let mut previous_module: Option<&str> = None;
    let mut outputs = BTreeSet::new();
    for binding in &artifact.bindings {
        if previous_module.is_some_and(|previous| previous >= binding.module.as_str()) {
            return Err("Python bindings must be sorted by unique module".to_string());
        }
        previous_module = Some(&binding.module);
        if !outputs.insert(binding.output.as_str()) {
            return Err(format!(
                "duplicate Python binding output '{}'",
                binding.output
            ));
        }
        validate_binding(package_root, binding, generated)?;
    }
    Ok(())
}

fn validate_binding(
    package_root: &Path,
    binding: &PythonBinding,
    generated: Option<(&str, &str, &[u8])>,
) -> Result<(), String> {
    if !is_dotted_name(&binding.module) || binding.soabi.trim().is_empty() {
        return Err("Python binding module and SOABI must be valid and non-empty".to_string());
    }
    if binding.symbols.is_empty()
        || !binding.symbols.iter().all(|symbol| is_identifier(symbol))
        || !binding.symbols.windows(2).all(|pair| pair[0] < pair[1])
    {
        return Err(format!(
            "Python binding '{}' symbols must be sorted, unique identifiers",
            binding.module
        ));
    }
    let generated_bytes = generated
        .filter(|(module, output, _)| *module == binding.module && *output == binding.output)
        .map(|(_, _, bytes)| bytes);
    let stored_bytes;
    let bytes = if let Some(bytes) = generated_bytes {
        let _ = safe_package_path(package_root, &binding.output, "binding output")?;
        bytes
    } else {
        let output = regular_package_file(package_root, &binding.output, "binding output")?;
        stored_bytes = std::fs::read(&output).map_err(|error| {
            format!(
                "could not read Python binding '{}': {error}",
                output.display()
            )
        })?;
        &stored_bytes
    };
    if python_binding_generated_digest(bytes) != binding.generated_digest {
        return Err(format!(
            "generated Python binding '{}' has drifted",
            binding.output
        ));
    }
    if let Some(distribution) = &binding.distribution {
        if distribution.name.trim().is_empty() || distribution.version.trim().is_empty() {
            return Err(format!(
                "Python binding '{}' has an incomplete distribution identity",
                binding.module
            ));
        }
    }
    if binding.sources.len() != binding.symbols.len() {
        return Err(format!(
            "Python binding '{}' must record one source for every symbol",
            binding.module
        ));
    }
    if !binding
        .stub_packages
        .windows(2)
        .all(|pair| pair[0] < pair[1])
        || binding
            .stub_packages
            .iter()
            .any(|package| package.trim().is_empty())
    {
        return Err(format!(
            "Python binding '{}' stub packages must be sorted, unique names",
            binding.module
        ));
    }
    for configured in binding.overrides.iter().chain(&binding.external_stubs) {
        let _ = regular_package_file(package_root, configured, "typing source")?;
    }
    validate_configured_source_digests(package_root, binding)?;
    for (source, symbol) in binding.sources.iter().zip(&binding.symbols) {
        if source.symbol != *symbol
            || source.identity.trim().is_empty()
            || source.digest.len() != 64
            || !source.digest.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(format!(
                "Python binding '{}' has invalid source evidence for '{symbol}'",
                binding.module
            ));
        }
    }
    let fingerprint = python_binding_source_fingerprint(
        &binding.module,
        &binding.symbols,
        &binding.soabi,
        binding.distribution.as_ref(),
        &binding.sources,
    );
    if fingerprint != binding.source_fingerprint {
        return Err(format!(
            "Python binding '{}' source fingerprint is invalid",
            binding.module
        ));
    }
    Ok(())
}

fn validate_configured_source_digests(
    package_root: &Path,
    binding: &PythonBinding,
) -> Result<(), String> {
    for (kind, configured) in [
        (PythonBindingSourceKind::Override, &binding.overrides),
        (
            PythonBindingSourceKind::ExternalStub,
            &binding.external_stubs,
        ),
    ] {
        for (index, relative) in configured.iter().enumerate() {
            let prefix = format!("{}:{index}:", source_kind_prefix(kind));
            let consumed = binding
                .sources
                .iter()
                .any(|source| source.kind == kind && source.identity.starts_with(&prefix));
            if !consumed {
                continue;
            }
            let path = regular_package_file(package_root, relative, "typing source")?;
            let bytes = std::fs::read(&path).map_err(|error| {
                format!(
                    "could not read Python typing source '{}': {error}",
                    path.display()
                )
            })?;
            let digest = python_binding_generated_digest(&bytes);
            if !binding.sources.iter().any(|source| {
                source.kind == kind
                    && source.identity.starts_with(&prefix)
                    && source.digest == digest
            }) {
                return Err(format!(
                    "Python binding typing source '{relative}' has drifted"
                ));
            }
        }
    }
    Ok(())
}

fn regular_package_file(package_root: &Path, value: &str, label: &str) -> Result<PathBuf, String> {
    let path = safe_package_path(package_root, value, label)?;
    let metadata = std::fs::symlink_metadata(&path).map_err(|error| {
        format!(
            "could not inspect Python {label} '{}': {error}",
            path.display()
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!(
            "Python {label} '{}' must be a regular package file",
            path.display()
        ));
    }
    let canonical_root = package_root
        .canonicalize()
        .map_err(|error| format!("could not resolve package root: {error}"))?;
    let canonical = path.canonicalize().map_err(|error| {
        format!(
            "could not resolve Python {label} '{}': {error}",
            path.display()
        )
    })?;
    if !canonical.starts_with(&canonical_root) {
        return Err(format!(
            "Python {label} '{value}' must stay inside the package"
        ));
    }
    Ok(canonical)
}

const fn source_kind_prefix(kind: PythonBindingSourceKind) -> &'static str {
    match kind {
        PythonBindingSourceKind::Override => "override",
        PythonBindingSourceKind::ExternalStub => "external",
        PythonBindingSourceKind::StubPackage
        | PythonBindingSourceKind::PyTyped
        | PythonBindingSourceKind::Introspection => "",
    }
}

fn is_dotted_name(value: &str) -> bool {
    !value.is_empty() && value.split('.').all(is_identifier)
}

fn is_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    chars
        .next()
        .is_some_and(|first| first == '_' || first.is_alphabetic())
        && chars.all(|character| character == '_' || character.is_alphanumeric())
}

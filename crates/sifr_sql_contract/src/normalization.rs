use crate::{
    DialectIdentity, ProviderIdentity, SCHEMA_IR_FORMAT_VERSION, SchemaContractError,
    SchemaContractErrorKind, SchemaIr, SchemaObject, SemanticValue,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SchemaDocumentKind {
    SqlDdl,
    ProviderMetadata,
    GeneratedDefinitions,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaDocument {
    pub kind: SchemaDocumentKind,
    pub document: String,
    pub objects: Vec<SchemaObject>,
}

pub fn normalize_schema(
    provider: ProviderIdentity,
    dialect: DialectIdentity,
    documents: impl IntoIterator<Item = SchemaDocument>,
) -> Result<SchemaIr, SchemaContractError> {
    validate_provider(&provider)?;
    validate_dialect(&dialect)?;
    let mut objects = BTreeMap::new();
    for document in documents {
        if document.document.is_empty() {
            return Err(invalid_schema("schema document identity must not be empty"));
        }
        for mut object in document.objects {
            validate_object(&object)?;
            if let Some(source) = &mut object.source {
                if source.document.is_empty() {
                    source.document.clone_from(&document.document);
                }
                if source.document != document.document {
                    return Err(invalid_schema(format!(
                        "object '{}' source document does not match its schema document",
                        object.identity
                    )));
                }
            }
            let identity = object.identity.clone();
            if objects.insert(identity.clone(), object).is_some() {
                return Err(SchemaContractError::new(
                    SchemaContractErrorKind::DuplicateObject,
                    format!("schema object '{identity}' is declared more than once"),
                ));
            }
        }
    }
    let schema = SchemaIr {
        format_version: SCHEMA_IR_FORMAT_VERSION,
        provider,
        dialect,
        objects,
    };
    validate_normalized_schema(&schema)?;
    Ok(schema)
}

pub(crate) fn validate_normalized_schema(schema: &SchemaIr) -> Result<(), SchemaContractError> {
    if schema.format_version != SCHEMA_IR_FORMAT_VERSION {
        return Err(invalid_schema(format!(
            "unsupported SchemaIR format version {}",
            schema.format_version
        )));
    }
    validate_provider(&schema.provider)?;
    validate_dialect(&schema.dialect)?;
    for (identity, object) in &schema.objects {
        if identity != &object.identity {
            return Err(invalid_schema(format!(
                "schema map key '{identity}' does not match object identity '{}'",
                object.identity
            )));
        }
        validate_object(object)?;
        for dependency in &object.dependencies {
            if !schema.objects.contains_key(dependency) {
                return Err(SchemaContractError::new(
                    SchemaContractErrorKind::MissingDependency,
                    format!(
                        "schema object '{}' depends on missing object '{dependency}'",
                        object.identity
                    ),
                ));
            }
        }
    }
    Ok(())
}

fn validate_provider(provider: &ProviderIdentity) -> Result<(), SchemaContractError> {
    if provider.package_id.is_empty()
        || provider.package_source.is_empty()
        || provider.package_graph_digest.is_empty()
    {
        return Err(SchemaContractError::new(
            SchemaContractErrorKind::InvalidProvider,
            "provider identity must include its locked package, source, and graph digest",
        ));
    }
    if provider.compiler_components.is_empty()
        || provider
            .compiler_components
            .values()
            .any(|digest| !is_sha256(digest))
    {
        return Err(SchemaContractError::new(
            SchemaContractErrorKind::InvalidProvider,
            "provider identity must include at least one exact compiler component SHA-256",
        ));
    }
    Ok(())
}

fn validate_dialect(dialect: &DialectIdentity) -> Result<(), SchemaContractError> {
    if dialect.family.is_empty() || dialect.server_version.is_empty() {
        return Err(invalid_schema(
            "dialect family and server version must not be empty",
        ));
    }
    if dialect.modes.keys().any(String::is_empty) || dialect.features.iter().any(String::is_empty) {
        return Err(invalid_schema(
            "dialect mode and feature names must not be empty",
        ));
    }
    Ok(())
}

fn validate_object(object: &SchemaObject) -> Result<(), SchemaContractError> {
    if !valid_qualified_name(object.identity.as_str()) {
        return Err(invalid_schema(format!(
            "schema object identity '{}' is not a canonical qualified name",
            object.identity
        )));
    }
    for (key, value) in &object.semantic {
        if key.is_empty() {
            return Err(invalid_schema(format!(
                "schema object '{}' has an empty semantic property name",
                object.identity
            )));
        }
        validate_value(value)?;
    }
    if let Some(source) = &object.source
        && (source.end < source.start || source.document.is_empty())
    {
        return Err(invalid_schema(format!(
            "schema object '{}' has an invalid source location",
            object.identity
        )));
    }
    Ok(())
}

fn validate_value(value: &SemanticValue) -> Result<(), SchemaContractError> {
    match value {
        SemanticValue::BytesHex(value) if value.len() % 2 != 0 || !value.is_ascii() => Err(
            invalid_schema("byte properties must use even-length ASCII hex"),
        ),
        SemanticValue::BytesHex(value) if !value.bytes().all(|byte| byte.is_ascii_hexdigit()) => {
            Err(invalid_schema(
                "byte properties must use hexadecimal digits",
            ))
        }
        SemanticValue::List(values) => {
            for value in values {
                validate_value(value)?;
            }
            Ok(())
        }
        SemanticValue::Set(values) => {
            for value in values {
                validate_value(value)?;
            }
            Ok(())
        }
        SemanticValue::Map(values) => {
            for (key, value) in values {
                if key.is_empty() {
                    return Err(invalid_schema("semantic map keys must not be empty"));
                }
                validate_value(value)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn valid_qualified_name(value: &str) -> bool {
    !value.is_empty()
        && value.split('.').all(|part| {
            let mut chars = part.chars();
            chars
                .next()
                .is_some_and(|first| first == '_' || first.is_alphabetic())
                && chars.all(|character| character == '_' || character.is_alphanumeric())
        })
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn invalid_schema(message: impl Into<String>) -> SchemaContractError {
    SchemaContractError::new(SchemaContractErrorKind::InvalidSchema, message)
}

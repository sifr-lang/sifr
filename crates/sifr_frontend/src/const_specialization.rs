//! Package-neutral contracts for deterministic const specialization.

use num_bigint::BigInt;
use ruff_text_size::TextRange;
use sifr_diagnostics::{DiagnosticArg, DiagnosticCode};
use sifr_lowering::HirDiagnostic;
use sifr_type_system::FixedIntType;
use std::collections::{BTreeMap, BTreeSet};

const MAX_ISSUES: usize = 32;
const MAX_ARGUMENTS: usize = 32;
const MAX_LABELS: usize = 8;
const MAX_NOTES: usize = 8;
const MAX_TEXT_BYTES: usize = 4096;
const JS_SAFE_INTEGER: i64 = 9_007_199_254_740_991;
const RESERVED_PACKAGE_ARGUMENTS: &[&str] = &["rule"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstValue {
    None,
    Bool(bool),
    Integer(BigInt),
    FloatBits(u64),
    String(String),
    Bytes(Vec<u8>),
    Tuple(Vec<Self>),
    List(Vec<Self>),
    Record(BTreeMap<String, Self>),
    /// Compiler-issued diagnostic token. Package const code can carry this
    /// value but cannot construct it or retain it in a static program.
    SourceOrigin(crate::class_declarations::SourceOriginId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstIssueSeverity {
    Fatal,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstIssueLabel {
    pub span: TextRange,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstPackageIssue {
    pub package: String,
    pub reason_code: String,
    pub severity: ConstIssueSeverity,
    pub arguments: BTreeMap<String, ConstValue>,
    pub primary_span: TextRange,
    pub labels: Vec<ConstIssueLabel>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstIssueTemplate {
    pub package: String,
    pub reason_code: String,
    pub argument_names: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstSpecializationOutcome<T> {
    Produced {
        value: T,
        warnings: Vec<ConstPackageIssue>,
    },
    Failed {
        issues: Vec<ConstPackageIssue>,
    },
}

/// Decode the closed record returned by a package-owned `@const_eval` specializer.
///
/// The record has exactly `status`, `value`, and `issues`. Issue records have exactly
/// `package`, `reason_code`, `severity`, `arguments`, `primary_origin`, `labels`, and `notes`;
/// the compiler resolves origins and never executes package rendering code.
pub(crate) fn decode_const_specialization_outcome(
    value: ConstValue,
    primary_span: TextRange,
    origins: &crate::class_declarations::SourceOriginTable,
) -> Result<ConstSpecializationOutcome<ConstValue>, Vec<HirDiagnostic>> {
    decode_outcome_record(value, origins).map_err(|problem| {
        vec![malformed_diagnostic(
            "unknown",
            "outcome_shape",
            problem,
            Some(primary_span),
        )]
    })
}

fn decode_outcome_record(
    value: ConstValue,
    origins: &crate::class_declarations::SourceOriginTable,
) -> Result<ConstSpecializationOutcome<ConstValue>, String> {
    let ConstValue::Record(mut fields) = value else {
        return Err("const specialization outcome must be a record".to_string());
    };
    if fields.len() != 3 {
        return Err(
            "const specialization outcome must contain exactly status, value, and issues"
                .to_string(),
        );
    }
    let status = take_string(&mut fields, "status")?;
    let value = fields
        .remove("value")
        .ok_or_else(|| "const specialization outcome is missing value".to_string())?;
    let issues = take_list(&mut fields, "issues")?
        .into_iter()
        .map(|issue| decode_issue(issue, origins))
        .collect::<Result<Vec<_>, _>>()?;
    if !fields.is_empty() {
        return Err("const specialization outcome contains unknown fields".to_string());
    }
    match status.as_str() {
        "produced" => Ok(ConstSpecializationOutcome::Produced {
            value,
            warnings: issues,
        }),
        "failed" => {
            if issues.is_empty() {
                return Err(
                    "a failed const specialization must contain at least one issue".to_string(),
                );
            }
            if value != ConstValue::None {
                return Err("a failed const specialization cannot contain a value".to_string());
            }
            Ok(ConstSpecializationOutcome::Failed { issues })
        }
        _ => Err("const specialization status must be produced or failed".to_string()),
    }
}

fn decode_issue(
    value: ConstValue,
    origins: &crate::class_declarations::SourceOriginTable,
) -> Result<ConstPackageIssue, String> {
    let ConstValue::Record(mut fields) = value else {
        return Err("const package issue must be a record".to_string());
    };
    if fields.len() != 7 {
        return Err(
            "const package issue must contain exactly package, reason_code, severity, arguments, primary_origin, labels, and notes"
                .to_string(),
        );
    }
    let package = take_string(&mut fields, "package")?;
    let reason_code = take_string(&mut fields, "reason_code")?;
    let severity = match take_string(&mut fields, "severity")?.as_str() {
        "fatal" => ConstIssueSeverity::Fatal,
        "warning" => ConstIssueSeverity::Warning,
        _ => return Err("const package issue severity must be fatal or warning".to_string()),
    };
    let arguments = match fields.remove("arguments") {
        Some(ConstValue::Record(arguments)) => arguments,
        Some(_) => return Err("const package issue arguments must be a record".to_string()),
        None => return Err("const package issue is missing arguments".to_string()),
    };
    let primary_origin = take_source_origin(&mut fields, "primary_origin")?;
    let primary_span = origins
        .resolve(primary_origin)
        .map(|origin| origin.range)
        .ok_or_else(|| {
            "const package issue primary_origin is not part of the adapted declaration".to_string()
        })?;
    let labels = take_list(&mut fields, "labels")?
        .into_iter()
        .map(|label| decode_label(label, origins))
        .collect::<Result<Vec<_>, _>>()?;
    let notes = take_list(&mut fields, "notes")?
        .into_iter()
        .map(|note| match note {
            ConstValue::String(note) => Ok(note),
            _ => Err("const package issue notes must contain only strings".to_string()),
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ConstPackageIssue {
        package,
        reason_code,
        severity,
        arguments,
        primary_span,
        labels,
        notes,
    })
}

fn decode_label(
    value: ConstValue,
    origins: &crate::class_declarations::SourceOriginTable,
) -> Result<ConstIssueLabel, String> {
    let ConstValue::Record(mut fields) = value else {
        return Err("const package issue label must be a record".to_string());
    };
    if fields.len() != 2 {
        return Err(
            "const package issue label must contain exactly origin and message".to_string(),
        );
    }
    let origin = take_source_origin(&mut fields, "origin")?;
    let span = origins
        .resolve(origin)
        .map(|origin| origin.range)
        .ok_or_else(|| {
            "const package issue label origin is not part of the adapted declaration".to_string()
        })?;
    let message = take_string(&mut fields, "message")?;
    Ok(ConstIssueLabel { span, message })
}

fn take_source_origin(
    fields: &mut BTreeMap<String, ConstValue>,
    name: &str,
) -> Result<crate::SourceOriginId, String> {
    match fields.remove(name) {
        Some(ConstValue::SourceOrigin(value)) => Ok(value),
        Some(_) => Err(format!(
            "const field '{name}' must be a compiler-issued source origin"
        )),
        None => Err(format!("const record is missing field '{name}'")),
    }
}

fn take_string(fields: &mut BTreeMap<String, ConstValue>, name: &str) -> Result<String, String> {
    match fields.remove(name) {
        Some(ConstValue::String(value)) => Ok(value),
        Some(_) => Err(format!("const field '{name}' must be a string")),
        None => Err(format!("const record is missing field '{name}'")),
    }
}

fn take_list(
    fields: &mut BTreeMap<String, ConstValue>,
    name: &str,
) -> Result<Vec<ConstValue>, String> {
    match fields.remove(name) {
        Some(ConstValue::List(value)) => Ok(value),
        Some(_) => Err(format!("const field '{name}' must be a list")),
        None => Err(format!("const record is missing field '{name}'")),
    }
}

#[derive(Debug, Clone)]
pub struct ValidatedConstSpecialization<T> {
    pub value: Option<T>,
    pub issues: Vec<ConstPackageIssue>,
}

impl<T> ConstSpecializationOutcome<T> {
    pub fn validate(
        self,
        templates: &[ConstIssueTemplate],
    ) -> Result<ValidatedConstSpecialization<T>, Vec<HirDiagnostic>> {
        let issues = match &self {
            Self::Produced { warnings, .. } => warnings,
            Self::Failed { issues } => issues,
        };
        let mut diagnostics = Vec::new();
        let mut validated_issues = Vec::new();
        if issues.len() > MAX_ISSUES {
            diagnostics.push(malformed_diagnostic(
                "unknown",
                "issue_limit",
                format!("at most {MAX_ISSUES} issues may be emitted"),
                None,
            ));
            return Err(diagnostics);
        }
        for issue in issues {
            if let Err(problem) = validate_issue(issue, templates) {
                diagnostics.push(malformed_diagnostic(
                    &issue.package,
                    &issue.reason_code,
                    problem,
                    Some(issue.primary_span),
                ));
                continue;
            }
            match (&self, issue.severity) {
                (Self::Produced { .. }, ConstIssueSeverity::Warning)
                | (Self::Failed { .. }, ConstIssueSeverity::Fatal) => {
                    validated_issues.push(issue.clone());
                }
                (Self::Produced { .. }, ConstIssueSeverity::Fatal) => {
                    diagnostics.push(malformed_diagnostic(
                        &issue.package,
                        &issue.reason_code,
                        "a produced value cannot carry a fatal issue".to_string(),
                        Some(issue.primary_span),
                    ));
                }
                (Self::Failed { .. }, ConstIssueSeverity::Warning) => {
                    diagnostics.push(malformed_diagnostic(
                        &issue.package,
                        &issue.reason_code,
                        "a failed outcome may contain only fatal issues".to_string(),
                        Some(issue.primary_span),
                    ));
                }
            }
        }
        if diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == Some(DiagnosticCode::META_MALFORMED_DECLARATION))
        {
            Err(diagnostics)
        } else {
            let value = match self {
                Self::Produced { value, .. } => Some(value),
                Self::Failed { .. } => None,
            };
            Ok(ValidatedConstSpecialization {
                value,
                issues: validated_issues,
            })
        }
    }
}

fn validate_issue(
    issue: &ConstPackageIssue,
    templates: &[ConstIssueTemplate],
) -> Result<(), String> {
    if !valid_package_name(&issue.package) {
        return Err("package must be a lowercase qualified package name".to_string());
    }
    if !valid_reason_code(&issue.reason_code) {
        return Err(
            "reason_code must use lowercase ASCII letters, digits, underscores, or dots"
                .to_string(),
        );
    }
    if issue.arguments.len() > MAX_ARGUMENTS {
        return Err(format!(
            "at most {MAX_ARGUMENTS} template arguments are allowed"
        ));
    }
    if issue.labels.len() > MAX_LABELS || issue.notes.len() > MAX_NOTES {
        return Err(format!(
            "at most {MAX_LABELS} labels and {MAX_NOTES} notes are allowed"
        ));
    }
    let template = templates
        .iter()
        .find(|template| {
            template.package == issue.package && template.reason_code == issue.reason_code
        })
        .ok_or_else(|| "reason_code is not declared by the package".to_string())?;
    let supplied = issue.arguments.keys().cloned().collect::<BTreeSet<_>>();
    if supplied != template.argument_names {
        return Err(
            "template argument names do not exactly match the static declaration".to_string(),
        );
    }
    if supplied
        .iter()
        .any(|name| RESERVED_PACKAGE_ARGUMENTS.contains(&name.as_str()))
    {
        return Err("template argument name is reserved by compiler or LSP rendering".to_string());
    }
    for (name, value) in &issue.arguments {
        if !valid_argument_name(name) {
            return Err(format!("invalid template argument name '{name}'"));
        }
        validate_const_value(value, 0)?;
    }
    if issue
        .labels
        .iter()
        .any(|label| label.message.len() > MAX_TEXT_BYTES)
        || issue.notes.iter().any(|note| note.len() > MAX_TEXT_BYTES)
    {
        return Err(format!(
            "label and note text is limited to {MAX_TEXT_BYTES} bytes"
        ));
    }
    Ok(())
}

fn validate_const_value(value: &ConstValue, depth: usize) -> Result<(), String> {
    if depth > 16 {
        return Err("template argument nesting exceeds 16 levels".to_string());
    }
    match value {
        ConstValue::String(value) if value.len() > MAX_TEXT_BYTES => Err(format!(
            "template string is limited to {MAX_TEXT_BYTES} bytes"
        )),
        ConstValue::Tuple(values) | ConstValue::List(values) => {
            if values.len() > 128 {
                return Err("template collection is limited to 128 values".to_string());
            }
            for value in values {
                validate_const_value(value, depth + 1)?;
            }
            Ok(())
        }
        ConstValue::Record(values) => {
            if values.len() > 128 {
                return Err("template record is limited to 128 fields".to_string());
            }
            for (name, value) in values {
                if !valid_argument_name(name) {
                    return Err(format!("invalid template record field '{name}'"));
                }
                validate_const_value(value, depth + 1)?;
            }
            Ok(())
        }
        ConstValue::Bytes(value) if value.len() > 128 => {
            Err("template bytes are limited to 128 values".to_string())
        }
        ConstValue::SourceOrigin(_) => {
            Err("source origins cannot be package template arguments".to_string())
        }
        ConstValue::None
        | ConstValue::Bool(_)
        | ConstValue::Integer(_)
        | ConstValue::FloatBits(_)
        | ConstValue::String(_)
        | ConstValue::Bytes(_) => Ok(()),
    }
}

fn valid_package_name(value: &str) -> bool {
    !value.is_empty()
        && value.split('.').all(|part| {
            !part.is_empty()
                && part.bytes().all(|byte| {
                    byte.is_ascii_lowercase()
                        || byte.is_ascii_digit()
                        || matches!(byte, b'_' | b'-')
                })
        })
}

fn valid_reason_code(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'.')
        })
}

fn valid_argument_name(value: &str) -> bool {
    let mut bytes = value.bytes();
    bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_lowercase() || byte == b'_')
        && bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

pub(crate) fn package_note(issue: &ConstPackageIssue) -> Option<String> {
    if issue.arguments.is_empty() && issue.notes.is_empty() {
        return None;
    }
    let mut parts = Vec::new();
    if !issue.arguments.is_empty() {
        let arguments = issue
            .arguments
            .iter()
            .map(|(name, value)| format!("{name}={value:?}"))
            .collect::<Vec<_>>()
            .join(", ");
        parts.push(format!("package arguments: {arguments}"));
    }
    parts.extend(issue.notes.iter().map(|note| format!("note: {note}")));
    Some(parts.join("; "))
}

fn malformed_diagnostic(
    package: &str,
    reason_code: &str,
    declaration_problem: String,
    primary_range: Option<TextRange>,
) -> HirDiagnostic {
    HirDiagnostic {
        code: Some(DiagnosticCode::META_MALFORMED_DECLARATION),
        message: format!(
            "package {package} declared malformed specialization issue {reason_code}: {declaration_problem}"
        ),
        args: BTreeMap::from([
            (
                "package".to_string(),
                DiagnosticArg::String(package.to_string()),
            ),
            (
                "reason_code".to_string(),
                DiagnosticArg::String(reason_code.to_string()),
            ),
            (
                "declaration_problem".to_string(),
                DiagnosticArg::String(declaration_problem),
            ),
        ]),
        help: None,
        primary_range,
        line: None,
        col: None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonIntegerProfile {
    Exact,
    Web,
    StringInts,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonIntegerRepresentation {
    ProfileDefault,
    Number,
    DecimalString,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonIntegerKind {
    Exact,
    Fixed(FixedIntType),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonIntegerBoundaryDescriptor {
    pub profile: Option<JsonIntegerProfile>,
    pub integer_kind: JsonIntegerKind,
    pub static_range: Option<(BigInt, BigInt)>,
    pub representation: JsonIntegerRepresentation,
    pub source_path: String,
    pub source_span: Option<TextRange>,
}

pub fn verify_json_integer_boundary(
    descriptor: &JsonIntegerBoundaryDescriptor,
) -> Result<(), Box<HirDiagnostic>> {
    let Some(profile) = descriptor.profile else {
        return Err(Box::new(integer_boundary_diagnostic(
            descriptor,
            "missing profile",
            "select json.exact, json.web, or json.string_ints",
        )));
    };
    if descriptor
        .static_range
        .as_ref()
        .is_some_and(|(minimum, maximum)| minimum > maximum)
    {
        return Err(Box::new(integer_boundary_diagnostic(
            descriptor,
            "invalid static range",
            "declare an ordered inclusive integer range",
        )));
    }
    match (profile, descriptor.representation) {
        (JsonIntegerProfile::StringInts, JsonIntegerRepresentation::Number) => {
            Err(Box::new(integer_boundary_diagnostic(
                descriptor,
                "json.string_ints cannot emit numeric integers",
                "use the profile-default decimal string representation",
            )))
        }
        (JsonIntegerProfile::Exact, JsonIntegerRepresentation::DecimalString) => {
            Err(Box::new(integer_boundary_diagnostic(
                descriptor,
                "json.exact requires canonical JSON numbers",
                "use the profile-default exact-number representation",
            )))
        }
        (JsonIntegerProfile::Web, JsonIntegerRepresentation::Number)
            if !range_is_javascript_safe(descriptor.static_range.as_ref()) =>
        {
            Err(Box::new(integer_boundary_diagnostic(
                descriptor,
                "json.web numeric output is not statically JavaScript-safe",
                "use profile-default decimal strings or declare a JavaScript-safe range",
            )))
        }
        _ => Ok(()),
    }
}

fn range_is_javascript_safe(range: Option<&(BigInt, BigInt)>) -> bool {
    let Some((minimum, maximum)) = range else {
        return false;
    };
    minimum >= &BigInt::from(-JS_SAFE_INTEGER) && maximum <= &BigInt::from(JS_SAFE_INTEGER)
}

fn integer_boundary_diagnostic(
    descriptor: &JsonIntegerBoundaryDescriptor,
    boundary: &str,
    suggested_policy: &str,
) -> HirDiagnostic {
    let profile = descriptor
        .profile
        .map_or_else(|| "missing".to_string(), |profile| format!("{profile:?}"));
    let static_range = descriptor.static_range.as_ref().map_or_else(
        || "unknown".to_string(),
        |(minimum, maximum)| format!("{minimum}..={maximum}"),
    );
    HirDiagnostic {
        code: Some(DiagnosticCode::INT_JSON_BOUNDARY_POLICY),
        message: format!(
            "integer JSON boundary policy is unsafe at {}: {boundary}",
            descriptor.source_path
        ),
        args: BTreeMap::from([
            (
                "path".to_string(),
                DiagnosticArg::String(descriptor.source_path.clone()),
            ),
            (
                "boundary".to_string(),
                DiagnosticArg::String(boundary.to_string()),
            ),
            ("profile".to_string(), DiagnosticArg::String(profile)),
            (
                "static_range".to_string(),
                DiagnosticArg::String(static_range),
            ),
            (
                "suggested_policy".to_string(),
                DiagnosticArg::String(suggested_policy.to_string()),
            ),
        ]),
        help: Some(suggested_policy.to_string()),
        primary_range: descriptor.source_span,
        line: None,
        col: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ruff_text_size::TextSize;

    fn span() -> TextRange {
        TextRange::new(TextSize::new(1), TextSize::new(2))
    }

    fn template() -> ConstIssueTemplate {
        ConstIssueTemplate {
            package: "example.mapper".to_string(),
            reason_code: "shape.unsupported".to_string(),
            argument_names: BTreeSet::from(["kind".to_string()]),
        }
    }

    #[test]
    fn valid_warning_preserves_value_and_maps_to_hard_warning() {
        let warning = ConstPackageIssue {
            package: "example.mapper".to_string(),
            reason_code: "shape.unsupported".to_string(),
            severity: ConstIssueSeverity::Warning,
            arguments: BTreeMap::from([(
                "kind".to_string(),
                ConstValue::String("callable".to_string()),
            )]),
            primary_span: span(),
            labels: Vec::new(),
            notes: Vec::new(),
        };
        let outcome = ConstSpecializationOutcome::Produced {
            value: "schema",
            warnings: vec![warning],
        };
        assert!(outcome.validate(&[template()]).is_ok());
    }

    #[test]
    fn malformed_and_fatal_outcomes_use_closed_meta_codes() {
        let fatal = ConstPackageIssue {
            package: "example.mapper".to_string(),
            reason_code: "shape.unsupported".to_string(),
            severity: ConstIssueSeverity::Fatal,
            arguments: BTreeMap::new(),
            primary_span: span(),
            labels: Vec::new(),
            notes: Vec::new(),
        };
        let errors = ConstSpecializationOutcome::<()>::Failed {
            issues: vec![fatal],
        }
        .validate(&[template()])
        .expect_err("argument mismatch must be rejected");
        assert_eq!(
            errors[0].code,
            Some(DiagnosticCode::META_MALFORMED_DECLARATION)
        );
    }

    #[test]
    fn integer_boundary_profiles_fail_closed() {
        let descriptor = JsonIntegerBoundaryDescriptor {
            profile: Some(JsonIntegerProfile::Web),
            integer_kind: JsonIntegerKind::Exact,
            static_range: None,
            representation: JsonIntegerRepresentation::Number,
            source_path: "Model.id".to_string(),
            source_span: Some(span()),
        };
        let diagnostic = verify_json_integer_boundary(&descriptor)
            .expect_err("unsafe web integer number must fail");
        assert_eq!(
            diagnostic.code,
            Some(DiagnosticCode::INT_JSON_BOUNDARY_POLICY)
        );
    }

    #[test]
    fn web_number_accepts_proven_safe_range() {
        let descriptor = JsonIntegerBoundaryDescriptor {
            profile: Some(JsonIntegerProfile::Web),
            integer_kind: JsonIntegerKind::Fixed(FixedIntType::I32),
            static_range: Some((BigInt::from(i32::MIN), BigInt::from(i32::MAX))),
            representation: JsonIntegerRepresentation::Number,
            source_path: "Record.value".to_string(),
            source_span: Some(span()),
        };
        assert!(verify_json_integer_boundary(&descriptor).is_ok());
    }
}

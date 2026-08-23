//! Package-issue collection and rendering for deterministic specialization.

use crate::specialization_support::malformed;
use crate::{
    ConstEvalError, ConstIssueSeverity, ConstIssueTemplate, ConstPackageIssue,
    FrontendDiagnosticStyle, FrontendSourceContext, diagnostic_with_source_ranges_args_help,
    package_note,
};
use ruff_text_size::TextRange;
use sifr_diagnostics::{DiagnosticArg, DiagnosticCode, RenderedDiagnostic};
use sifr_lowering::{ExternalDefs, HirDiagnostic};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) enum SpecializationDiagnostic {
    Hir(HirDiagnostic),
    Package(ConstPackageIssue),
}

pub(crate) struct SpecializationDiagnostics(Vec<SpecializationDiagnostic>);

impl SpecializationDiagnostics {
    pub(crate) fn from_hir(diagnostics: Vec<HirDiagnostic>) -> Self {
        Self(
            diagnostics
                .into_iter()
                .map(SpecializationDiagnostic::Hir)
                .collect(),
        )
    }

    pub(crate) fn push(&mut self, diagnostic: HirDiagnostic) {
        self.0.push(SpecializationDiagnostic::Hir(diagnostic));
    }

    pub(crate) fn push_package(&mut self, diagnostic: ConstPackageIssue) {
        self.0.push(SpecializationDiagnostic::Package(diagnostic));
    }

    pub(crate) fn extend(&mut self, diagnostics: impl IntoIterator<Item = HirDiagnostic>) {
        self.0
            .extend(diagnostics.into_iter().map(SpecializationDiagnostic::Hir));
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub(crate) fn into_vec(self) -> Vec<SpecializationDiagnostic> {
        self.0
    }
}

pub(crate) fn render_package_issue(
    module_name: &str,
    diagnostic_style: FrontendDiagnosticStyle,
    source_context: Option<FrontendSourceContext<'_>>,
    issue: &ConstPackageIssue,
) -> RenderedDiagnostic {
    let code = match issue.severity {
        ConstIssueSeverity::Fatal => DiagnosticCode::META_SPECIALIZATION_FATAL,
        ConstIssueSeverity::Warning => DiagnosticCode::META_SPECIALIZATION_WARNING,
    };
    let kind = match issue.severity {
        ConstIssueSeverity::Fatal => "failed",
        ConstIssueSeverity::Warning => "warning",
    };
    let bare_message = format!(
        "package {} specialization {kind}: {}",
        issue.package, issue.reason_code
    );
    let message = match diagnostic_style {
        FrontendDiagnosticStyle::Bare => bare_message,
        FrontendDiagnosticStyle::ModulePrefixed => format!("[{module_name}] {bare_message}"),
    };
    let args = BTreeMap::from([
        (
            "package".to_string(),
            DiagnosticArg::String(issue.package.clone()),
        ),
        (
            "reason_code".to_string(),
            DiagnosticArg::String(issue.reason_code.clone()),
        ),
    ]);
    let related = issue
        .labels
        .iter()
        .map(|label| (label.span, label.message.clone()))
        .collect::<Vec<_>>();
    if let Some(context) = source_context {
        return diagnostic_with_source_ranges_args_help(
            code,
            context,
            issue.primary_span,
            &related,
            "{message}",
            BTreeMap::from([("message".to_string(), DiagnosticArg::String(message))]),
            args,
            package_note(issue),
        );
    }
    let mut rendered = crate::diagnostic_with_code(message, code);
    rendered.args.extend(args);
    rendered.help = package_note(issue);
    rendered
}

pub(crate) fn issue_templates(
    external_defs: &ExternalDefs,
    package: &str,
    function: &str,
) -> Vec<ConstIssueTemplate> {
    external_defs
        .declaration_metadata
        .get(package)
        .into_iter()
        .flatten()
        .filter(|metadata| metadata.owner == function && metadata.key == "sifr.meta.issue_template")
        .filter_map(|metadata| {
            let value = crate::structural_shape::const_value_from_hir(&metadata.value)?;
            let crate::ConstValue::Tuple(mut parts) = value else {
                return None;
            };
            if parts.len() != 2 {
                return None;
            }
            let crate::ConstValue::List(arguments) = parts.pop()? else {
                return None;
            };
            let crate::ConstValue::String(reason_code) = parts.pop()? else {
                return None;
            };
            let argument_names = arguments
                .into_iter()
                .map(|argument| match argument {
                    crate::ConstValue::String(argument) => Some(argument),
                    _ => None,
                })
                .collect::<Option<BTreeSet<_>>>()?;
            Some(ConstIssueTemplate {
                package: package.to_string(),
                reason_code,
                argument_names,
            })
        })
        .collect()
}

pub(crate) fn evaluation_error(
    package: &str,
    error: &ConstEvalError,
    range: TextRange,
) -> HirDiagnostic {
    malformed(
        package,
        "const_evaluation",
        format!("{:?}: {}", error.kind, error.detail),
        range,
    )
}

pub(crate) fn replace_unknown_package(diagnostic: &mut HirDiagnostic, package: &str) {
    if matches!(diagnostic.args.get("package"), Some(DiagnosticArg::String(value)) if value == "unknown")
    {
        diagnostic.args.insert(
            "package".to_string(),
            DiagnosticArg::String(package.to_string()),
        );
        diagnostic.message =
            diagnostic
                .message
                .replacen("package unknown", &format!("package {package}"), 1);
    }
}

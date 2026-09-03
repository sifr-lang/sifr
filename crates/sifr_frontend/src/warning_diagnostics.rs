//! Rendering for non-fatal lowering diagnostics shared by CLI and editor analysis.

use crate::{
    FrontendSourceContext, diagnostic_with_source_range, diagnostic_with_source_ranges_args_help,
};
use sifr_diagnostics::{DiagnosticArg, DiagnosticCode, RenderedDiagnostic};
use sifr_lowering::{LoweringWarningDiagnostic, RevealTypeDiagnostic};
use std::collections::BTreeMap;

pub fn reveal_type_diagnostics(
    source_context: Option<FrontendSourceContext<'_>>,
    reveal_types: &[RevealTypeDiagnostic],
) -> Vec<RenderedDiagnostic> {
    reveal_types
        .iter()
        .map(|diagnostic| reveal_type_diagnostic(source_context, diagnostic))
        .collect()
}

pub fn warning_diagnostics(
    source_context: Option<FrontendSourceContext<'_>>,
    warnings: &[LoweringWarningDiagnostic],
) -> Vec<RenderedDiagnostic> {
    warnings
        .iter()
        .map(|diagnostic| warning_diagnostic(source_context, diagnostic))
        .collect()
}

fn warning_diagnostic(
    source_context: Option<FrontendSourceContext<'_>>,
    diagnostic: &LoweringWarningDiagnostic,
) -> RenderedDiagnostic {
    let structured_help = match diagnostic {
        LoweringWarningDiagnostic::MetaPackageIssue { help, .. } => help.clone(),
        LoweringWarningDiagnostic::UnreachableStatement { .. } => None,
    };
    let (code, message, message_template, args, primary_range) = match diagnostic {
        LoweringWarningDiagnostic::UnreachableStatement { primary_range } => (
            DiagnosticCode::FLOW_UNREACHABLE_STATEMENT,
            "unreachable statement ignored".to_string(),
            "unreachable statement ignored",
            Vec::new(),
            *primary_range,
        ),
        LoweringWarningDiagnostic::MetaPackageIssue {
            package,
            reason_code,
            help: _,
            primary_range,
            related_ranges: _,
        } => (
            DiagnosticCode::META_SPECIALIZATION_WARNING,
            format!("package {package} specialization warning: {reason_code}"),
            "package {package} specialization warning: {reason_code}",
            vec![
                ("package", DiagnosticArg::String(package.clone())),
                ("reason_code", DiagnosticArg::String(reason_code.clone())),
            ],
            *primary_range,
        ),
    };
    let related_ranges = match diagnostic {
        LoweringWarningDiagnostic::MetaPackageIssue { related_ranges, .. } => {
            related_ranges.as_slice()
        }
        LoweringWarningDiagnostic::UnreachableStatement { .. } => &[],
    };
    if let (Some(context), Some(range)) = (source_context, primary_range) {
        let args = args
            .iter()
            .map(|(name, value)| ((*name).to_string(), value.clone()))
            .collect();
        return diagnostic_with_source_ranges_args_help(
            code,
            context,
            range,
            related_ranges,
            message_template,
            args,
            BTreeMap::new(),
            structured_help,
        );
    }
    let mut rendered = rendered_spanless_diagnostic(code, message, message_template, &args);
    rendered.help = structured_help;
    rendered
}

fn reveal_type_diagnostic(
    source_context: Option<FrontendSourceContext<'_>>,
    diagnostic: &RevealTypeDiagnostic,
) -> RenderedDiagnostic {
    let code = DiagnosticCode::TYPE_REVEAL_TYPE;
    let message = format!("revealed type is {}", diagnostic.revealed_type);
    let args = [(
        "revealed_type",
        DiagnosticArg::String(diagnostic.revealed_type.clone()),
    )];
    if let (Some(context), Some(range)) = (source_context, diagnostic.primary_range) {
        return diagnostic_with_source_range(
            code,
            context,
            range,
            "revealed type is {revealed_type}",
            &args,
        );
    }
    rendered_spanless_diagnostic(code, message, "revealed type is {revealed_type}", &args)
}

fn rendered_spanless_diagnostic(
    code: DiagnosticCode,
    message: String,
    message_template: &'static str,
    args: &[(&'static str, DiagnosticArg)],
) -> RenderedDiagnostic {
    let mut rendered_args = BTreeMap::new();
    for (name, value) in args {
        rendered_args.insert((*name).to_string(), value.clone());
    }
    RenderedDiagnostic {
        code: code.code().to_string(),
        severity: code.declared_severity(),
        message,
        message_template: message_template.to_string(),
        args: rendered_args,
        url: code.docs_url(),
        spans: Vec::new(),
        children: Vec::new(),
        help: None,
        suggestions: Vec::new(),
    }
}

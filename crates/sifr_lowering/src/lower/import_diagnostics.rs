use ruff_text_size::TextRange;
use sifr_diagnostics::{DiagnosticArg, DiagnosticCode};
use sifr_stdlib_imports::BareStdlibMatch;
use std::collections::BTreeMap;

use super::LowerCtx;

pub(in crate::lower) fn forbidden_intrinsic(ctx: &mut LowerCtx, module: &str, range: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::IMPORT_FORBIDDEN_INTRINSIC,
        format!(
            "cannot import from '{module}' — private sysroot declarations can only be imported by public sysroot stdlib source"
        ),
        range,
    );
}

pub(in crate::lower) fn unknown_import_target(ctx: &mut LowerCtx, module: &str, range: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::IMPORT_UNKNOWN_SOURCE_MODULE,
        format!("unknown import target: '{module}'"),
        range,
    );
}

pub(in crate::lower) fn deferred_compat_module(
    ctx: &mut LowerCtx,
    module: &str,
    reason: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::IMPORT_UNKNOWN_SOURCE_MODULE,
        format!("module '{module}' is intentionally deferred: {reason}"),
        range,
    );
}

pub(in crate::lower) fn unsupported_form(ctx: &mut LowerCtx, form: &str, range: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::IMPORT_UNSUPPORTED_FORM,
        format!("unsupported import form: {form}"),
        range,
    );
}

pub(in crate::lower) fn bare_stdlib(
    ctx: &mut LowerCtx,
    stdlib_match: &BareStdlibMatch,
    imported_names: &str,
    range: TextRange,
) {
    let mut args = BTreeMap::new();
    args.insert(
        "bare_module".to_string(),
        DiagnosticArg::String(stdlib_match.bare_module.clone()),
    );
    args.insert(
        "suggested_module".to_string(),
        DiagnosticArg::String(stdlib_match.suggested_module.clone()),
    );
    args.insert(
        "imported_names".to_string(),
        DiagnosticArg::String(imported_names.to_string()),
    );
    ctx.error_with_code_args_help_at(
        DiagnosticCode::IMPORT_BARE_STDLIB,
        format!(
            "bare stdlib import '{}'; Sifr stdlib lives under 'sifr.*'",
            stdlib_match.bare_module
        ),
        args,
        Some(bare_stdlib_help(stdlib_match, imported_names)),
        range,
    );
}

pub(in crate::lower) fn bare_stdlib_help(
    stdlib_match: &BareStdlibMatch,
    imported_names: &str,
) -> String {
    let suggestion = if imported_names.is_empty() {
        format!("use 'from {} import <name>'", stdlib_match.suggested_module)
    } else {
        format!(
            "use 'from {} import {}'",
            stdlib_match.suggested_module, imported_names
        )
    };
    if stdlib_match.exact_public_module_exists {
        return suggestion;
    }
    format!(
        "{suggestion}; no embedded sifr.{} module exists",
        stdlib_match.bare_module
    )
}

pub(in crate::lower) fn private_member(
    ctx: &mut LowerCtx,
    module: &str,
    name: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::IMPORT_PRIVATE_MEMBER,
        format!("cannot import private name '{name}' from module '{module}'"),
        range,
    );
}

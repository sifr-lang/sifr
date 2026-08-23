use super::{
    LowerCtx,
    diagnostics::{collect_enum_variants, is_enum_class},
    simple_expr::lower_expr_simple,
};
use ruff_text_size::Ranged;
use sifr_diagnostics::{DiagnosticArg, DiagnosticCode};
use sifr_ir::{
    ConstSpecializationRequest, DeclarationMetadataTargetKind, JsonIntegerBoundaryRequest,
    TypedDeclarationMetadata,
};
use sifr_python_ast::{Decorator, Expr, Stmt, StmtClassDef, StmtFunctionDef};
use std::collections::{BTreeMap, BTreeSet};

pub(in crate::lower) fn collect(stmts: &[Stmt], ctx: &mut LowerCtx) {
    for stmt in stmts {
        match stmt {
            Stmt::FunctionDef(function) => collect_function(function, None, ctx),
            Stmt::ClassDef(class) => collect_class(class, ctx),
            _ => {}
        }
    }
}

fn collect_class(class: &StmtClassDef, ctx: &mut LowerCtx) {
    let owner = class.name.to_string();
    let mut allowed_targets = BTreeMap::new();
    if is_enum_class(class) {
        allowed_targets.insert(
            DeclarationMetadataTargetKind::EnumVariant,
            collect_enum_variants(class)
                .into_iter()
                .map(|variant| variant.name)
                .collect(),
        );
    } else {
        allowed_targets.insert(
            DeclarationMetadataTargetKind::Field,
            class
                .body
                .iter()
                .filter_map(|stmt| match stmt {
                    Stmt::AnnAssign(assign) => match assign.target.as_ref() {
                        Expr::Name(name) => Some(name.id.to_string()),
                        _ => None,
                    },
                    _ => None,
                })
                .collect(),
        );
    }
    collect_specialization_requests(&class.decorator_list, &owner, ctx);
    collect_integer_boundary_requests(&class.decorator_list, &owner, ctx);
    collect_decorators(
        &class.decorator_list,
        &owner,
        DeclarationMetadataTargetKind::Type,
        &allowed_targets,
        ctx,
    );
    for stmt in &class.body {
        if let Stmt::FunctionDef(function) = stmt {
            collect_function(function, Some(&owner), ctx);
        }
    }
}

fn collect_integer_boundary_requests(decorators: &[Decorator], owner: &str, ctx: &mut LowerCtx) {
    for decorator in decorators {
        let Expr::Call(call) = &decorator.expression else {
            continue;
        };
        if !matches!(call.func.as_ref(), Expr::Name(name) if name.id.as_str() == "json_integer_boundary")
        {
            continue;
        }
        if call.arguments.args.len() != 5 || !call.arguments.keywords.is_empty() {
            malformed(
                ctx,
                "sifr.meta",
                "integer_boundary",
                "@json_integer_boundary requires field, profile, representation, minimum, and maximum",
                call.range(),
            );
            continue;
        }
        let Some(field) = string_literal(&call.arguments.args[0]) else {
            malformed(
                ctx,
                "sifr.meta",
                "integer_boundary",
                "integer boundary field must be a string literal",
                call.arguments.args[0].range(),
            );
            continue;
        };
        let profile = match &call.arguments.args[1] {
            Expr::NoneLiteral(_) => None,
            value => {
                let Some(value) = string_literal(value) else {
                    malformed(
                        ctx,
                        "sifr.meta",
                        "integer_boundary",
                        "integer boundary profile must be a string literal or None",
                        value.range(),
                    );
                    continue;
                };
                Some(value)
            }
        };
        let Some(representation) = string_literal(&call.arguments.args[2]) else {
            malformed(
                ctx,
                "sifr.meta",
                "integer_boundary",
                "integer boundary representation must be a string literal",
                call.arguments.args[2].range(),
            );
            continue;
        };
        let Some(static_minimum) = optional_integer(&call.arguments.args[3]) else {
            malformed(
                ctx,
                "sifr.meta",
                "integer_boundary",
                "integer boundary minimum must be a const integer or None",
                call.arguments.args[3].range(),
            );
            continue;
        };
        let Some(static_maximum) = optional_integer(&call.arguments.args[4]) else {
            malformed(
                ctx,
                "sifr.meta",
                "integer_boundary",
                "integer boundary maximum must be a const integer or None",
                call.arguments.args[4].range(),
            );
            continue;
        };
        ctx.json_integer_boundary_requests
            .push(JsonIntegerBoundaryRequest {
                owner: owner.to_string(),
                field,
                profile,
                representation,
                static_minimum,
                static_maximum,
                range: decorator.expression.range(),
            });
    }
}

fn optional_integer(expr: &Expr) -> Option<Option<num_bigint::BigInt>> {
    if matches!(expr, Expr::NoneLiteral(_)) {
        return Some(None);
    }
    match lower_expr_simple(expr)? {
        sifr_ir::HirExpr::IntLiteral(value) => Some(Some(value.into())),
        sifr_ir::HirExpr::LargeIntLiteral(value) => value.parse().ok().map(Some),
        _ => None,
    }
}

fn collect_specialization_requests(decorators: &[Decorator], owner: &str, ctx: &mut LowerCtx) {
    let mut seen_request = false;
    for decorator in decorators {
        let Expr::Call(call) = &decorator.expression else {
            continue;
        };
        if !matches!(call.func.as_ref(), Expr::Name(name) if name.id.as_str() == "const_specialize")
        {
            continue;
        }
        if seen_request {
            malformed(
                ctx,
                "unknown",
                "duplicate_specialization_request",
                "a class may declare exactly one @const_specialize decorator",
                call.range(),
            );
            continue;
        }
        seen_request = true;
        if call.arguments.args.len() != 2 || !call.arguments.keywords.is_empty() {
            malformed(
                ctx,
                "unknown",
                "specialization_request",
                "@const_specialize requires package module and function string literals",
                call.range(),
            );
            continue;
        }
        let Some(package_module) = string_literal(&call.arguments.args[0]) else {
            malformed(
                ctx,
                "unknown",
                "specialization_request",
                "specialization package module must be a string literal",
                call.arguments.args[0].range(),
            );
            continue;
        };
        let Some(function) = string_literal(&call.arguments.args[1]) else {
            malformed(
                ctx,
                package_module.as_str(),
                "specialization_request",
                "specialization function must be a string literal",
                call.arguments.args[1].range(),
            );
            continue;
        };
        if !valid_metadata_key(&format!("{package_module}.request")) {
            malformed(
                ctx,
                package_module.as_str(),
                "specialization_request",
                "specialization package module must be a lowercase qualified name",
                call.arguments.args[0].range(),
            );
            continue;
        }
        ctx.specialization_requests
            .push(ConstSpecializationRequest {
                owner: owner.to_string(),
                package_module,
                function,
                range: decorator.expression.range(),
            });
    }
}

fn collect_function(function: &StmtFunctionDef, class: Option<&str>, ctx: &mut LowerCtx) {
    let function_name = function.name.to_string();
    let owner = class.map_or_else(
        || function_name.clone(),
        |class| format!("{class}.{function_name}"),
    );
    let direct_kind = if class.is_some() {
        DeclarationMetadataTargetKind::Method
    } else {
        DeclarationMetadataTargetKind::Function
    };
    let parameter_names = function
        .parameters
        .posonlyargs
        .iter()
        .chain(&function.parameters.args)
        .map(|parameter| parameter.parameter.name.to_string())
        .chain(
            function
                .parameters
                .vararg
                .iter()
                .map(|parameter| parameter.name.to_string()),
        )
        .chain(
            function
                .parameters
                .kwonlyargs
                .iter()
                .map(|parameter| parameter.parameter.name.to_string()),
        )
        .chain(
            function
                .parameters
                .kwarg
                .iter()
                .map(|parameter| parameter.name.to_string()),
        )
        .collect::<BTreeSet<_>>();
    let allowed_targets =
        BTreeMap::from([(DeclarationMetadataTargetKind::Parameter, parameter_names)]);
    collect_decorators(
        &function.decorator_list,
        &owner,
        direct_kind,
        &allowed_targets,
        ctx,
    );
}

fn collect_decorators(
    decorators: &[Decorator],
    owner: &str,
    direct_kind: DeclarationMetadataTargetKind,
    allowed_targets: &BTreeMap<DeclarationMetadataTargetKind, BTreeSet<String>>,
    ctx: &mut LowerCtx,
) {
    for decorator in decorators {
        let Expr::Call(call) = &decorator.expression else {
            continue;
        };
        if !matches!(call.func.as_ref(), Expr::Name(name) if name.id.as_str() == "metadata") {
            continue;
        }
        if !call.arguments.keywords.is_empty() || !matches!(call.arguments.args.len(), 2 | 4) {
            malformed(
                ctx,
                "unknown",
                "metadata_declaration",
                "@metadata accepts either (key, value) or (target_kind, target_name, key, value)",
                call.range(),
            );
            continue;
        }

        let (target_kind, target_name, key_expr, value_expr) = if call.arguments.args.len() == 2 {
            (
                direct_kind,
                None,
                &call.arguments.args[0],
                &call.arguments.args[1],
            )
        } else {
            let Some(kind_name) = string_literal(&call.arguments.args[0]) else {
                malformed(
                    ctx,
                    "unknown",
                    "metadata_target",
                    "metadata target kind must be a string literal",
                    call.arguments.args[0].range(),
                );
                continue;
            };
            let Some(target_name) = string_literal(&call.arguments.args[1]) else {
                malformed(
                    ctx,
                    "unknown",
                    "metadata_target",
                    "metadata target name must be a string literal",
                    call.arguments.args[1].range(),
                );
                continue;
            };
            let Some(target_kind) = targeted_kind(kind_name.as_str(), direct_kind) else {
                malformed(
                    ctx,
                    "unknown",
                    "metadata_target",
                    "metadata target kind is not valid for this declaration",
                    call.arguments.args[0].range(),
                );
                continue;
            };
            (
                target_kind,
                Some(target_name),
                &call.arguments.args[2],
                &call.arguments.args[3],
            )
        };

        let Some(key) = string_literal(key_expr) else {
            malformed(
                ctx,
                "unknown",
                "metadata_key",
                "metadata key must be a string literal",
                key_expr.range(),
            );
            continue;
        };
        let package = metadata_package(&key).unwrap_or("unknown");
        if !valid_metadata_key(&key) {
            malformed(
                ctx,
                package,
                "metadata_key",
                "metadata key must be a lowercase package-qualified name",
                key_expr.range(),
            );
            continue;
        }
        if let Some(target_name) = &target_name {
            if !allowed_targets
                .get(&target_kind)
                .is_some_and(|names| names.contains(target_name))
            {
                malformed(
                    ctx,
                    package,
                    "metadata_target",
                    "metadata target does not exist on the declaration",
                    call.arguments.args[1].range(),
                );
                continue;
            }
        }
        let Some(value) = lower_expr_simple(value_expr) else {
            malformed(
                ctx,
                package,
                "metadata_value",
                "metadata value must be a statically typed const expression",
                value_expr.range(),
            );
            continue;
        };
        ctx.declaration_metadata.push(TypedDeclarationMetadata {
            owner: owner.to_string(),
            target_kind,
            target_name,
            key,
            value_type: value.ty().clone(),
            value,
            range: decorator.expression.range(),
        });
    }
}

fn targeted_kind(
    name: &str,
    direct_kind: DeclarationMetadataTargetKind,
) -> Option<DeclarationMetadataTargetKind> {
    match (direct_kind, name) {
        (DeclarationMetadataTargetKind::Type, "field") => {
            Some(DeclarationMetadataTargetKind::Field)
        }
        (DeclarationMetadataTargetKind::Type, "enum_variant") => {
            Some(DeclarationMetadataTargetKind::EnumVariant)
        }
        (
            DeclarationMetadataTargetKind::Function | DeclarationMetadataTargetKind::Method,
            "parameter",
        ) => Some(DeclarationMetadataTargetKind::Parameter),
        _ => None,
    }
}

fn string_literal(expr: &Expr) -> Option<String> {
    match expr {
        Expr::StringLiteral(value) => Some(value.value.to_str().to_string()),
        _ => None,
    }
}

fn valid_metadata_key(key: &str) -> bool {
    let segments = key.split('.').collect::<Vec<_>>();
    segments.len() >= 2
        && segments.iter().all(|segment| {
            !segment.is_empty()
                && segment
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
                && segment
                    .as_bytes()
                    .first()
                    .is_some_and(u8::is_ascii_lowercase)
        })
}

fn metadata_package(key: &str) -> Option<&str> {
    key.rsplit_once('.').map(|(package, _)| package)
}

fn malformed(
    ctx: &mut LowerCtx,
    package: &str,
    reason_code: &str,
    problem: &str,
    range: ruff_text_size::TextRange,
) {
    let args = BTreeMap::from([
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
            DiagnosticArg::String(problem.to_string()),
        ),
    ]);
    ctx.error_with_code_args_help_at(
        DiagnosticCode::META_MALFORMED_DECLARATION,
        format!(
            "package {package} declared malformed specialization issue {reason_code}: {problem}"
        ),
        args,
        None,
        range,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lower::lower_module;
    use sifr_python_parser::parse_module;

    fn errors<T>(result: Result<T, Vec<crate::HirDiagnostic>>) -> Vec<crate::HirDiagnostic> {
        match result {
            Ok(_) => panic!("lowering unexpectedly succeeded"),
            Err(errors) => errors,
        }
    }

    #[test]
    fn collects_typed_metadata_for_all_declaration_target_families() {
        let source = r#"
@metadata("fixture.model", "record")
@metadata("field", "value", "fixture.description", "payload")
class Model:
    value: int

    @metadata("fixture.callback", True)
    @metadata("parameter", "scale", "fixture.unit", "ratio")
    def transform(self, scale: int) -> int:
        return self.value * scale

@metadata("fixture.function", 7)
@metadata("parameter", "value", "fixture.unit", "count")
def inspect(value: int) -> int:
    return value
"#;
        let parsed = parse_module(source).expect("fixture parses");
        let lowered = lower_module(parsed.suite()).expect("fixture lowers");
        assert_eq!(lowered.declaration_metadata.len(), 6);
        assert!(
            lowered
                .declaration_metadata
                .iter()
                .any(|item| item.target_kind == DeclarationMetadataTargetKind::Field)
        );
        assert!(
            lowered
                .declaration_metadata
                .iter()
                .any(|item| item.target_kind == DeclarationMetadataTargetKind::Parameter)
        );
    }

    #[test]
    fn rejects_dynamic_metadata_values_with_meta_code() {
        let source = "@metadata(\"fixture.value\", build())\ndef inspect(value: int) -> int:\n    return value\n";
        let parsed = parse_module(source).expect("fixture parses");
        let diagnostics = errors(lower_module(parsed.suite()));
        assert_eq!(
            diagnostics[0].code,
            Some(DiagnosticCode::META_MALFORMED_DECLARATION)
        );
    }

    #[test]
    fn rejects_metadata_for_missing_declaration_child() {
        let source = r#"
@metadata("field", "missing", "fixture.description", "payload")
class Model:
    value: int

@metadata("parameter", "missing", "fixture.unit", "count")
def inspect(value: int) -> int:
    return value
"#;
        let parsed = parse_module(source).expect("fixture parses");
        let diagnostics = errors(lower_module(parsed.suite()));
        assert_eq!(diagnostics.len(), 2);
        assert!(diagnostics.iter().all(|diagnostic| {
            diagnostic.code == Some(DiagnosticCode::META_MALFORMED_DECLARATION)
        }));
    }

    #[test]
    fn rejects_duplicate_const_specialization_decorators() {
        let source = r#"
@const_specialize("fixture.first", "derive")
@const_specialize("fixture.second", "derive")
class Model:
    value: int
"#;
        let parsed = parse_module(source).expect("fixture parses");
        let diagnostics = errors(lower_module(parsed.suite()));
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.code == Some(DiagnosticCode::META_MALFORMED_DECLARATION)
                && diagnostic.message.contains("exactly one @const_specialize")
        }));
    }
}

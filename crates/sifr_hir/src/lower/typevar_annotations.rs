use ruff_text_size::{Ranged, TextRange};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::{Expr, ExprCall};

use super::LowerCtx;

const TYPEVAR_CONSTRAINT_PREFIX: &str = "__constraint__:";

pub(crate) fn encode_typevar_constraint(name: &str) -> String {
    format!("{TYPEVAR_CONSTRAINT_PREFIX}{name}")
}

pub(crate) fn decode_typevar_constraint(encoded: &str) -> Option<&str> {
    encoded.strip_prefix(TYPEVAR_CONSTRAINT_PREFIX)
}

fn invalid_typevar_shape(ctx: &mut LowerCtx, message: impl Into<String>, range: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::TYPE_INVALID_ANNOTATION,
        message.into(),
        range,
    );
}

fn warn_bigint_transition_name(ctx: &mut LowerCtx, name: &str, range: TextRange) {
    if name == "bigint" {
        ctx.warn_bigint_transition_alias(range);
    }
}

/// Parse a `TypeVar` bound/constraint expression from PEP 695 syntax.
/// `T: Bound` is treated as a hard bound; `T: (A, B)` is treated as constraints.
pub(crate) fn parse_typevar_bound_expr(expr: &Expr, ctx: &mut LowerCtx) -> Vec<String> {
    match expr {
        Expr::Name(name) => {
            warn_bigint_transition_name(ctx, name.id.as_str(), name.range());
            vec![name.id.to_string()]
        }
        Expr::Tuple(tuple) => {
            let mut specs = Vec::new();
            for elt in &tuple.elts {
                if let Expr::Name(name) = elt {
                    warn_bigint_transition_name(ctx, name.id.as_str(), name.range());
                    specs.push(encode_typevar_constraint(&name.id));
                } else {
                    invalid_typevar_shape(
                        ctx,
                        "TypeVar constraints must be simple type names",
                        elt.range(),
                    );
                }
            }
            specs
        }
        _ => {
            invalid_typevar_shape(
                ctx,
                "TypeVar bound must be a type name or tuple of type names",
                expr.range(),
            );
            Vec::new()
        }
    }
}

/// Parse `TypeVar(...)` declaration bounds/constraints.
/// Supports:
/// - `TypeVar("T")`
/// - `TypeVar("T", int, str)` (constraints)
/// - `TypeVar("T", bound=Comparable)`
/// - `TypeVar("T", constraints=(int, str))`
pub(crate) fn parse_typevar_declaration_specs(call: &ExprCall, ctx: &mut LowerCtx) -> Vec<String> {
    let mut specs = Vec::new();
    let mut saw_bound = false;
    let mut saw_constraints = false;
    let mut reported_bound_constraints_conflict = false;

    for arg in call.arguments.args.iter().skip(1) {
        saw_constraints = true;
        match arg {
            Expr::Name(name) => {
                warn_bigint_transition_name(ctx, name.id.as_str(), name.range());
                specs.push(encode_typevar_constraint(&name.id));
            }
            _ => invalid_typevar_shape(
                ctx,
                "TypeVar positional constraints must be simple type names",
                arg.range(),
            ),
        }
    }

    for kw in &call.arguments.keywords {
        let Some(arg_name) = &kw.arg else {
            continue;
        };
        match arg_name.as_str() {
            "bound" => {
                if saw_constraints && !reported_bound_constraints_conflict {
                    invalid_typevar_shape(
                        ctx,
                        "TypeVar cannot declare both 'bound' and 'constraints'",
                        arg_name.range(),
                    );
                    reported_bound_constraints_conflict = true;
                }
                saw_bound = true;
                match &kw.value {
                    Expr::Name(name) => {
                        warn_bigint_transition_name(ctx, name.id.as_str(), name.range());
                        specs.push(name.id.to_string());
                    }
                    _ => invalid_typevar_shape(
                        ctx,
                        "TypeVar bound must be a simple type name",
                        kw.value.range(),
                    ),
                }
            }
            "constraints" => {
                if saw_bound && !reported_bound_constraints_conflict {
                    invalid_typevar_shape(
                        ctx,
                        "TypeVar cannot declare both 'bound' and 'constraints'",
                        arg_name.range(),
                    );
                    reported_bound_constraints_conflict = true;
                }
                saw_constraints = true;
                match &kw.value {
                    Expr::Tuple(tuple) => {
                        for elt in &tuple.elts {
                            if let Expr::Name(name) = elt {
                                warn_bigint_transition_name(ctx, name.id.as_str(), name.range());
                                specs.push(encode_typevar_constraint(&name.id));
                            } else {
                                invalid_typevar_shape(
                                    ctx,
                                    "TypeVar constraints must be simple type names",
                                    elt.range(),
                                );
                            }
                        }
                    }
                    Expr::Name(name) => {
                        warn_bigint_transition_name(ctx, name.id.as_str(), name.range());
                        specs.push(encode_typevar_constraint(&name.id));
                    }
                    _ => invalid_typevar_shape(
                        ctx,
                        "TypeVar constraints must be a type name or tuple of type names",
                        kw.value.range(),
                    ),
                }
            }
            _ => {}
        }
    }

    specs
}

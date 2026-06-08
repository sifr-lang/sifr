//! Structured runtime diagnostic intrinsic lowerers.

use crate::{render_expr, RustExpr};

pub(crate) fn lower_runtime_emit_diagnostic(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 4 {
        return None;
    }
    let level = render_expr(&args[0]);
    let target = render_expr(&args[1]);
    let name = render_expr(&args[2]);
    let message = render_expr(&args[3]);

    Some(RustExpr::Ident(format!(
        r#"{{
            let __sifr_diagnostic_level = ({level}).as_str();
            let __sifr_diagnostic_target = ({target}).as_str();
            let __sifr_diagnostic_name = ({name}).as_str();
            let __sifr_diagnostic_message = ({message}).as_str();
            match __sifr_diagnostic_level {{
                "trace" => {{
                    tracing::event!(
                        target: "sifr.runtime",
                        tracing::Level::TRACE,
                        diagnostic_target = __sifr_diagnostic_target,
                        diagnostic_name = __sifr_diagnostic_name,
                        diagnostic_message = __sifr_diagnostic_message
                    );
                    metrics::counter!(
                        "sifr.runtime.diagnostic.emitted",
                        "level" => "trace",
                        "surface" => "runtime"
                    )
                    .increment(1);
                    Ok(())
                }}
                "debug" => {{
                    tracing::event!(
                        target: "sifr.runtime",
                        tracing::Level::DEBUG,
                        diagnostic_target = __sifr_diagnostic_target,
                        diagnostic_name = __sifr_diagnostic_name,
                        diagnostic_message = __sifr_diagnostic_message
                    );
                    metrics::counter!(
                        "sifr.runtime.diagnostic.emitted",
                        "level" => "debug",
                        "surface" => "runtime"
                    )
                    .increment(1);
                    Ok(())
                }}
                "info" => {{
                    tracing::event!(
                        target: "sifr.runtime",
                        tracing::Level::INFO,
                        diagnostic_target = __sifr_diagnostic_target,
                        diagnostic_name = __sifr_diagnostic_name,
                        diagnostic_message = __sifr_diagnostic_message
                    );
                    metrics::counter!(
                        "sifr.runtime.diagnostic.emitted",
                        "level" => "info",
                        "surface" => "runtime"
                    )
                    .increment(1);
                    Ok(())
                }}
                "warn" => {{
                    tracing::event!(
                        target: "sifr.runtime",
                        tracing::Level::WARN,
                        diagnostic_target = __sifr_diagnostic_target,
                        diagnostic_name = __sifr_diagnostic_name,
                        diagnostic_message = __sifr_diagnostic_message
                    );
                    metrics::counter!(
                        "sifr.runtime.diagnostic.emitted",
                        "level" => "warn",
                        "surface" => "runtime"
                    )
                    .increment(1);
                    Ok(())
                }}
                "error" => {{
                    tracing::event!(
                        target: "sifr.runtime",
                        tracing::Level::ERROR,
                        diagnostic_target = __sifr_diagnostic_target,
                        diagnostic_name = __sifr_diagnostic_name,
                        diagnostic_message = __sifr_diagnostic_message
                    );
                    metrics::counter!(
                        "sifr.runtime.diagnostic.emitted",
                        "level" => "error",
                        "surface" => "runtime"
                    )
                    .increment(1);
                    Ok(())
                }}
                _ => {{
                    metrics::counter!(
                        "sifr.runtime.diagnostic.rejected",
                        "reason" => "unsupported_level",
                        "surface" => "runtime"
                    )
                    .increment(1);
                    Err(DiagnosticError::new(format!(
                        "unsupported diagnostic level: {{}}",
                        __sifr_diagnostic_level
                    )))
                }}
            }}
        }}"#,
        level = level,
        target = target,
        name = name,
        message = message,
    )))
}

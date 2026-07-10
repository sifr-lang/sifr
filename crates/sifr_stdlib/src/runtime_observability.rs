/// Emit a structured runtime diagnostic and its bounded observability metric.
///
/// The string-shaped boundary keeps tracing and metrics as private
/// implementation dependencies of `sifr_stdlib` rather than dependencies of
/// generated user projects.
pub fn emit_diagnostic(level: &str, target: &str, name: &str, message: &str) -> Result<(), String> {
    match level {
        "error" => tracing::event!(
            target: "sifr.runtime",
            tracing::Level::ERROR,
            diagnostic_target = target,
            diagnostic_name = name,
            diagnostic_message = message
        ),
        "warn" => tracing::event!(
            target: "sifr.runtime",
            tracing::Level::WARN,
            diagnostic_target = target,
            diagnostic_name = name,
            diagnostic_message = message
        ),
        "info" => tracing::event!(
            target: "sifr.runtime",
            tracing::Level::INFO,
            diagnostic_target = target,
            diagnostic_name = name,
            diagnostic_message = message
        ),
        "debug" => tracing::event!(
            target: "sifr.runtime",
            tracing::Level::DEBUG,
            diagnostic_target = target,
            diagnostic_name = name,
            diagnostic_message = message
        ),
        "trace" => tracing::event!(
            target: "sifr.runtime",
            tracing::Level::TRACE,
            diagnostic_target = target,
            diagnostic_name = name,
            diagnostic_message = message
        ),
        _ => {
            metrics::counter!(
                "sifr.runtime.diagnostic.rejected",
                "reason" => "unsupported_level",
                "surface" => "runtime"
            )
            .increment(1);
            return Err(format!("unsupported diagnostic level: {level}"));
        }
    }

    metrics::counter!(
        "sifr.runtime.diagnostic.emitted",
        "level" => level.to_string(),
        "surface" => "runtime"
    )
    .increment(1);
    Ok(())
}

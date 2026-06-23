pub fn emit_diagnostic(level: tracing::Level, target: &str, name: &str, message: &str) {
    match level {
        tracing::Level::ERROR => tracing::event!(
            target: "sifr.runtime",
            tracing::Level::ERROR,
            diagnostic_target = target,
            diagnostic_name = name,
            diagnostic_message = message
        ),
        tracing::Level::WARN => tracing::event!(
            target: "sifr.runtime",
            tracing::Level::WARN,
            diagnostic_target = target,
            diagnostic_name = name,
            diagnostic_message = message
        ),
        tracing::Level::INFO => tracing::event!(
            target: "sifr.runtime",
            tracing::Level::INFO,
            diagnostic_target = target,
            diagnostic_name = name,
            diagnostic_message = message
        ),
        tracing::Level::DEBUG => tracing::event!(
            target: "sifr.runtime",
            tracing::Level::DEBUG,
            diagnostic_target = target,
            diagnostic_name = name,
            diagnostic_message = message
        ),
        tracing::Level::TRACE => tracing::event!(
            target: "sifr.runtime",
            tracing::Level::TRACE,
            diagnostic_target = target,
            diagnostic_name = name,
            diagnostic_message = message
        ),
    }
}

//! Structured signal intrinsic lowerers.

use crate::RustExpr;

fn boxed_async_signal_block(body: &str) -> RustExpr {
    RustExpr::Ident(format!("Box::pin(async move {{ {body} }})"))
}

pub(crate) fn lower_signal_ctrl_c(args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(boxed_async_signal_block(
        r#"match tokio::signal::ctrl_c().await {
            Ok(()) => Ok(Signal { number: 2_i64, name: "SIGINT".to_string(), supported: true }),
            Err(error) => Err(SignalError::new(format!("failed to wait for SIGINT: {}", error))),
        }"#,
    ))
}

pub(crate) fn lower_signal_sigterm_supported(args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::Ident("cfg!(unix)".to_string()))
}

pub(crate) fn lower_signal_terminate(args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(boxed_async_signal_block(
        r#"#[cfg(unix)]
        {
            let mut __sifr_sigterm = tokio::signal::unix::signal(
                tokio::signal::unix::SignalKind::terminate(),
            )
            .map_err(|error| SignalError::new(format!("failed to install SIGTERM listener: {}", error)))?;
            let _ = __sifr_sigterm.recv().await;
            Ok(Signal { number: 15_i64, name: "SIGTERM".to_string(), supported: true })
        }
        #[cfg(not(unix))]
        {
            Err(SignalError::new("SIGTERM is unsupported on this host".to_string()))
        }"#,
    ))
}

pub(crate) fn lower_signal_shutdown(args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(boxed_async_signal_block(
        r#"#[cfg(unix)]
        {
            let mut __sifr_sigterm = tokio::signal::unix::signal(
                tokio::signal::unix::SignalKind::terminate(),
            )
            .map_err(|error| SignalError::new(format!("failed to install SIGTERM listener: {}", error)))?;
            tokio::select! {
                __sifr_ctrl_c = tokio::signal::ctrl_c() => {
                    match __sifr_ctrl_c {
                        Ok(()) => Ok(Signal { number: 2_i64, name: "SIGINT".to_string(), supported: true }),
                        Err(error) => Err(SignalError::new(format!("failed to wait for SIGINT: {}", error))),
                    }
                }
                _ = __sifr_sigterm.recv() => Ok(Signal { number: 15_i64, name: "SIGTERM".to_string(), supported: true }),
            }
        }
        #[cfg(not(unix))]
        {
            match tokio::signal::ctrl_c().await {
                Ok(()) => Ok(Signal { number: 2_i64, name: "SIGINT".to_string(), supported: true }),
                Err(error) => Err(SignalError::new(format!("failed to wait for SIGINT: {}", error))),
            }
        }"#,
    ))
}

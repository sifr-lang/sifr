use std::{future::Future, pin::Pin};

use sifr_runtime::interop::SifrIntBridge;

type SignalFuture = Pin<Box<dyn Future<Output = Result<SifrIntBridge, String>> + Send>>;

#[must_use]
pub const fn feature_name() -> &'static str {
    "signals"
}

pub fn signal_ctrl_c() -> SignalFuture {
    Box::pin(async move {
        tokio::signal::ctrl_c()
            .await
            .map_err(|error| format!("failed to wait for SIGINT: {error}"))?;
        Ok(SifrIntBridge::from(2_i64))
    })
}

pub fn signal_terminate() -> SignalFuture {
    Box::pin(async move {
        #[cfg(unix)]
        {
            let mut sigterm =
                tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                    .map_err(|error| format!("failed to install SIGTERM listener: {error}"))?;
            let _ = sigterm.recv().await;
            Ok(SifrIntBridge::from(15_i64))
        }
        #[cfg(not(unix))]
        {
            Err("SIGTERM is unsupported on this host".to_string())
        }
    })
}

pub fn signal_shutdown() -> SignalFuture {
    Box::pin(async move {
        #[cfg(unix)]
        {
            let mut sigterm =
                tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                    .map_err(|error| format!("failed to install SIGTERM listener: {error}"))?;
            tokio::select! {
                ctrl_c = tokio::signal::ctrl_c() => {
                    ctrl_c.map_err(|error| format!("failed to wait for SIGINT: {error}"))?;
                    Ok(SifrIntBridge::from(2_i64))
                }
                _ = sigterm.recv() => Ok(SifrIntBridge::from(15_i64)),
            }
        }
        #[cfg(not(unix))]
        {
            tokio::signal::ctrl_c()
                .await
                .map_err(|error| format!("failed to wait for SIGINT: {error}"))?;
            Ok(SifrIntBridge::from(2_i64))
        }
    })
}

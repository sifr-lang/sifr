//! Ingress and publication share one lock: ignored cancellation cannot publish
//! a result after a source/configuration event has entered the server.
use crate::errors::{LspError, LspResult};
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub(crate) struct Generations(Arc<Mutex<u64>>);
impl Generations {
    pub(crate) fn observe(&self, method: Option<&str>) -> LspResult<u64> {
        let mut generation = self
            .0
            .lock()
            .map_err(|_| LspError::internal("generation lock poisoned"))?;
        if matches!(
            method,
            Some(
                "textDocument/didOpen"
                    | "textDocument/didChange"
                    | "textDocument/didSave"
                    | "textDocument/didClose"
                    | "workspace/didChangeConfiguration"
                    | "workspace/didChangeWatchedFiles"
            )
        ) {
            *generation = generation
                .checked_add(1)
                .ok_or_else(|| LspError::internal("generation exhausted"))?;
        }
        Ok(*generation)
    }
    pub(crate) fn publish<T>(
        &self,
        captured: u64,
        operation: impl FnOnce(bool) -> LspResult<T>,
    ) -> LspResult<T> {
        let current = self
            .0
            .lock()
            .map_err(|_| LspError::internal("generation lock poisoned"))?;
        operation(*current == captured)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dx11_stale_publication_is_independent_of_cancellation_and_document_version() {
        let owner = Generations::default();
        let old = owner.observe(Some("textDocument/didOpen")).unwrap();
        for method in [
            "textDocument/didSave",
            "textDocument/didClose",
            "textDocument/didOpen",
            "workspace/didChangeConfiguration",
            "workspace/didChangeWatchedFiles",
        ] {
            let new = owner.observe(Some(method)).unwrap();
            owner
                .publish(old, |current| {
                    assert!(!current);
                    Ok(())
                })
                .unwrap();
            owner
                .publish(new, |current| {
                    assert!(current);
                    Ok(())
                })
                .unwrap();
        }
    }
}

/// A cancellation boundary for one embedded compiler-component operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmbeddedAnalysisCheckpoint {
    BeforeComponentEntry,
    BetweenProviderOperations,
    BeforeResultPublication,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EmbeddedAnalysisCancelled {
    pub checkpoint: EmbeddedAnalysisCheckpoint,
}

/// Run an ordered provider pipeline with deterministic cancellation boundaries.
///
/// The caller supplies the cancellation state because the frontend must not own
/// an LSP transport token. A cancelled pipeline never publishes a partial value.
pub fn run_embedded_provider_operations<T, E>(
    cancelled: impl FnMut(EmbeddedAnalysisCheckpoint) -> bool,
    operations: impl IntoIterator<Item = Box<dyn FnOnce() -> Result<T, E>>>,
) -> Result<Vec<T>, EmbeddedProviderOperationError<E>> {
    run_embedded_provider_items(cancelled, operations, |operation| operation())
}

pub fn run_embedded_provider_items<I, T, E>(
    mut cancelled: impl FnMut(EmbeddedAnalysisCheckpoint) -> bool,
    items: impl IntoIterator<Item = I>,
    mut operation: impl FnMut(I) -> Result<T, E>,
) -> Result<Vec<T>, EmbeddedProviderOperationError<E>> {
    check(
        &mut cancelled,
        EmbeddedAnalysisCheckpoint::BeforeComponentEntry,
    )?;
    let mut values = Vec::new();
    for item in items {
        if !values.is_empty() {
            check(
                &mut cancelled,
                EmbeddedAnalysisCheckpoint::BetweenProviderOperations,
            )?;
        }
        values.push(operation(item).map_err(EmbeddedProviderOperationError::Provider)?);
    }
    check(
        &mut cancelled,
        EmbeddedAnalysisCheckpoint::BeforeResultPublication,
    )?;
    Ok(values)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EmbeddedProviderOperationError<E> {
    Cancelled(EmbeddedAnalysisCancelled),
    Provider(E),
}

fn check<E>(
    cancelled: &mut impl FnMut(EmbeddedAnalysisCheckpoint) -> bool,
    checkpoint: EmbeddedAnalysisCheckpoint,
) -> Result<(), EmbeddedProviderOperationError<E>> {
    if cancelled(checkpoint) {
        return Err(EmbeddedProviderOperationError::Cancelled(
            EmbeddedAnalysisCancelled { checkpoint },
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancellation_is_checked_at_every_provider_boundary() {
        for expected in [
            EmbeddedAnalysisCheckpoint::BeforeComponentEntry,
            EmbeddedAnalysisCheckpoint::BetweenProviderOperations,
            EmbeddedAnalysisCheckpoint::BeforeResultPublication,
        ] {
            let result = run_embedded_provider_operations(
                |checkpoint| checkpoint == expected,
                vec![
                    Box::new(|| Ok::<_, ()>(1)) as Box<dyn FnOnce() -> Result<_, _>>,
                    Box::new(|| Ok::<_, ()>(2)),
                ],
            );
            assert_eq!(
                result,
                Err(EmbeddedProviderOperationError::Cancelled(
                    EmbeddedAnalysisCancelled {
                        checkpoint: expected
                    }
                ))
            );
        }
    }
}

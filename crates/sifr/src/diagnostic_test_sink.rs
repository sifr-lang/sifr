use sifr_diagnostics::RenderedDiagnostic;
use std::cell::RefCell;

thread_local! {
    static CAPTURED: RefCell<Option<Vec<RenderedDiagnostic>>> = const { RefCell::new(None) };
}

pub(super) fn capture<T>(operation: impl FnOnce() -> T) -> (T, Vec<RenderedDiagnostic>) {
    CAPTURED.with(|captured| {
        let previous = captured.replace(Some(Vec::new()));
        assert!(
            previous.is_none(),
            "diagnostic capture must not be nested on one thread"
        );
    });
    let result = operation();
    let diagnostics = CAPTURED.with(|captured| {
        captured
            .replace(None)
            .expect("diagnostic capture should remain active")
    });
    (result, diagnostics)
}

pub(super) fn record(diagnostics: &[RenderedDiagnostic]) {
    CAPTURED.with(|captured| {
        if let Some(output) = captured.borrow_mut().as_mut() {
            output.extend_from_slice(diagnostics);
        }
    });
}

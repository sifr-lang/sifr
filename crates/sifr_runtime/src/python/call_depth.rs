use std::cell::Cell;

thread_local! {
    static PYTHON_CALL_DEPTH: Cell<usize> = const { Cell::new(0) };
}

pub(super) fn python_call_depth() -> usize {
    PYTHON_CALL_DEPTH.with(Cell::get)
}

pub(super) fn enter_python_call() -> PythonCallDepthGuard {
    PYTHON_CALL_DEPTH.with(|depth| depth.set(depth.get().saturating_add(1)));
    PythonCallDepthGuard
}

pub(super) struct PythonCallDepthGuard;

impl Drop for PythonCallDepthGuard {
    fn drop(&mut self) {
        PYTHON_CALL_DEPTH.with(|depth| depth.set(depth.get().saturating_sub(1)));
    }
}

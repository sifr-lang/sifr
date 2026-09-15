//! Scoped observation of the application-selection boundary for driver regressions.
use std::cell::Cell;

thread_local! {
    static SELECTIONS: Cell<Option<usize>> = const { Cell::new(None) };
}

pub(super) fn selected() {
    SELECTIONS.with(|count| {
        if let Some(value) = count.get() {
            count.set(Some(value + 1));
        }
    });
}

#[doc(hidden)]
pub fn observe_stdlib_interop_selection<T>(operation: impl FnOnce() -> T) -> (T, usize) {
    struct Restore(Option<usize>);
    impl Drop for Restore {
        fn drop(&mut self) {
            SELECTIONS.with(|count| count.set(self.0));
        }
    }
    let _restore = Restore(SELECTIONS.with(|count| count.replace(Some(0))));
    let value = operation();
    let count = SELECTIONS.with(|count| count.get().unwrap_or(0));
    (value, count)
}

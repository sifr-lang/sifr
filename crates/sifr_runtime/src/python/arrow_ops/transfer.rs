use super::PythonError;
use super::arrow_error;
use std::collections::HashSet;
use std::sync::{LazyLock, Mutex};

static TRANSFERS: LazyLock<Mutex<HashSet<usize>>> = LazyLock::new(|| Mutex::new(HashSet::new()));

/// Admission covers prepared arguments even after removal from the handle store.
pub(super) struct TransferLease {
    pointers: Vec<usize>,
}

impl TransferLease {
    pub(super) fn acquire(pointers: &[usize]) -> Result<Self, PythonError> {
        let mut active = TRANSFERS
            .lock()
            .map_err(|_| arrow_error("Arrow transfer admission is unavailable"))?;
        if pointers.iter().any(|pointer| active.contains(pointer)) {
            return Err(arrow_error(
                "Arrow payload already has an active owned transfer",
            ));
        }
        active.extend(pointers.iter().copied());
        Ok(Self {
            pointers: pointers.to_vec(),
        })
    }
}

impl Drop for TransferLease {
    fn drop(&mut self) {
        if let Ok(mut active) = TRANSFERS.lock() {
            for pointer in &self.pointers {
                active.remove(pointer);
            }
        }
    }
}

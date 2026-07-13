use super::super::PythonRuntimeError;
use super::state::{CallbackOwnerInner, CallbackOwnerState};
use std::collections::BTreeMap;
use std::sync::{Arc, LazyLock, Mutex, Weak};

static RETAINED_CALLBACK_OWNERS: LazyLock<Mutex<BTreeMap<u64, Weak<CallbackOwnerInner>>>> =
    LazyLock::new(|| Mutex::new(BTreeMap::new()));

pub(super) fn register(owner_id: u64, owner: &Arc<CallbackOwnerInner>) {
    lock_registry().insert(owner_id, Arc::downgrade(owner));
}

pub(super) fn unregister(owner_id: u64) {
    lock_registry().remove(&owner_id);
}

pub(super) fn shutdown_registered_callback_owners() -> Result<(), PythonRuntimeError> {
    let owners = {
        let mut registry = lock_registry();
        let owners = registry
            .iter()
            .filter_map(|(_, owner)| owner.upgrade())
            .map(CallbackOwnerState::from_inner)
            .collect::<Vec<_>>();
        registry.retain(|_, owner| owner.strong_count() > 0);
        owners
    };

    let mut first_error = None;
    for owner in owners {
        if let Err(error) = owner.shutdown_from_runtime() {
            if first_error.is_none() {
                first_error = Some(PythonRuntimeError::AsyncRuntimeFailed(format!(
                    "callback owner shutdown failed: {error}"
                )));
            }
        }
    }
    first_error.map_or(Ok(()), Err)
}

fn lock_registry() -> std::sync::MutexGuard<'static, BTreeMap<u64, Weak<CallbackOwnerInner>>> {
    RETAINED_CALLBACK_OWNERS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[cfg(test)]
pub(super) fn retained_owner_count() -> usize {
    let mut registry = lock_registry();
    registry.retain(|_, owner| owner.strong_count() > 0);
    registry.len()
}

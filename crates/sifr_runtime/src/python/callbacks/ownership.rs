use super::CallbackOwnerState;
use crate::cancellation::{CancellationCarrier, CancellationScopeLease};
use crate::python::{
    context_exit_normal, context_exit_python_error, context_exit_sifr_cause, object_ops,
    semantic_close, ObjectHandle, PythonError, PythonExitDecision, SifrExitCause,
};
use std::fmt;
use std::sync::{Arc, Mutex};

type UnregisterAction = Arc<dyn Fn() -> Result<(), PythonError> + Send + Sync + 'static>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RetainedCallbackCleanup {
    Close,
    Context,
    AsyncClose,
    AsyncContext,
}

pub struct RetainedCallbackGroup {
    owner: CallbackOwnerState,
    unregister: Arc<Mutex<Option<UnregisterAction>>>,
    committed: bool,
}

pub struct CallbackOwnerSlot {
    owner: Mutex<Option<CallbackOwnerState>>,
}

impl fmt::Debug for CallbackOwnerSlot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let owner = self
            .owner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        formatter
            .debug_struct("CallbackOwnerSlot")
            .field(
                "owner_id",
                &owner.as_ref().map(CallbackOwnerState::owner_id),
            )
            .finish_non_exhaustive()
    }
}

impl RetainedCallbackGroup {
    pub fn new() -> Result<Self, PythonError> {
        let unregister = Arc::new(Mutex::new(None::<UnregisterAction>));
        let deferred = Arc::clone(&unregister);
        let owner = CallbackOwnerState::new_retained(move || {
            let action = deferred
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone();
            action.map_or(Ok(()), |action| action())
        })?;
        Ok(Self {
            owner,
            unregister,
            committed: false,
        })
    }

    #[must_use]
    pub fn owner(&self) -> &CallbackOwnerState {
        &self.owner
    }

    pub fn commit_for_object(
        &mut self,
        object: &ObjectHandle,
        cleanup: RetainedCallbackCleanup,
    ) -> Result<CallbackOwnerState, PythonError> {
        let action = unregister_action(object.clone(), cleanup);
        let mut unregister = self
            .unregister
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if unregister.is_some() {
            return Err(super::errors::unavailable(
                "retained callback owner already committed",
            ));
        }
        *unregister = Some(action);
        self.committed = true;
        Ok(self.owner.clone())
    }

    pub async fn rollback_async(&mut self) -> Result<(), PythonError> {
        if self.committed {
            return Ok(());
        }
        let unregister = self.owner.begin_owner_unregister()?;
        drop(unregister);
        let outcome = self
            .owner
            .close_after_owner_unregister_with_typed_observer_async()
            .await;
        self.committed = true;
        outcome
    }
}

impl Drop for RetainedCallbackGroup {
    fn drop(&mut self) {
        if self.committed {
            return;
        }
        if let Ok(guard) = self.owner.begin_owner_unregister() {
            drop(guard);
        }
        if let Ok(runtime) = tokio::runtime::Handle::try_current() {
            let owner = self.owner.clone();
            std::mem::drop(runtime.spawn(async move {
                let _ignored = owner.close_after_owner_unregister_async().await;
            }));
        } else {
            let _ignored = self.owner.close_after_owner_unregister();
        }
    }
}

pub async fn rollback_retained_callbacks_on_error<T>(
    outcome: Result<T, PythonError>,
    group: &mut RetainedCallbackGroup,
) -> Result<T, PythonError> {
    let Err(mut primary) = outcome else {
        return outcome;
    };
    if let Err(secondary) = group.rollback_async().await {
        let evidence = format!("secondary retained callback rollback failure: {secondary}");
        if primary.context.is_empty() {
            primary.context = evidence;
        } else {
            primary.context.push_str("; ");
            primary.context.push_str(&evidence);
        }
    }
    Err(primary)
}

pub async fn finalize_retained_callbacks<T, E>(
    outcome: Result<T, E>,
    group: &mut RetainedCallbackGroup,
) -> Result<T, E> {
    if outcome.is_err() {
        if let Err(secondary) = group.rollback_async().await {
            super::super::record_context_cleanup_evidence(
                "retained-callback-finalization",
                &secondary,
            );
        }
    }
    outcome
}

pub fn retained_callback_finalization_scope(
    parent: Option<&CancellationCarrier>,
) -> Result<Option<CancellationScopeLease>, PythonError> {
    parent
        .map(CancellationScopeLease::claim)
        .transpose()
        .map_err(|error| {
            PythonError::without_replay(
                "runtime",
                "SifrPythonAsyncCallbackError",
                format!("retained callback cancellation scope failed: {error:?}"),
                String::new(),
                "retained callback finalization",
            )
        })
}

pub async fn finish_retained_callback_finalization<T>(
    outcome: T,
    scope: Option<CancellationScopeLease>,
) -> T {
    let Some(scope) = scope else {
        return outcome;
    };
    if !scope.notification().is_notified() {
        return outcome;
    }
    let _resume = scope.release_and_resume_parent();
    tokio::task::yield_now().await;
    outcome
}

impl CallbackOwnerSlot {
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            owner: Mutex::new(None),
        }
    }

    #[must_use]
    pub fn from_owner(owner: CallbackOwnerState) -> Self {
        Self {
            owner: Mutex::new(Some(owner)),
        }
    }

    pub fn owner_or_insert(
        &self,
        object: &ObjectHandle,
        cleanup: RetainedCallbackCleanup,
    ) -> Result<CallbackOwnerState, PythonError> {
        let mut slot = self
            .owner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(owner) = slot.as_ref() {
            return Ok(owner.clone());
        }
        let action = unregister_action(object.clone(), cleanup);
        let owner = CallbackOwnerState::new_retained(move || action())?;
        *slot = Some(owner.clone());
        Ok(owner)
    }

    #[must_use]
    pub fn owner(&self) -> Option<CallbackOwnerState> {
        self.owner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    pub(crate) fn take(self) -> Option<CallbackOwnerState> {
        self.owner
            .into_inner()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

pub fn semantic_close_with_callbacks(
    object: ObjectHandle,
    method: impl AsRef<str>,
    callbacks: CallbackOwnerSlot,
) -> Result<(), PythonError> {
    let Some(owner) = callbacks.take() else {
        return semantic_close(object, method);
    };
    let unregister = owner.begin_owner_unregister()?;
    let primary = semantic_close(object, method);
    drop(unregister);
    let callback_close = owner.close_after_owner_unregister_with_typed_observer();
    match primary {
        Err(primary) => finish_primary_with_callback_close(primary, callback_close, &owner),
        Ok(()) => callback_close,
    }
}

pub fn context_exit_normal_with_callbacks(
    object: ObjectHandle,
    callbacks: CallbackOwnerSlot,
) -> Result<PythonExitDecision, PythonError> {
    finish_context_callbacks(callbacks, true, || context_exit_normal(object))
}

pub fn context_exit_python_error_with_callbacks(
    object: ObjectHandle,
    error: &PythonError,
    callbacks: CallbackOwnerSlot,
) -> Result<PythonExitDecision, PythonError> {
    finish_context_callbacks(callbacks, false, || {
        context_exit_python_error(object, error)
    })
}

pub fn context_exit_sifr_cause_with_callbacks(
    object: ObjectHandle,
    cause: &SifrExitCause,
    callbacks: CallbackOwnerSlot,
) -> Result<PythonExitDecision, PythonError> {
    finish_context_callbacks(callbacks, false, || context_exit_sifr_cause(object, cause))
}

fn finish_context_callbacks(
    callbacks: CallbackOwnerSlot,
    typed_observer: bool,
    exit: impl FnOnce() -> Result<PythonExitDecision, PythonError>,
) -> Result<PythonExitDecision, PythonError> {
    let Some(owner) = callbacks.take() else {
        return exit();
    };
    let unregister = owner.begin_owner_unregister()?;
    let primary = exit();
    drop(unregister);
    let callback_close = if typed_observer {
        owner.close_after_owner_unregister_with_typed_observer()
    } else {
        owner.close_after_owner_unregister()
    };
    match primary {
        Err(primary) => finish_primary_with_callback_close(primary, callback_close, &owner),
        Ok(decision) => callback_close.map(|()| decision),
    }
}

fn finish_primary_with_callback_close<T>(
    primary: PythonError,
    callback_close: Result<(), PythonError>,
    owner: &CallbackOwnerState,
) -> Result<T, PythonError> {
    let mut primary = primary;
    super::execution::attach_callback_failure_evidence_to_error(&mut primary, &[owner]);
    if let Err(secondary) = callback_close {
        super::super::attach_secondary_python_error(&mut primary, &secondary);
    }
    Err(primary)
}

fn unregister_action(object: ObjectHandle, cleanup: RetainedCallbackCleanup) -> UnregisterAction {
    Arc::new(move || match cleanup {
        RetainedCallbackCleanup::Close => semantic_close(object.clone(), "close"),
        RetainedCallbackCleanup::Context => context_exit_normal(object.clone()).map(|_decision| ()),
        RetainedCallbackCleanup::AsyncClose => {
            let request = super::super::PythonAsyncRequest::semantic_close_method(
                object.clone(),
                "aclose".to_string(),
            )?;
            super::super::async_declaration::submit_async_declaration_cleanup_blocking(request)
                .map(|_value| ())
        }
        RetainedCallbackCleanup::AsyncContext => {
            let request =
                super::super::async_value::PythonAsyncRequest::semantic_context_exit_method(
                    object.clone(),
                    super::super::PythonAsyncExitCause::Normal,
                )?;
            super::super::async_declaration::submit_async_context_cleanup_blocking(request)
                .map(|_decision| ())
        }
    })
}

pub(super) fn release_callable(object: ObjectHandle) {
    let _ignored = object_ops::close_object(object);
}

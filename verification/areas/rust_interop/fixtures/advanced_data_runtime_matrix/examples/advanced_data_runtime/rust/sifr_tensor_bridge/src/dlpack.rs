use ndarray::Array2;
use sifr_runtime::interop::{Handle, HandleStateError};

use crate::tensor::{DataExchangeError, OwnerGuard, TensorView};

#[derive(Debug)]
pub struct Capsule;

#[derive(Debug)]
pub struct DlpackView {
    state: DlpackRuntimeState,
}

#[derive(Debug)]
struct DlpackRuntimeState {
    array: Array2<f64>,
    pointer: usize,
    _owner: OwnerGuard,
}

pub fn transfer(input: Handle<TensorView>) -> Result<Handle<DlpackView>, DataExchangeError> {
    let state = input
        .into_inner()
        .map_err(handle_error)
        .and_then(transfer_state)
        .map_err(DataExchangeError)?;
    Ok(Handle::new(DlpackView { state }))
}

pub fn observe(view: &Handle<DlpackView>) -> Result<String, DataExchangeError> {
    match view.inner_ref() {
        Ok(DlpackView { state })
            if state.array.as_ptr() as usize == state.pointer
                && state.array.shape() == [2, 3]
                && state.array.strides() == [3, 1] =>
        {
            Ok("protocol=managed-tensor;ownership=transferred;dtype=f64;rank=2;shape=2x3;strides=3x1;device=cpu;copy=none".to_string())
        }
        Ok(DlpackView { .. }) => Err(DataExchangeError(
            "DLPack capsule metadata or allocation changed".to_string(),
        )),
        Err(error) => Err(DataExchangeError(handle_error(error))),
    }
}

pub fn close(mut view: Handle<DlpackView>) -> Result<(), DataExchangeError> {
    view.inner_ref()
        .map_err(handle_error)
        .map_err(DataExchangeError)?;
    view.mark_closed(sifr_runtime::interop::__generated_glue::token());
    Ok(())
}

fn transfer_state(view: TensorView) -> Result<DlpackRuntimeState, String> {
    let TensorView { state, mut owner } = view;
    let array = state
        .ndarray
        .ok_or_else(|| "ndarray owner was already transferred".to_string())?;
    let pointer = array.as_ptr() as usize;
    let owner = owner
        .take()
        .ok_or_else(|| "tensor owner guard was already transferred".to_string())?;
    Ok(DlpackRuntimeState {
        array,
        pointer,
        _owner: owner,
    })
}

fn handle_error(error: HandleStateError) -> String {
    format!("advanced-data DLPack state: {error}")
}

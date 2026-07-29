use std::sync::atomic::{AtomicUsize, Ordering};

use candle::{CpuStorage, DType, Device, Storage, Tensor};
use ndarray::Array2;
use sifr_runtime::interop::{Handle, HandleStateError};

static ACTIVE_OWNERS: AtomicUsize = AtomicUsize::new(0);
static RELEASED_OWNERS: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
pub struct DataExchangeError(pub(crate) String);

impl std::fmt::Display for DataExchangeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for DataExchangeError {}

#[derive(Debug)]
pub struct TensorView {
    pub(crate) state: TensorRuntimeState,
    pub(crate) owner: Option<OwnerGuard>,
}

#[derive(Debug)]
pub(crate) struct TensorRuntimeState {
    pub(crate) ndarray: Option<Array2<f64>>,
    candle: Tensor,
    ndarray_pointer: usize,
    candle_pointer: usize,
}

#[derive(Debug)]
pub(crate) struct OwnerGuard;

impl OwnerGuard {
    fn new() -> Self {
        ACTIVE_OWNERS.fetch_add(1, Ordering::SeqCst);
        Self
    }
}

impl Drop for OwnerGuard {
    fn drop(&mut self) {
        ACTIVE_OWNERS.fetch_sub(1, Ordering::SeqCst);
        RELEASED_OWNERS.fetch_add(1, Ordering::SeqCst);
    }
}

pub fn create(
    ndarray_values: Vec<f64>,
    candle_values: Vec<f64>,
) -> Result<Handle<TensorView>, DataExchangeError> {
    RELEASED_OWNERS.store(0, Ordering::SeqCst);
    let ndarray_pointer = ndarray_values.as_ptr() as usize;
    let candle_pointer = candle_values.as_ptr() as usize;
    let state = build_state(
        ndarray_values,
        candle_values,
        ndarray_pointer,
        candle_pointer,
    )
    .map_err(DataExchangeError)?;
    Ok(Handle::new(TensorView {
        state,
        owner: Some(OwnerGuard::new()),
    }))
}

pub fn observe(view: &Handle<TensorView>) -> Result<String, DataExchangeError> {
    match view.inner_ref() {
        Ok(view) => observe_state(&view.state).map_err(DataExchangeError),
        Err(error) => Err(DataExchangeError(handle_error(error))),
    }
}

pub fn close(mut view: Handle<TensorView>) -> Result<(), DataExchangeError> {
    view.inner_ref()
        .map_err(handle_error)
        .map_err(DataExchangeError)?;
    view.mark_closed(sifr_runtime::interop::__generated_glue::token());
    Ok(())
}

pub fn release_observation() -> String {
    let active = ACTIVE_OWNERS.load(Ordering::SeqCst);
    let released = RELEASED_OWNERS.load(Ordering::SeqCst);
    format!("tensor-released={released};active={active}")
}

fn build_state(
    ndarray_values: Vec<f64>,
    candle_values: Vec<f64>,
    ndarray_pointer: usize,
    candle_pointer: usize,
) -> Result<TensorRuntimeState, String> {
    let ndarray =
        Array2::from_shape_vec((2, 3), ndarray_values).map_err(|error| error.to_string())?;
    if ndarray.as_ptr() as usize != ndarray_pointer {
        return Err("ndarray copied the owned input allocation".to_string());
    }
    let candle =
        Tensor::from_vec(candle_values, (2, 3), &Device::Cpu).map_err(|error| error.to_string())?;
    let observed_candle_pointer = candle_pointer_from_storage(&candle)?;
    if observed_candle_pointer != candle_pointer {
        return Err("Candle copied the owned CPU input allocation".to_string());
    }

    Ok(TensorRuntimeState {
        ndarray: Some(ndarray),
        candle,
        ndarray_pointer,
        candle_pointer,
    })
}

fn observe_state(state: &TensorRuntimeState) -> Result<String, String> {
    let Some(array) = state.ndarray.as_ref() else {
        return Err("ndarray owner already transferred".to_string());
    };
    if array.as_ptr() as usize != state.ndarray_pointer
        || array.shape() != [2, 3]
        || array.strides() != [3, 1]
        || state.candle.dims() != [2, 3]
        || state.candle.dtype() != DType::F64
        || !state.candle.device().is_cpu()
    {
        return Err("tensor metadata identity mismatch".to_string());
    }
    match candle_pointer_from_storage(&state.candle) {
        Ok(pointer) if pointer == state.candle_pointer => Ok(
            "dtype=f64;rank=2;shape=2x3;layout=c;strides=3x1;device=cpu;ndarray-copy=none;candle-copy=none".to_string(),
        ),
        Ok(_) => Err("Candle storage allocation changed".to_string()),
        Err(error) => Err(error),
    }
}

fn candle_pointer_from_storage(tensor: &Tensor) -> Result<usize, String> {
    let (storage, _layout) = tensor.storage_and_layout();
    match &*storage {
        Storage::Cpu(CpuStorage::F64(values)) => Ok(values.as_ptr() as usize),
        _ => Err("Candle tensor lost CPU f64 storage identity".to_string()),
    }
}

fn handle_error(error: HandleStateError) -> String {
    format!("advanced-data tensor view state: {error}")
}

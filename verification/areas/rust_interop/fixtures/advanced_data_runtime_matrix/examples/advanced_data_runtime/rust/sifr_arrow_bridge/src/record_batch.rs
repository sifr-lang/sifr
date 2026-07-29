use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use arrow::array::{Array, ArrayRef, Float64Array};
use arrow::datatypes::{DataType as ArrowDataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use datafusion::datasource::MemTable;
use datafusion::execution::context::SessionContext;
use polars::prelude::{DataFrame, DataType as PolarsDataType, IntoColumn, NamedFrom, Series};
use sifr_runtime::interop::{Handle, HandleStateError};

static ACTIVE_VIEWS: AtomicUsize = AtomicUsize::new(0);
static RELEASED_VIEWS: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
pub struct DataExchangeError(String);

impl std::fmt::Display for DataExchangeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for DataExchangeError {}

#[derive(Debug)]
pub struct RecordBatchView {
    state: ArrowRuntimeState,
}

struct ArrowRuntimeState {
    record_batch: RecordBatch,
    datafusion: SessionContext,
    polars: DataFrame,
    input_pointer: usize,
}

impl std::fmt::Debug for ArrowRuntimeState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ArrowRuntimeState")
            .field("record_batch", &self.record_batch)
            .field("polars", &self.polars)
            .field("input_pointer", &self.input_pointer)
            .finish_non_exhaustive()
    }
}

impl Drop for RecordBatchView {
    fn drop(&mut self) {
        ACTIVE_VIEWS.fetch_sub(1, Ordering::SeqCst);
        RELEASED_VIEWS.fetch_add(1, Ordering::SeqCst);
    }
}

pub fn create(input: Vec<f64>) -> Result<Handle<RecordBatchView>, DataExchangeError> {
    RELEASED_VIEWS.store(0, Ordering::SeqCst);
    let input_pointer = input.as_ptr() as usize;
    let state = build_state(input, input_pointer).map_err(DataExchangeError)?;
    ACTIVE_VIEWS.fetch_add(1, Ordering::SeqCst);
    Ok(Handle::new(RecordBatchView { state }))
}

pub fn observe(view: &Handle<RecordBatchView>) -> Result<String, DataExchangeError> {
    match view.inner_ref() {
        Ok(view) => observe_state(&view.state).map_err(DataExchangeError),
        Err(error) => Err(DataExchangeError(handle_error(error))),
    }
}

pub fn close(mut view: Handle<RecordBatchView>) -> Result<(), DataExchangeError> {
    view.inner_ref()
        .map_err(handle_error)
        .map_err(DataExchangeError)?;
    view.mark_closed(sifr_runtime::interop::__generated_glue::token());
    Ok(())
}

pub fn release_observation() -> String {
    let active = ACTIVE_VIEWS.load(Ordering::SeqCst);
    let released = RELEASED_VIEWS.load(Ordering::SeqCst);
    format!("arrow-released={released};active={active}")
}

fn build_state(input: Vec<f64>, input_pointer: usize) -> Result<ArrowRuntimeState, String> {
    let array = Float64Array::from(input);
    if array.values().as_ptr() as usize != input_pointer {
        return Err("Arrow copied the owned input allocation".to_string());
    }
    let polars_values = array.values().to_vec();

    let schema = Arc::new(Schema::new(vec![Field::new(
        "value",
        ArrowDataType::Float64,
        false,
    )]));
    let values: ArrayRef = Arc::new(array);
    let record_batch =
        RecordBatch::try_new(Arc::clone(&schema), vec![values]).map_err(display_error)?;
    let table = MemTable::try_new(Arc::clone(&schema), vec![vec![record_batch.clone()]])
        .map_err(display_error)?;
    let datafusion = SessionContext::new();
    datafusion
        .register_table("input", Arc::new(table))
        .map_err(display_error)?;

    let series = Series::new("value".into(), polars_values);
    let polars = DataFrame::new(record_batch.num_rows(), vec![series.into_column()])
        .map_err(display_error)?;

    Ok(ArrowRuntimeState {
        record_batch,
        datafusion,
        polars,
        input_pointer,
    })
}

fn observe_state(state: &ArrowRuntimeState) -> Result<String, String> {
    let Some(column) = state.record_batch.columns().first() else {
        return Err("Arrow record batch lost its column".to_string());
    };
    let Some(array) = column.as_any().downcast_ref::<Float64Array>() else {
        return Err("Arrow column lost Float64 identity".to_string());
    };
    if array.values().as_ptr() as usize != state.input_pointer {
        return Err("Arrow record batch changed the input allocation".to_string());
    }
    let schema = state.record_batch.schema();
    let Some(arrow_field) = schema.fields().first() else {
        return Err("Arrow record batch lost its schema field".to_string());
    };
    let Some(polars_column) = state.polars.columns().first() else {
        return Err("Polars dataframe lost its column".to_string());
    };
    let datafusion_registered = state.datafusion.table_exist("input").unwrap_or(false);
    if arrow_field.name() != "value"
        || arrow_field.data_type() != &ArrowDataType::Float64
        || polars_column.name().as_str() != "value"
        || polars_column.dtype() != &PolarsDataType::Float64
        || state.polars.height() != state.record_batch.num_rows()
        || !datafusion_registered
    {
        return Err("crate-backed schema identity mismatch".to_string());
    }
    Ok(format!(
        "schema=value:float64;rows={};datafusion=registered;polars=value:float64x{};polars-copy=explicit;copy=input->arrow:none",
        state.record_batch.num_rows(),
        state.polars.height()
    ))
}

fn display_error(error: impl std::fmt::Display) -> String {
    error.to_string()
}

fn handle_error(error: HandleStateError) -> String {
    format!("advanced-data Arrow view state: {error}")
}

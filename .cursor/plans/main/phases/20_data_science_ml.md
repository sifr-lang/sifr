# Data Science and Machine Learning

**Why now:** With the language feature-complete, async runtime available, and typed serialization in place, Sifr can target the data science and ML ecosystem — one of Python's strongest domains. Data processing (Polars DataFrames) and ML inference are independent of the web stack and developer tools, so this phase can proceed based on its actual dependencies (typed serde + async runtime). Keeping this as a dedicated phase recognizes that data science and ML are a distinct use case from web development.

---

## milestone_data_processing: Data Processing

status: pending

**Goal:** Enable data science and data engineering workflows with a Pythonic API over Polars.

**Depends on:** milestone_typed_serde_core (typed serialization for CSV/Parquet type mapping), milestone_async_core (async runtime for potential lazy evaluation)

### Work Items

- `sifr.data` (wraps `polars`) — DataFrame library with lazy evaluation, expressions, CSV/Parquet I/O

### Definition of Done (milestone_data_processing)

- `sifr.data.DataFrame` wraps polars with Pythonic API
- Lazy evaluation chain (filter, group_by, agg, sort) compiles correctly
- CSV/Parquet read/write works end-to-end
- All existing E2E tests still pass (no regressions)
- `cargo test` passes, `cargo clippy -- -D warnings` passes, no new `unsafe` without justification
- E2E pass tests: dataframe_basic, csv_roundtrip, parquet_roundtrip, dataframe_filter, dataframe_groupby
- Milestone demo in `./demos/milestone_data_processing_demo.sifr`

---

## milestone_ml_inference: Machine Learning and Inference

status: pending

**Goal:** Enable ML model inference and LLM integration in Sifr, making it viable for AI-powered applications.

**Depends on:** milestone_data_processing (data processing primitives should be available for feature engineering and data pipelines)

### Work Items

- ML inference runtime: wraps a Rust ML inference library for running trained models
- Tensor/array primitives: basic N-dimensional array support for model inputs/outputs
- LLM client integration: typed API for calling LLM inference endpoints (local and remote)
- Model loading: load serialized models from disk with typed configuration

### Definition of Done (milestone_ml_inference)

- ML model inference works end-to-end (load model, prepare input, run inference, get typed output)
- Basic tensor/array operations compile correctly
- LLM client can call inference endpoints with typed request/response
- All existing E2E tests still pass (no regressions)
- `cargo test` passes, `cargo clippy -- -D warnings` passes, no new `unsafe` without justification
- E2E pass tests: model_inference_basic, tensor_operations, llm_client_basic
- Milestone demo in `./demos/milestone_ml_inference_demo.sifr`

---

## Milestone Ordering

- **milestone_data_processing first:** DataFrame and CSV/Parquet I/O are the foundation for data workflows. Depends on typed serde and async runtime from Phase 14.
- **milestone_ml_inference second:** ML inference builds on data processing primitives for feature engineering and data pipelines.

# Advanced Data Matrix Fixture

This fixture records contract-only passing coverage for advanced data metadata
across Arrow/dataframe and tensor/DLPack bridge categories. The focused unit coverage
checks schema identity, shared bridge crate boundaries, tensor dtype/rank/layout
metadata, tensor shape/strides metadata, CPU-only device metadata, and invalid
dtype/shape metadata rejection.

Runtime-observed certification for `datafusion`, `polars`, `ndarray`, and
`candle` remains staged for ecosystem certification.

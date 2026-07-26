# Advanced Data Runtime Matrix Fixture

This future-owned fixture reserves runtime-observed certification for Arrow,
DataFusion, Polars, ndarray, and CPU-only Candle. The positive direction will
observe schema, dtype, rank, shape, layout, stride, device, DLPack ownership,
cleanup, and no-copy behavior. The negative direction will observe deterministic
schema, shape, and device mismatch rejection.

The existing `arrow_record_batch`, `tensor_dlpack_bridge`, and
`advanced_data_matrix` fixtures remain contract-only and do not provide this
runtime evidence.

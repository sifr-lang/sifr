# Advanced Data Runtime Matrix Fixture

This runtime-observed fixture certifies generated package exchange through the
exact root-lock versions of Arrow, DataFusion, Polars, ndarray, and CPU-only
Candle. The positive direction observes Arrow/DataFusion/Polars schema
identity, DataFusion 55 NaN-fill planning through its borrowed API, ndarray and
Polars 0.55 dataframe-level sortedness, Candle and ndarray
dtype/rank/shape/layout/stride/device identity,
allocation-preserving owned inputs, one-shot DLPack-style transfer, and
deterministic cleanup before and after consuming close. The Polars dataframe
is derived from the crossed Arrow values through an explicit copy; no
Arrow-to-Polars zero-copy claim is made. The managed-capsule bridge models
DLPack ownership and metadata without exposing the unsafe C ABI. The negative
direction observes compiler rejection of schema, shape/rank, and device
mismatches before Cargo probing.

The existing `arrow_record_batch`, `tensor_dlpack_bridge`, and
`advanced_data_matrix` fixtures remain contract-only and do not provide this
runtime evidence. The generated scenario's shared bridges use only safe Rust
and its manifest grants only the seven-entry native-link envelope covering the
locked default-feature graph on the supported arm64 and x86_64 validation
hosts. DataFusion catalog errors remain typed failures; the bridge does not
convert them to a missing-table value.

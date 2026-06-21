# Tensor and DLPack Fixture

This fixture records contract-only passing coverage for tensor and DLPack view
metadata. The driver validates `dtype=`, `shape=`, `rank=`,
`layout=`, `strides=`, `device=`, and `ownership=` for tensor views, requires
CPU-only device metadata for this verification surface, and rejects DLPack handoff unless
`ownership=transfer`, an owned owner parameter, and `protocol=` are explicit.

Runtime-observed `ndarray` and DLPack crate exchange is pending for the
ecosystem certification fixture pass.

# Zero-Copy Runtime Matrix Fixture

This runtime-observed fixture certifies `bytes`, `memmap2`, `bytemuck`, and
`zerocopy` through a generated package bridge.

The positive direction moves the Sifr byte owner into `bytes::Bytes` without
changing its allocation, retains a sliced alias after the original Rust owner
binding is dropped, seals a mutable anonymous `memmap2` mapping without
changing its address, and observes pointer-identical `bytemuck` and `zerocopy`
views. The bridge type is checked against its declared `Send + Sync`
obligations, mutation occurs only before immutable aliases are published, and
consuming close records deterministic release with zero active views.

The negative direction rejects a mutable view from a shared owner, a returned
call-lifetime borrow, and an owner-lifetime view crossing async suspension with
`SIFR-RUST-ZC-0001` before Cargo probing. A separate mandatory integration
mutation makes the bridge type non-`Send` and non-`Sync` to prove those
obligations are enforced by the direct type probe.

The existing `zero_copy_bytes` and `zero_copy_view_matrix` fixtures remain
contract-only; their support claims are intentionally narrower than this row.

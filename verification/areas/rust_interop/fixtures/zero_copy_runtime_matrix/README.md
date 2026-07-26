# Zero-Copy Runtime Matrix Fixture

This future-owned fixture reserves runtime-observed certification for
`bytes`, `memmap2`, `bytemuck`, and `zerocopy`. The positive direction will
observe alias-preserving views, owner lifetime, mutation exclusivity,
Send/Sync obligations, release, and async-suspension rejection. The negative
direction will prove that borrow escape and invalid mutability cannot bypass
the generated runtime boundary.

The existing `zero_copy_bytes` and `zero_copy_view_matrix` fixtures remain
contract-only and do not provide this runtime evidence.

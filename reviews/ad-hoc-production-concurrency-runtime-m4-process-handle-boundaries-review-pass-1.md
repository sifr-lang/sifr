PASS

Verified at `crates/sifr_lowering/src/lower/task_scope_calls.rs:436-444`: `process_owned_handle_type_label_by_name` returns `Some(...)` for exactly `Child`, `AsyncChild`, `PipeReader`, `PipeWriter`, `AsyncPipeReader`, `AsyncPipeWriter`, and `None` for everything else (so `Command`/`Status`/`Output`/`TextOutput`/`Stdio` correctly fall through). It's invoked from both `non_send_reason_inner` (task-boundary + channel-send paths) and `non_share_safe_reason_inner` (Shared constructor), so all three boundary checks share the same allow-list - no skew between rules.

Fail fixtures match the matrix as claimed:
- `process_child_handle_task_boundary_rejected.sifr` - `Child` via `scope.spawn` -> SIFR-OWN-0010
- `process_handle_task_boundary_rejected.sifr` - `AsyncChild` via `scope.spawn` -> SIFR-OWN-0010
- `process_handle_channel_send_rejected.sifr` - `PipeReader` via `ChannelSender.send` -> SIFR-OWN-0011
- `process_handle_shared_rejected.sifr` - `AsyncPipeReader` via `Shared(...)` -> SIFR-OWN-0012

Coverage hits both handle categories (child + pipe), both sync/async variants, and all three boundary entry points. Blocker-correct.

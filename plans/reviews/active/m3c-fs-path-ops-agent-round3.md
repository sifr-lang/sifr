Pass 3 review: verified behavior parity for all migrated fs ops (mkdir/touch/glob/rglob/walk_dir/listdir/iterdir/wildcard_match) against retired codegen intrinsics. Pass-2 finding is fixed — `mkdir` now in `RETIRED_INTRINSICS`. `api_behavior.rs` covers the full new surface. No stale issues, no new blockers.

READY

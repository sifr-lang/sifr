# Arrow Record Batch Fixture

This fixture records contract-only passing coverage for advanced data views.
The driver validates `@rust.view(..., data=arrow_record_batch, ...)`
metadata, requires explicit schema identity through `schema=`, requires explicit
borrowed or owned view ownership, and enforces the `sifr_arrow_bridge` shared
bridge crate boundary.

Runtime-observed `arrow` crate record batch exchange remains staged for the
ecosystem certification fixture pass.

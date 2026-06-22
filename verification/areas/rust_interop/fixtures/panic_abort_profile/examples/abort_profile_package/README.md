# fixture: panic_abort_profile
# scenario-example: abort_profile_package

This scenario models a package that explicitly opts a legacy Rust backend into
process-aborting panic semantics. The positive evidence permits
`@rust(..., panic=abort)` only because `[trust].rust-panic-abort` names the
target; the paired negative evidence omits that trust entry and must produce
`SIFR-RUST-PANIC-0001`.

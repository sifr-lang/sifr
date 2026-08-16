# Method slot runtime example

# fixture: method_slot_table
# scenario-example: method_slot_runtime

This package proves compiler-emitted method-slot tables with mutable, shared,
and runtime-owned no-context dispatch. The Rust bridge invokes slots by their
stable table index and uses only structural arena input and output.

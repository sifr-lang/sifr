# Static program and arena bridge

This fixture proves compiler-emitted static program data and a checked structural arena.

The positive case uses one generic, monomorphized Rust call. The call reads the sealed program
bytes and the compiler-emitted typed value. It consumes a validated document, constructs a typed
Sifr record, and projects the output.
The arena runtime test separately proves that a 30-digit exact integer moves into `SifrInt`
without narrowing.

The negative case changes the expected program envelope. The bridge returns a typed error before it
processes the arena. Runtime unit tests reject invalid indices, scalar kinds, and cycles.

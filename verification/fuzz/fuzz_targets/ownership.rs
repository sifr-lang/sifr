#![no_main]

use libfuzzer_sys::fuzz_target;
use sifr_fuzz::ownership_program;

fuzz_target!(|data: &[u8]| {
    let source = ownership_program(data);
    let _ = sifr_driver::lower_source(&source);
});

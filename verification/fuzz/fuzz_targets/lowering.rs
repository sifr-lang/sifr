#![no_main]

use libfuzzer_sys::fuzz_target;
use sifr_fuzz::bounded_text;

fuzz_target!(|data: &[u8]| {
    let source = bounded_text(data, 16 * 1024);
    let _ = sifr_driver::lower_source(&source);
});

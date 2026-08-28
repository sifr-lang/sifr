#![no_main]

use libfuzzer_sys::fuzz_target;
use sifr_fuzz::bounded_text;

fuzz_target!(|data: &[u8]| {
    let source = bounded_text(data, 16 * 1024);
    let _ = sifr_syntax::parse_module(&source, Some("fuzz/parser.sifr"));
});

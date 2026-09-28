#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let source = String::from_utf8_lossy(&data[..data.len().min(16 * 1024)]);
    let _ = sifr_syntax::parse_module(&source, Some("fuzz/parser.sifr"));
});

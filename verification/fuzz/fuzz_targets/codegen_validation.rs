#![no_main]

use libfuzzer_sys::fuzz_target;
use sifr_driver::CompileResult;
use sifr_fuzz::valid_program;

fuzz_target!(|data: &[u8]| {
    let source = valid_program(data);
    if let CompileResult::Success { rust_source } = sifr_driver::compile(&source) {
        let _ = sifr_codegen::validate_generated_rust_source(&rust_source);
    }
});

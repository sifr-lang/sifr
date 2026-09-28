#![no_main]

use libfuzzer_sys::fuzz_target;
use sifr_frontend::guided_targets::{GuidedTarget, replay};

fuzz_target!(|data: &[u8]| {
    replay(GuidedTarget::Ownership, data);
});

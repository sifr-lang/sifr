//! Cache-key bridge from early class adaptation to static specialization.

use sifr_lowering::LoweringResult;
use std::fmt::Write;

pub(crate) fn post_adapter_hex(result: &LoweringResult, owner: &str) -> String {
    result
        .class_adapter_selections
        .iter()
        .find(|selection| selection.owner == owner)
        .map_or_else(String::new, |selection| {
            selection.post_adapter_identity.iter().fold(
                String::with_capacity(64),
                |mut encoded, byte| {
                    let _ = write!(encoded, "{byte:02x}");
                    encoded
                },
            )
        })
}

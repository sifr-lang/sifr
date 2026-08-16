use super::{canonical_node, canonical_value, ShapeNode};
use crate::ConstValue;

pub(super) fn canonical_bytes(value: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(value.len() * 2);
    for byte in value {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}

pub(super) fn canonical_values(kind: &str, values: &[ConstValue]) -> String {
    format!(
        "{kind}[{}]",
        values
            .iter()
            .map(canonical_value)
            .collect::<Vec<_>>()
            .join(",")
    )
}

pub(super) fn canonical_sequence(kind: &str, elements: &[ShapeNode]) -> String {
    format!(
        "{kind}[{}]",
        elements
            .iter()
            .map(canonical_node)
            .collect::<Vec<_>>()
            .join(",")
    )
}

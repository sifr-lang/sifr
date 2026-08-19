use sifr_structural_identity::ShapeIdentity;

const STRUCTURAL: &str = "::sifr_runtime::interop::structural";

pub(super) fn static_expression(value: ShapeIdentity) -> String {
    let bytes = value
        .as_bytes()
        .iter()
        .map(u8::to_string)
        .collect::<Vec<_>>()
        .join(", ");
    format!("{STRUCTURAL}::ShapeIdentity::from_bytes([{bytes}])")
}

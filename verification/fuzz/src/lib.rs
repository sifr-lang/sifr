use std::fmt::Write as _;

#[must_use]
pub fn bounded_text(data: &[u8], limit: usize) -> String {
    String::from_utf8_lossy(&data[..data.len().min(limit)]).into_owned()
}

#[must_use]
pub fn valid_program(data: &[u8]) -> String {
    let mut source = String::from("def main():\n");
    let values = if data.is_empty() { &[0_u8][..] } else { data };
    for (index, byte) in values.iter().take(24).enumerate() {
        let value = i64::from(*byte);
        let _ = writeln!(source, "    value_{index}: int = {value}");
    }
    source.push_str("    print(str(value_0))\n");
    source
}

#[must_use]
pub fn ownership_program(data: &[u8]) -> String {
    let mut source = String::from(
        "class Box:\n    value: int\n\n    def __init__(self, value: int):\n        self.value = value\n\n",
    );
    source.push_str("def main():\n    first: Box = Box(1)\n");
    for (index, byte) in data.iter().take(16).enumerate() {
        match byte % 3 {
            0 => {
                let _ = writeln!(source, "    alias_{index}: Box = first");
            }
            1 => {
                let _ = writeln!(source, "    value_{index}: int = first.value");
            }
            _ => {
                let _ = writeln!(source, "    first.value = {}", byte % 17);
            }
        }
    }
    source.push_str("    print(str(first.value))\n");
    source
}

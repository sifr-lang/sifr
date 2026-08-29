use std::fmt::Write as _;

#[must_use]
pub fn bounded_text(data: &[u8], limit: usize) -> String {
    String::from_utf8_lossy(&data[..data.len().min(limit)]).into_owned()
}

#[must_use]
pub fn valid_program(data: &[u8]) -> String {
    let mut source = String::from(
        "def adjust(value: int, delta: int) -> int:\n    return value + delta\n\ndef main():\n    total: int = 0\n    values: list[int] = []\n",
    );
    let values = if data.is_empty() { &[0_u8][..] } else { data };
    for (index, byte) in values.iter().take(24).enumerate() {
        let value = i64::from(*byte);
        match byte % 4 {
            0 => {
                let _ = writeln!(source, "    value_{index}: int = adjust({value}, {index})");
                let _ = writeln!(source, "    values.append(value_{index})");
            }
            1 => {
                let _ = writeln!(source, "    if {value} % 2 == 0:");
                let _ = writeln!(source, "        total = total + {value}");
                let _ = writeln!(source, "    else:");
                let _ = writeln!(source, "        total = total - {value}");
            }
            2 => {
                let _ = writeln!(source, "    text_{index}: str = str({value})");
                let _ = writeln!(source, "    total = total + len(text_{index})");
            }
            _ => {
                let _ = writeln!(source, "    for item_{index} in range({value} % 4):");
                let _ = writeln!(source, "        total = total + item_{index}");
            }
        }
    }
    source.push_str("    print(str(total + len(values)))\n");
    source
}

#[must_use]
pub fn ownership_program(data: &[u8]) -> String {
    let mut source = String::from(
        "class Box:\n    value: int\n\n    def __init__(self, value: int):\n        self.value = value\n\n",
    );
    source.push_str(
        "def read_box(item: Box) -> int:\n    return item.value\n\ndef main():\n    first: Box = Box(1)\n    boxes: list[Box] = []\n",
    );
    for (index, byte) in data.iter().take(16).enumerate() {
        match byte % 5 {
            0 => {
                let _ = writeln!(source, "    alias_{index}: Box = first");
            }
            1 => {
                let _ = writeln!(source, "    value_{index}: int = first.value");
            }
            _ => {
                if byte & 1 == 0 {
                    let _ = writeln!(source, "    boxes.append(Box({}))", byte % 17);
                } else {
                    let _ = writeln!(source, "    read_{index}: int = read_box(first)");
                }
            }
        }
    }
    source.push_str("    print(str(first.value))\n");
    source
}

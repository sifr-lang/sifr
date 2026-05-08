mod formatter {
    pub fn render_value() -> String {
        42_i64.to_string()
    }
}

mod helper {
    use super::formatter;

    pub fn render() -> String {
        formatter::render_value()
    }
}

fn main() {
    println!("{}", helper::render());
}

use num_bigint::BigInt;

mod formatter {
    use num_bigint::BigInt;

    pub fn render_value() -> String {
        BigInt::from(42_u8).to_string()
    }
}

mod helper {
    use super::formatter;

    pub fn render() -> String {
        formatter::render_value()
    }
}

fn main() {
    let _project_dependency_marker = BigInt::from(0_u8);
    println!("{}", helper::render());
}

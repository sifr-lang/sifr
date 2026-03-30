use num_bigint::BigInt;

mod helper {
    use num_bigint::BigInt;

    pub fn render() -> String {
        let value = BigInt::from(42_u8);
        if value.to_string() == "42" {
            "adhoc milestone 3 manifest unification demo: pass".to_string()
        } else {
            "adhoc milestone 3 manifest unification demo: fail".to_string()
        }
    }
}

fn main() {
    let _manifest_relevant_type = BigInt::from(0_u8);
    println!("{}", helper::render());
}

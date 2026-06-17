mod helper {
    pub fn render() -> String {
        let value = 42_i64;
        if value.to_string() == "42" {
            "manifest unification demo: pass".to_string()
        } else {
            "manifest unification demo: fail".to_string()
        }
    }
}

fn main() {
    println!("{}", helper::render());
}

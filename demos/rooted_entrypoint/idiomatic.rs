mod shared {
    pub fn phase_label() -> &'static str {
        "adhoc milestone 1 rooted entrypoint demo: pass"
    }
}

mod helper {
    use crate::shared::phase_label;

    pub fn render() -> &'static str {
        phase_label()
    }
}

fn main() {
    println!("{}", helper::render());
}

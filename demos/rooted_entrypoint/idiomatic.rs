mod shared {
    pub fn label() -> &'static str {
        "rooted entrypoint demo: pass"
    }
}

mod helper {
    use crate::shared::label;

    pub fn render() -> &'static str {
        label()
    }
}

fn main() {
    println!("{}", helper::render());
}

mod a_provider {
    pub fn a() -> &'static str {
        "A"
    }
}

mod z_provider {
    pub fn z() -> &'static str {
        "Z"
    }
}

mod consumer {
    use crate::{a_provider, z_provider};

    pub fn joined() -> String {
        format!("{}-{}", a_provider::a(), z_provider::z())
    }
}

fn main() {
    println!("m19_2 deterministic assembly demo:");
    println!("{}", consumer::joined());
}

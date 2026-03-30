mod z_provider {
    pub fn value() -> i64 {
        41
    }
}

mod a_consumer {
    use super::z_provider;

    pub fn fetch() -> i64 {
        z_provider::value() + 1
    }
}

fn main() {
    println!("m23_2 deterministic module graph and cycle diagnostics demo:");
    println!("{}", a_consumer::fetch());
}

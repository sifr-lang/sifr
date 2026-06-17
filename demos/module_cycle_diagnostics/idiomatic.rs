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
    println!("module_cycle_diagnostics deterministic module graph and cycle diagnostics demo:");
    println!("{}", a_consumer::fetch());
}

mod provider {
    pub fn base() -> i64 {
        9
    }
}

mod consumer {
    use super::provider;

    pub fn value() -> i64 {
        provider::base() + 10
    }
}

fn main() {
    println!("module_ordering dependency-safe module ordering demo:");
    println!("{}", consumer::value());
}

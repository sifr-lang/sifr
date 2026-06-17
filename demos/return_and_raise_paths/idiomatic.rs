#[derive(Debug, Clone)]
struct ValueError {
    message: String,
}

impl ValueError {
    fn new(message: String) -> Self {
        return Self { message: message };
    }
}

impl std::fmt::Display for ValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for ValueError {}

fn classify(n: i64) -> i64 {
    if n > 0 {
        n
    } else {
        let _error = ValueError::new("non-positive".to_string());
        99
    }
}

fn main() {
    println!("return_and_raise_paths control-flow effect query unification demo:");
    println!("{}", classify(7));
    println!("{}", classify(0));
}

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

fn classify(flag: bool) -> i64 {
    if flag {
        5
    } else {
        let _error = ValueError::new("bad value".to_string());
        77
    }
}

fn main() {
    println!("reachability canonical flow truth queries demo:");
    println!("{}", classify(true));
    println!("{}", classify(false));
}

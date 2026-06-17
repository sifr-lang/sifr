fn keep_comparable<T: Clone + std::fmt::Display + PartialOrd + 'static>(x: &T) -> T {
    return x.clone();
}

fn relay_comparable<U: Clone + std::fmt::Display + PartialOrd + 'static>(x: &U) -> U {
    return keep_comparable(x);
}

fn main() {
    println!("protocol_bounds protocol bound strictness closure demo:");
    println!("{}", relay_comparable(&(9 as i64)));
    println!("{}", relay_comparable(&"ok".to_string()));
}

fn keep_comparable<T: PartialOrd>(x: T) -> T {
    x
}

fn relay_comparable<U: PartialOrd>(x: U) -> U {
    keep_comparable(x)
}

fn main() {
    println!("protocol_bounds protocol bound strictness closure demo:");
    println!("{}", relay_comparable(9_i64));
    println!("{}", relay_comparable("ok".to_string()));
}

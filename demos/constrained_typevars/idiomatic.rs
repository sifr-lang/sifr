fn echo<T: std::fmt::Display + PartialOrd + 'static>(x: T) -> T {
    return x;
}

fn smallest<U: std::fmt::Display + PartialOrd + 'static>(a: U, b: U) -> U {
    if a < b {
        return a;
    }
    return b;
}

fn main() {
    println!("m26_1 typevar constraint enforcement demo:");
    println!("{}", echo(7 as i64));
    println!("{}", echo("ok".to_string()));
    println!("{}", smallest(10 as i64, 3 as i64));
}

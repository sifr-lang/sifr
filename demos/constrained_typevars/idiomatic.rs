trait EchoType {}

impl EchoType for i64 {}
impl EchoType for String {}

trait Comparable: PartialOrd {}

impl<T: PartialOrd> Comparable for T {}

fn echo<T: EchoType>(x: T) -> T {
    x
}

fn smallest<U: Comparable>(a: U, b: U) -> U {
    if a < b {
        a
    } else {
        b
    }
}

fn main() {
    println!("constrained_typevars typevar constraint enforcement demo:");
    println!("{}", echo(7_i64));
    println!("{}", echo("ok".to_string()));
    println!("{}", smallest(10_i64, 3_i64));
}

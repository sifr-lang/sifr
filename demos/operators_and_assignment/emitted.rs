// src/main.rs
fn main() {
    let a: i64 = 10_i64;
    let b: i64 = 12_i64;
    println!("{}", a & b);
    println!("{}", a | b);
    println!("{}", a ^ b);
    println!("{}", a << (2_i64));
    println!("{}", a >> (1_i64));
    let x: i64 = 42_i64;
    println!("{}", !(x));
    println!("{}", !(0_i64));
    let y: i64 = -(7_i64);
    println!("{}", (y));
    println!("{}", (42_i64));
    let mut flags: i64 = 0_i64;
    flags |= 1_i64;
    flags |= 4_i64;
    println!("{}", flags);
    flags &= 3_i64;
    println!("{}", flags);
    flags ^= 3_i64;
    println!("{}", flags);
    flags <<= 3_i64;
    println!("{}", flags);
    flags >>= 2_i64;
    println!("{}", flags);
    let mut p: i64 = 0_i64;
    let mut q: i64 = 0_i64;
    let mut r: i64 = 0_i64;
    r = 99_i64;
    q = r;
    p = q;
    println!("{}", p);
    println!("{}", q);
    println!("{}", r);
}

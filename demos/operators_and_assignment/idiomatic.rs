fn main() {
    let a = 0b1010_i64;
    let b = 0b1100_i64;
    println!("{}", a & b);
    println!("{}", a | b);
    println!("{}", a ^ b);
    println!("{}", a << 2);
    println!("{}", a >> 1);

    let x = 42_i64;
    println!("{}", !x);
    println!("{}", !0_i64);

    let y = -7_i64;
    println!("{}", y);
    println!("{}", 42_i64);

    let mut flags = 0_i64;
    flags |= 0b0001;
    flags |= 0b0100;
    println!("{flags}");
    flags &= 0b0011;
    println!("{flags}");
    flags ^= 0b0011;
    println!("{flags}");
    flags <<= 3;
    println!("{flags}");
    flags >>= 2;
    println!("{flags}");

    let value = 99_i64;
    let (p, q, r) = (value, value, value);
    println!("{p}");
    println!("{q}");
    println!("{r}");
}

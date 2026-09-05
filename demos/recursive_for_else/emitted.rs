// src/main.rs
use ::sifr_runtime::SifrInt;
fn main() {
    fn rec(n: &SifrInt) -> SifrInt {
        let items: Vec<SifrInt> = vec![SifrInt::from_i64(1)];
        {
            let sifr_generated_broke: bool = false;
            #[expect(
                clippy::explicit_iter_loop,
                reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
            )]
            for _ in items.iter() {}
            if !sifr_generated_broke && n > &SifrInt::from_i64(0) {
                return rec(&::std::ops::Sub::sub(n, &SifrInt::from_i64(1)));
            }
        }
        SifrInt::from_i64(0)
    }
    println!("recursive_for_else canonical walker coverage demo:");
    println!("{}", rec(&SifrInt::from_i64(3)));
}

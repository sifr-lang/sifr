// src/main.rs
use ::sifr_runtime::SifrInt;
fn main() {
    fn rec(n: SifrInt) -> SifrInt {
        let items: Vec<SifrInt> = vec![SifrInt::from_i64(1)];
        {
            let sifr_generated_broke: bool = false;
            for _ in items.iter().cloned() {}
            if !sifr_generated_broke && &n > &SifrInt::from_i64(0) {
                return rec(&n - &SifrInt::from_i64(1));
            }
        }
        SifrInt::from_i64(0)
    }
    println!("recursive_for_else canonical walker coverage demo:");
    println!("{}", rec(SifrInt::from_i64(3)));
}

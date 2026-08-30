// src/main.rs
use ::sifr_runtime::SifrInt;

fn main() {
    fn rec(n: SifrInt) -> SifrInt {
        let items: Vec<SifrInt> = vec![SifrInt::from_i64(1)];
        {
            let _broke: bool = false;
            for i in items.iter().cloned() {
            }
            if !(_broke) {
                if (&n > &SifrInt::from_i64(0)) {
                    return rec(&n - &SifrInt::from_i64(1));
                }
            }
        }
        return SifrInt::from_i64(0);
    }
    println!("recursive_for_else canonical walker coverage demo:");
    println!("{}", rec(SifrInt::from_i64(3)));
}

// src/main.rs
use ::sifr_runtime::SifrInt;

fn main() {
    fn recurse(n: SifrInt) -> SifrInt {
        {
            let _broke: bool = false;
            for i in vec![SifrInt::from_i64(1)].into_iter() {
            }
            if !(_broke) {
                if (&n > &SifrInt::from_i64(0)) {
                    return recurse(&n - &SifrInt::from_i64(1));
                }
            }
        }
        return SifrInt::from_i64(0);
    }
    println!("recursive_calls semantic query layer standardization demo:");
    println!("{}", recurse(SifrInt::from_i64(4)));
}

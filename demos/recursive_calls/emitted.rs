// src/main.rs
use ::sifr_runtime::SifrInt;
fn main() {
    fn recurse(n: &SifrInt) -> SifrInt {
        {
            let sifr_generated_broke: bool = false;
            for _i in vec![SifrInt::from_i64(1)] {}
            if !sifr_generated_broke && n > SifrInt::from_i64(0) {
                return recurse(&::std::ops::Sub::sub(n, &SifrInt::from_i64(1)));
            }
        }
        SifrInt::from_i64(0)
    }
    println!("recursive_calls semantic query layer standardization demo:");
    println!("{}", recurse(SifrInt::from_i64(4)));
}

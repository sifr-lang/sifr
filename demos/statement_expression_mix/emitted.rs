// src/main.rs
use ::sifr_runtime::SifrInt;
fn main() {
    let mut acc: SifrInt = SifrInt::from_i64(0);
    let nums: Vec<SifrInt> = vec![
        SifrInt::from_i64(1),
        SifrInt::from_i64(2),
        SifrInt::from_i64(3),
    ];
    {
        let sifr_generated_broke: bool = false;
        for n in nums.iter().cloned() {
            acc = &acc + &n;
        }
        if !sifr_generated_broke {
            acc = &acc + &SifrInt::from_i64(1);
        }
    }
    let mut i: SifrInt = SifrInt::from_i64(0);
    {
        let sifr_generated_broke: bool = false;
        while &i < &SifrInt::from_i64(3) {
            acc = &acc + &i;
            i = &i + &SifrInt::from_i64(1);
        }
        if !sifr_generated_broke {
            acc = &acc + &SifrInt::from_i64(2);
        }
    }
    let ready: bool = true;
    if ready {
        acc = &acc + &SifrInt::from_i64(10);
    } else {
        acc = &acc + &SifrInt::from_i64(100);
    }
    assert!(&acc > &SifrInt::from_i64(0));
    println!("acc = {acc}");
    assert_eq!(format!("acc = {acc}").to_string(), "acc = 22");
}

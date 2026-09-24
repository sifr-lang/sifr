// src/main.rs
use ::sifr_runtime::SifrInt;
use ::sifr_runtime::SifrRange;
fn main() {
    println!(
        "max(3, 7) = {}",
        ::std::cmp::max(SifrInt::from_i64(3), SifrInt::from_i64(7))
    );
    assert_eq!(
        format!(
            "max(3, 7) = {}",
            ::std::cmp::max(SifrInt::from_i64(3), SifrInt::from_i64(7))
        ),
        "max(3, 7) = 7"
    );
    println!(
        "min(3, 7) = {}",
        ::std::cmp::min(SifrInt::from_i64(3), SifrInt::from_i64(7))
    );
    assert_eq!(
        format!(
            "min(3, 7) = {}",
            ::std::cmp::min(SifrInt::from_i64(3), SifrInt::from_i64(7))
        ),
        "min(3, 7) = 3"
    );
    println!(
        "pow(2, 10) = {}",
        SifrInt::from_i64(2).pow_known_valid(10_u32)
    );
    assert_eq!(
        format!(
            "pow(2, 10) = {}",
            SifrInt::from_i64(2).pow_known_valid(10_u32)
        ),
        "pow(2, 10) = 1024"
    );
    let mut result: String = String::new();
    let mut sifr_generated_chars_result: Vec<char> = result.chars().collect::<Vec<char>>();
    for i in SifrRange::new_known_nonzero(
        SifrInt::from_i64(0),
        SifrInt::from_i64(10),
        SifrInt::from_i64(2),
    ) {
        if sifr_generated_chars_result.len() > SifrInt::from_i64(0) {
            result.push(' ');
            sifr_generated_chars_result.push(' ');
        }
        let sifr_generated_string_concat_result_0 = i.to_string();
        result.push_str(sifr_generated_string_concat_result_0.as_str());
        sifr_generated_chars_result.extend(sifr_generated_string_concat_result_0.as_str().chars());
    }
    println!("{result}");
    assert_eq!(result, "0 2 4 6 8");
}

use sha2::{Digest, Sha224};
use std::error::Error;

tokio::task_local! {
    static CONTEXT_LABEL: String;
}

fn current_context() -> String {
    CONTEXT_LABEL
        .try_with(Clone::clone)
        .unwrap_or_else(|_| "Context".to_owned())
}

fn assert_eq(left: i64, right: i64) -> i64 {
    left + right
}

fn decode_hex(input: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let digits: String = input.chars().filter(|c| !c.is_ascii_whitespace()).collect();
    if !digits.len().is_multiple_of(2) {
        return Err("hex input must have an even number of digits".into());
    }
    digits
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| Ok(u8::from_str_radix(std::str::from_utf8(pair)?, 16)?))
        .collect()
}

fn main() -> Result<(), Box<dyn Error>> {
    assert_eq!(assert_eq(20, 22), 42);
    assert_eq!(current_context(), "Context");

    let payload = decode_hex("53 69 66 72")?;
    assert_eq!(std::str::from_utf8(&payload)?, "Sifr");
    assert_eq!(
        Sha224::digest(b"typed boundary")
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
            .len(),
        56
    );

    println!("typed compiler boundary: ok");
    Ok(())
}

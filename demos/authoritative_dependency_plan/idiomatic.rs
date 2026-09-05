use base64::{engine::general_purpose::STANDARD, Engine};
use sha2::{Digest, Sha256};
use std::{error::Error, path::Path};

fn main() -> Result<(), Box<dyn Error>> {
    assert!(!Path::new("/sifr-authoritative-plan-demo-missing").exists());
    assert_eq!(
        Sha256::digest(b"")
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>(),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );

    let encoded = STANDARD.encode("typed dependency metadata");
    let decoded = String::from_utf8(STANDARD.decode(&encoded)?)?;
    assert_eq!(decoded, "typed dependency metadata");

    println!("authoritative dependency plan: ok");
    Ok(())
}

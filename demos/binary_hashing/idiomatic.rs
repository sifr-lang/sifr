use base64::{engine::general_purpose::STANDARD, Engine as _};
use sha2::{Digest, Sha256};

fn hex_string(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = b"binary_hashing-bytes-demo";
    let digest = Sha256::digest(data);

    assert_eq!(digest.len(), 32);
    assert_eq!(hex_string(&digest).len(), 64);

    let encoded = STANDARD.encode(data).into_bytes();
    let decoded = STANDARD.decode(&encoded)?;
    assert_eq!(decoded, data);
    Ok(())
}

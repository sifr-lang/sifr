#[derive(Debug)]
pub struct HashErrorBridge {
    pub message: String,
}

pub fn hash_bytes(input: &[u8]) -> Result<Vec<u8>, HashErrorBridge> {
    Ok(blake3::hash(input).to_be_bytes().to_vec())
}

pub fn hash_hex(input: &[u8]) -> Result<String, HashErrorBridge> {
    Ok(format!("{:016x}", blake3::hash(input)))
}

pub fn map_panic(message: &str) -> HashErrorBridge {
    HashErrorBridge {
        message: message.to_owned(),
    }
}

use std::fmt;

#[derive(Debug)]
pub struct NativeErrorBridge {
    pub message: String,
}

impl fmt::Display for NativeErrorBridge {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for NativeErrorBridge {}

pub fn compress(input: &[u8]) -> Result<Vec<u8>, NativeErrorBridge> {
    zstd::encode(input).map_err(|message| NativeErrorBridge { message })
}

pub fn decompress(input: &[u8]) -> Result<Vec<u8>, NativeErrorBridge> {
    zstd::decode(input).map_err(|message| NativeErrorBridge { message })
}

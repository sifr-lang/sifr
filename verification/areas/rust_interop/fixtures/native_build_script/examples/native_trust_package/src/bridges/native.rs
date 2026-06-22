pub struct NativeErrorBridge {
    pub message: String,
}

pub fn compress(input: &[u8]) -> Result<Vec<u8>, NativeErrorBridge> {
    let mut output = zstd::encode(input);
    output.extend_from_slice(&cc::probe());
    output.extend_from_slice(&bindgen::probe());
    output.extend_from_slice(&cxx::probe());
    Ok(output)
}

pub fn map_panic(message: &str) -> NativeErrorBridge {
    NativeErrorBridge {
        message: message.to_owned(),
    }
}

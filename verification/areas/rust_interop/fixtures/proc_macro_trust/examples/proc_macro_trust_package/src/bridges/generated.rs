pub struct DecodeErrorBridge {
    pub message: String,
}

pub struct GeneratedMessageBridge {
    pub id: u64,
    pub payload: Vec<u8>,
}

#[derive(serde_derive::SifrGenerated)]
struct GeneratedSchema;

pub fn decode(input: &[u8]) -> Result<GeneratedMessageBridge, DecodeErrorBridge> {
    let _schema = GeneratedSchema;
    Ok(GeneratedMessageBridge {
        id: prost_build::schema_version(),
        payload: input.to_vec(),
    })
}

pub fn map_panic(message: &str) -> DecodeErrorBridge {
    DecodeErrorBridge {
        message: message.to_owned(),
    }
}

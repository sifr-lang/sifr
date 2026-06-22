pub struct CliErrorBridge {
    pub message: String,
}

pub fn parse_and_trace(args: &[String]) -> Result<u32, CliErrorBridge> {
    tracing::info("parse");
    tracing_subscriber::init("info");
    let parsed = clap::parse(args);
    Ok(parsed + anyhow::context_len("cli") as u32)
}

pub fn map_panic(message: &str) -> CliErrorBridge {
    CliErrorBridge {
        message: message.to_owned(),
    }
}

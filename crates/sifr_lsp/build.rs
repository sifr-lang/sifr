fn main() -> Result<(), Box<dyn std::error::Error>> {
    sifr_identity::build::emit_local_token(&["third_party/ruff"])?;
    Ok(())
}

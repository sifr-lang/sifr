fn main() -> Result<(), Box<dyn std::error::Error>> {
    sifr_identity::build::emit_local_token(&["crates/sifr_structural_identity"])?;
    Ok(())
}

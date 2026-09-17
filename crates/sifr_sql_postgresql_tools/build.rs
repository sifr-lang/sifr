fn main() -> Result<(), Box<dyn std::error::Error>> {
    sifr_identity::build::emit_local_token(&[])?;
    sifr_identity::build::emit_product_identity()?;
    Ok(())
}

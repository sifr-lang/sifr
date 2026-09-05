use sifr_stdlib::runtime_observability::emit_diagnostic;

fn main() -> Result<(), String> {
    emit_diagnostic("info", "sifr.demo", "accepted", "stdlib boundary")?;

    let rejected = emit_diagnostic("verbose", "sifr.demo", "rejected", "stdlib boundary");
    assert_eq!(
        rejected,
        Err("unsupported diagnostic level: verbose".to_owned())
    );
    Ok(())
}

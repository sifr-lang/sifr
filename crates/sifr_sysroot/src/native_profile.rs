//! Application boundary overrides shared by captured Cargo configurations.
pub(crate) fn validate_flags(flags: &str) -> Result<(), String> {
    let compact: String = flags
        .chars()
        .filter(|c| !c.is_whitespace() && !matches!(c, '\x1f' | '\"' | ','))
        .collect();
    if [
        "panic=abort",
        "overflow-checks=off",
        "overflow-checks=no",
        "overflow-checks=false",
        "overflow-checks=0",
    ]
    .iter()
    .any(|bad| compact.contains(bad))
    {
        Err("native Rust flags invalidate required unwind/overflow boundary".into())
    } else {
        Ok(())
    }
}

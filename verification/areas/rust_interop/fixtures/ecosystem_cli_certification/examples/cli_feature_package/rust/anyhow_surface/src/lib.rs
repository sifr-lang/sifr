pub fn direct_error(message: &str) -> anyhow::Error {
    anyhow::anyhow!("{message}").context("direct anyhow surface")
}

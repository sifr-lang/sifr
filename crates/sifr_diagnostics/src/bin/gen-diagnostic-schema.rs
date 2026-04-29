use std::io::{self, Write};

fn main() -> io::Result<()> {
    let schema = sifr_diagnostics::schema::diagnostic_schema_pretty_json();
    let mut stdout = io::stdout().lock();
    stdout.write_all(schema.as_bytes())?;
    stdout.write_all(b"\n")
}

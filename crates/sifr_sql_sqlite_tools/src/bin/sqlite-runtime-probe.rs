use rusqlite::Connection;
use serde::Serialize;
use std::io::Write;

#[derive(Serialize)]
struct Probe {
    version: &'static str,
    version_number: i32,
    compile_options: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let connection = Connection::open_in_memory()?;
    let mut statement = connection.prepare("PRAGMA compile_options")?;
    let mut compile_options = statement
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    compile_options.sort();
    let mut payload = serde_json::to_vec(&Probe {
        version: rusqlite::version(),
        version_number: rusqlite::version_number(),
        compile_options,
    })?;
    payload.push(b'\n');
    std::io::stdout().lock().write_all(&payload)?;
    Ok(())
}
